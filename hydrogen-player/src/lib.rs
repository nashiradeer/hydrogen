use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    result,
    sync::{Arc, Mutex},
};

use backend::Backend;
pub use songbird::{
    error::JoinError,
    id::{ChannelId, GuildId, UserId},
};

pub mod backend;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "lavalink")]
    Lavalink(hydrolink::Error),

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

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub struct VoiceState {
    pub channel_id: Option<ChannelId>,
    pub session_id: String,
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
pub struct QueueAddResult {
    pub track: Vec<Track>,
    pub offset: usize,
    pub truncated: bool,
}

#[derive(Clone)]
pub struct SeekResult {
    pub position: usize,
    pub total: usize,
    pub track: Track,
}

pub struct PlayerManager<T: Backend, D> {
    backend: T,
    data: Arc<Mutex<HashMap<GuildId, D>>>,
}
