//! Hydrogen // Utils
//!
//! Utility functions for Hydrogen's commands and components.

use std::sync::Arc;

use hydrogen_i18n::I18n;
use serenity::{
    all::{ChannelId, CommandInteraction, Guild, GuildId, UserId},
    client::Context,
};
use songbird::Songbird;
use tracing::{error, warn};

use crate::{manager::HydrogenManager, HydrogenContext, HYDROGEN_BUG_URL};

/// Common data used by music commands and components.
pub struct MusicCommonData {
    /// Hydrogen's manager.
    pub manager: HydrogenManager,

    /// Songbird's voice manager.
    pub voice_manager: Arc<Songbird>,

    /// Guild.
    pub guild: Guild,

    /// Guild's ID.
    pub guild_id: GuildId,
}

impl MusicCommonData {
    /// Creates a new instance of `MusicCommonData`.
    pub async fn new(
        hydrogen: &HydrogenContext,
        context: &Context,
        guild_id: Option<GuildId>,
    ) -> Option<Self> {
        let Some(manager) = hydrogen.manager.read().await.clone() else {
            error!("cannot get the manager");
            return None;
        };

        let Some(voice_manager) = songbird::get(context).await else {
            error!("cannot get the Songbird's voice manager");
            return None;
        };

        let Some(guild_id) = guild_id else {
            warn!("cannot get the guild ID");
            return None;
        };
        let Some(guild) = context.cache.guild(guild_id) else {
            warn!("cannot get the guild {} from the cache", guild_id);
            return None;
        };

        Some(Self {
            manager,
            voice_manager,
            guild_id,
            // Guild needs to be cloned because it's not `Send`.
            guild: guild.clone(),
        })
    }

    /// Gets the voice channel ID of the user.
    pub fn get_connected_channel(&self, user_id: UserId) -> Option<ChannelId> {
        self.guild.voice_states.get(&user_id)?.channel_id
    }
}

/// Creates an error embed's description.
pub fn error_message(i18n: &I18n, locale: &str, error: &str) -> String {
    format!(
        "{}\n\n{}",
        error,
        i18n.translate(locale, "error", "not_intentional",)
            .replace("{url}", HYDROGEN_BUG_URL)
    )
}

/// Gets a string option from a command.
pub fn get_str_option(command: &CommandInteraction, index: usize) -> Option<&str> {
    command.data.options.get(index)?.value.as_str()
}

/// Converts a time in seconds to a string.
pub fn time_to_string(seconds: i32) -> String {
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
    format!(
        "{:02}:{:02}:{:02}",
        hours as u32, minutes as u32, seconds as u32
    )
}

/// Creates a progress bar.
pub fn progress_bar(current: i32, total: i32) -> String {
    let item_total = 30usize;
    let item_count = (current as f32 / (total as f32 / item_total as f32)).round();
    let bar = "▓".repeat(item_count as usize);
    format!("╣{:░<width$.width$}╠", bar, width = item_total)
}
