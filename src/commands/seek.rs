use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType, ChannelId,
        Guild, UserId,
    },
    prelude::Context,
};
use tracing::warn;

use crate::{
    i18n::HydrogenI18n, HydrogenCommandListener, HydrogenContext, HYDROGEN_ERROR_COLOR,
    HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

pub struct SeekCommand;

impl SeekCommand {
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
    fn parse_time(time: &str) -> Result<i32, String> {
        if let Some(time) = time.strip_suffix("m") {
            let minutes = time
                .parse::<u16>()
                .map_err(|_| "minute syntax detected but with a weird number".to_owned())?
                * 60
                * 1000;
            return Ok(minutes.into());
        }

        if let Some(time) = time.strip_suffix("h") {
            let hours = time
                .parse::<u16>()
                .map_err(|_| "hour syntax detected but with a weird number".to_owned())?
                * 60
                * 60
                * 1000;
            return Ok(hours.into());
        }

        let components: Result<Vec<_>, _> = time
            .split(":")
            .map(|i| {
                i.parse::<u16>()
                    .map_err(|_| "minute syntax detected but with a weird number".to_owned())
            })
            .collect();

        let components = components?;

        if components.len() == 0 {
            return Err("not detected any time syntax".to_owned());
        } else if components.len() == 1 {
            let seconds = components[0] * 1000;
            return Ok(seconds.into());
        } else if components.len() == 2 {
            let mut seconds = components[1] * 1000;
            seconds += components[0] * 60 * 1000;
            return Ok(seconds.into());
        }

        let mut seconds = components[2] * 1000;
        seconds += components[1] * 60 * 1000;
        seconds += components[0] * 60 * 60 * 1000;
        Ok(seconds.into())
    }

    fn time_to_string(seconds: i32) -> String {
        if seconds < 60 {
            return format!("00:{:02}", seconds);
        } else if seconds < 60 * 60 {
            let time = seconds as f32;
            let minutes = (time / 60.0).floor();
            let seconds = time - minutes * 60.0;
            return format!("{:02}:{:02}", minutes as u32, seconds as u32);
        }

        let time = seconds as f32;
        let hours = (time / 60.0 / 60.0).floor();
        let minutes = (time - hours * 60.0 * 60.0).floor();
        let seconds = time - minutes * 60.0 - hours * 60.0 * 60.0;
        return format!(
            "{:02}:{:02}:{:02}",
            hours as u32, minutes as u32, seconds as u32
        );
    }

    async fn _execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ApplicationCommandInteraction,
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
                                    "seek",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "seek",
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
                let paused = !manager.get_paused(guild_id).await;

                if let Err(e) = manager.set_paused(guild_id, paused).await {
                    if let Err(e) = interaction
                        .edit_original_interaction_response(&context.http, |response| {
                            response.embed(|embed| {
                                embed
                                    .title(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "seek",
                                        "embed_title",
                                    ))
                                    .description(hydrogen.i18n.translate(
                                        &interaction.locale,
                                        "seek",
                                        "cant_pause",
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

                    return Err(format!("can't resume/pause the player: {}", e));
                }

                let mut translation_key = "resumed";

                if paused {
                    translation_key = "paused";
                }

                if let Err(e) = interaction
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "seek",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "seek",
                                    translation_key,
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
            } else {
                if let Err(e) = interaction
                    .edit_original_interaction_response(&context.http, |response| {
                        response.embed(|embed| {
                            embed
                                .title(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "seek",
                                    "embed_title",
                                ))
                                .description(hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "seek",
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
                                "seek",
                                "embed_title",
                            ))
                            .description(hydrogen.i18n.translate(
                                &interaction.locale,
                                "seek",
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
impl HydrogenCommandListener for SeekCommand {
    fn register<'a, 'b>(
        &'a self,
        i18n: HydrogenI18n,
        command: &'b mut CreateApplicationCommand,
    ) -> &'b mut CreateApplicationCommand {
        i18n.translate_application_command_name("seek", "name", command);
        i18n.translate_application_command_description("seek", "description", command);

        command
            .description("Sets the current playback time of the song..")
            .create_option(|option| {
                i18n.translate_application_command_option_name("seek", "time_name", option);
                i18n.translate_application_command_option_description(
                    "seek",
                    "time_description",
                    option,
                );

                option
                    .kind(CommandOptionType::String)
                    .name("time")
                    .description("The time to be set on the player.")
                    .required(true)
            })
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
