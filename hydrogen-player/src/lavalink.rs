//! Hydrogen Player // Lavalink
//!
//! Implementation of a backend for [`hydrolink`].
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use async_trait::async_trait;
pub use hydrolink::Error as LavalinkError;
use hydrolink::{Handler, Session, Track as LavalinkTrack, UpdatePlayer, VoiceState};
use songbird::{
    id::{ChannelId, GuildId, UserId},
    ConnectionInfo, Songbird,
};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::warn;

use crate::{utils::Queue, Error, Result, Track as HydrogenTrack};

/// Track internally used by [`Lavalink`].
#[derive(Clone)]
pub struct Track {
    /// Track internally used in [`hydrolink`].
    pub track: LavalinkTrack,

    /// Who has requested this track.
    pub requester_id: UserId,
}

impl Into<HydrogenTrack> for Track {
    fn into(self) -> HydrogenTrack {
        HydrogenTrack {
            length: self.track.info.length,
            author: self.track.info.author,
            title: self.track.info.title,
            uri: self.track.info.uri,
            thumbnail_uri: None,
            requester_id: self.requester_id,
        }
    }
}

#[derive(Clone)]
/// Player internally used in [`Lavalink`].
pub struct Player {
    guild_id: GuildId,
    lavalink: Node,
    queue: Queue<Track>,
    voice_manager: Arc<Songbird>,
}

impl Player {
    /// Fetches the [`ConnectionInfo`] from the voice call connection.
    async fn get_connection(&self) -> Option<ConnectionInfo> {
        self.voice_manager
            .get(self.guild_id)?
            .lock()
            .await
            .current_connection()
            .cloned()
    }

    /// Pauses/resumes the player in Lavalink.
    pub async fn set_pause(&self, paused: bool) -> Result<()> {
        // Prepare to update the player.
        let mut player = UpdatePlayer {
            paused: Some(paused),
            ..Default::default()
        };

        // Get the player from Lavalink server.
        let lavalink_player = self.lavalink.session.get_player(self.guild_id.0).await.ok();
        // Check if player exists and store the bool to be used later.
        let has_player = lavalink_player.is_some();

        // If isn't any track playing now, get a track from the queue and update the voice state.
        if let Some(lavalink_player) = lavalink_player {
            if lavalink_player.track.is_none() && !paused {
                // Fetch the connection from the voice chat.
                let connection = self.get_connection().await.ok_or(Error::NotConnected)?;
                // Check if should have a song playing now accordingly to the in-memory queue.
                if let Some(music) = self.queue.now() {
                    player.encoded_track = Some(Some(music.track.encoded.clone()));
                    player.voice = Some(VoiceState::new(
                        &connection.token,
                        &connection.endpoint,
                        &connection.session_id,
                    ));
                }
            }
        }

        // If already have a player in Lavalink server, request the resume.
        if has_player {
            self.lavalink
                .session
                .update_player(self.guild_id.0, true, player)
                .await
                .map_err(|e| Error::Lavalink(e))?;
        }

        // If not, request to start the current track.
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

/// Lavalink node wrapper, used to redirect events and to track the players created in it.
#[derive(Clone)]
struct Node {
    /// Players created in this node.
    pub players: Arc<Mutex<Vec<GuildId>>>,

    /// The node connection.
    pub session: Session,

    /// Who are managing this node and its players.
    pub manager: Lavalink,
}

#[async_trait]
impl Handler for Node {
    async fn disconnect(&self, _: Session) {
        let owned_players = self.players.lock().unwrap().clone();
        self.manager.disconnection_handler(owned_players).await;
    }
}

/// Engine/backend that manages players in a pool of Lavalink nodes using [`hydrolink`].
#[derive(Clone)]
pub struct Lavalink {
    /// Players from this manager.
    players: Arc<AsyncRwLock<HashMap<GuildId, Player>>>,

    /// Lavalink node pool.
    nodes: Arc<RwLock<Vec<Node>>>,

    /// Index used to balance the load across the nodes on the pool.
    index: Arc<Mutex<usize>>,

    /// Voice Manager used to connect to Discord.
    voice_manager: Arc<Songbird>,

    /// Max queue size.
    max_size: usize,
}

impl Lavalink {
    /// Initialize the Lavalink engine.
    pub fn new(songbird: Arc<Songbird>, max_size: usize) -> Self {
        Self {
            players: Arc::new(AsyncRwLock::new(HashMap::new())),
            nodes: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(Mutex::new(0)),
            voice_manager: songbird,
            max_size,
        }
    }

    /// Gets a Lavalink node from the pool, balancing the load.
    fn get_node(&self) -> Option<Node> {
        let nodes = self.nodes.read().unwrap();
        let mut index = self.index.lock().unwrap();

        for _ in 0..5 {
            if let Some(node) = nodes.get(*index) {
                *index = *index + 1;
                if *index >= nodes.len() {
                    *index = 0;
                }

                if node.session.is_connected() {
                    return Some(node.clone());
                }
            } else {
                break;
            }
        }

        None
    }

    /// Handle the Lavalink Node disconnection, leaving from the voice chat and removing the players.
    async fn disconnection_handler(&self, players: Vec<GuildId>) {
        let mut all_players = self.players.write().await;

        for guild_id in players {
            if let Err(v) = self.voice_manager.leave(guild_id).await {
                warn!("PLAYER [LAVALINK]: can't leave from voice chat: {:?}", v);
            }

            all_players.remove(&guild_id);
        }
    }
}

impl Lavalink {
    async fn join(&self, guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
        let connection_info = match self.voice_manager.get(guild_id) {
            Some(v) => v.lock().await.current_connection().unwrap().clone(),
            None => self
                .voice_manager
                .join_gateway(guild_id, channel_id)
                .await
                .1
                .map_err(Error::Join)?,
        };

        let mut players = self.players.write().await;
        let node = self.get_node().ok_or(Error::NotConnected)?;

        let player = Player {
            guild_id,
            lavalink: node.clone(),
            queue: Queue::new(self.max_size),
            voice_manager: self.voice_manager.clone(),
        };

        players.insert(guild_id, player.clone());

        Ok(())
    }
}
