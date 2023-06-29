use serenity::{async_trait, builder::CreateApplicationCommand, prelude::Context, model::prelude::{interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, command::CommandOptionType}};
use tracing::warn;

use crate::{HydrogenCommandListener, HydrogenContext};

pub struct PingCommand;

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
            warn!("can't response to interaction: {:?}", e);
        }
    }
}

pub struct PlayCommand;

#[async_trait]
impl HydrogenCommandListener for PlayCommand {
    fn register<'a, 'b>(&'a self,command: &'b mut CreateApplicationCommand) ->  &'b mut CreateApplicationCommand {
        command
            .description("Searches and plays the requested song, initializing the player if necessary.")
            .create_option(|option| option
                .kind(CommandOptionType::String)
                .name("query")
                .description("The query to search for.")
                .required(true)
            )
            .dm_permission(false)
    }

    async fn execute(&self, _: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) {
        let query = {
            let Some(option) = interaction.data.options.get(0) else {
                warn!("no 'play:query' provided");
                return;
            };

            let Some(value) = &option.value else {
                warn!("no 'play:query provided");
                return;
            };

            let Some(data) = value.as_str() else {
                warn!("invalid 'play:query' provided");
                return;
            };

            data.to_owned()
        };

        if let Err(e) = interaction.create_interaction_response(&context.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!("Requested query: {}", query))
                })
        }).await {
            warn!("can't response to interaction: {:?}", e);
        }
    }
}