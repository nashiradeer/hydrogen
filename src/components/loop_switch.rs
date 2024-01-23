//! Hydrogen // Components // Loop
//!
//! 'loop' component execution.

use serenity::{all::ComponentInteraction, client::Context};
use tracing::{error, warn};

use crate::{
    handler::{Response, Result},
    player::LoopType,
    utils::{error_message, MusicCommonData},
    HydrogenContext, HYDROGEN_BUG_URL,
};

/// Executes the `loop` command.
pub async fn execute(
    hydrogen: &HydrogenContext,
    context: &Context,
    interaction: &ComponentInteraction,
) -> Result {
    // Get the translation for the command's title.
    let title = hydrogen
        .i18n
        .translate(&interaction.locale, "loop", "embed_title");

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
                    .translate(&interaction.locale, "error", "unknown_voice_state")
                    .replace("{url}", HYDROGEN_BUG_URL),
            ),
        });
    };

    // Get the voice channel ID of the bot.
    if let Some(my_channel_id) = data.manager.get_voice_channel_id(data.guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            // Get the current loop type.
            let current_loop_type = data.manager.get_loop_type(data.guild_id).await;

            // Cycle through the loop types.
            let new_loop_type = match current_loop_type {
                LoopType::None => LoopType::NoAutostart,
                LoopType::NoAutostart => LoopType::Music,
                LoopType::Music => LoopType::Queue,
                LoopType::Queue => LoopType::Random,
                LoopType::Random => LoopType::None,
            };

            // Set the new loop type.
            data.manager
                .set_loop_type(data.guild_id, new_loop_type.clone())
                .await;

            // Get the translation key for the new loop type.
            let loop_type_translation_key = match new_loop_type {
                LoopType::None => "autostart",
                LoopType::NoAutostart => "no_autostart",
                LoopType::Music => "music",
                LoopType::Queue => "queue",
                LoopType::Random => "random",
            };

            // Get the translation for the new loop type.
            let loop_type_translation =
                hydrogen
                    .i18n
                    .translate(&interaction.locale, "loop", loop_type_translation_key);

            Ok(Response::Generic {
                title,
                description: hydrogen
                    .i18n
                    .translate(&interaction.locale, "loop", "looping")
                    .replace("{loop}", &loop_type_translation),
            })
        } else {
            // User isn't in the same voice channel as the bot.
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
