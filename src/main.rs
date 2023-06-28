use std::{env, process::exit};

use serenity::{prelude::{EventHandler, GatewayIntents, Context}, Client, model::prelude::{Ready, interaction::{Interaction, InteractionResponseType}, command::Command}, async_trait};
use tracing::{error, info, debug, warn};
use tracing_subscriber::{registry, fmt::layer, layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};
struct HydrogenHandler;

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("client initialized and connected to: {}", ready.user.name);

        debug!("registering commands...");
        if let Err(e) = Command::create_global_application_command(ctx.http, |command| {
            command
                .name("ping")
                .description("Ping!")
        }).await {
            error!("can't register command: {}", e);
        }

        info!("commands registered");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let command_name = command.data.name.as_str();
                debug!("executing application command: {}", command_name);
                match command_name {
                    "ping" => {
                        if let Err(e) = command.create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content("pong!")
                                })
                        }).await {
                            error!("can't response to interaction: {:?}", e);
                        }
                    }
                    _ => {
                        warn!("unknown command: {}", command_name);
                        ()
                    },
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

    debug!("initializing client...");
    let mut client = match Client::builder(match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        Err(e) => {
            error!("you need to set DISCORD_TOKEN environment variable: {:?}", e);
            exit(1);
        }
    }, GatewayIntents::default())
        .event_handler(HydrogenHandler)
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
