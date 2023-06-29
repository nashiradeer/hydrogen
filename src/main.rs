use std::{env, process::exit, collections::HashMap};

use serenity::{prelude::{EventHandler, GatewayIntents, Context}, Client, model::prelude::{Ready, interaction::{Interaction, InteractionResponseType, application_command::ApplicationCommandInteraction}, command::Command}, async_trait, builder::CreateApplicationCommand};
use tracing::{error, info, debug, warn};
use tracing_subscriber::{registry, fmt::layer, layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};

#[derive(Clone)]
struct HydrogenContext;

struct HydrogenHandler {
    context: HydrogenContext,
    commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>>
}

#[async_trait]
trait HydrogenCommandListener {
    fn register<'a, 'b>(&'a self, command: &'b mut CreateApplicationCommand) -> &'b mut CreateApplicationCommand;
    async fn execute(&self, hydrogen_context: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction);
}

struct PingCommand;

#[async_trait]
impl HydrogenCommandListener for PingCommand {
    fn register<'a, 'b>(&'a self,command: &'b mut CreateApplicationCommand) ->  &'b mut CreateApplicationCommand {
        command
            .description("Ping!")
    }

    async fn execute(&self, _: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) {
        if let Err(e) = interaction.create_interaction_response(&context.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("pong!")
                })
        }).await {
            error!("can't response to interaction: {:?}", e);
        }
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

    let mut commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>> = HashMap::new();
    commands.insert("ping".to_owned(), Box::new(PingCommand));

    debug!("initializing client...");
    let mut client = match Client::builder(match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        Err(e) => {
            error!("you need to set DISCORD_TOKEN environment variable: {:?}", e);
            exit(1);
        }
    }, GatewayIntents::default())
        .event_handler(HydrogenHandler {
            context: HydrogenContext,
            commands
        })
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
