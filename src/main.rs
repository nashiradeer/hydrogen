use std::{env, collections::HashMap, sync::Arc, process::exit};

use commands::play::PlayCommand;
use i18n::HydrogenI18n;
use lavalink::LavalinkNodeInfo;
use manager::HydrogenManager;
use serenity::{prelude::{EventHandler, GatewayIntents, Context}, Client, model::{prelude::{Ready, interaction::{Interaction, application_command::ApplicationCommandInteraction}, command::Command, VoiceServerUpdateEvent}, voice::VoiceState}, async_trait, builder::CreateApplicationCommand};
use songbird::SerenityInit;
use tokio::sync::RwLock;
use tracing::{error, info, debug, warn};
use tracing_subscriber::{registry, fmt::layer, layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};

mod commands;
mod i18n;
mod lavalink;
mod manager;
mod player;

#[derive(Clone)]
struct HydrogenContext {
    pub i18n: HydrogenI18n,
    pub manager: Arc<RwLock<Option<HydrogenManager>>>
}

#[derive(Clone)]
struct HydrogenHandler {
    context: HydrogenContext,
    lavalink_nodes: Arc<Vec<LavalinkNodeInfo>>,
    commands: Arc<HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>>>
}

#[async_trait]
trait HydrogenCommandListener {
    fn register<'a, 'b>(&'a self, i18n: HydrogenI18n, command: &'b mut CreateApplicationCommand) -> &'b mut CreateApplicationCommand;
    async fn execute(&self, hydrogen_context: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction);
}

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("client initialized and connected to: {}", ready.user.name);

        debug!("initializing hydrogen manager...");
        let manager = HydrogenManager::new(ctx.cache.clone(), ctx.http.clone(), self.context.i18n.clone());
        *self.context.manager.write().await = Some(manager.clone());
        info!("hydrogen manager initialized");

        debug!("registering commands...");
        for (name, command) in self.commands.iter() {
            debug!("registering '{}' command...", name);
            if let Err(e) = Command::create_global_application_command(ctx.http.clone(), |create_command| {
                command.register(self.context.i18n.clone(), create_command).name(name)
            }).await {
                error!("can't register '{}' command: {}", name, e);
            }
        }
        info!("commands registered");


        debug!("connecting to lavalink server...");
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
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let command_name = command.data.name.as_str();
                debug!("executing application command: {}", command_name);

                if let Some(listener) = self.commands.get(command_name) {
                    listener.execute(self.context.clone(), ctx, command).await;
                }
                else {
                    warn!("unknown command: {}", command_name);
                }
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, _: Context, _: Option<VoiceState>, new: VoiceState) {
        debug!("voice state update: {:?}", new);
        let option_manager = {
            let manager_locked = self.context.manager.read().await;
            manager_locked.clone()
        };
        if let Some(manager) = option_manager {
            if let Err(e) = manager.update_lavalink_connection(new).await {
                warn!("error when updating lavalink connection: {}", e);
            }
        }
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        debug!("voice server update: {:?}", voice_server);
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
        let path = env::var("LANGUAGE_PATH").expect("you need to set LANGUAGE_PATH environment variable");
        HydrogenI18n::new(path, HydrogenI18n::DEFAULT_LANGUAGE)
    }.expect("can't initialize i18n");

    debug!("parsing lavalink config...");
    let lavalink_nodes = {
        let mut lavalink_nodes = Vec::new();
        let lavalink_env = env::var("LAVALINK").expect("you need to set LAVALINK environment variable");

        for single_node in lavalink_env.split(";") {
            let mut node_components = single_node.split(",");
            let Some(host) = node_components.next() else {
                break;
            };

            let password = node_components.next().expect("some lavalink node doesn't have password set");
            let tls = node_components.next().unwrap_or("");

            lavalink_nodes.push(LavalinkNodeInfo {
                host: host.to_owned(),
                password: password.to_owned(),
                tls: tls == "true" || tls == "enabled" || tls == "on"
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
            i18n
        },
        commands: {
            let mut commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>> =  HashMap::new();
            
            commands.insert("play".to_owned(), Box::new(PlayCommand));

            Arc::new(commands)
        },
        lavalink_nodes
    };

    debug!("initializing client...");
    Client::builder(env::var("DISCORD_TOKEN").expect("you need to set DISCORD_TOKEN environment variable"), GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES)
        .event_handler(app)
        .register_songbird()
        .await.expect("can't initialize the client")
        .start().await.expect("can't start the client");
}
