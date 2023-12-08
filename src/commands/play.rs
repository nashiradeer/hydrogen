use std::sync::Arc;

use async_trait::async_trait;
use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, Guild, GuildId, UserId},
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, EditInteractionResponse,
    },
    cache::CacheRef,
    client::Context,
};
use songbird::{Call, Songbird};
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    i18n::HydrogenI18n, player::HydrogenPlayCommand, HydrogenCommandListener, HydrogenContext,
    HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

pub struct PlayCommand;

impl PlayCommand {
    #[inline]
    fn get_channel_id(
        guild: CacheRef<'_, GuildId, Guild>,
        user_id: UserId,
    ) -> Result<ChannelId, Result<(), String>> {
        Ok(guild
            .voice_states
            .get(&user_id)
            .ok_or(Err(
                "can't find the user voice state in the origin guild".to_owned()
            ))?
            .channel_id
            .ok_or(Err(
                "can't get the channel id from the voice state".to_owned()
            ))?)
    }

    #[inline]
    async fn join_channel<'a>(
        hydrogen: &'a HydrogenContext,
        context: &'a Context,
        interaction: &'a CommandInteraction,
        voice_manager: &'a Arc<Songbird>,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<Arc<Mutex<Call>>, String> {
        Ok(
            match voice_manager.join_gateway(guild_id, channel_id).await {
                Ok(v) => v.1,
                Err(e) => {
                    if let Err(e) = interaction
                        .edit_response(
                            &context.http,
                            EditInteractionResponse::new().embed(
                                CreateEmbed::new()
                                    .title(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "play",
                                        "embed_title",
                                    ))
                                    .description(format!(
                                        "{}\n\n{}",
                                        hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "cant_connect",
                                        ),
                                        hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "not_intentional",
                                        )
                                    ))
                                    .color(HYDROGEN_ERROR_COLOR)
                                    .footer(
                                        CreateEmbedFooter::new(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "generic",
                                            "embed_footer",
                                        ))
                                        .icon_url(HYDROGEN_LOGO_URL),
                                    ),
                            ),
                        )
                        .await
                    {
                        warn!("can't response to interaction: {:?}", e);
                    }

                    return Err(format!("can't connect to voice chat: {}", e));
                }
            },
        )
    }

    #[inline]
    fn get_message<'a>(
        result: HydrogenPlayCommand,
        hydrogen: &'a HydrogenContext,
        interaction: &'a CommandInteraction,
    ) -> String {
        if let Some(track) = result.track {
            if result.playing && result.count == 1 {
                if let Some(uri) = track.uri {
                    return hydrogen
                        .i18n
                        .translate(&interaction.locale, "play", "play_single_url")
                        .replace("{name}", &track.title)
                        .replace("{author}", &track.author)
                        .replace("{url}", &uri);
                } else {
                    return hydrogen
                        .i18n
                        .translate(&interaction.locale, "play", "play_single")
                        .replace("{name}", &track.title)
                        .replace("{author}", &track.author);
                }
            } else if result.count == 1 {
                if let Some(uri) = track.uri {
                    return hydrogen
                        .i18n
                        .translate(&interaction.locale, "play", "enqueue_single_url")
                        .replace("{name}", &track.title)
                        .replace("{author}", &track.author)
                        .replace("{url}", &uri);
                } else {
                    return hydrogen
                        .i18n
                        .translate(&interaction.locale, "play", "enqueue_single")
                        .replace("{name}", &track.title)
                        .replace("{author}", &track.author);
                }
            } else if result.playing {
                if !result.truncated {
                    if let Some(uri) = track.uri {
                        return hydrogen
                            .i18n
                            .translate(&interaction.locale, "play", "play_multi_url")
                            .replace("{name}", &track.title)
                            .replace("{author}", &track.author)
                            .replace("{url}", &uri)
                            .replace("{count}", &result.count.to_string());
                    } else {
                        return hydrogen
                            .i18n
                            .translate(&interaction.locale, "play", "play_multi")
                            .replace("{name}", &track.title)
                            .replace("{author}", &track.author)
                            .replace("{count}", &result.count.to_string());
                    }
                } else {
                    if let Some(uri) = track.uri {
                        return format!(
                            "{}\n\n{}",
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "play", "truncated_warn",),
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "play", "play_multi_url",)
                                .replace("{name}", &track.title)
                                .replace("{author}", &track.author)
                                .replace("{url}", &uri)
                                .replace("{count}", &result.count.to_string())
                        );
                    } else {
                        return format!(
                            "{}\n\n{}",
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "play", "truncated_warn",),
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "play", "play_multi")
                                .replace("{name}", &track.title)
                                .replace("{author}", &track.author)
                                .replace("{count}", &result.count.to_string())
                        );
                    }
                }
            }
        }

        if result.truncated {
            return format!(
                "{}\n\n{}",
                hydrogen
                    .i18n
                    .translate(&interaction.locale, "play", "truncated_warn",),
                hydrogen
                    .i18n
                    .translate(&interaction.locale, "play", "enqueue_multi")
                    .replace("{count}", &result.count.to_string())
            );
        }

        hydrogen
            .i18n
            .translate(&interaction.locale, "play", "enqueue_multi")
            .replace("{count}", &result.count.to_string())
    }

    async fn _execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: CommandInteraction,
    ) -> Result<(), String> {
        let query = interaction
            .data
            .options
            .get(0)
            .ok_or("required 'query' parameter missing".to_owned())?
            .value
            .clone()
            .as_str()
            .ok_or("can't convert required 'query' to str".to_owned())?
            .to_owned();

        interaction
            .defer_ephemeral(&context.http)
            .await
            .map_err(|e| format!("can't defer the response: {}", e))?;

        let manager = hydrogen
            .manager
            .read()
            .await
            .clone()
            .ok_or("manager not initialized".to_owned())?;
        let voice_manager = songbird::get(&context)
            .await
            .ok_or("songbird not registered".to_owned())?;
        let guild_id = interaction
            .guild_id
            .ok_or("interaction doesn't have a guild_id".to_owned())?;
        let guild = context
            .cache
            .guild(guild_id)
            .ok_or("guild isn't present in the cache".to_owned())?;

        let voice_channel_id = match Self::get_channel_id(guild, interaction.user.id) {
            Ok(v) => v,
            Err(e) => {
                if let Err(e) = interaction
                    .edit_response(
                        &context.http,
                        EditInteractionResponse::new().embed(
                            CreateEmbed::new()
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "play",
                                    "embed_title",
                                ))
                                .description(format!(
                                    "{}\n\n{}",
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "error",
                                        "unknown_voice_state",
                                    ),
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "error",
                                        "not_intentional",
                                    )
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(
                                    CreateEmbedFooter::new(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "generic",
                                        "embed_footer",
                                    ))
                                    .icon_url(HYDROGEN_LOGO_URL),
                                ),
                        ),
                    )
                    .await
                {
                    warn!("can't response to interaction: {:?}", e);
                }
                return e;
            }
        };

        let call = match voice_manager.get(guild_id) {
            Some(v) => {
                if v.lock().await.current_connection().cloned().is_none() {
                    Self::join_channel(
                        &hydrogen,
                        &context,
                        &interaction,
                        &voice_manager,
                        guild_id,
                        voice_channel_id,
                    )
                    .await?;
                }

                v
            }
            None => {
                Self::join_channel(
                    &hydrogen,
                    &context,
                    &interaction,
                    &voice_manager,
                    guild_id,
                    voice_channel_id,
                )
                .await?
            }
        };

        if let Some(connection_info) = call.lock().await.current_connection().cloned() {
            if let Some(channel_id) = connection_info.channel_id {
                if channel_id != voice_channel_id.into() {
                    if let Err(e) = interaction
                        .edit_response(
                            &context.http,
                            EditInteractionResponse::new().embed(
                                CreateEmbed::new()
                                    .title(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "play",
                                        "embed_title",
                                    ))
                                    .description(format!(
                                        "{}\n\n{}",
                                        hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "player_exists",
                                        ),
                                        hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "not_intentional",
                                        )
                                    ))
                                    .color(HYDROGEN_ERROR_COLOR)
                                    .footer(
                                        CreateEmbedFooter::new(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "generic",
                                            "embed_footer",
                                        ))
                                        .icon_url(HYDROGEN_LOGO_URL),
                                    ),
                            ),
                        )
                        .await
                    {
                        warn!("can't response to interaction: {:?}", e);
                    }

                    return Err("user isn't in the same channel".to_owned());
                }
            }
        }
        let result = manager
            .init_or_play(
                guild_id,
                &interaction
                    .guild_locale
                    .clone()
                    .unwrap_or(interaction.locale.clone()),
                &query,
                interaction.user.id,
                voice_manager.clone(),
                interaction.channel_id,
            )
            .await
            .map_err(|e| e.to_string())?;

        if result.count > 0 {
            if let Err(e) = interaction
                .edit_response(
                    &context.http,
                    EditInteractionResponse::new().embed(
                        CreateEmbed::new()
                            .title(hydrogen.i18n.translate(
                                &interaction.locale,
                                "play",
                                "embed_title",
                            ))
                            .description(Self::get_message(result, &hydrogen, &interaction))
                            .color(HYDROGEN_PRIMARY_COLOR)
                            .footer(
                                CreateEmbedFooter::new(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "generic",
                                    "embed_footer",
                                ))
                                .icon_url(HYDROGEN_LOGO_URL),
                            ),
                    ),
                )
                .await
            {
                warn!("can't response to interaction: {:?}", e);
            }
        } else {
            if !result.truncated {
                if let Err(e) = interaction
                    .edit_response(
                        &context.http,
                        EditInteractionResponse::new().embed(
                            CreateEmbed::new()
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "play",
                                    "embed_title",
                                ))
                                .description(format!(
                                    "{}\n\n{}",
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "play",
                                        "not_found",
                                    ),
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "error",
                                        "not_intentional",
                                    )
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(
                                    CreateEmbedFooter::new(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "generic",
                                        "embed_footer",
                                    ))
                                    .icon_url(HYDROGEN_LOGO_URL),
                                ),
                        ),
                    )
                    .await
                {
                    warn!("can't response to interaction: {:?}", e);
                }
            } else {
                if let Err(e) = interaction
                    .edit_response(
                        &context.http,
                        EditInteractionResponse::new().embed(
                            CreateEmbed::new()
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "play",
                                    "embed_title",
                                ))
                                .description(format!(
                                    "{}\n\n{}",
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "play",
                                        "truncated",
                                    ),
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "error",
                                        "not_intentional",
                                    )
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(
                                    CreateEmbedFooter::new(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "generic",
                                        "embed_footer",
                                    ))
                                    .icon_url(HYDROGEN_LOGO_URL),
                                ),
                        ),
                    )
                    .await
                {
                    warn!("can't response to interaction: {:?}", e);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl HydrogenCommandListener for PlayCommand {
    fn register<'a, 'b>(&'a self, i18n: HydrogenI18n) -> CreateCommand {
        let mut command = CreateCommand::new("play");

        command = i18n.translate_application_command_name("play", "name", command);
        command = i18n.translate_application_command_description("play", "description", command);

        command
            .description(
                "Request music to be played, enqueuing it in the queue or playing immediately if empty.",
            )
            .add_option({
                let mut option = CreateCommandOption::new(
                    CommandOptionType::String,
                    "query",
                    "A music or playlist URL, or a search term.",
                )
                .required(true);

                option =
                    i18n.translate_application_command_option_name("play", "query_name", option);
                option = i18n.translate_application_command_option_description(
                    "play",
                    "query_description",
                    option,
                );

                option
            })
            .dm_permission(false)
    }

    async fn execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: CommandInteraction,
    ) {
        if let Err(e) = self._execute(hydrogen, context, interaction).await {
            warn!("{}", e);
        }
    }
}
