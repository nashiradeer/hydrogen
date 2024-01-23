//! Hydrogen // Components // Pause
//!
//! 'pause' component execution.

use serenity::{all::ComponentInteraction, client::Context};
use tracing::{error, warn};

use crate::{
    handler::{Response, Result},
    utils::{error_message, MusicCommonData},
    HydrogenContext, HYDROGEN_BUG_URL,
};

/// Executes the `pause` command.
pub async fn execute(
    hydrogen: &HydrogenContext,
    context: &Context,
    interaction: &ComponentInteraction,
) -> Result {
    // Get the translation for the command's title.
    let title = hydrogen
        .i18n
        .translate(&interaction.locale, "pause", "embed_title");

    // Get the common data used by music commands and components.
    let Some(data) = MusicCommonData::new(&hydrogen, &context, interaction.guild_id).await else {
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
                    .translate(&interaction.locale, "error", "unknown_voice_state"),
            ),
        });
    };

    // Get the voice channel ID of the bot.
    if let Some(my_channel_id) = data.manager.get_voice_channel_id(data.guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            // Get the pause state and invert it.
            let paused = !data.manager.get_paused(data.guild_id).await;

            // Pause or resume the player.
            if let Err(e) = data.manager.set_paused(data.guild_id, paused).await {
                warn!(
                    "cannot resume/pause the player in the guild {}: {}",
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

            // Get the translation key.
            let translation_key = if paused { "paused" } else { "resumed" };

            Ok(Response::Generic {
                title,
                description: hydrogen
                    .i18n
                    .translate(&interaction.locale, "pause", translation_key),
            })
        } else {
            // Not in the same voice channel as the bot.
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
        // Player doesn't exist.
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
