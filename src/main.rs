use std::{env, process::exit, collections::HashMap, sync::Arc};

use lavalink::{HydrogenLavalinkHandler, LavalinkSocketReady};
use serenity::{prelude::{EventHandler, GatewayIntents, Context}, Client, model::prelude::{Ready, interaction::{Interaction, application_command::ApplicationCommandInteraction}, command::Command}, async_trait, builder::CreateApplicationCommand};
use tracing::{error, info, debug, warn};
use tracing_subscriber::{registry, fmt::layer, layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};

mod commands;
use crate::commands::{PingCommand, PlayCommand};

mod lavalink;
use crate::lavalink::HydrogenLavalink;

#[derive(Clone)]
struct HydrogenContext;

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
impl HydrogenLavalinkHandler for HydrogenHandler {
    async fn lavalink_ready(&self, _: LavalinkSocketReady) {
        info!("lavalink initialized and connected");
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

        debug!("initializing lavalink...");
        {
            let uri = match env::var("LAVALINK_URL") {
                Ok(v) => v,
                Err(e) => {
                    error!("you need to set LAVALINK_URL environment variable: {:?}", e);
                    exit(1);
                }
            };

            let password = match env::var("LAVALINK_PASSWORD") {
                Ok(v) => v,
                Err(e) => {
                    error!("you need to set LAVALINK_PASSWORD environment variable: {:?}", e);
                    exit(1);
                }
            };

            match HydrogenLavalink::new(&uri, &password, &ready.user.id.0.to_string(), self.clone()).await {
                Ok(v) => v,
                Err(e) => {
                    error!("can't initialize lavalink: {}", e);
                    exit(4);
                }
            }
        };
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

                ()
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
    let app = HydrogenHandler {
        context: HydrogenContext,
        commands: {
            let mut commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>> =  HashMap::new();
            
            commands.insert("ping".to_owned(), Box::new(PingCommand));
            commands.insert("play".to_owned(), Box::new(PlayCommand));

            Arc::new(commands)
        }
    };

    debug!("initializing client...");
    let mut client = match Client::builder(match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        Err(e) => {
            error!("you need to set DISCORD_TOKEN environment variable: {:?}", e);
            exit(1);
        }
    }, GatewayIntents::default())
        .event_handler(app)
        .await {
            Ok(v) => v,
            Err(e) => {
                error!("can't initialize the client: {:?}", e);
                exit(2);
            }
        };

    if let Err(e) = client.start().await {
        error!("can't start the client: {:?}", e);
        exit(3);
    }
}
