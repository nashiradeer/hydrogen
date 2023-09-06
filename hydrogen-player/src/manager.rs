use std::{
    collections::HashMap,
    fmt::Display,
    process::exit,
    result,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use serenity::{
    builder::{CreateComponents, CreateEmbedAuthor},
    client::Cache,
    http::{CacheHttp, Http},
    model::{
        prelude::{
            component::ButtonStyle, Channel, ChannelId, ChannelType, GuildId, MessageId,
            ReactionType, UserId, VoiceServerUpdateEvent,
        },
        voice::VoiceState,
    },
};
use songbird::Songbird;
use tokio::{spawn, sync::RwLock, task::JoinHandle, time::sleep};
use tracing::{debug, error, warn};

use crate::{
    i18n::HydrogenI18n,
    lavalink::{
        websocket::{LavalinkTrackEndEvent, LavalinkTrackEndReason, LavalinkTrackStartEvent},
        Lavalink, LavalinkError, LavalinkHandler, LavalinkNodeInfo,
    },
    player::{
        HydrogenMusic, HydrogenPlayCommand, HydrogenPlayer, HydrogenPlayerError,
        HydrogenSeekCommand, LoopType,
    },
    HYDROGEN_EMPTY_CHAT_TIMEOUT, HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

#[derive(Debug)]
pub enum HydrogenManagerError {
    Lavalink(LavalinkError),
    Serenity(serenity::Error),
    Player(HydrogenPlayerError),
    LavalinkNotConnected,
    VoiceManagerNotConnected,
    GuildIdMissing,
    GuildChannelNotFound,
    PlayerNotFound,
}

impl Display for HydrogenManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lavalink(e) => e.fmt(f),
            Self::Serenity(e) => e.fmt(f),
            Self::Player(e) => e.fmt(f),
            Self::LavalinkNotConnected => write!(f, "there're no lavalink nodes connected"),
            Self::VoiceManagerNotConnected => {
                write!(f, "voice manager doesn't have a call for this guild")
            }
            Self::GuildIdMissing => write!(f, "guild id missing"),
            Self::GuildChannelNotFound => write!(f, "guild channel not found"),
            Self::PlayerNotFound => write!(f, "player not found"),
        }
    }
}

pub type Result<T> = result::Result<T, HydrogenManagerError>;

#[derive(Clone, PartialEq, Eq)]
enum HydrogenPlayerState {
    Nothing,
    Playing,
    Thinking,
}

#[derive(Clone)]
pub struct HydrogenManager {
    cache: Arc<Cache>,
    destroy_handle: Arc<RwLock<HashMap<GuildId, JoinHandle<()>>>>,
    http: Arc<Http>,
    lavalink: Arc<RwLock<Vec<Lavalink>>>,
    load_balancer: Arc<AtomicUsize>,
    player: Arc<RwLock<HashMap<GuildId, HydrogenPlayer>>>,
}

impl HydrogenManager {
    pub fn new(cache: Arc<Cache>, http: Arc<Http>, i18n: HydrogenI18n) -> Self {
        Self {
            lavalink: Arc::new(RwLock::new(Vec::new())),
            destroy_handle: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: Arc::new(AtomicUsize::new(0)),
            message: Arc::new(RwLock::new(HashMap::new())),
            player: Arc::new(RwLock::new(HashMap::new())),
            cache,
            http,
            i18n,
        }
    }

    pub async fn connect_lavalink(&self, node: LavalinkNodeInfo) -> Result<()> {
        let mut lavalink_vector = self.lavalink.write().await;
        let lavalink = Lavalink::connect(node, self.cache.current_user().id.0, self.clone())
            .await
            .map_err(|e| HydrogenManagerError::Lavalink(e))?;
        lavalink_vector.push(lavalink);
        Ok(())
    }

    pub async fn lavalink_node_count(&self) -> usize {
        let nodes = self.lavalink.read().await;
        nodes.len()
    }

    async fn increment_load_balancer(&self) -> usize {
        let index = self.load_balancer.fetch_add(1, Ordering::AcqRel);
        let lavalink = self.lavalink.read().await;

        if index + 1 >= lavalink.len() {
            self.load_balancer.store(0, Ordering::Release);
        }

        if index >= lavalink.len() {
            return 0;
        }

        index
    }

    pub async fn init(
        &self,
        guild_id: GuildId,
        guild_locale: &str,
        voice_manager: Arc<Songbird>,
        text_channel_id: ChannelId,
    ) -> Result<HydrogenPlayer> {
        let player = {
            let call = voice_manager
                .get(guild_id)
                .ok_or(HydrogenManagerError::VoiceManagerNotConnected)?;
            let connection_info = call
                .lock()
                .await
                .current_connection()
                .cloned()
                .ok_or(HydrogenManagerError::VoiceManagerNotConnected)?;

            let mut players = self.player.write().await;
            let lavalink_nodes = self.lavalink.read().await;

            let lavalink_index = self.increment_load_balancer().await;

            let lavalink = lavalink_nodes
                .get(lavalink_index)
                .cloned()
                .ok_or(HydrogenManagerError::LavalinkNotConnected)?;
            let player = HydrogenPlayer::new(
                lavalink,
                guild_id,
                voice_manager,
                connection_info.into(),
                text_channel_id,
                guild_locale,
            );

            players.insert(guild_id, player.clone());

            player
        };

        self.update_now_playing(guild_id).await;

        Ok(player)
    }

    pub async fn init_or_play(
        &self,
        guild_id: GuildId,
        guild_locale: &str,
        music: &str,
        requester_id: UserId,
        voice_manager: Arc<Songbird>,
        text_channel_id: ChannelId,
    ) -> Result<HydrogenPlayCommand> {
        let player_option = {
            let option = self.player.read().await;
            option.get(&guild_id).cloned()
        };

        if let Some(player) = player_option {
            return Ok(player
                .play(music, requester_id)
                .await
                .map_err(|e| HydrogenManagerError::Player(e))?);
        }

        let player = self
            .init(guild_id, guild_locale, voice_manager, text_channel_id)
            .await?;

        Ok(player
            .play(music, requester_id)
            .await
            .map_err(|e| HydrogenManagerError::Player(e))?)
    }

    pub async fn contains_player(&self, guild_id: GuildId) -> bool {
        self.player.read().await.contains_key(&guild_id)
    }

    pub async fn get_voice_channel_id(&self, guild_id: GuildId) -> Option<songbird::id::ChannelId> {
        let players = self.player.read().await;
        let connection = players.get(&guild_id)?.connection.read().await;
        connection.channel_id.clone()
    }

    pub async fn skip(&self, guild_id: GuildId) -> Result<Option<HydrogenMusic>> {
        let players = self.player.read().await;

        let player = players
            .get(&guild_id)
            .ok_or(HydrogenManagerError::PlayerNotFound)?;

        player
            .skip()
            .await
            .map_err(|e| HydrogenManagerError::Player(e))
    }

    pub async fn prev(&self, guild_id: GuildId) -> Result<Option<HydrogenMusic>> {
        let players = self.player.read().await;

        let player = players
            .get(&guild_id)
            .ok_or(HydrogenManagerError::PlayerNotFound)?;

        player
            .prev()
            .await
            .map_err(|e| HydrogenManagerError::Player(e))
    }

    pub async fn seek(
        &self,
        guild_id: GuildId,
        milliseconds: i32,
    ) -> Result<Option<HydrogenSeekCommand>> {
        let players = self.player.read().await;

        let player = players
            .get(&guild_id)
            .ok_or(HydrogenManagerError::PlayerNotFound)?;

        player
            .seek(milliseconds)
            .await
            .map_err(|e| HydrogenManagerError::Player(e))
    }

    pub async fn update_voice_state(
        &self,
        old_voice_state: Option<VoiceState>,
        voice_state: VoiceState,
    ) -> Result<()> {
        let players = self.player.read().await;

        let guild_id = voice_state
            .guild_id
            .ok_or(HydrogenManagerError::GuildIdMissing)?;
        let Some(player) = players.get(&guild_id) else {
            return Ok(());
        };

        {
            if old_voice_state.is_some() {
                if voice_state.user_id == self.cache.current_user().id {
                    if let Some(channel_id) = voice_state.channel_id {
                        let mut connection = player.connection.write().await;

                        connection.session_id = voice_state.session_id;

                        if let Some(token) = voice_state.token {
                            connection.token = token;
                        }

                        connection.channel_id = Some(channel_id.into());
                    } else {
                        let is_connected = player.connection.read().await.channel_id.is_some();
                        if is_connected {
                            drop(players);

                            self.destroy(guild_id).await?;

                            return Ok(());
                        }
                    }
                }
            }
        }

        let connection = player.connection.read().await;
        if let Some(channel_id) = connection.channel_id {
            if let Channel::Guild(channel) = self
                .cache
                .channel(channel_id.0)
                .ok_or(HydrogenManagerError::GuildChannelNotFound)?
            {
                if channel.kind == ChannelType::Voice || channel.kind == ChannelType::Stage {
                    let members_count = channel
                        .members(self.cache.clone())
                        .await
                        .map_err(|e| HydrogenManagerError::Serenity(e))?
                        .len();

                    if members_count <= 1 {
                        self.timed_destroy(
                            guild_id,
                            Duration::from_secs(HYDROGEN_EMPTY_CHAT_TIMEOUT),
                        )
                        .await;

                        self.update_play_message(
                            guild_id,
                            &self
                                .i18n
                                .translate(&player.guild_locale(), "playing", "timeout_trigger")
                                .replace("${time}", &HYDROGEN_EMPTY_CHAT_TIMEOUT.to_string()),
                            HYDROGEN_PRIMARY_COLOR,
                            HydrogenPlayerState::Thinking,
                            player.pause(),
                            player.loop_type().await,
                            None,
                        )
                        .await;
                    } else {
                        self.cancel_destroy(guild_id).await;
                        self.update_now_playing(guild_id).await;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn update_voice_server(&self, voice_server: VoiceServerUpdateEvent) -> Result<()> {
        let players = self.player.read().await;

        let guild_id = voice_server
            .guild_id
            .ok_or(HydrogenManagerError::GuildIdMissing)?;
        let Some(player) = players.get(&guild_id) else {
            return Ok(());
        };

        {
            let mut connection = player.connection.write().await;

            connection.token = voice_server.token;

            if let Some(endpoint) = voice_server.endpoint {
                connection.endpoint = endpoint;
            }
        }

        player
            .update_connection()
            .await
            .map_err(|e| HydrogenManagerError::Player(e))?;

        Ok(())
    }

    pub async fn destroy(&self, guild_id: GuildId) -> Result<()> {
        let mut players = self.player.write().await;
        let mut messages = self.message.write().await;
        let mut destroy_handles = self.destroy_handle.write().await;

        if let Some(player) = players.get(&guild_id) {
            player
                .destroy()
                .await
                .map_err(|e| HydrogenManagerError::Player(e))?;

            if let Some(message) = messages.get(&guild_id) {
                self.http
                    .delete_message(player.text_channel_id().0, message.0)
                    .await
                    .map_err(|e| HydrogenManagerError::Serenity(e))?;
            }
        }

        if let Some(destroy_handle) = destroy_handles.get(&guild_id) {
            destroy_handle.abort();
        }

        players.remove(&guild_id);
        messages.remove(&guild_id);
        destroy_handles.remove(&guild_id);

        Ok(())
    }

    pub async fn timed_destroy(&self, guild_id: GuildId, duration: Duration) {
        let players = self.player.read().await;
        let mut destroy_handles = self.destroy_handle.write().await;

        if players.get(&guild_id).is_some() {
            if destroy_handles.get(&guild_id).is_none() {
                let self_clone = self.clone();
                let guild_id_clone = guild_id.clone();
                destroy_handles.insert(
                    guild_id,
                    spawn(async move {
                        sleep(duration).await;

                        {
                            let mut _destroy_handles = self_clone.destroy_handle.write().await;
                            _destroy_handles.remove(&guild_id_clone);
                        }

                        _ = self_clone.destroy(guild_id_clone).await;
                    }),
                );
            }
        }
    }

    pub async fn cancel_destroy(&self, guild_id: GuildId) {
        let mut destroy_handles = self.destroy_handle.write().await;

        if let Some(handle) = destroy_handles.get(&guild_id) {
            handle.abort();
            destroy_handles.remove(&guild_id);
        }
    }

    pub async fn get_loop_type(&self, guild_id: GuildId) -> LoopType {
        let players = self.player.read().await;

        if let Some(player) = players.get(&guild_id) {
            return player.loop_type().await;
        }

        return LoopType::None;
    }

    pub async fn set_loop_type(&self, guild_id: GuildId, loop_type: LoopType) {
        let players = self.player.read().await;

        if let Some(player) = players.get(&guild_id) {
            player.set_loop_type(loop_type).await;
        }

        drop(players);
        self.update_now_playing(guild_id).await;
    }

    pub async fn get_paused(&self, guild_id: GuildId) -> bool {
        let players = self.player.read().await;

        if let Some(player) = players.get(&guild_id) {
            return player.pause();
        }

        return false;
    }

    pub async fn set_paused(&self, guild_id: GuildId, paused: bool) -> Result<()> {
        let players = self.player.read().await;

        if let Some(player) = players.get(&guild_id) {
            player
                .set_pause(paused)
                .await
                .map_err(|e| HydrogenManagerError::Player(e))?;
        }

        drop(players);
        self.update_now_playing(guild_id).await;
        Ok(())
    }
}

#[async_trait]
impl LavalinkHandler for HydrogenManager {
    async fn lavalink_ready(&self, node: Lavalink, _: bool) {
        debug!("processing lavalink ready...");
        let lavalink_nodes = self.lavalink.read().await;
        if let Some(index) = find_lavalink(&lavalink_nodes, &node).await {
            debug!("managed lavalink {} initialized and connected", index);
        } else {
            debug!("unknown lavalink initialized and connected");
        }
        debug!("processed lavalink ready");
    }

    async fn lavalink_disconnect(&self, node: Lavalink) {
        debug!("processing lavalink disconnect...");
        let mut lavalink_nodes = self.lavalink.write().await;
        if let Some(index) = find_lavalink(&lavalink_nodes, &node).await {
            warn!("managed lavalink {} has disconnected", index);
            lavalink_nodes.remove(index);
        } else {
            warn!("unknown lavalink has disconnected");
        }

        if lavalink_nodes.len() == 0 {
            error!("there's not lavalink available anymore, exiting...");
            exit(1);
        }

        warn!("destroying all players that are using this lavalink...");
        let mut players = self.player.write().await;
        let players_clone = players.clone();
        for (guild_id, player) in players_clone.iter() {
            if node.eq(&player.lavalink()).await {
                players.remove(guild_id);
                if let Err(e) = player.destroy().await {
                    error!("can't cleanup some player: {}", e);
                }
            }
        }
        debug!("processed lavalink disconnect");
    }

    async fn lavalink_track_start(&self, _: Lavalink, message: LavalinkTrackStartEvent) {
        debug!("processing lavalink track start...");
        let guild_id = match message.guild_id.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                warn!("invalid guild id in track start event: {}", e);
                return;
            }
        };

        self.update_now_playing(guild_id.into()).await;
        debug!("processed lavalink track start");
    }

    async fn lavalink_track_end(&self, _: Lavalink, message: LavalinkTrackEndEvent) {
        debug!("processing lavalink track end...");
        if message.reason == LavalinkTrackEndReason::Finished {
            let guild_id = match message.guild_id.parse::<u64>() {
                Ok(v) => v,
                Err(e) => {
                    warn!("invalid guild id in track end event: {}", e);
                    return;
                }
            };
            if let Some(player) = self.player.read().await.get(&guild_id.into()) {
                if let Err(e) = player.next().await {
                    warn!("can't go to the next music: {}", e);
                }

                self.update_now_playing(guild_id.into()).await;
            }
        }
        debug!("processed lavalink track end");
    }
}

impl CacheHttp for HydrogenManager {
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
    fn http(&self) -> &Http {
        &self.http
    }
}

async fn find_lavalink(nodes: &Vec<Lavalink>, lavalink: &Lavalink) -> Option<usize> {
    for i in 0..nodes.len() {
        if let Some(node) = nodes.get(i) {
            if node.eq(lavalink).await {
                return Some(i);
            }
        }
    }
    None
}
