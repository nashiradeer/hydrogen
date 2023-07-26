use async_trait::async_trait;
use serenity::{
    model::prelude::{message_component::MessageComponentInteraction, ChannelId, Guild, UserId},
    prelude::Context,
};
use tracing::warn;

use crate::{
    player::HydrogenMusic, HydrogenComponentListener, HydrogenContext, HYDROGEN_ERROR_COLOR,
    HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

pub struct SkipComponent;

impl SkipComponent {
    #[inline]
    fn get_channel_id(guild: Guild, user_id: UserId) -> Result<ChannelId, Result<(), String>> {
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
        interaction: &'a MessageComponentInteraction,
    ) -> String {
        if let Some(uri) = track.uri {
            return hydrogen
                .i18n
                .translate(&interaction.locale, "skip", "success_uri")
                .replace("${music}", &track.title)
                .replace("${author}", &track.author)
                .replace("${uri}", &uri);
        } else {
            return hydrogen
                .i18n
                .translate(&interaction.locale, "skip", "success")
                .replace("${music}", &track.title)
                .replace("${author}", &track.author);
        }
    }

    async fn _execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: MessageComponentInteraction,
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
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "skip",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "skip",
                                    "unknown_voice_state",
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(|footer| {
                                    footer
                                        .text(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "embed",
                                            "footer_text",
                                        ))
                                        .icon_url(HYDROGEN_LOGO_URL)
                                })
                        })
                    })
                    .await
                {
                    warn!("can't response to interaction: {:?}", e);
                }
                return e;
            }
        };

        if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
            if my_channel_id == voice_channel_id.into() {
                let music = match manager.skip(guild_id).await {
                    Ok(v) => v,
                    Err(e) => {
                        if let Err(e) = interaction
                            .edit_original_interaction_response(&context.http, |response| {
                                response.embed(|embed| {
                                    embed
                                        .title(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "skip",
                                            "embed_title",
                                        ))
                                        .description(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "skip",
                                            "player_not_exists",
                                        ))
                                        .color(HYDROGEN_ERROR_COLOR)
                                        .footer(|footer| {
                                            footer
                                                .text(hydrogen.i18n.translate(
                                                    &interaction.locale,
                                                    "embed",
                                                    "footer_text",
                                                ))
                                                .icon_url(HYDROGEN_LOGO_URL)
                                        })
                                })
                            })
                            .await
                        {
                            warn!("can't response to interaction: {:?}", e);
                        }

                        return Err(format!("can't skip the current music: {}", e));
                    }
                };

                let Some(music) = music else {
                    if let Err(e) = interaction
                        .edit_original_interaction_response(&context.http, |response| {
                            response.embed(|embed| {
                                embed
                                    .title(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "skip",
                                        "embed_title",
                                    ))
                                    .description(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "skip",
                                        "empty_queue",
                                    ))
                                    .color(HYDROGEN_ERROR_COLOR)
                                    .footer(|footer| {
                                        footer
                                            .text(hydrogen.i18n.translate(
                                                &interaction.locale,
                                                "embed",
                                                "footer_text",
                                            ))
                                            .icon_url(HYDROGEN_LOGO_URL)
                                    })
                            })
                        })
                        .await
                    {
                        warn!("can't response to interaction: {:?}", e);
                    }

                    return Ok(());
                };

                if let Err(e) = interaction
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "skip",
                                    "embed_title",
                                ))
                                .description(Self::get_message(music, &hydrogen, &interaction))
                                .color(HYDROGEN_PRIMARY_COLOR)
                                .footer(|footer| {
                                    footer
                                        .text(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "embed",
                                            "footer_text",
                                        ))
                                        .icon_url(HYDROGEN_LOGO_URL)
                                })
                        })
                    })
                    .await
                {
                    warn!("can't response to interaction: {:?}", e);
                }
            } else {
                if let Err(e) = interaction
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "skip",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "skip",
                                    "not_same_voice_chat",
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(|footer| {
                                    footer
                                        .text(hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "embed",
                                            "footer_text",
                                        ))
                                        .icon_url(HYDROGEN_LOGO_URL)
                                })
                        })
                    })
                    .await
                {
                    warn!("can't response to interaction: {:?}", e);
                }
            }
        } else {
            if let Err(e) = interaction
                .edit_original_interaction_response(&context.http, |response| {
                    response.embed(|embed| {
                        embed
                            .title(hydrogen.i18n.translate(
                                &interaction.locale,
                                "skip",
                                "embed_title",
                            ))
                            .description(hydrogen.i18n.translate(
                                &interaction.locale,
                                "skip",
                                "player_not_exists",
                            ))
                            .color(HYDROGEN_ERROR_COLOR)
                            .footer(|footer| {
                                footer
                                    .text(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "embed",
                                        "footer_text",
                                    ))
                                    .icon_url(HYDROGEN_LOGO_URL)
                            })
                    })
                })
                .await
            {
                warn!("can't response to interaction: {:?}", e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl HydrogenComponentListener for SkipComponent {
    async fn execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: MessageComponentInteraction,
    ) {
        if let Err(e) = self._execute(hydrogen, context, interaction).await {
            warn!("{}", e);
        }
    }
}
