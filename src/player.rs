use std::{sync::{Arc, atomic::{AtomicUsize, Ordering, AtomicBool}}, fmt::Display, result};

use rand::Rng;
use serenity::model::prelude::{GuildId, UserId, ChannelId, MessageId};
use songbird::{Call, error::JoinError};
use tokio::{sync::{RwLock, Mutex}, task::JoinHandle};

use crate::lavalink::{Lavalink, LavalinkError, rest::{LavalinkTrack, LavalinkUpdatePlayer, LavalinkVoiceState, LavalinkLoadResultType}, LavalinkConnection};

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

pub struct HydrogenPlayCommand {
    pub track: Option<HydrogenMusic>,
    pub count: usize,
    pub playing: bool
}

#[derive(Clone)]
pub struct HydrogenPlayer {
    call: Arc<Mutex<Call>>,
    destroyed: Arc<AtomicBool>,
    guild_id: String,
    guild_locale: String,
    index: Arc<AtomicUsize>,
    lavalink: Lavalink,
    queue: Arc<RwLock<Vec<HydrogenMusic>>>,
    queue_loop: Arc<RwLock<LoopType>>,
    text_channel_id: ChannelId,
    pub destroy_handle: Arc<RwLock<Option<Arc<JoinHandle<()>>>>>,
    pub message_id: Arc<RwLock<Option<MessageId>>>
}

impl HydrogenPlayer {
    pub fn new(lavalink: Lavalink, guild_id: GuildId, text_channel_id: ChannelId, guild_locale: &str, call: Arc<Mutex<Call>>) -> Self {
        Self {
            destroy_handle: Arc::new(RwLock::new(None)),
            destroyed: Arc::new(AtomicBool::new(false)),
            guild_id: guild_id.0.to_string(),
            index: Arc::new(AtomicUsize::new(0)),
            lavalink: lavalink,
            queue: Arc::new(RwLock::new(Vec::new())),
            queue_loop: Arc::new(RwLock::new(LoopType::None)),
            message_id: Arc::new(RwLock::new(None)),
            guild_locale: guild_locale.to_owned(),
            call,
            text_channel_id
        }
    }

    pub fn lavalink(&self) -> Lavalink {
        self.lavalink.clone()
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
        self.queue.read().await.get(self.index.load(Ordering::Relaxed)).cloned()
    }

    pub async fn next(&self) -> Result<()> {
        let queue_loop = self.queue_loop.read().await;
        let queue = self.queue.read().await;

        if queue_loop.ne(&LoopType::NoAutostart) {
            if queue_loop.ne(&LoopType::Music) {
                if queue_loop.ne(&LoopType::Random) {
                    let index = self.index.fetch_add(1, Ordering::Relaxed) + 1;
                    if index >= queue.len() {
                        if queue_loop.eq(&LoopType::Queue) {
                            self.index.store(0, Ordering::Relaxed);
                            self.start_playing().await?;
                        } else {
                            self.index.store(queue.len(), Ordering::Relaxed);
                        }
                    } else {
                        self.start_playing().await?;
                    }
                } else {
                    let random_index = rand::thread_rng().gen_range(0..queue.len());
                    self.index.store(random_index, Ordering::Relaxed);
                    self.start_playing().await?;
                }
            }
        }
        Ok(())
    }

    pub async fn play(&self, music: &str, requester_id: UserId) -> Result<HydrogenPlayCommand> {
        let musics = {
            let mut musics = self.lavalink.track_load(music).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;

            if musics.tracks.len() == 0 {
                musics = self.lavalink.track_load(&format!("ytsearch:{}", music)).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;
            }

            musics
        };

        let starting_index = self.queue.read().await.len();
        if musics.load_type == LavalinkLoadResultType::SearchResult {
            if let Some(music) = musics.tracks.get(0) {
                self.queue.write().await.push(HydrogenMusic::from(music.clone(), requester_id));
                
            } else {
                return Ok(HydrogenPlayCommand {
                    track: None,
                    count: 0,
                    playing: false
                });
            }
        } else {
            for music in musics.tracks.iter() {
                self.queue.write().await.push(HydrogenMusic::from(music.clone(), requester_id));
            }
        }

        let mut playing = false;

        let lavalink_not_playing = match self.lavalink.get_player(&self.guild_id).await {
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

        let mut this_play_track = self.queue.read().await.get(starting_index).cloned();

        if lavalink_not_playing {
            let index = starting_index + musics.playlist_info.selected_track.unwrap_or(0).try_into().unwrap_or(0);
            self.index.store(index, Ordering::Relaxed);
            playing = self.start_playing().await?;
            if playing {
                this_play_track = self.queue.read().await.get(index).cloned();
            }
        }

        Ok(HydrogenPlayCommand {
            track: this_play_track,
            count: self.queue.read().await.len() - starting_index,
            playing
        })
    }

    async fn start_playing(&self) -> Result<bool> {
        if let Some(connection_info) = self.call.lock().await.current_connection().cloned() {
            if let Some(music) = self.queue.read().await.get(self.index.load(Ordering::Relaxed)) {
                let player = LavalinkUpdatePlayer::new()
                        .encoded_track(&music.encoded_track)
                        .voice_state(LavalinkVoiceState::new(&connection_info.token, &connection_info.endpoint, &connection_info.session_id));

                self.lavalink.update_player(&self.guild_id, false, player).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;

                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn destroy(&self) -> Result<()> {
        if !self.destroyed.load(Ordering::Acquire) {
            *self.destroy_handle.write().await = None;

            self.call.lock().await.leave().await.map_err(|e| HydrogenPlayerError::Join(e))?;

            if self.lavalink.connected().await == LavalinkConnection::Connected {
                self.lavalink.destroy_player(&self.guild_id).await.map_err(|e| HydrogenPlayerError::Lavalink(e))?;
            }
        }
        self.destroyed.store(true, Ordering::Release);

        Ok(())
    }
}