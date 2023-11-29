use async_trait::async_trait;
use serenity::{
    all::{ChannelId, ComponentInteraction, Guild, GuildId, UserId},
    builder::{CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    cache::CacheRef,
    client::Context,
};
use tracing::warn;

use crate::{
    HydrogenComponentListener, HydrogenContext, HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL,
    HYDROGEN_PRIMARY_COLOR,
};

pub struct StopComponent;

impl StopComponent {
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
                                    "stop",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "stop",
                                    "unknown_voice_state",
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(
                                    CreateEmbedFooter::new(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "embed",
                                        "footer_text",
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
                manager
                    .destroy(guild_id)
                    .await
                    .map_err(|e| format!("destroy error: {}", e))?;

                if let Err(e) = interaction
                    .edit_response(
                        &context.http,
                        EditInteractionResponse::new().embed(
                            CreateEmbed::new()
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "stop",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "stop",
                                    "success",
                                ))
                                .color(HYDROGEN_PRIMARY_COLOR)
                                .footer(
                                    CreateEmbedFooter::new(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "embed",
                                        "footer_text",
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
                                    "stop",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "stop",
                                    "not_same_voice_chat",
                                ))
                                .color(HYDROGEN_ERROR_COLOR)
                                .footer(
                                    CreateEmbedFooter::new(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "embed",
                                        "footer_text",
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
                                "stop",
                                "embed_title",
                            ))
                            .description(hydrogen.i18n.translate(
                                &interaction.locale,
                                "stop",
                                "player_not_exists",
                            ))
                            .color(HYDROGEN_ERROR_COLOR)
                            .footer(
                                CreateEmbedFooter::new(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "embed",
                                    "footer_text",
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
impl HydrogenComponentListener for StopComponent {
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
