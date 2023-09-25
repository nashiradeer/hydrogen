//! Hydrogen Player // Lavalink
//!
//! Implementation of a backend for [`hydrolink`].

use std::sync::{atomic::AtomicUsize, Arc, RwLock};

use async_trait::async_trait;
pub use hydrolink::Error as LavalinkError;
use hydrolink::{Session, Track as LavalinkTrack};
use songbird::{
    id::{GuildId, UserId},
    Songbird,
};

use crate::Connection;

use super::{Player, QueueNext, Track};

#[derive(Clone)]
pub struct LavalinkMusic {
    pub encoded_track: String,
    pub track: Track,
}

impl LavalinkMusic {
    pub fn new(value: LavalinkTrack, requester_id: UserId) -> Self {
        Self {
            encoded_track: value.encoded,
            track: Track {
                length: value.info.length,
                author: value.info.author,
                title: value.info.title,
                uri: value.info.uri,
                thumbnail_uri: None,
                requester_id,
            },
        }
    }

    pub fn track(&self) -> Track {
        self.track.clone()
    }
}

#[derive(Clone)]
pub struct LavalinkPlayer {
    pub connection: Arc<RwLock<Connection>>,
    guild_id: GuildId,
    index: Arc<AtomicUsize>,
    lavalink: Session,
    queue: Arc<RwLock<Vec<LavalinkMusic>>>,
    queue_next: Arc<RwLock<QueueNext>>,
    voice_manager: Arc<Songbird>,
}

#[async_trait]
impl Player for LavalinkPlayer {
    async fn new(guild_id: GuildId, voice_manager: Arc<Songbird>, connection: Connection) -> Self {
        Self {
            connection: Arc::new(RwLock::new(connection)),
            destroyed: Arc::new(AtomicBool::new(false)),
            index: Arc::new(AtomicUsize::new(0)),
            paused: Arc::new(AtomicBool::new(false)),
            queue: Arc::new(RwLock::new(Vec::new())),
            queue_loop: Arc::new(RwLock::new(LoopType::None)),
            guild_locale: guild_locale.to_owned(),
            guild_id,
            lavalink,
            text_channel_id,
            voice_manager,
        }
    }

    pub async fn loop_type(&self) -> LoopType {
        self.queue_loop.read().await.clone()
    }

    pub async fn set_loop_type(&self, loop_type: LoopType) {
        *self.queue_loop.write().await = loop_type;
    }

    pub fn pause(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub async fn set_pause(&self, paused: bool) -> Result<()> {
        let mut player = LavalinkUpdatePlayer::new();

        player.paused(paused);

        let lavalink_player = self.lavalink.get_player(self.guild_id.0).await.ok();
        let has_player = lavalink_player.is_some();

        if let Some(lavalink_player) = lavalink_player {
            if lavalink_player.track.is_none() && !paused {
                let connection = self.connection.read().await;
                if let Some(music) = self
                    .queue
                    .read()
                    .await
                    .get(self.index.load(Ordering::Relaxed))
                {
                    player
                        .encoded_track(&music.encoded_track)
                        .voice_state(connection.clone().into());
                }
            }
        }

        if has_player {
            self.lavalink
                .update_player(self.guild_id.0, true, &player)
                .await
                .map_err(|e| HydrogenPlayerError::Lavalink(e))?;
        }

        self.paused.store(paused, Ordering::Relaxed);

        if !has_player && !paused {
            self.start_playing().await?;
        }

        Ok(())
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

    pub async fn now(&self) -> Option<HydrogenMusic> {
        self.queue
            .read()
            .await
            .get(self.index.load(Ordering::Relaxed))
            .cloned()
    }

    pub async fn queue(&self) -> Vec<HydrogenMusic> {
        self.queue.read().await.clone()
    }

    pub async fn skip(&self) -> Result<Option<HydrogenMusic>> {
        let queue = self.queue.read().await;
        let mut index = self.index.fetch_add(1, Ordering::Relaxed) + 1;
        if index >= queue.len() {
            self.index.store(0, Ordering::Relaxed);
            index = 0;
        }
        self.start_playing().await?;
        Ok(queue.get(index).cloned())
    }

    pub async fn prev(&self) -> Result<Option<HydrogenMusic>> {
        let queue = self.queue.read().await;
        let mut index = self.index.load(Ordering::Relaxed);
        if index == 0 {
            index = queue.len() - 1;
        } else {
            index -= 1;
        }
        self.index.store(index, Ordering::Relaxed);
        self.start_playing().await?;
        Ok(queue.get(index).cloned())
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
                            self.index.store(queue.len() - 1, Ordering::Relaxed);
                            self.paused.store(true, Ordering::Relaxed);
                        }
                    } else {
                        self.start_playing().await?;
                    }
                } else {
                    let random_index = rand::thread_rng().gen_range(0..queue.len());
                    self.index.store(random_index, Ordering::Relaxed);
                    self.start_playing().await?;
                }
            } else {
                self.start_playing().await?;
            }
        } else {
            let index = self.index.fetch_add(1, Ordering::Relaxed) + 1;
            if index >= queue.len() {
                self.index.store(queue.len() - 1, Ordering::Relaxed);
            }
            self.paused.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn play(&self, music: &str, requester_id: UserId) -> Result<HydrogenPlayCommand> {
        let musics = {
            let mut musics = self
                .lavalink
                .track_load(music)
                .await
                .map_err(|e| HydrogenPlayerError::Lavalink(e))?;

            if musics.tracks.len() == 0 {
                musics = self
                    .lavalink
                    .track_load(&format!("{}{}", HYDROGEN_SEARCH_PREFIX, music))
                    .await
                    .map_err(|e| HydrogenPlayerError::Lavalink(e))?;
            }

            musics
        };

        let mut truncated = false;
        let starting_index = self.queue.read().await.len();
        if musics.load_type == LavalinkLoadResultType::SearchResult {
            if let Some(music) = musics.tracks.get(0) {
                let queue_length = self.queue.read().await.len();
                if queue_length < HYDROGEN_QUEUE_LIMIT {
                    self.queue
                        .write()
                        .await
                        .push(HydrogenMusic::from(music.clone(), requester_id));
                } else {
                    truncated = true;
                }
            } else {
                return Ok(HydrogenPlayCommand {
                    track: None,
                    count: 0,
                    playing: false,
                    truncated: false,
                });
            }
        } else {
            for music in musics.tracks.iter() {
                let queue_length = self.queue.read().await.len();
                if queue_length < HYDROGEN_QUEUE_LIMIT {
                    self.queue
                        .write()
                        .await
                        .push(HydrogenMusic::from(music.clone(), requester_id));
                } else {
                    truncated = true;
                    break;
                }
            }
        }

        let mut playing = false;

        let lavalink_not_playing = match self.lavalink.get_player(self.guild_id.0).await {
            Ok(v) => v.track.is_none(),
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
            let mut index = starting_index
                + musics
                    .playlist_info
                    .selected_track
                    .unwrap_or(0)
                    .try_into()
                    .unwrap_or(0);

            if index >= self.queue.read().await.len() {
                index = starting_index;
            }

            self.index.store(index, Ordering::Relaxed);
            playing = self.start_playing().await?;
            if playing {
                this_play_track = self.queue.read().await.get(index).cloned();
            }
        }

        Ok(HydrogenPlayCommand {
            track: this_play_track,
            count: self.queue.read().await.len() - starting_index,
            playing,
            truncated,
        })
    }

    pub async fn seek(&self, milliseconds: i32) -> Result<Option<HydrogenSeekCommand>> {
        let mut update_player = LavalinkUpdatePlayer::new();
        update_player.position(milliseconds);
        let player = self
            .lavalink
            .update_player(self.guild_id.0, false, &update_player)
            .await
            .map_err(|e| HydrogenPlayerError::Lavalink(e))?;
        if let Some(track) = player.track {
            if let Some(music) = self.now().await {
                return Ok(Some(HydrogenSeekCommand {
                    position: track.info.position,
                    total: track.info.length,
                    track: music,
                }));
            }
        }
        Ok(None)
    }

    async fn start_playing(&self) -> Result<bool> {
        let connection = self.connection.read().await;
        if let Some(music) = self
            .queue
            .read()
            .await
            .get(self.index.load(Ordering::Relaxed))
        {
            let mut player = LavalinkUpdatePlayer::new();
            player
                .encoded_track(&music.encoded_track)
                .voice_state(connection.clone().into())
                .paused(self.paused.load(Ordering::Relaxed));

            self.lavalink
                .update_player(self.guild_id.0, false, &player)
                .await
                .map_err(|e| HydrogenPlayerError::Lavalink(e))?;

            return Ok(true);
        }

        Ok(false)
    }

    pub async fn destroy(&self) -> Result<()> {
        if !self.destroyed.load(Ordering::Acquire) {
            self.voice_manager
                .leave(self.guild_id)
                .await
                .map_err(|e| HydrogenPlayerError::Join(e))?;

            if self.lavalink.connected().await == LavalinkConnection::Connected {
                self.lavalink
                    .destroy_player(self.guild_id.0)
                    .await
                    .map_err(|e| HydrogenPlayerError::Lavalink(e))?;
            }
        }
        self.destroyed.store(true, Ordering::Release);

        Ok(())
    }

    pub async fn update_connection(&self) -> Result<()> {
        let connection = self.connection.read().await;
        let mut player = LavalinkUpdatePlayer::new();
        player.voice_state(connection.clone().into());

        self.lavalink
            .update_player(self.guild_id.0, true, &player)
            .await
            .map_err(|e| HydrogenPlayerError::Lavalink(e))?;

        Ok(())
    }
}
