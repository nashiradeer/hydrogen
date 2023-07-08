use std::sync::Arc;

use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, ChannelId, Guild, GuildId, UserId,
    },
    prelude::Context,
};
use songbird::{Call, Songbird};
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    i18n::HydrogenI18n, HydrogenCommandListener, HydrogenContext, HYDROGEN_ERROR_COLOR,
    HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

pub struct JoinCommand;

impl JoinCommand {
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
    async fn join_channel<'a>(
        hydrogen: &'a HydrogenContext,
        context: &'a Context,
        interaction: &'a ApplicationCommandInteraction,
        voice_manager: &'a Arc<Songbird>,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<Arc<Mutex<Call>>, String> {
        let voice = voice_manager.join_gateway(guild_id, channel_id).await;
        Ok(match voice.1 {
            Ok(_) => voice.0,
            Err(e) => {
                if let Err(e) = interaction
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "join",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "join",
                                    "cant_connect",
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

                return Err(format!("can't connect to voice chat: {}", e));
            }
        })
    }

    async fn _execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ApplicationCommandInteraction,
    ) -> Result<(), String> {
        let manager = hydrogen
            .manager
            .read()
            .await
            .clone()
            .ok_or("manager not initialized".to_owned())?;
        let voice_manager = songbird::get(&context)
            .await
            .ok_or("songbird not initialized".to_owned())?;
        let guild_id = interaction
            .guild_id
            .ok_or("interaction doesn't have a guild_id".to_owned())?;
        let guild = context
            .cache
            .guild(guild_id)
            .ok_or("guild isn't present in the cache".to_owned())?;

        if manager.contains_player(guild_id).await {
            if let Err(e) = interaction
                .create_interaction_response(&context.http, |response| {
                    response.interaction_response_data(|data| {
                        data.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "join",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "join",
                                    "player_exists",
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
                })
                .await
            {
                warn!("can't response to interaction: {:?}", e);
            }
        }

        interaction
            .defer_ephemeral(&context.http)
            .await
            .map_err(|e| format!("can't defer the response: {}", e))?;

        let voice_channel_id = match Self::get_channel_id(guild, interaction.user.id) {
            Ok(v) => v,
            Err(e) => {
                if let Err(e) = interaction
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "join",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "join",
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

        Self::join_channel(
            &hydrogen,
            &context,
            &interaction,
            &voice_manager,
            guild_id,
            voice_channel_id,
        )
        .await?;

        manager
            .init(
                guild_id,
                &interaction
                    .guild_locale
                    .clone()
                    .unwrap_or(interaction.locale.clone()),
                voice_manager.clone(),
                interaction.channel_id,
            )
            .await
            .map_err(|e| e.to_string())?;

        if let Err(e) = interaction
            .edit_original_interaction_response(&context.http, |response| {
                response.embed(|embed| {
                    embed
                        .title(
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "join", "embed_title"),
                        )
                        .description(hydrogen.i18n.translate(
                            &interaction.locale,
                            "join",
                            "success",
                        ))
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

        Ok(())
    }
}

#[async_trait]
impl HydrogenCommandListener for JoinCommand {
    fn register<'a, 'b>(
        &'a self,
        i18n: HydrogenI18n,
        command: &'b mut CreateApplicationCommand,
    ) -> &'b mut CreateApplicationCommand {
        i18n.translate_application_command_name("join", "name", command);
        i18n.translate_application_command_description("join", "description", command);

        command
            .description("Connects me to your voice chat by starting a music player without playing anything.")
            .dm_permission(false)
    }

    async fn execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ApplicationCommandInteraction,
    ) {
        if let Err(e) = self._execute(hydrogen, context, interaction).await {
            warn!("{}", e);
        }
    }
}
