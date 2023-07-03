use std::{sync::{Arc, atomic::AtomicUsize}, fmt::Display, result};

use serenity::model::prelude::{GuildId, UserId};
use songbird::ConnectionInfo;
use tokio::sync::RwLock;

use crate::lavalink::{Lavalink, LavalinkError, rest::{LavalinkTrack, LavalinkUpdatePlayer, LavalinkVoiceState}};

#[derive(PartialEq, Eq)]
pub enum LoopType {
    None,
    NoAutostart,
    Music,
    Queue
}

#[derive(Clone)]
pub struct HydrogenMusic {
    pub encoded_track: String,
    pub length: i32,
    pub author: String,
    pub title: String,
    pub uri: Option<String>,
    pub requester_id: UserId
}

impl HydrogenMusic {
    pub fn from(value: LavalinkTrack, requester_id: UserId) -> Self {
        HydrogenMusic {
            encoded_track: value.encoded,
            length: value.info.length,
            author: value.info.author,
            title: value.info.title,
            uri: value.info.uri,
            requester_id
        }
    }
}

#[derive(Debug)]
pub enum HydrogenPlayerError {
    Lavalink(LavalinkError),
}

impl Display for HydrogenPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lavalink(e) => e.fmt(f)
        }
    }
}

pub type Result<T> = result::Result<T, HydrogenPlayerError>;

pub struct HydrogenPlayResult {
    pub track: Option<HydrogenMusic>,
    pub count: usize,
    pub playing: bool
}

#[derive(Clone)]
pub struct HydrogenPlayer {
    pub musics: Arc<RwLock<Vec<HydrogenMusic>>>,
    pub index: Arc<AtomicUsize>,
    pub queue_loop: Arc<RwLock<LoopType>>,
    pub guild_id: String,
    pub connection_info: ConnectionInfo
}

impl HydrogenPlayer {
    pub fn new(guild_id: GuildId, connection_info: ConnectionInfo) -> Self {
        Self {
            musics: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(AtomicUsize::new(0)),
            queue_loop: Arc::new(RwLock::new(LoopType::None)),
            guild_id: guild_id.0.to_string(),
            connection_info
        }
    }

    pub async fn play(&self, lavalink: Lavalink, music: &str, requester_id: UserId) -> Result<HydrogenPlayResult> {
        let musics = lavalink.track_load(music).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;
        for music in musics.tracks.iter() {
            self.musics.write().await.push(HydrogenMusic::from(music.clone(), requester_id));
        }

        let mut playing = false;

        if lavalink.get_player(&self.guild_id).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?.track.is_none() {
            if let Some(music) = musics.tracks.get(0) {
                let player = LavalinkUpdatePlayer::new()
                    .encoded_track(&music.encoded)
                    .voice_state(LavalinkVoiceState::new(&self.connection_info.token, &self.connection_info.endpoint, &self.connection_info.session_id));

                lavalink.update_player(&self.guild_id, false, player).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;

                playing = true;
            }
        }

        Ok(HydrogenPlayResult {
            track: musics.tracks.get(0).map(|v| HydrogenMusic::from(v.clone(), requester_id)),
            count: musics.tracks.len(),
            playing
        })
    }
}