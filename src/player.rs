use std::{sync::{Arc, atomic::{AtomicUsize, Ordering, AtomicBool}}, fmt::Display, result, time::Duration};

use rand::Rng;
use serenity::model::prelude::{GuildId, UserId, ChannelId, MessageId};
use songbird::{Call, error::JoinError};
use tokio::{sync::{RwLock, Mutex}, time::sleep, task::JoinHandle, spawn};

use crate::lavalink::{Lavalink, LavalinkError, rest::{LavalinkTrack, LavalinkUpdatePlayer, LavalinkVoiceState, LavalinkLoadResultType}};

#[derive(PartialEq, Eq)]
pub enum LoopType {
    None,
    NoAutostart,
    Music,
    Queue,
    Random
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
    Join(JoinError),
}

impl Display for HydrogenPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lavalink(e) => e.fmt(f),
            Self::Join(e) => e.fmt(f)
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
    musics: Arc<RwLock<Vec<HydrogenMusic>>>,
    index: Arc<AtomicUsize>,
    queue_loop: Arc<RwLock<LoopType>>,
    guild_id: String,
    call: Arc<Mutex<Call>>,
    text_channel_id: ChannelId,
    guild_locale: String,
    destroy_handle: Arc<RwLock<Option<Arc<JoinHandle<()>>>>>,
    destroyed: Arc<AtomicBool>,
    pub message_id: Arc<RwLock<Option<MessageId>>>
}

impl HydrogenPlayer {
    pub fn new(guild_id: GuildId, text_channel_id: ChannelId, guild_locale: String, call: Arc<Mutex<Call>>) -> Self {
        Self {
            musics: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(AtomicUsize::new(0)),
            queue_loop: Arc::new(RwLock::new(LoopType::None)),
            guild_id: guild_id.0.to_string(),
            message_id: Arc::new(RwLock::new(None)),
            destroyed: Arc::new(AtomicBool::new(false)),
            destroy_handle: Arc::new(RwLock::new(None)),
            text_channel_id,
            call,
            guild_locale
        }
    }

    pub fn text_channel_id(&self) -> ChannelId {
        self.text_channel_id
    }

    pub fn guild_locale(&self) -> String {
        self.guild_locale.clone()
    }

    pub async fn call(&self) -> Arc<Mutex<Call>> {
        self.call.clone()
    }

    pub async fn now(&self) -> Option<HydrogenMusic> {
        self.musics.read().await.get(self.index.load(Ordering::Relaxed)).cloned()
    }

    pub async fn next(&self, lavalink: Lavalink) -> Result<()> {
        let queue_loop = self.queue_loop.read().await;
        if queue_loop.ne(&LoopType::NoAutostart) {
            if queue_loop.ne(&LoopType::Music) {
                if queue_loop.ne(&LoopType::Random) {
                    let index = self.index.fetch_add(1, Ordering::Relaxed) + 1;
                    let queue_len = self.musics.read().await.len();
                    if index >= queue_len {
                        if queue_loop.eq(&LoopType::Queue) {
                            self.index.store(0, Ordering::Relaxed);
                            self.start_playing(lavalink).await?;
                        } else {
                            self.index.store(queue_len, Ordering::Relaxed);
                        }
                    } else {
                        self.start_playing(lavalink).await?;
                    }
                } else {
                    let queue_len = self.musics.read().await.len();
                    let new_index = rand::thread_rng().gen_range(0..queue_len);
                    self.index.store(new_index, Ordering::Relaxed);
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
            self.index.store(index, Ordering::Relaxed);
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
        if let Some(connection_info) = self.call.lock().await.current_connection().cloned() {
            if let Some(music) = self.musics.read().await.get(self.index.load(Ordering::Relaxed)) {
                let player = LavalinkUpdatePlayer::new()
                        .encoded_track(&music.encoded_track)
                        .voice_state(LavalinkVoiceState::new(&connection_info.token, &connection_info.endpoint, &connection_info.session_id));

                lavalink.update_player(&self.guild_id, false, player).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;

                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn destroy(&self, lavalink: Lavalink) -> Result<()> {
        if !self.destroyed.load(Ordering::Relaxed) {
            self.destroyed.store(true, Ordering::Relaxed);
            *self.destroy_handle.write().await = None;
            lavalink.destroy_player(&self.guild_id).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;
            self.call.lock().await.leave().await.map_err(|e| HydrogenPlayerError::Join(e))?;
        }

        Ok(())
    }

    pub async fn timed_destroy(&self, lavalink: Lavalink, duration: Duration) {
        let self_cloned = self.clone();
        let lavalink_cloned = lavalink.clone();
        *self.destroy_handle.write().await = Some(Arc::new(spawn(async move {
            sleep(duration).await;
            _ = self_cloned.destroy(lavalink_cloned).await;
        })));
    }

    pub async fn cancel_destroy(&self) {
        let maybe_handle = self.destroy_handle.read().await.clone();
        if let Some(handle) = maybe_handle {
            handle.abort();
            *self.destroy_handle.write().await = None;
        }
    }
}