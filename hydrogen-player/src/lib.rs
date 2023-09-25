//! # Hydrogen // Player
//!
//! An abstraction between your bot and the backends that you can use to play songs in a Discord's voice chat, facilitating the development of music bots while allowing the use of various audio systems (called `engines`), from the internal driver system of [`songbird`] to the client [`hydrolink`] for Lavalink.
//!
//! ## Features
//!
//! `lavalink` = Enables [`hydrolink`] and the engine [`engine::lavalink::Lavalink`]. (default)
//! `serenity-rustls-webpki` = Enables the `serenity` compatibility and the usage of `rustls` with `webpki-roots`.
//! `serenity-rustls-native` = Enables the `serenity` compatibility and the usage of `rustls` with the native roots.
//! `serenity-native` = Enables the `serenity` compatibility and the usage of `native-tls`.
//! `serenity-native-vendored` = Enables the `serenity` compatibility and the usage of `native-tls` with `vendored` feature.
//! `twilight-rustls-webpki` = Enables the `twilight` compatibility and the usage of `rustls` with `webpki-roots`.
//! `twilight-rustls-native` = Enables the `twilight` compatibility and the usage of `rustls` with the native roots.
//! `twilight-native` = Enables the `twilight` compatibility and the usage of `native-tls`.
//! `twilight-native-vendored` = Enables the `twilight` compatibility and the usage of `native-tls` with `vendored` feature.
use std::{
    fmt::{self, Display, Formatter},
    result,
};

use async_trait::async_trait;
pub use songbird;
use songbird::{
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
    #[cfg_attr(docsrs, doc(cfg(feature = "lavalink")))]
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

/// Information about a track in the queue.
#[derive(Clone, PartialEq, Eq)]
pub struct Track {
    /// Track length/time in seconds.
    pub length: i32,

    /// ID from the user that has requested this track.
    pub requester_id: UserId,

    /// Track title.
    pub title: String,

    /// Author of this track.
    pub author: String,

    /// URI where this track can be found by the users.
    pub uri: Option<String>,

    /// URI from the thumbnail of this track.
    pub thumbnail_uri: Option<String>,
}

/// Information about where and what is added to the queue.
#[derive(Clone)]
pub struct QueueAdd {
    /// A list of tracks added to the queue.
    pub track: Vec<Track>,

    /// Queue offset of this tracks.
    pub offset: usize,

    /// Is true if the max size of the queue has reached.
    pub truncated: bool,
}

/// Information about the current track playing.
#[derive(Clone)]
pub struct TrackPlaying {
    /// The current playback position in seconds.
    pub position: usize,

    /// Total track length/time in seconds.
    pub total: usize,

    /// The current track playing.
    pub track: Track,

    /// The current track index playing.
    pub index: usize,
}

/// Use this trait instead of using the players directly, this trait is used as an interface to guarantee that all players will have the same methods and behavior independently of the backend used.
///
/// # Why are all methods asynchronous?
///
/// The motivation behind making all methods asynchronous comes from the possibility that some player accesses the queue or the music player in an external server, as in the case of Lavalink.
#[async_trait]
pub trait Player {
    /// Initiates the voice chat connection using [`songbird`] and the backend, managing it internally.
    async fn join(&self, guild_id: GuildId) -> Result<()>;

    /// Leaves from the voice chat, closing the connection and destroying the backend.
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

    /// Adds a song or playlist to the queue, searching for it if needed.
    async fn queue_add(&self, guild_id: GuildId, song: &str) -> Result<QueueAdd>;

    /// Gets a part of the queue.
    async fn queue(&self, guild_id: GuildId, offset: usize, size: usize) -> Result<Vec<Track>>;

    /// Removes a song from the queue.
    async fn queue_remove(&self, guild_id: GuildId, index: usize) -> Result<bool>;

    /// Gets the currently playing song.
    async fn now(&self, guild_id: GuildId) -> Result<Option<Track>>;

    /// Starts playing a song from the queue, replacing it if there is one currently playing.
    ///
    /// This method should not resume the song, this is a function of [`Player::set_pause`].
    async fn play(&self, guild_id: GuildId, index: usize) -> Result<TrackPlaying>;

    /// Skips to the next song in the queue, returning to the beginning of the queue if it is already at the end.
    ///
    /// This method should ignore reproduction rules such as random next and cyclic queue.
    async fn skip(&self, guild_id: GuildId) -> Result<Option<Track>>;

    /// Skips to the previous song in the queue, returning to the end of the queue if it is already at the beginning.
    ///
    /// This method should ignore reproduction rules such as random next and cyclic queue.
    async fn prev(&self, guild_id: GuildId) -> Result<Option<Track>>;

    /// Sets the music playback time.
    async fn seek(&self, guild_id: GuildId, seconds: i64) -> Result<TrackPlaying>;

    /// Updates the voice state and the voice chat connection.
    async fn voice_state(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        new: VoiceState,
        old: Option<VoiceState>,
    ) -> Result<()>;

    /// Updates the voice server used to establish the voice chat connection.
    async fn voice_server(&self, guild_id: GuildId, token: &str, endpoint: &str) -> Result<()>;
}
