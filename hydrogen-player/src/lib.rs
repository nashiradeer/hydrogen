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

pub use songbird;
use songbird::{
    error::JoinError,
    id::{ChannelId, UserId},
};

#[cfg(feature = "lavalink")]
#[cfg_attr(docsrs, doc(cfg(feature = "lavalink")))]
pub mod lavalink;
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
