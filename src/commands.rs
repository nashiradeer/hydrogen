use serenity::{async_trait, builder::CreateApplicationCommand, prelude::Context, model::prelude::{interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, command::CommandOptionType}};
use tracing::warn;

use crate::{HydrogenCommandListener, HydrogenContext, lavalink::{LavalinkRestUpdatePlayer, LavalinkRestVoiceState}};

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

    async fn execute(&self, hydrogen: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) {
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

        
        if let Err(e) = interaction.defer_ephemeral(&context.http).await {
            warn!("can't defer the response: {}", e);
        }

        let Some(guild_id) = interaction.guild_id else {
            warn!("command hasn't executed in a guild");
            return;
        };

        let Some(guild) = context.cache.guild(guild_id) else {
            warn!("guild aren't in the cache");
            return;
        };

        let Some(voice_state) = guild.voice_states.get(&interaction.user.id) else {
            if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                response
                    .content("You aren't in a voice chat!")
            }).await {
                warn!("can't response to interaction: {:?}", e);
            }
            return;
        };

        let Some(channel_id) = voice_state.channel_id else {
            if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                response
                    .content("You aren't in a voice chat!")
            }).await {
                warn!("can't response to interaction: {:?}", e);
            }
            return;
        };

        let Some(voice_manager) = songbird::get(&context).await else {
            warn!("songbird not registered");
            return;
        };

        let voice = match voice_manager.get(guild_id) {
            Some(_) => None,
            None => {
                let connection = match voice_manager.join_gateway(guild_id, channel_id).await.1 {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("can't join the voice chat: {}", e);

                        if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                            response
                                .content("Can't connect to the voice chat!")
                        }).await {
                            warn!("can't response to interaction: {:?}", e);
                        }

                        return;
                    }
                };

                Some(LavalinkRestVoiceState {
                    endpoint: connection.endpoint,
                    session_id: connection.session_id,
                    token: connection.token,
                    connected: false,
                    ping: 0
                })
            }
        };

        let music = match hydrogen.lavalink.update_player(&guild_id.0.to_string(), false, LavalinkRestUpdatePlayer {
            encoded_track: None,
            identifier: Some(Some(query.to_owned())),
            position: None,
            end_time: None,
            volume: None,
            paused: None,
            voice
        }).await {
            Ok(v) => v,
            Err(e) => {
                warn!("can't update lavalink player: {:?}", e);
                return;
            }
        };

        let Some(track) = music.track else {
            warn!("lavalink update player don't has returned the track");
            return;
        };

        if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
            response
                .content(format!("Requested music: {}", track.info.title))
        }).await {
            warn!("can't response to interaction: {:?}", e);
        }
    }
}