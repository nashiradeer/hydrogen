use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, fmt::Display, result};

use serenity::model::prelude::{GuildId, UserId};
use songbird::ConnectionInfo;
use tokio::sync::RwLock;

use crate::lavalink::{Lavalink, LavalinkError, rest::{LavalinkTrack, LavalinkUpdatePlayer, LavalinkVoiceState, LavalinkLoadResultType}};

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

    pub async fn next(&self, lavalink: Lavalink) -> Result<()> {
        let queue_loop = self.queue_loop.read().await;
        if queue_loop.ne(&LoopType::NoAutostart) {
            if queue_loop.ne(&LoopType::Music) {
                let index = self.index.fetch_add(1, Ordering::AcqRel) + 1;
                let queue_len = self.musics.read().await.len();
                if index >= queue_len {
                    if queue_loop.eq(&LoopType::Queue) {
                        self.index.store(0, Ordering::Release);
                        self.start_playing(lavalink).await?;
                    } else {
                        self.index.store(queue_len, Ordering::Release);
                    }
                } else {
                    self.start_playing(lavalink).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn play(&self, lavalink: Lavalink, music: &str, requester_id: UserId) -> Result<HydrogenPlayResult> {
        let musics = {
            let mut musics = lavalink.track_load(music).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;

            if musics.tracks.len() == 0 {
                musics = lavalink.track_load(&format!("ytsearch:{}", music)).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;
            }

            musics
        };

        let starting_index = self.musics.read().await.len();
        if musics.load_type == LavalinkLoadResultType::SearchResult {
            if let Some(music) = musics.tracks.get(0) {
                self.musics.write().await.push(HydrogenMusic::from(music.clone(), requester_id));
                
            } else {
                return Ok(HydrogenPlayResult {
                    track: None,
                    count: 0,
                    playing: false
                });
            }
        } else {
            for music in musics.tracks.iter() {
                self.musics.write().await.push(HydrogenMusic::from(music.clone(), requester_id));
            }
        }

        let mut playing = false;

        let lavalink_not_playing = match lavalink.get_player(&self.guild_id).await {
            Ok(v) => {
                v.track.is_none()
            },
            Err(e) => {
                if let LavalinkError::RestError(er) = e {
                    if er.status != 404 {
                        return Err(HydrogenPlayerError::Lavalink(LavalinkError::RestError(er)));
                    }
                } else {
                    return Err(HydrogenPlayerError::Lavalink(e));
                }
                
                true
            }
        };

        let mut this_play_track = self.musics.read().await.get(starting_index).cloned();

        if lavalink_not_playing {
            let index = starting_index + musics.playlist_info.selected_track.unwrap_or(0).try_into().unwrap_or(0);
            self.index.store(index, Ordering::Release);
            playing = self.start_playing(lavalink.clone()).await?;
            if playing {
                this_play_track = self.musics.read().await.get(index).cloned();
            }
        }

        Ok(HydrogenPlayResult {
            track: this_play_track,
            count: self.musics.read().await.len() - starting_index,
            playing
        })
    }

    async fn start_playing(&self, lavalink: Lavalink) -> Result<bool> {
        if let Some(music) = self.musics.read().await.get(self.index.load(Ordering::Acquire)) {
            let player = LavalinkUpdatePlayer::new()
                    .encoded_track(&music.encoded_track)
                    .voice_state(LavalinkVoiceState::new(&self.connection_info.token, &self.connection_info.endpoint, &self.connection_info.session_id));

            lavalink.update_player(&self.guild_id, false, player).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;

            return Ok(true);
        }

        Ok(false)
    }
}