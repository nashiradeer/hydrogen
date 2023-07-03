use std::sync::Arc;

use async_trait::async_trait;
use serenity::{prelude::Context, model::{prelude::{application_command::ApplicationCommandInteraction, command::CommandOptionType, ChannelId, Guild, UserId, GuildId}, channel}, builder::CreateApplicationCommand};
use songbird::{ConnectionInfo, Songbird};
use tracing::warn;

use crate::{HydrogenContext, lavalink::rest::{LavalinkUpdatePlayer, LavalinkVoiceState}, HydrogenCommandListener, i18n::HydrogenI18n, player::HydrogenPlayer};

pub struct PlayCommand;

impl PlayCommand {
    fn get_channel_id(guild: Guild, user_id: UserId) -> Result<ChannelId, Result<(), String>> {
        Ok(guild.voice_states.get(&user_id).ok_or(Err("can't find the user voice state in the origin guild".to_owned()))?
            .channel_id.ok_or(Err("can't get the channel id from the voice state".to_owned()))?)
    }

    async fn join_channel(hydrogen: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction, voice_manager: Arc<Songbird>, guild_id: GuildId, channel_id: ChannelId) -> Result<ConnectionInfo, String> {
        Ok(match voice_manager.join_gateway(guild_id, channel_id).await.1 {
            Ok(v) => v,
            Err(e) => {
                if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                    response
                        .content(hydrogen.i18n.translate(&interaction.locale, "play", "cant_connect"))
                }).await {
                    warn!("can't response to interaction: {:?}", e);
                }

                return Err("can't connect to voice chat".to_owned());
            }
        })
    }

    async fn _execute(&self, hydrogen: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) -> Result<(), String> {
        let query = interaction
            .data.options.get(0).ok_or("required 'query' parameter missing".to_owned())?
            .value.clone().ok_or("required 'query' parameter missing".to_owned())?
            .as_str().ok_or("can't convert required 'query' to str".to_owned())?
            .to_owned();

        
        if let Err(e) = interaction.defer_ephemeral(&context.http).await {
            warn!("can't defer the response: {}", e);
        }

        let voice_manager = songbird::get(&context).await.ok_or("songbird not registered".to_owned())?;
        let guild_id = interaction.guild_id.ok_or("interaction doesn't have a guild_id".to_owned())?;
        let guild = context.cache.guild(guild_id).ok_or("guild isn't present in the cache".to_owned())?;

        let channel_id = match Self::get_channel_id(guild, interaction.user.id) {
            Ok(v) => v,
            Err(e) => {
                if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                    response
                        .content(hydrogen.i18n.translate(&interaction.locale, "play", "unknown_voice_state"))
                }).await {
                    warn!("can't response to interaction: {:?}", e);
                }
                return e;
            }
        };

        
        let player = match hydrogen.players.read().await.get(&guild_id).cloned() {
            Some(v) => v,
            None => {
                let connection_info = match voice_manager.get(guild_id) {
                    Some(v) => {
                        match v.lock().await.current_connection().cloned() {
                            Some(v) => v,
                            None => Self::join_channel(hydrogen, context, interaction, voice_manager, guild_id, channel_id).await?
                        }
                    },
                    None => Self::join_channel(hydrogen, context, interaction, voice_manager, guild_id, channel_id).await?
                };

                let player = HydrogenPlayer::new(guild_id, connection_info);
                hydrogen.players.blocking_write().insert(guild_id, player);
                player
            }
        };

        let result = player.play(hydrogen.lavalink, &query, interaction.user.id).await.map_err(|e| e.to_string())?;


        let mut message = String::new();
        
        

        if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
            response
                .content()
        }).await {
            warn!("can't response to interaction: {:?}", e);
        }

        Ok(())
    }
}

#[async_trait]
impl HydrogenCommandListener for PlayCommand {
    fn register<'a, 'b>(&'a self, i18n: HydrogenI18n, command: &'b mut CreateApplicationCommand) ->  &'b mut CreateApplicationCommand {
        i18n.translate_application_command_name("play", "name", command);
        i18n.translate_application_command_description("play", "description", command);

        command
            .description("Searches and plays the requested song, initializing the player if necessary.")
            .create_option(|option| {
                i18n.translate_application_command_option_name("play", "query_name", option);
                i18n.translate_application_command_option_description("play", "query_description", option);

                option
                    .kind(CommandOptionType::String)
                    .name("query")
                    .description("The query to search for.")
                    .required(true)
            })
            .dm_permission(false)
    }

    async fn execute(&self, hydrogen: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) {
        if let Err(e) = self._execute(hydrogen, context, interaction).await {
            warn!("{}", e);
        }
    }
}