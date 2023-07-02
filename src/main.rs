use std::{env, collections::HashMap, sync::Arc, process::exit};

use lavalink::{websocket::LavalinkReadyEvent, LavalinkHandler};
use serenity::{prelude::{EventHandler, GatewayIntents, Context}, Client, model::prelude::{Ready, interaction::{Interaction, application_command::ApplicationCommandInteraction}, command::Command}, async_trait, builder::CreateApplicationCommand};
use songbird::SerenityInit;
use tracing::{error, info, debug, warn};
use tracing_subscriber::{registry, fmt::layer, layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};

mod commands;
use crate::commands::play::PlayCommand;

mod lavalink;
use crate::lavalink::Lavalink;

#[derive(Clone)]
struct HydrogenContext {
    pub lavalink: Arc<Lavalink>
}

#[derive(Clone)]
struct HydrogenHandler {
    context: HydrogenContext,
    commands: Arc<HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>>>
}

#[async_trait]
trait HydrogenCommandListener {
    fn register<'a, 'b>(&'a self, command: &'b mut CreateApplicationCommand) -> &'b mut CreateApplicationCommand;
    async fn execute(&self, hydrogen_context: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction);
}

#[async_trait]
impl LavalinkHandler for HydrogenHandler {
    async fn lavalink_ready(&self, _: Lavalink, _: LavalinkReadyEvent) {
        info!("lavalink initialized and connected");
    }

    async fn lavalink_disconnect(&self, _node: Lavalink) {
        error!("lavalink has disconnected");
        exit(1);
    }
}

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("client initialized and connected to: {}", ready.user.name);

        debug!("registering commands...");
        for (name, command) in self.commands.iter() {
            debug!("registering '{}' command...", name);
            if let Err(e) = Command::create_global_application_command(ctx.http.clone(), |create_command| {
                command.register(create_command).name(name)
            }).await {
                error!("can't register '{}' command: {}", name, e);
            }
        }

        info!("commands registered");

        debug!("connecting to lavalink server...");
        if let Err(e) = self.context.lavalink.connect(&ready.user.id.0.to_string(), self.clone()).await {
            error!("can't connect to the lavalink server: {}", e);
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
}

#[tokio::main]
async fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("starting up...");

    debug!("initializing lavalink...");
    let lavalink = {
        let uri = env::var("LAVALINK_URL").expect("you need to set LAVALINK_URL environment variable");
        let password = env::var("LAVALINK_PASSWORD").expect("you need to set LAVALINK_PASSWORD environment variable");
        let tls = env::var("LAVALINK_TLS").unwrap_or_default().to_lowercase();

        Arc::new(Lavalink::new(&uri, &password, tls == "true" || tls == "enabled" || tls == "on").expect("can't initialize lavalink"))
    };

    debug!("initializing handler...");
    let app = HydrogenHandler {
        context: HydrogenContext {
            lavalink
        },
        commands: {
            let mut commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>> =  HashMap::new();
            
            commands.insert("play".to_owned(), Box::new(PlayCommand));

            Arc::new(commands)
        }
    };

    

    debug!("initializing client...");
    Client::builder(env::var("DISCORD_TOKEN").expect("you need to set DISCORD_TOKEN environment variable"), GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES)
        .event_handler(app)
        .register_songbird()
        .await.expect("can't initialize the client")
        .start().await.expect("can't start the client");
}
