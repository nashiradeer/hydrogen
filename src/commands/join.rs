use std::sync::Arc;

use async_trait::async_trait;
use serenity::{
    all::{ChannelId, CommandInteraction, Guild, GuildId, UserId},
    builder::{CreateCommand, CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    client::Context,
};
use songbird::{Call, Songbird};
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    i18n::HydrogenI18n, HydrogenCommandListener, HydrogenContext, HYDROGEN_BUG_URL,
    HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
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
                                        "join",
                                        "embed_title",
                                    ))
                                    .description(format!(
                                        "{}\n\n{}",
                                        hydrogen.i18n.translate(
                                            &interaction.locale,
                                            "error",
                                            "cant_connect",
                                        ),
                                        hydrogen
                                            .i18n
                                            .translate(
                                                &interaction.locale,
                                                "error",
                                                "not_intentional",
                                            )
                                            .replace("{url}", HYDROGEN_BUG_URL)
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

    async fn _execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: CommandInteraction,
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
            .ok_or("guild isn't present in the cache".to_owned())?
            .clone();

        interaction
            .defer_ephemeral(&context.http)
            .await
            .map_err(|e| format!("can't defer the response: {}", e))?;

        if manager.contains_player(guild_id).await {
            if let Err(e) = interaction
                .edit_response(
                    &context.http,
                    EditInteractionResponse::new().embed(
                        CreateEmbed::new()
                            .title(hydrogen.i18n.translate(
                                &interaction.locale,
                                "join",
                                "embed_title",
                            ))
                            .description(format!(
                                "{}\n\n{}",
                                hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "error",
                                    "player_exists",
                                ),
                                hydrogen
                                    .i18n
                                    .translate(&interaction.locale, "error", "not_intentional",)
                                    .replace("{url}", HYDROGEN_BUG_URL)
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
                                    "join",
                                    "embed_title",
                                ))
                                .description(format!(
                                    "{}\n\n{}",
                                    hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "error",
                                        "unknown_voice_state",
                                    ),
                                    hydrogen
                                        .i18n
                                        .translate(&interaction.locale, "error", "not_intentional",)
                                        .replace("{url}", HYDROGEN_BUG_URL)
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
            .edit_response(
                &context.http,
                EditInteractionResponse::new().embed(
                    CreateEmbed::new()
                        .title(
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "join", "embed_title"),
                        )
                        .description(
                            hydrogen
                                .i18n
                                .translate(&interaction.locale, "join", "joined"),
                        )
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

        Ok(())
    }
}

#[async_trait]
impl HydrogenCommandListener for JoinCommand {
    fn register(&self, i18n: HydrogenI18n) -> CreateCommand {
        let mut command = CreateCommand::new("join");

        command = i18n.translate_application_command_name("join", "name", command);
        command = i18n.translate_application_command_description("join", "description", command);

        command
            .description("Make me join your voice chat without playing anything.")
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
