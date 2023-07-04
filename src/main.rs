use std::{env, collections::HashMap, sync::Arc, process::exit, time::Duration};

use commands::play::PlayCommand;
use i18n::HydrogenI18n;
use lavalink::{websocket::{LavalinkReadyEvent, LavalinkTrackEndEvent, LavalinkTrackStartEvent, LavalinkTrackEndReason}, LavalinkHandler, Lavalink};
use player::HydrogenPlayer;
use serenity::{prelude::{EventHandler, GatewayIntents, Context}, Client, model::{prelude::{Ready, interaction::{Interaction, application_command::ApplicationCommandInteraction}, command::Command, GuildId, ReactionType, Channel, ChannelType}, application::component::ButtonStyle, voice::VoiceState}, async_trait, builder::{CreateApplicationCommand, CreateComponents, CreateEmbedAuthor}, http::Http, client::Cache};
use songbird::SerenityInit;
use tokio::sync::RwLock;
use tracing::{error, info, debug, warn};
use tracing_subscriber::{registry, fmt::layer, layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};

mod commands;
mod i18n;
mod lavalink;
mod player;

#[derive(Clone)]
struct HydrogenContext {
    pub i18n: HydrogenI18n,
    pub players: Arc<RwLock<HashMap<GuildId, HydrogenPlayer>>>,
    pub lavalink: Lavalink
}

#[derive(Clone)]
struct HydrogenHandler {
    context: HydrogenContext,
    cache: Arc<RwLock<Option<Arc<Cache>>>>,
    http: Arc<RwLock<Option<Arc<Http>>>>,
    commands: Arc<HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>>>
}

#[async_trait]
trait HydrogenCommandListener {
    fn register<'a, 'b>(&'a self, i18n: HydrogenI18n, command: &'b mut CreateApplicationCommand) -> &'b mut CreateApplicationCommand;
    async fn execute(&self, hydrogen_context: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction);
}

#[async_trait]
impl LavalinkHandler for HydrogenHandler {
    async fn lavalink_ready(&self, _: Lavalink, _: LavalinkReadyEvent) {
        info!("lavalink initialized and connected");
    }

    async fn lavalink_disconnect(&self, _node: Lavalink) {
        error!("lavalink has disconnected");
        exit(1);
    }

    async fn lavalink_track_start(&self, _: Lavalink, message: LavalinkTrackStartEvent) {
        let guild_id = match message.guild_id.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                warn!("invalid guild id in track end event: {}", e);
                return;
            }
        };

        self.update_player_message(guild_id.into()).await;
    }

    async fn lavalink_track_end(&self, node: Lavalink, message: LavalinkTrackEndEvent) {
        if message.reason == LavalinkTrackEndReason::Finished {
            let guild_id = match message.guild_id.parse::<u64>() {
                Ok(v) => v,
                Err(e) => {
                    warn!("invalid guild id in track end event: {}", e);
                    return;
                }
            };
            if let Some(player) = self.context.players.read().await.get(&guild_id.into()) {
                if let Err(e) = player.next(node).await {
                    warn!("track end event error: {}", e);
                }

                self.update_player_message(guild_id.into()).await;
            }
        }
    }
}

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("client initialized and connected to: {}", ready.user.name);
        *self.cache.write().await = Some(ctx.cache.clone());
        *self.http.write().await = Some(ctx.http.clone());

        debug!("registering commands...");
        for (name, command) in self.commands.iter() {
            debug!("registering '{}' command...", name);
            if let Err(e) = Command::create_global_application_command(ctx.http.clone(), |create_command| {
                command.register(self.context.i18n.clone(), create_command).name(name)
            }).await {
                error!("can't register '{}' command: {}", name, e);
            }
        }

        info!("commands registered");

        debug!("connecting to lavalink server...");
        if let Err(e) = self.context.lavalink.connect(&ready.user.id.0.to_string(), self.clone()).await {
            error!("can't connect to the lavalink server: {}", e);
            exit(1);
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let command_name = command.data.name.as_str();
                debug!("executing application command: {}", command_name);

                if let Some(listener) = self.commands.get(command_name) {
                    listener.execute(self.context.clone(), ctx, command).await;
                }
                else {
                    warn!("unknown command: {}", command_name);
                }
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, ctx: Context, _: Option<VoiceState>, new: VoiceState) {
        if let Err(e) = self._voice_state_update(ctx, new).await {
            warn!("voice state update: {}", e);
        }
    }
}

impl HydrogenHandler {
    async fn _update_play_message(&self, player: HydrogenPlayer, http: Arc<Http>, description: &str, disable_comps: bool, color: i32, author_obj: Option<CreateEmbedAuthor>) {
        if let Some(message_id) = player.message_id.read().await.clone() {
            match player.text_channel_id().edit_message(http.clone(), message_id, |message|
                message
                    .embed(|embed| {
                        if let Some(author_obj) = author_obj.clone() {
                            embed.set_author(author_obj);
                        }

                        embed
                            .title(self.context.i18n.translate(&player.guild_locale(), "playing", "title"))
                            .description(description)
                            .color(color)
                            .footer(|footer|
                                footer
                                    .text(self.context.i18n.translate(&player.guild_locale(), "embed", "footer_text"))
                                    .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                            )
                    })
                    .set_components(Self::play_components(disable_comps))
            ).await {
                Ok(_) => return,
                Err(e) => {
                    warn!("can't edit player message: {}", e);
                }
            }
        }

        match player.text_channel_id().send_message(http.clone(), |message|
            message
                .embed(|embed| {
                    if let Some(author_obj) = author_obj {
                        embed.set_author(author_obj);
                    }

                    embed
                        .title(self.context.i18n.translate(&player.guild_locale(), "playing", "title"))
                        .description(description)
                        .color(color)
                        .footer(|footer|
                            footer
                                .text(self.context.i18n.translate(&player.guild_locale(), "embed", "footer_text"))
                                .icon_url("https://gitlab.com/uploads/-/system/project/avatar/45361202/hydrogen_icon.png")
                        )
                })
                .set_components(Self::play_components(disable_comps))
        ).await {
            Ok(v) => *player.message_id.write().await = Some(v.id),
            Err(e) => warn!("can't send a new playing message: {}", e)
        };
    }

    async fn _voice_state_update(&self, ctx: Context, new: VoiceState) -> Result<(), String> {
        if new.user_id != ctx.cache.current_user().id {
            let Some(guild_id) = new.guild_id else {
                return Ok(());
            };

            let Some(player) = self.context.players.read().await.get(&guild_id).cloned() else {
                return Ok(());
            };

            let Some(connection_info) = player.call().await.lock().await.current_connection().cloned() else {
                return Ok(());
            };

            let Some(channel_id) = connection_info.channel_id else {
                return Ok(());
            };

            let Some(Channel::Guild(channel)) = ctx.cache.channel(channel_id.0) else {
                return Ok(());
            };

            if channel.kind == ChannelType::Voice || channel.kind == ChannelType::Stage {
                let members_count = channel.members(&ctx.cache).await.map_err(|e| format!("can't get voice channel member count: {}", e))?.len();
                
                if members_count <= 1 {
                    player.timed_destroy(self.context.lavalink.clone(), Duration::from_secs(10)).await;
                    self._update_play_message(player.clone(), ctx.http, &self.context.i18n.translate(&player.guild_locale(), "playing", "timeout_trigger"), true, 0x5865f2, None).await;
                } else {
                    player.cancel_destroy().await;
                    self.update_player_message(guild_id).await;
                }
            }
        }

        Ok(())
    }

    pub async fn update_player_message(&self, guild_id: GuildId) {
        if let Some(http) = self.http.read().await.clone() {
            let cache_http = self.cache.read().await.clone().map(|v| (v, http.clone()));

            if let Some(player) = self.context.players.read().await.get(&guild_id.into()) {
                let (translated_message, requester) = match player.now().await {
                    Some(v) => {
                        let message = match v.uri {
                            Some(v) => self.context.i18n.translate(&player.guild_locale(), "playing", "description_uri")
                                        .replace("${uri}", &v),
                            None => self.context.i18n.translate(&player.guild_locale(), "playing", "description")
                        }
                            .replace("${music}", &v.title)
                            .replace("${author}", &v.author);

                        (message, Some(v.requester_id))
                    },
                    None => (
                        self.context.i18n.translate(&player.guild_locale(), "playing", "empty"),
                        None
                    )
                };

                let mut author_obj: Option<CreateEmbedAuthor> = None;
                if let Some(author) = requester {
                    let result_author_user = match cache_http {
                        Some(v) => author.to_user((&v.0, v.1.as_ref())).await,
                        None => author.to_user(http.clone()).await
                    };

                    if let Ok(author_user) = result_author_user {
                        let mut inner_author_obj = CreateEmbedAuthor::default();

                        inner_author_obj.name(author_user.name.clone());

                        if let Some(avatar_url) = author_user.avatar_url() {
                            inner_author_obj.icon_url(avatar_url);
                        }

                        author_obj = Some(inner_author_obj.to_owned());
                    }
                }

                self._update_play_message(player.clone(), http, &translated_message, requester.is_none(), 0x5865f2, author_obj).await;
            }
        } else {
            error!("http client not initialized");
        }
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
}

#[tokio::main]
async fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("starting up...");

    debug!("initializing i18n...");
    let i18n = {
        let path = env::var("LANGUAGE_PATH").expect("you need to set LANGUAGE_PATH environment variable");
        HydrogenI18n::new(path, HydrogenI18n::DEFAULT_LANGUAGE)
    }.expect("can't initialize i18n");

    debug!("initializing lavalink...");
    let lavalink = {
        let uri = env::var("LAVALINK_URL").expect("you need to set LAVALINK_URL environment variable");
        let password = env::var("LAVALINK_PASSWORD").expect("you need to set LAVALINK_PASSWORD environment variable");
        let tls = env::var("LAVALINK_TLS").unwrap_or_default().to_lowercase();

        Lavalink::new(&uri, &password, tls == "true" || tls == "enabled" || tls == "on").expect("can't initialize lavalink")
    };

    debug!("initializing handler...");
    let app = HydrogenHandler {
        context: HydrogenContext {
            players: Arc::new(RwLock::new(HashMap::new())),
            lavalink,
            i18n
        },
        cache: Arc::new(RwLock::new(None)),
        http: Arc::new(RwLock::new(None)),
        commands: {
            let mut commands: HashMap<String, Box<dyn HydrogenCommandListener + Sync + Send>> =  HashMap::new();
            
            commands.insert("play".to_owned(), Box::new(PlayCommand));

            Arc::new(commands)
        }
    };

    debug!("initializing client...");
    Client::builder(env::var("DISCORD_TOKEN").expect("you need to set DISCORD_TOKEN environment variable"), GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES)
        .event_handler(app)
        .register_songbird()
        .await.expect("can't initialize the client")
        .start().await.expect("can't start the client");
}
