use std::{collections::HashMap, env, process::exit, sync::Arc};

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
        info!("client initialized and connected to: {}", ready.user.name);

        debug!("initializing hydrogen manager...");
        let manager = HydrogenManager::new(
            ctx.cache.clone(),
            ctx.http.clone(),
            self.context.i18n.clone(),
        );
        *self.context.manager.write().await = Some(manager.clone());
        info!("hydrogen manager initialized");

        debug!("registering commands...");
        for (name, command) in self.commands.iter() {
            debug!("registering '{}' command...", name);
            if let Err(e) = Command::create_global_command(
                ctx.http.clone(),
                command.register(self.context.i18n.clone()),
            )
            .await
            {
                error!("can't register '{}' command: {}", name, e);
            }
        }
        info!("commands registered");

        info!("connecting to the lavalink nodes...");
        for i in 0..self.lavalink_nodes.len() {
            if let Some(node) = self.lavalink_nodes.get(i) {
                if let Err(e) = manager.connect_lavalink(node.clone()).await {
                    error!("can't connect to the lavalink node {}: {}", i, e);
                }
            }
        }

        if manager.lavalink_node_count().await == 0 {
            error!("there's not lavalink nodes connected");
            exit(1);
        }

        info!(
            "connected to {} lavalink nodes",
            manager.lavalink_node_count().await
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.clone();
                debug!("executing application command: {}", command_name);

                if let Some(listener) = self.commands.get(&command_name) {
                    listener.execute(self.context.clone(), ctx, command).await;
                } else {
                    warn!("unknown command: {}", command_name);
                }

                debug!("application command executed: {}", command_name);
            }
            Interaction::Component(component) => {
                let component_name = component.data.custom_id.clone();
                debug!("executing message component: {}", component_name);

                if let Some(listener) = self.components.get(&component_name) {
                    listener.execute(self.context.clone(), ctx, component).await;
                } else {
                    warn!("unknown component: {}", component_name);
                }

                debug!("message component executed: {}", component_name);
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, _: Context, old: Option<VoiceState>, new: VoiceState) {
        debug!("processing voice state update...");
        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            if let Err(e) = manager.update_voice_state(old, new).await {
                warn!("error when updating player voice state: {}", e);
            }
        }
        debug!("processed voice state update");
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        debug!("processing voice server update...");
        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            if let Err(e) = manager.update_voice_server(voice_server).await {
                warn!("error when updating player voice server: {}", e);
            }
        }
        debug!("processed voice server update");
    }
}

#[tokio::main]
async fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("starting up...");

    debug!("initializing i18n...");
    let i18n = {
        let path =
            env::var("LANGUAGE_PATH").expect("you need to set LANGUAGE_PATH environment variable");
        HydrogenI18n::new(path, HydrogenI18n::DEFAULT_LANGUAGE)
    }
    .expect("can't initialize i18n");

    debug!("parsing lavalink config...");
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
                .expect("some lavalink node doesn't have password set");
            let tls = node_components.next().unwrap_or("");

            lavalink_nodes.push(LavalinkNodeInfo {
                host: host.to_owned(),
                password: password.to_owned(),
                tls: tls == "true" || tls == "enabled" || tls == "on",
            });
        }

        if lavalink_nodes.len() == 0 {
            error!("at least one lavalink node is required to work");
            exit(1);
        }

        Arc::new(lavalink_nodes)
    };

    debug!("initializing handler...");
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

    debug!("initializing client...");
    Client::builder(
        env::var("DISCORD_TOKEN").expect("you need to set DISCORD_TOKEN environment variable"),
        GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
    )
    .event_handler(app)
    .register_songbird()
    .await
    .expect("can't initialize the client")
    .start()
    .await
    .expect("can't start the client");
}
