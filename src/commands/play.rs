//! Hydrogen // Commands // Play
//!
//! '/play' command registration and execution.

use hydrogen_i18n::I18n;
use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use tracing::{error, warn};

use crate::{
    handler::{Response, Result},
    player::HydrogenPlayCommand,
    utils::{error_message, get_str_option, MusicCommonData},
    HydrogenContext, HYDROGEN_BUG_URL,
};

/// Executes the `/play` command.
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
    let Some(query) = get_str_option(interaction, 0) else {
        warn!("cannot get the 'query' option");

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

    // Try to get the voice connection, or join the channel if it doesn't exist.
    let call = match data.voice_manager.get(data.guild_id) {
        Some(v) => {
            // Check if has connection and drop the lock.
            let has_connection = v.lock().await.current_connection().is_some();

            if !has_connection {
                // Join the voice channel.
                match data
                    .voice_manager
                    .join_gateway(data.guild_id, voice_channel_id)
                    .await
                {
                    Ok(e) => e.1,
                    Err(e) => {
                        warn!(
                            "cannot connect to the voice channel in the guild {}: {}",
                            data.guild_id, e
                        );

                        return Err(Response::Generic {
                            title,
                            description: error_message(
                                &hydrogen.i18n,
                                &interaction.locale,
                                &hydrogen.i18n.translate(
                                    &interaction.locale,
                                    "error",
                                    "cant_connect",
                                ),
                            ),
                        });
                    }
                }
            } else {
                v
            }
        }
        None => {
            // Join the voice channel.
            match data
                .voice_manager
                .join_gateway(data.guild_id, voice_channel_id)
                .await
            {
                Ok(e) => e.1,
                Err(e) => {
                    warn!(
                        "cannot connect to the voice channel in the guild {}: {}",
                        data.guild_id, e
                    );

                    return Err(Response::Generic {
                        title,
                        description: error_message(
                            &hydrogen.i18n,
                            &interaction.locale,
                            &hydrogen
                                .i18n
                                .translate(&interaction.locale, "error", "cant_connect"),
                        ),
                    });
                }
            }
        }
    };

    // Fetch the connection info.
    if let Some(connection_info) = call.lock().await.current_connection() {
        if let Some(channel_id) = connection_info.channel_id {
            if channel_id != voice_channel_id.into() {
                return Err(Response::Generic {
                    title,
                    description: error_message(
                        &hydrogen.i18n,
                        &interaction.locale,
                        &hydrogen
                            .i18n
                            .translate(&interaction.locale, "error", "player_exists"),
                    ),
                });
            }
        }
    }

    // Initialize the player or enqueue/play the music.
    let result = match data
        .manager
        .init_or_play(
            data.guild_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
            &query,
            interaction.user.id,
            data.voice_manager.clone(),
            interaction.channel_id,
        )
        .await
    {
        Ok(e) => e,
        Err(e) => {
            warn!("cannot play the music: {}", e);

            return Err(Response::Generic {
                title,
                description: error_message(
                    &hydrogen.i18n,
                    &interaction.locale,
                    &hydrogen
                        .i18n
                        .translate(&interaction.locale, "error", "unknown")
                        .replace("{url}", HYDROGEN_BUG_URL),
                ),
            });
        }
    };

    if result.count > 0 {
        // Success.
        Ok(Response::Generic {
            title,
            description: get_message(result, &hydrogen, &interaction),
        })
    } else {
        // Error.
        if !result.truncated {
            // The music was not found.
            Err(Response::Generic {
                title,
                description: error_message(
                    &hydrogen.i18n,
                    &interaction.locale,
                    &hydrogen
                        .i18n
                        .translate(&interaction.locale, "play", "not_found"),
                ),
            })
        } else {
            // The queue is full.
            Err(Response::Generic {
                title,
                description: error_message(
                    &hydrogen.i18n,
                    &interaction.locale,
                    &hydrogen
                        .i18n
                        .translate(&interaction.locale, "play", "truncated"),
                ),
            })
        }
    }
}

/// Registers the `/play` command.
///
/// If `i18n` is `None`, the translation will be ignored.
pub fn register(i18n: Option<&I18n>) -> CreateCommand {
    let mut command = CreateCommand::new("play");

    if let Some(i18n) = i18n {
        command = i18n.serenity_command_name("play", "name", command);
        command = i18n.serenity_command_description("play", "description", command);
    }

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

                if let Some(i18n) = i18n {
                    option =
                        i18n.serenity_command_option_name("play", "query_name", option);
                    option = i18n.serenity_command_option_description(
                        "play",
                        "query_description",
                        option,
                    );
                }

                option
            })
            .dm_permission(false)
}

/// Get the message to send to the user.
fn get_message(
    result: HydrogenPlayCommand,
    hydrogen: &HydrogenContext,
    interaction: &CommandInteraction,
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
