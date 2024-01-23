//! Hydrogen // Commands // Join
//!
//! '/join' command registration and execution.

use hydrogen_i18n::I18n;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};
use tracing::{error, warn};

use crate::{
    handler::{Response, Result},
    utils::{error_message, MusicCommonData},
    HydrogenContext, HYDROGEN_BUG_URL,
};

/// Executes the `/join` command.
pub async fn execute(
    hydrogen: &HydrogenContext,
    context: &Context,
    interaction: &CommandInteraction,
) -> Result {
    // Get the translation for the command's title.
    let title = hydrogen
        .i18n
        .translate(&interaction.locale, "join", "embed_title");

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

    // Check if a player already exists.
    if data.manager.contains_player(data.guild_id).await {
        warn!("a player already exists in the guild {}", data.guild_id);

        return Err(Response::Generic {
            title,
            description: error_message(
                &hydrogen.i18n,
                &interaction.locale,
                &hydrogen
                    .i18n
                    .translate(&interaction.locale, "error", "player_exists")
                    .replace("{url}", HYDROGEN_BUG_URL),
            ),
        });
    }

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

    // Join the voice channel.
    if let Err(e) = data
        .voice_manager
        .join_gateway(data.guild_id, voice_channel_id)
        .await
    {
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

    // Initialize the player.
    if let Err(e) = data
        .manager
        .init(
            data.guild_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
            data.voice_manager.clone(),
            interaction.channel_id,
        )
        .await
    {
        error!(
            "cannot initialize the player in the guild {}: {}",
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

    // Get play command's mention.
    let play_command = match hydrogen.commands_id.read().await.get("play") {
        Some(v) => format!("</play:{}>", v.get()),
        None => "`/play`".to_owned(),
    };

    Ok(Response::Generic {
        title,
        description: hydrogen
            .i18n
            .translate(&interaction.locale, "join", "joined")
            .replace("{play}", &play_command),
    })
}

/// Registers the `/join` command.
///
/// If `i18n` is `None`, the translation will be ignored.
pub fn register(i18n: Option<&I18n>) -> CreateCommand {
    let mut command = CreateCommand::new("join");

    if let Some(i18n) = i18n {
        command = i18n.serenity_command_name("join", "name", command);
        command = i18n.serenity_command_description("join", "description", command);
    }

    command
        .description("Make me join your voice chat without playing anything.")
        .dm_permission(false)
}
