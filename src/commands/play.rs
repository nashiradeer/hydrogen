use std::sync::Arc;

use async_trait::async_trait;
use serenity::{prelude::Context, model::prelude::{application_command::ApplicationCommandInteraction, command::CommandOptionType, ChannelId, Guild, UserId, GuildId}, builder::CreateApplicationCommand};
use songbird::{Songbird, Call};
use tokio::sync::Mutex;
use tracing::warn;

use crate::{HydrogenContext, HydrogenCommandListener, i18n::HydrogenI18n, player::HydrogenPlayCommand};

pub struct PlayCommand;

impl PlayCommand {
    #[inline]
    fn get_channel_id(guild: Guild, user_id: UserId) -> Result<ChannelId, Result<(), String>> {
        Ok(guild.voice_states.get(&user_id).ok_or(Err("can't find the user voice state in the origin guild".to_owned()))?
            .channel_id.ok_or(Err("can't get the channel id from the voice state".to_owned()))?)
    }

    #[inline]
    async fn join_channel<'a>(hydrogen: &'a HydrogenContext, context: &'a Context, interaction: &'a ApplicationCommandInteraction, voice_manager: &'a Arc<Songbird>, guild_id: GuildId, channel_id: ChannelId) -> Result<Arc<Mutex<Call>>, String> {
        let voice = voice_manager.join_gateway(guild_id, channel_id).await;
        Ok(match voice.1 {
            Ok(_) => voice.0,
            Err(e) => {
                if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                    response
                        .embed(|embed|
                            embed
                                .title(hydrogen.i18n.translate(&interaction.locale, "play", "embed_title"))
                                .description(hydrogen.i18n.translate(&interaction.locale, "play", "cant_connect"))
                                .color(0xf04747)
                                .footer(|footer|
                                    footer
                                        .text(hydrogen.i18n.translate(&interaction.locale, "embed", "footer_text"))
                                        .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                                )
                        )
                }).await {
                    warn!("can't response to interaction: {:?}", e);
                }

                return Err(format!("can't connect to voice chat: {}", e));
            }
        })
    }

    #[inline]
    fn get_message<'a>(result: HydrogenPlayCommand, hydrogen: &'a HydrogenContext, interaction: &'a ApplicationCommandInteraction) -> String {
        if let Some(track) = result.track {
            if result.playing && result.count == 1 {
                if let Some(uri) = track.uri {
                    return hydrogen.i18n.translate(&interaction.locale, "play", "playing_one_uri")
                        .replace("${music}", &track.title)
                        .replace("${author}", &track.author)
                        .replace("${uri}", &uri);
                } else {
                    return hydrogen.i18n.translate(&interaction.locale, "play", "playing_one")
                        .replace("${music}", &track.title)
                        .replace("${author}", &track.author);
                }
            } else if result.count == 1 {
                if let Some(uri) = track.uri {
                    return hydrogen.i18n.translate(&interaction.locale, "play", "enqueue_one_uri")
                        .replace("${music}", &track.title)
                        .replace("${author}", &track.author)
                        .replace("${uri}", &uri);
                } else {
                    return hydrogen.i18n.translate(&interaction.locale, "play", "enqueue_one")
                        .replace("${music}", &track.title)
                        .replace("${author}", &track.author);
                }
            } else if result.playing {
                if let Some(uri) = track.uri {
                    return hydrogen.i18n.translate(&interaction.locale, "play", "playing_playlist_uri")
                        .replace("${music}", &track.title)
                        .replace("${author}", &track.author)
                        .replace("${uri}", &uri)
                        .replace("${count}", &result.count.to_string());
                } else {
                    return hydrogen.i18n.translate(&interaction.locale, "play", "playing_playlist")
                        .replace("${music}", &track.title)
                        .replace("${author}", &track.author)
                        .replace("${count}", &result.count.to_string());
                }
            }
        }
        
        return hydrogen.i18n.translate(&interaction.locale, "play", "enqueue_playlist")
            .replace("${count}", &result.count.to_string());
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

        let manager = hydrogen.manager.read().await.clone().ok_or("manager not initialized".to_owned())?;
        let voice_manager = songbird::get(&context).await.ok_or("songbird not registered".to_owned())?;
        let guild_id = interaction.guild_id.ok_or("interaction doesn't have a guild_id".to_owned())?;
        let guild = context.cache.guild(guild_id).ok_or("guild isn't present in the cache".to_owned())?;

        let voice_channel_id = match Self::get_channel_id(guild, interaction.user.id) {
            Ok(v) => v,
            Err(e) => {
                if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                    response
                        .embed(|embed|
                            embed
                                .title(hydrogen.i18n.translate(&interaction.locale, "play", "embed_title"))
                                .description(hydrogen.i18n.translate(&interaction.locale, "play", "unknown_voice_state"))
                                .color(0xf04747)
                                .footer(|footer|
                                    footer
                                        .text(hydrogen.i18n.translate(&interaction.locale, "embed", "footer_text"))
                                        .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                                )
                        )
                }).await {
                    warn!("can't response to interaction: {:?}", e);
                }
                return e;
            }
        };

        let call = match voice_manager.get(guild_id) {
            Some(v) => {
                if v.lock().await.current_connection().cloned().is_none() {
                    Self::join_channel(&hydrogen, &context, &interaction, &voice_manager, guild_id, voice_channel_id).await?;
                }

                v
            },
            None => Self::join_channel(&hydrogen, &context, &interaction, &voice_manager, guild_id, voice_channel_id).await?
        };

        if let Some(connection_info) = call.lock().await.current_connection().cloned() {
            if let Some(channel_id) = connection_info.channel_id {
                if channel_id != voice_channel_id.into() {
                    if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                        response
                            .embed(|embed|
                                embed
                                    .title(hydrogen.i18n.translate(&interaction.locale, "play", "embed_title"))
                                    .description(hydrogen.i18n.translate(&interaction.locale, "play", "is_not_same_voice"))
                                    .color(0xf04747)
                                    .footer(|footer|
                                        footer
                                            .text(hydrogen.i18n.translate(&interaction.locale, "embed", "footer_text"))
                                            .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                                    )
                            )
                    }).await {
                        warn!("can't response to interaction: {:?}", e);
                    }

                    return Err("user isn't in the same channel".to_owned());
                }
            }
        }
        let result = manager.init_or_play(
            guild_id,
            &interaction.guild_locale.clone().unwrap_or(HydrogenI18n::DEFAULT_LANGUAGE.to_owned()),
            &query,
            interaction.user.id,
            interaction.channel_id,
            call
        ).await.map_err(|e| e.to_string())?;

        if result.count > 0 {
            if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                response
                    .embed(|embed|
                        embed
                            .title(hydrogen.i18n.translate(&interaction.locale, "play", "embed_title"))
                            .description(Self::get_message(result, &hydrogen, &interaction))
                            .color(0x5865f2)
                            .footer(|footer|
                                footer
                                    .text(hydrogen.i18n.translate(&interaction.locale, "embed", "footer_text"))
                                    .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                            )
                    )
            }).await {
                warn!("can't response to interaction: {:?}", e);
            }
        } else {
            if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                response
                    .embed(|embed|
                        embed
                            .title(hydrogen.i18n.translate(&interaction.locale, "play", "embed_title"))
                            .description(hydrogen.i18n.translate(&interaction.locale, "play", "not_found"))
                            .color(0xf04747)
                            .footer(|footer|
                                footer
                                    .text(hydrogen.i18n.translate(&interaction.locale, "embed", "footer_text"))
                                    .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                            )
                    )
            }).await {
                warn!("can't response to interaction: {:?}", e);
            }
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