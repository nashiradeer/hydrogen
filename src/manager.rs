use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, collections::HashMap, fmt::Display, result, time::Duration, process::exit};

use async_trait::async_trait;
use serenity::{client::Cache, http::{Http, CacheHttp}, model::{prelude::{GuildId, Channel, ChannelType, ReactionType, component::ButtonStyle, UserId, ChannelId}, voice::VoiceState}, builder::{CreateEmbedAuthor, CreateComponents}};
use songbird::Call;
use tokio::{sync::{RwLock, Mutex}, spawn, time::sleep};
use tracing::{info, error, warn};

use crate::{i18n::HydrogenI18n, lavalink::{Lavalink, LavalinkError, LavalinkHandler, websocket::{LavalinkTrackStartEvent, LavalinkTrackEndEvent, LavalinkTrackEndReason}, LavalinkNodeInfo}, player::{HydrogenPlayer, HydrogenPlayCommand, HydrogenPlayerError}};

#[derive(Debug)]
pub enum HydrogenManagerError {
    Lavalink(LavalinkError),
    Serenity(serenity::Error),
    Player(HydrogenPlayerError),
    WithoutNodes
}

impl Display for HydrogenManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lavalink(e) => e.fmt(f),
            Self::Serenity(e) => e.fmt(f),
            Self::Player(e) => e.fmt(f),
            Self::WithoutNodes => write!(f, "there's no lavalink nodes connected")
        }
    }
}

pub type Result<T> = result::Result<T, HydrogenManagerError>;

#[derive(Clone)]
pub struct HydrogenManager {
    cache: Arc<Cache>,
    http: Arc<Http>,
    i18n: HydrogenI18n,
    lavalink: Arc<RwLock<Vec<Lavalink>>>,
    load_balancer: Arc<AtomicUsize>,
    player: Arc<RwLock<HashMap<GuildId, HydrogenPlayer>>>
}

impl HydrogenManager {
    pub fn new(cache: Arc<Cache>, http: Arc<Http>, i18n: HydrogenI18n) -> Self {
        Self {
            lavalink: Arc::new(RwLock::new(Vec::new())),
            load_balancer: Arc::new(AtomicUsize::new(0)),
            player: Arc::new(RwLock::new(HashMap::new())),
            cache,
            http,
            i18n
        }
    }

    pub async fn connect_lavalink(&self, node: LavalinkNodeInfo) -> Result<()> {
        self.lavalink.write().await.push(Lavalink::connect(node, self.cache.current_user().id.0, self.clone()).await.map_err(|e| HydrogenManagerError::Lavalink(e))?);
        Ok(())
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

    pub async fn new_or_play(&self, guild_id: GuildId, guild_locale: &str, music: &str, requester_id: UserId, text_channel_id: ChannelId, call: Arc<Mutex<Call>>) -> Result<HydrogenPlayCommand> {
        let player_option = self.player.read().await.get(&guild_id).cloned();
        if let Some(player) = player_option {
            return Ok(player.play(music, requester_id).await.map_err(|e| HydrogenManagerError::Player(e))?);
        } else {
            let lavalink = self.lavalink.read().await.get(self.increment_load_balancer().await).cloned().ok_or(HydrogenManagerError::WithoutNodes)?;
            let player = HydrogenPlayer::new(lavalink, guild_id, text_channel_id, guild_locale, call);
            self.player.write().await.insert(guild_id, player.clone());
            return Ok(player.play(music, requester_id).await.map_err(|e| HydrogenManagerError::Player(e))?);
        }
    }

    pub async fn voice_state_update(&self, new: VoiceState) -> Result<()> {
        if new.user_id != self.cache.current_user().id {
            let Some(guild_id) = new.guild_id else {
                return Ok(());
            };
            
            let Some(player) = self.player.read().await.get(&guild_id).cloned() else {
                return Ok(());
            };

            let Some(connection_info) = player.call().await.lock().await.current_connection().cloned() else {
                return Ok(());
            };

            let Some(channel_id) = connection_info.channel_id else {
                return Ok(());
            };

            let Some(Channel::Guild(channel)): Option<Channel> = self.cache.channel(channel_id.0) else {
                return Ok(());
            };

            if channel.kind == ChannelType::Voice || channel.kind == ChannelType::Stage {
                let members_count = channel.members(self.cache.clone()).await.map_err(|e| HydrogenManagerError::Serenity(e))?.len();
                
                if members_count <= 1 {
                    self.timed_destroy(guild_id, Duration::from_secs(10)).await;
                    self.update_play_message(player.clone(), &self.i18n.translate(&player.guild_locale(), "playing", "timeout_trigger"), 0x5865f2, true, None).await;
                } else {
                    self.cancel_destroy(guild_id).await;
                    self.update_now_playing(guild_id).await;
                }
            }
        }

        Ok(())
    }

    pub async fn timed_destroy(&self, guild_id: GuildId, duration: Duration) {
        let self_cloned = self.clone();
        let player_option = self.player.read().await.get(&guild_id).cloned();
        if let Some(player) = player_option {
            let player_cloned = player.clone();
            let mut player_abort = player.destroy_handle.write().await;
            if player_abort.is_none() {
                *player_abort = Some(Arc::new(spawn(async move {
                    sleep(duration).await;
                    self_cloned.player.write().await.remove(&guild_id);
                    _ = player_cloned.destroy().await;
                })));
            }
        }
    }

    pub async fn cancel_destroy(&self, guild_id: GuildId) {
        if let Some(player) = self.player.read().await.get(&guild_id) {
            let mut handle_guard = player.destroy_handle.write().await;
            if let Some(handle) = handle_guard.clone() {
                handle.abort();
                *handle_guard = None;
            }
        }
    }

    async fn update_now_playing(&self, guild_id: GuildId) {
        if let Some(player) = self.player.read().await.get(&guild_id.into()) {
            let (translated_message, requester) = match player.now().await {
                Some(v) => {
                    let message = match v.uri {
                        Some(v) => self.i18n.translate(&player.guild_locale(), "playing", "description_uri")
                                    .replace("${uri}", &v),
                        None => self.i18n.translate(&player.guild_locale(), "playing", "description")
                    }
                        .replace("${music}", &v.title)
                        .replace("${author}", &v.author);

                    (message, Some(v.requester_id))
                },
                None => (
                    self.i18n.translate(&player.guild_locale(), "playing", "empty"),
                    None
                )
            };

            let mut author_obj = None;
            if let Some(author) = requester {
                if let Ok(author_user) = author.to_user(self).await {
                    let mut inner_author_obj = CreateEmbedAuthor::default();

                    inner_author_obj.name(author_user.name.clone());

                    if let Some(avatar_url) = author_user.avatar_url() {
                        inner_author_obj.icon_url(avatar_url);
                    }

                    author_obj = Some(inner_author_obj.to_owned());
                }
            }

            self.update_play_message(player.clone(), &translated_message, 0x5865f2, requester.is_none(), author_obj).await;
        }
    }

    async fn update_play_message(&self, player: HydrogenPlayer, description: &str, color: i32, disable_buttons: bool, author_obj: Option<CreateEmbedAuthor>) {
        if let Some(message_id) = player.message_id.read().await.clone() {
            match player.text_channel_id().edit_message(self.http.clone(), message_id, |message|
                message
                    .embed(|embed| {
                        if let Some(author_obj) = author_obj.clone() {
                            embed.set_author(author_obj);
                        }

                        embed
                            .title(self.i18n.translate(&player.guild_locale(), "playing", "title"))
                            .description(description)
                            .color(color)
                            .footer(|footer|
                                footer
                                    .text(self.i18n.translate(&player.guild_locale(), "embed", "footer_text"))
                                    .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                            )
                    })
                    .set_components(Self::play_components(disable_buttons))
            ).await {
                Ok(_) => return,
                Err(e) => {
                    warn!("can't edit player message: {}", e);
                }
            }
        }

        match player.text_channel_id().send_message(self.http.clone(), |message|
            message
                .embed(|embed| {
                    if let Some(author_obj) = author_obj {
                        embed.set_author(author_obj);
                    }

                    embed
                        .title(self.i18n.translate(&player.guild_locale(), "playing", "title"))
                        .description(description)
                        .color(color)
                        .footer(|footer|
                            footer
                                .text(self.i18n.translate(&player.guild_locale(), "embed", "footer_text"))
                                .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                        )
                })
                .set_components(Self::play_components(disable_buttons))
        ).await {
            Ok(v) => *player.message_id.write().await = Some(v.id),
            Err(e) => warn!("can't send a new playing message: {}", e)
        };
    }

    pub fn play_components(disable_all: bool) -> CreateComponents {
        CreateComponents::default()
            .create_action_row(|action_row|
                action_row
                    .create_button(|button|
                        button
                            .custom_id("prev")
                            .disabled(disable_all)
                            .emoji('⏮')
                            .style(ButtonStyle::Secondary)
                    )
                    .create_button(|button|
                        button
                            .custom_id("pause")
                            .disabled(disable_all)
                            .emoji('⏸')
                            .style(ButtonStyle::Secondary)
                    )
                    .create_button(|button|
                        button
                            .custom_id("skip")
                            .disabled(disable_all)
                            .emoji('⏭')
                            .style(ButtonStyle::Secondary)
                    )
            )
            .create_action_row(|action_row|
                action_row
                    .create_button(|button|
                        button
                            .custom_id("loop")
                            .disabled(disable_all)
                            .emoji(ReactionType::Unicode("⤵️".to_owned()))
                            .style(ButtonStyle::Secondary)
                    )
                    .create_button(|button|
                        button
                            .custom_id("stop")
                            .disabled(disable_all)
                            .emoji('⏹')
                            .style(ButtonStyle::Secondary)
                    )
                    .create_button(|button|
                        button
                            .custom_id("queue")
                            .disabled(disable_all)
                            .emoji(ReactionType::Unicode("ℹ️".to_owned()))
                            .style(ButtonStyle::Secondary)
                    )
            ).to_owned()
    }

    async fn find_lavalink(&self, lavalink: &Lavalink) -> Option<usize> {
        let nodes = self.lavalink.read().await;
        for i in 0..nodes.len() {
            if let Some(node) = nodes.get(i) {
                if node.eq(lavalink).await {
                    return Some(i);
                }
            }
        }
        None
    }
}

#[async_trait]
impl LavalinkHandler for HydrogenManager {
    async fn lavalink_ready(&self, node: Lavalink, _: bool) {
        if let Some(index) = self.find_lavalink(&node).await {
            info!("managed lavalink {} initialized and connected", index);
        } else {
            info!("unknown lavalink initialized and connected");
        }
    }

    async fn lavalink_disconnect(&self, node: Lavalink) {
        if let Some(index) = self.find_lavalink(&node).await {
            warn!("managed lavalink {} has disconnected", index);
        } else {
            warn!("unknown lavalink has disconnected");
        }

        if self.lavalink.read().await.len() == 0 {
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
    }

    async fn lavalink_track_start(&self, _: Lavalink, message: LavalinkTrackStartEvent) {
        let guild_id = match message.guild_id.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                warn!("invalid guild id in track start event: {}", e);
                return;
            }
        };

        self.update_now_playing(guild_id.into()).await;
    }

    async fn lavalink_track_end(&self, _: Lavalink, message: LavalinkTrackEndEvent) {
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