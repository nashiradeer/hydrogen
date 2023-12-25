use std::{collections::HashMap, env, process::exit, sync::Arc, time::Instant};

use async_trait::async_trait;
use commands::play::PlayCommand;
use i18n::HydrogenI18n;
use lavalink::LavalinkNodeInfo;
use manager::HydrogenManager;
use serenity::{
    all::{
        Client, Command, CommandInteraction, ComponentInteraction, GatewayIntents, Interaction,
        Ready, VoiceServerUpdateEvent, VoiceState,
    },
    builder::CreateCommand,
    client::{Context, EventHandler},
};
use songbird::SerenityInit;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter,
};

use crate::{
    commands::{join::JoinCommand, seek::SeekCommand},
    components::{
        loop_switch::LoopComponent, pause::PauseComponent, prev::PrevComponent,
        skip::SkipComponent, stop::StopComponent,
    },
};

mod commands;
mod components;
mod i18n;
mod lavalink;
mod manager;
mod player;

pub const HYDROGEN_PRIMARY_COLOR: i32 = 0x5865f2;
pub const HYDROGEN_ERROR_COLOR: i32 = 0xf04747;
pub const HYDROGEN_EMPTY_CHAT_TIMEOUT: u64 = 10;
pub const HYDROGEN_QUEUE_LIMIT: usize = 1000;
pub const HYDROGEN_SEARCH_PREFIX: &str = "ytsearch:";
pub const LAVALINK_CONNECTION_TIMEOUT: u64 = 5000;

pub static HYDROGEN_LOGO_URL: &str =
    "https://raw.githubusercontent.com/nashiradeer/hydrogen/main/icon.png";
pub static HYDROGEN_BUG_URL: &str = "https://github.com/nashiradeer/hydrogen/issues";

#[derive(Clone)]
struct HydrogenContext {
    pub i18n: HydrogenI18n,
    pub manager: Arc<RwLock<Option<HydrogenManager>>>,
}

#[derive(Clone)]
struct HydrogenHandler {
    context: HydrogenContext,
    lavalink_nodes: Arc<Vec<LavalinkNodeInfo>>,
    commands: Arc<HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>>>,
    components: Arc<HashMap<String, Box<dyn HydrogenComponentListener + Sync + Send>>>,
}

#[async_trait]
trait HydrogenCommandListener {
    fn register(&self, i18n: HydrogenI18n) -> CreateCommand;

    async fn execute(
        &self,
        hydrogen_context: HydrogenContext,
        context: Context,
        interaction: CommandInteraction,
    );
}

#[async_trait]
trait HydrogenComponentListener {
    async fn execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ComponentInteraction,
    );
}

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let timer = Instant::now();
        debug!("(ready): processing...");

        let manager = HydrogenManager::new(
            ctx.cache.clone(),
            ctx.http.clone(),
            self.context.i18n.clone(),
        );
        *self.context.manager.write().await = Some(manager.clone());
        debug!("(ready): HydrogenManager initialized");

        for (name, command) in self.commands.iter() {
            debug!("(ready): registering command: {}", name);
            if let Err(e) = Command::create_global_command(
                ctx.http.clone(),
                command.register(self.context.i18n.clone()),
            )
            .await
            {
                error!("(ready): cannot register the command '{}': {}", name, e);
            }
        }
        debug!("(ready): commands registered");

        for i in 0..self.lavalink_nodes.len() {
            if let Some(node) = self.lavalink_nodes.get(i) {
                if let Err(e) = manager.connect_lavalink(node.clone()).await {
                    error!("(ready): cannot connect to the lavalink node {}: {}", i, e);
                }
            }
        }

        if manager.lavalink_node_count().await == 0 {
            error!("(ready): no lavalink nodes connected.");
            exit(1);
        }

        info!(
            "(ready): connected to {} lavalink nodes",
            manager.lavalink_node_count().await
        );

        info!("(ready): client connected to '{}' in {}ms", ready.user.name, timer.elapsed().as_millis());
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let timer = Instant::now();
        debug!("(interaction_create): processing...");

        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.clone();

                if let Some(listener) = self.commands.get(&command_name) {
                    listener.execute(self.context.clone(), ctx, command).await;
                } else {
                    warn!("(interaction_create): command not found: {}", command_name);
                }

                info!("(interaction_create): command '{}' executed in {}ms", command_name, timer.elapsed().as_millis());
            }
            Interaction::Component(component) => {
                let component_name = component.data.custom_id.clone();

                if let Some(listener) = self.components.get(&component_name) {
                    listener.execute(self.context.clone(), ctx, component).await;
                } else {
                    warn!("(interaction_create): component not found: {}", component_name);
                }

                info!("(interaction_create): component '{}' executed in {}ms", component_name, timer.elapsed().as_millis());
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, _: Context, old: Option<VoiceState>, new: VoiceState) {
        let timer = Instant::now();
        debug!("(voice_state_update): processing...");

        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            if let Err(e) = manager.update_voice_state(old, new).await {
                warn!("(voice_state_update): cannot update the HydrogenManager's player voice state: {}", e);
            }
        }

        info!("(voice_state_update): processed in {}ms", timer.elapsed().as_millis());
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        let timer = Instant::now();
        debug!("(voice_server_update): processing...");

        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            if let Err(e) = manager.update_voice_server(voice_server).await {
                warn!("(voice_server_update): cannot update HydrogenManager's player voice server: {}", e);
            }
        }

        info!("(voice_server_update): processed in {}ms...", timer.elapsed().as_millis());
    }
}

#[tokio::main]
async fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    let i18n = {
        let path =
            env::var("LANGUAGE_PATH").expect("you need to set LANGUAGE_PATH environment variable");
        HydrogenI18n::new(path, HydrogenI18n::DEFAULT_LANGUAGE)
    }
    .expect("cannot initialize HydrogenI18n");

    let lavalink_nodes = {
        let mut lavalink_nodes = Vec::new();
        let lavalink_env =
            env::var("LAVALINK").expect("you need to set LAVALINK environment variable");

        for single_node in lavalink_env.split(";") {
            let mut node_components = single_node.split(",");
            let Some(host) = node_components.next() else {
                break;
            };

            let password = node_components
                .next()
                .expect("lavalink node doesn't have a password set");
            let tls = node_components.next().unwrap_or("");

            lavalink_nodes.push(LavalinkNodeInfo {
                host: host.to_owned(),
                password: password.to_owned(),
                tls: tls == "true" || tls == "enabled" || tls == "on",
            });
        }

        if lavalink_nodes.len() == 0 {
            panic!("at least one lavalink node is required to work");
        }

        Arc::new(lavalink_nodes)
    };

    let app = HydrogenHandler {
        context: HydrogenContext {
            manager: Arc::new(RwLock::new(None)),
            i18n,
        },
        commands: {
            let mut commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>> =
                HashMap::new();

            commands.insert("play".to_owned(), Box::new(PlayCommand));
            commands.insert("join".to_owned(), Box::new(JoinCommand));
            commands.insert("seek".to_owned(), Box::new(SeekCommand));

            Arc::new(commands)
        },
        components: {
            let mut components: HashMap<String, Box<dyn HydrogenComponentListener + Sync + Send>> =
                HashMap::new();

            components.insert("stop".to_owned(), Box::new(StopComponent));
            components.insert("loop".to_owned(), Box::new(LoopComponent));
            components.insert("pause".to_owned(), Box::new(PauseComponent));
            components.insert("skip".to_owned(), Box::new(SkipComponent));
            components.insert("prev".to_owned(), Box::new(PrevComponent));

            Arc::new(components)
        },
        lavalink_nodes,
    };

    Client::builder(
        env::var("DISCORD_TOKEN").expect("you need to set DISCORD_TOKEN environment variable"),
        GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
    )
    .event_handler(app)
    .register_songbird()
    .await
    .expect("cannot initialize client")
    .start()
    .await
    .expect("cannot start client");
}
