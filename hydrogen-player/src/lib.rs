//! # Hydrogen // Player
//!
//! A standardized abstraction of different ways to play audio in Discord voice calls, facilitating the development of music bots while allowing the use of various audio systems (called `engines`), from the internal driver system of [`songbird`] to the client [`hydrolink`] for Lavalink.
//!
//! ## Features
//!
//! `lavalink` = Enables [`hydrolink`] and the engine [`engine::lavalink::Lavalink`].
use std::{
    fmt::{self, Display, Formatter},
    result,
};

use async_trait::async_trait;
pub use songbird::{
    error::JoinError,
    id::{ChannelId, GuildId, UserId},
};

pub mod engine;
pub mod utils;

/// Enum that groups all errors produced by this crate.
#[derive(Debug)]
pub enum Error {
    /// Errors produced by the Lavalink engine.
    #[cfg(feature = "lavalink")]
    #[cfg_attr(docsrs, doc(cfg(feature = "time")))]
    Lavalink(hydrolink::Error),

    /// Errors produced by [`songbird`] when connecting to the Discord Gateway.
    Join(JoinError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "lavalink")]
            Self::Lavalink(e) => e.fmt(f),

            Self::Join(e) => e.fmt(f),
        }
    }
}

/// Only [`result::Result`] with the type of [`Err`] set to [`Error`].
pub type Result<T> = result::Result<T, Error>;

/// A simple representation of the Discord Voice State.
#[derive(Clone)]
pub struct VoiceState {
    /// Channel ID associated with this voice state.
    pub channel_id: Option<ChannelId>,

    /// Session ID associated with this voice state.
    pub session_id: String,

    /// Token associated with this voice state.
    pub token: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Track {
    pub length: i32,
    pub requester_id: UserId,
    pub title: String,
    pub author: String,
    pub uri: Option<String>,
    pub thumbnail_uri: Option<String>,
}

#[derive(Clone)]
pub struct QueueAdd {
    pub track: Vec<Track>,
    pub offset: usize,
    pub truncated: bool,
}

#[derive(Clone)]
pub struct Seek {
    pub position: usize,
    pub total: usize,
    pub track: Track,
}

/// Trait that defines and standardizes a communication interface between [`crate::PlayerManager`] and whatever backend is implemented.
///
/// # Why are all methods asynchronous?
///
/// The motivation behind making all methods asynchronous by default comes from the possibility that some backend needs an external server, as in the case of Lavalink, and the possibility that this server can implement all the requirements for the proper functioning of the Hydrogen Player.
#[async_trait]
pub trait Player {
    /// This method should initiate the voice chat connection using [`songbird`] and initialize the player used by this backend, in addition to saving it in a [`std::collections::HashMap`].
    ///
    /// The reason the voice chat connection is not initiated by [`crate::PlayerManager`] is because different backends may use different things from [`songbird`], such as [`songbird::driver::Driver`] which is not required by all backends but may be required by some.
    async fn join(&self, guild_id: GuildId) -> Result<()>;

    /// The opposite of [`Backend::join()`], this method must destroy the player, freeing all resources related to it.
    async fn leave(&self, guild_id: GuildId) -> Result<()>;

    /// Gets the pause state of the player from a given guild.
    async fn pause(&self, guild_id: GuildId) -> Result<bool>;

    /// Gets the repeat music state of the player from a given guild.
    async fn repeat_music(&self, guild_id: GuildId) -> Result<bool>;

    /// Gets the random next state of the player from a given guild.
    async fn random_next(&self, guild_id: GuildId) -> Result<bool>;

    /// Gets the cyclic queue of the player from a given guild.
    async fn cyclic_queue(&self, guild_id: GuildId) -> Result<bool>;

    /// Gets the autoplay state of the player from a given guild.
    async fn autoplay(&self, guild_id: GuildId) -> Result<bool>;

    /// Sets the pause state of the player from a given guild.
    async fn set_pause(&self, guild_id: GuildId, pause: bool) -> Result<()>;

    /// Sets the repeat music state of the player from a given guild.
    async fn set_repeat_music(&self, guild_id: GuildId, repeat_music: bool) -> Result<()>;

    /// Sets the random next state of the player from a given guild.
    async fn set_random_next(&self, guild_id: GuildId, random_next: bool) -> Result<()>;

    /// Sets the cyclic queue state of the player from a given guild.
    async fn set_cyclic_queue(&self, guild_id: GuildId, cyclic_queue: bool) -> Result<()>;

    /// Sets the autoplay state of the player from a given guild.
    async fn set_autoplay(&self, guild_id: GuildId, autoplay: bool) -> Result<()>;

    /// Fetches and adds the song to the queue, which can also be a playlist.
    async fn queue_add(&self, guild_id: GuildId, song: &str) -> Result<QueueAdd>;

    /// Gets a part of the queue.
    async fn queue(&self, guild_id: GuildId, offset: usize, size: usize) -> Result<Vec<Track>>;

    /// Removes a song from the queue.
    async fn queue_remove(&self, guild_id: GuildId, index: usize) -> Result<bool>;

    /// Gets the currently playing song.
    async fn now(&self, guild_id: GuildId) -> Result<Option<Track>>;

    /// Starts playing a song from the queue, replacing it if there is one currently playing.
    ///
    /// This method should not resume the song, this is a function of [`Backend::set_pause`].
    async fn play(&self, guild_id: GuildId, index: usize) -> Result<Track>;

    /// Skips to the next song in the queue, returning to the beginning of the queue if it is already at the end.
    ///
    /// This method should ignore reproduction rules such as random next and cyclic queue.
    async fn skip(&self, guild_id: GuildId) -> Result<Option<Track>>;

    /// Skips to the previous song in the queue, returning to the end of the queue if it is already at the beginning.
    ///
    /// This method should ignore reproduction rules such as random next and cyclic queue.
    async fn prev(&self, guild_id: GuildId) -> Result<Option<Track>>;

    /// Sets the music playback time.
    async fn seek(&self, guild_id: GuildId, seconds: i64) -> Result<Seek>;

    /// Updates the voice state, necessary if the backend uses a third party to establish the voice call such as [`lavalink`].
    async fn voice_state(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        new: VoiceState,
        old: Option<VoiceState>,
    ) -> Result<()>;

    /// Updates the voice server, necessary if the backend uses a third party to establish the voice call such as [`lavalink`].
    async fn voice_server(&self, guild_id: GuildId, token: &str, endpoint: &str) -> Result<()>;
}
