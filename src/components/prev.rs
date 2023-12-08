use async_trait::async_trait;
use serenity::{
    all::{ChannelId, ComponentInteraction, Guild, GuildId, UserId},
    builder::{CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    cache::CacheRef,
    client::Context,
};
use tracing::warn;

use crate::{
    player::HydrogenMusic, HydrogenComponentListener, HydrogenContext, HYDROGEN_ERROR_COLOR,
    HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

pub struct PrevComponent;

impl PrevComponent {
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
    fn get_message<'a>(
        track: HydrogenMusic,
        hydrogen: &'a HydrogenContext,
        interaction: &'a ComponentInteraction,
    ) -> String {
        if let Some(uri) = track.uri {
            return hydrogen
                .i18n
                .translate(&interaction.locale, "prev", "returning_url")
                .replace("{name}", &track.title)
                .replace("{author}", &track.author)
                .replace("{url}", &uri);
        } else {
            return hydrogen
                .i18n
                .translate(&interaction.locale, "prev", "returning")
                .replace("${name}", &track.title)
                .replace("${author}", &track.author);
        }
    }

    async fn _execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ComponentInteraction,
    ) -> Result<(), String> {
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
                                    "prev",
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

        if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
            if my_channel_id == voice_channel_id.into() {
                let music = match manager.prev(guild_id).await {
                    Ok(v) => v,
                    Err(e) => {
                        if let Err(e) = interaction
                            .edit_response(
                                &context.http,
                                EditInteractionResponse::new().embed(
                                    CreateEmbed::new()
                                        .title(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "prev",
                                            "embed_title",
                                        ))
                                        .description(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "unknown",
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

                        return Err(format!("can't skip the current music: {}", e));
                    }
                };

                let Some(music) = music else {
                    if let Err(e) = interaction
                        .edit_response(
                            &context.http,
                            EditInteractionResponse::new().embed(
                                CreateEmbed::new()
                                    .title(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "prev",
                                        "embed_title",
                                    ))
                                    .description(format!(
                                        "{}\n\n{}",
                                        hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "empty_queue",
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

                    return Ok(());
                };

                if let Err(e) = interaction
                    .edit_response(
                        &context.http,
                        EditInteractionResponse::new().embed(
                            CreateEmbed::new()
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "prev",
                                    "embed_title",
                                ))
                                .description(Self::get_message(music, &hydrogen, &interaction))
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
                if let Err(e) = interaction
                    .edit_response(
                        &context.http,
                        EditInteractionResponse::new().embed(
                            CreateEmbed::new()
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "prev",
                                    "embed_title",
                                ))
                                .description(format!(
                                    "{}\n\n{}",
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "error",
                                        "not_in_voice_chat",
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
        } else {
            if let Err(e) = interaction
                .edit_response(
                    &context.http,
                    EditInteractionResponse::new().embed(
                        CreateEmbed::new()
                            .title(hydrogen.i18n.translate(
                                &interaction.locale,
                                "prev",
                                "embed_title",
                            ))
                            .description(format!(
                                "{}\n\n{}",
                                hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "error",
                                    "player_not_exists",
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

        Ok(())
    }
}

#[async_trait]
impl HydrogenComponentListener for PrevComponent {
    async fn execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ComponentInteraction,
    ) {
        if let Err(e) = self._execute(hydrogen, context, interaction).await {
            warn!("{}", e);
        }
    }
}
