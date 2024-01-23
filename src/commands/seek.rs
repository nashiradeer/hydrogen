//! Hydrogen // Commands // Seek
//!
//! '/seek' command registration and execution.

use hydrogen_i18n::I18n;
use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use tracing::{error, warn};

use crate::{
    handler::{Response, Result},
    utils::{error_message, get_str_option, progress_bar, time_to_string, MusicCommonData},
    HydrogenContext, HYDROGEN_BUG_URL,
};

/// Executes the `/seek` command.
pub async fn execute(
    hydrogen: &HydrogenContext,
    context: &Context,
    interaction: &CommandInteraction,
) -> Result {
    // Get the title of the embed.
    let title = hydrogen
        .i18n
        .translate(&interaction.locale, "seek", "embed_title");

    // Get the time option value.
    let Some(time) = get_str_option(interaction, 0) else {
        warn!("cannot get the 'time' option");

        return Err(Response::Generic {
            title,
            description: hydrogen
                .i18n
                .translate(&interaction.locale, "error", "unknown")
                .replace("{url}", HYDROGEN_BUG_URL),
        });
    };

    // Get the common data used by music commands and components.
    let Some(data) = MusicCommonData::new(&hydrogen, &context, &interaction).await else {
        error!("cannot get common music data");

        return Err(Response::Generic {
            title,
            description: hydrogen
                .i18n
                .translate(&interaction.locale, "error", "unknown")
                .replace("{url}", HYDROGEN_BUG_URL),
        });
    };

    // Get the user's voice channel ID.
    let Some(voice_channel_id) = data.get_connected_channel(interaction.user.id) else {
        warn!(
            "cannot get the voice channel ID of the user {} in the guild {}",
            interaction.user.id, data.guild_id
        );

        return Err(Response::Generic {
            title,
            description: error_message(
                &hydrogen.i18n,
                &interaction.locale,
                &hydrogen
                    .i18n
                    .translate(&interaction.locale, "error", "unknown_voice_state")
                    .replace("{url}", HYDROGEN_BUG_URL),
            ),
        });
    };

    // Get the player's voice channel ID.
    if let Some(my_channel_id) = data.manager.get_voice_channel_id(data.guild_id).await {
        // Checks if the user is in the same voice channel as the bot.
        if my_channel_id == voice_channel_id.into() {
            // Try to parse the suffix syntax.
            let seek_time = match hydrogen.time_parsers.suffix_syntax(time) {
                Some(v) => v,
                // Try to parse the semicolon syntax.
                None => match hydrogen.time_parsers.semicolon_syntax(time) {
                    Some(v) => v,
                    None => {
                        warn!("cannot parse the time syntax: {}", time);

                        return Err(Response::Generic {
                            title,
                            description: error_message(
                                &hydrogen.i18n,
                                &interaction.locale,
                                &hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "error",
                                    "invalid_syntax",
                                ),
                            ),
                        });
                    }
                },
            };

            // Convert the seek time to a i32 for the player.
            // TODO: Remove this when the player supports u32.
            let converted_seek_time = match seek_time.try_into() {
                Ok(v) => v,
                Err(e) => {
                    error!("cannot convert the seek time to a i32: {}", e);

                    return Err(Response::Generic {
                        title,
                        description: hydrogen
                            .i18n
                            .translate(&interaction.locale, "error", "unknown")
                            .replace("{url}", HYDROGEN_BUG_URL),
                    });
                }
            };

            // Seek the player.
            let seek_result = match data.manager.seek(data.guild_id, converted_seek_time).await {
                Ok(Some(v)) => v,
                Ok(None) => {
                    // The queue is empty.
                    warn!("guild {} has a empty queue", data.guild_id);

                    return Err(Response::Generic {
                        title,
                        description: error_message(
                            &hydrogen.i18n,
                            &interaction.locale,
                            &hydrogen
                                .i18n
                                .translate(&interaction.locale, "error", "empty_queue"),
                        ),
                    });
                }
                Err(e) => {
                    // An error occurred.
                    error!(
                        "cannot seek time the player in the guild {}: {}",
                        data.guild_id, e
                    );

                    return Err(Response::Generic {
                        title,
                        description: hydrogen
                            .i18n
                            .translate(&interaction.locale, "error", "unknown")
                            .replace("{url}", HYDROGEN_BUG_URL),
                    });
                }
            };

            // Get the current time, total time and progress bar.
            let current_time = time_to_string(seek_result.position / 1000);
            let total_time = time_to_string(seek_result.total / 1000);
            let progress_bar = progress_bar(seek_result.position, seek_result.total);

            // Get the translation message.
            let translation_message = if let Some(uri) = seek_result.track.uri {
                hydrogen
                    .i18n
                    .translate(&interaction.locale, "seek", "seeking_url")
                    .replace("{name}", &seek_result.track.title)
                    .replace("{author}", &seek_result.track.author)
                    .replace("{url}", &uri)
                    .replace("{current}", &current_time)
                    .replace("{total}", &total_time)
                    .replace("{progress}", &progress_bar)
            } else {
                hydrogen
                    .i18n
                    .translate(&interaction.locale, "seek", "seeking")
                    .replace("{name}", &seek_result.track.title)
                    .replace("{author}", &seek_result.track.author)
                    .replace("{current}", &current_time)
                    .replace("{total}", &total_time)
                    .replace("{progress}", &progress_bar)
            };

            Ok(Response::Generic {
                title,
                description: translation_message,
            })
        } else {
            // The user is not in the same voice channel as the bot.
            Err(Response::Generic {
                title,
                description: error_message(
                    &hydrogen.i18n,
                    &interaction.locale,
                    &hydrogen
                        .i18n
                        .translate(&interaction.locale, "error", "not_in_voice_chat"),
                ),
            })
        }
    } else {
        // The player doesn't exists.
        Err(Response::Generic {
            title,
            description: error_message(
                &hydrogen.i18n,
                &interaction.locale,
                &hydrogen
                    .i18n
                    .translate(&interaction.locale, "error", "player_not_exists"),
            ),
        })
    }
}

/// Registers the `/seek` command.
///
/// If `i18n` is `None`, the translation will be ignored.
pub fn register(i18n: Option<&I18n>) -> CreateCommand {
    let mut command = CreateCommand::new("seek");

    if let Some(i18n) = i18n {
        command = i18n.serenity_command_name("seek", "name", command);
        command = i18n.serenity_command_description("seek", "description", command);
    }

    command
        .description("Seek for the time in the current music playing.")
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "time",
                "Time in seconds or a supported syntax.",
            )
            .required(true);

            if let Some(i18n) = i18n {
                option = i18n.serenity_command_option_name("seek", "time_name", option);
                option =
                    i18n.serenity_command_option_description("seek", "time_description", option);
            }

            option
        })
        .dm_permission(false)
}
