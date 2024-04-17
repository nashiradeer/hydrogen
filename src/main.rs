use std::{collections::HashMap, env, process::exit, sync::Arc, time::Instant};

use async_trait::async_trait;
use config::load_configuration;
use dashmap::DashMap;
use handler::{register_commands, AutoRemoverKey};
use hydrogen_i18n::I18n;
use lavalink::LavalinkNodeInfo;
use manager::HydrogenManager;
use parsers::{RollParser, TimeParser};
use serenity::{
    all::{
        Client, CommandId, ComponentInteraction, GatewayIntents, Interaction, Message, Ready,
        ShardId, UserId, VoiceServerUpdateEvent, VoiceState,
    },
    client::{Context, EventHandler},
    gateway::ShardRunnerInfo,
    prelude::TypeMapKey,
};
use songbird::SerenityInit;
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter,
};

use crate::handler::{handle_command, handle_component};

mod commands;
mod components;
mod config;
mod handler;
mod lavalink;
mod manager;
mod parsers;
mod player;
mod roll;
mod utils;

pub const HYDROGEN_PRIMARY_COLOR: i32 = 0x5865f2;
pub const HYDROGEN_ERROR_COLOR: i32 = 0xf04747;
pub const HYDROGEN_EMPTY_CHAT_TIMEOUT: u64 = 10;
pub const HYDROGEN_QUEUE_LIMIT: usize = 1000;
pub const HYDROGEN_SEARCH_PREFIX: &str = "ytsearch:";
pub const HYDROGEN_WARNING_TIMEOUT: u64 = 10;
pub const HYDROGEN_WARNING_PROBABILITY: f64 = 0.1;
pub const HYDROGEN_COLOR: i32 = 0x009b60;
pub const LAVALINK_CONNECTION_TIMEOUT: u64 = 5000;
/// The public instance ID.
pub const HYDROGEN_PUBLIC_INSTANCE_ID: u64 = 1128087591179268116;

pub static HYDROGEN_LOGO_URL: &str =
    "https://raw.githubusercontent.com/nashiradeer/hydrogen/main/assets/icons/hydrogen-circular.png";
pub static HYDROGEN_BUG_URL: &str = "https://github.com/nashiradeer/hydrogen/issues";

/// Hydrogen version.
pub static HYDROGEN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Hydrogen repository URL.
pub static HYDROGEN_REPOSITORY_URL: &str = "https://github.com/nashiradeer/hydrogen";

/// Hydrogen's project name.
pub static HYDROGEN_NAME: &str = "Hydrogen";

#[cfg(feature = "builtin-language")]
/// Default language file already loaded in the binary.
pub static HYDROGEN_DEFAULT_LANGUAGE: &str = include_str!("../assets/langs/en-US.json");

/// The public instance and other roll bots IDs.
pub static OTHER_ROLL_BOTS: [u64; 1] = [
    // Rollem bot ID.
    240732567744151553,
];

#[derive(Clone)]
struct HydrogenContext {
    pub i18n: Arc<I18n>,
    pub manager: Arc<RwLock<Option<HydrogenManager>>>,

    /// Parsers used to parse different time syntaxes.
    pub time_parsers: Arc<TimeParser>,

    /// Parser used to parse rolls.
    pub roll_parser: Arc<RollParser>,

    pub commands_id: Arc<RwLock<HashMap<String, CommandId>>>,

    /// The responses from the components.
    pub components_responses: Arc<DashMap<AutoRemoverKey, (JoinHandle<()>, ComponentInteraction)>>,
    /// Whether this is the public instance.
    pub public_instance: bool,
}

#[derive(Clone)]
struct HydrogenHandler {
    context: HydrogenContext,
    lavalink_nodes: Arc<Vec<LavalinkNodeInfo>>,
    /// Other roll bots IDs.
    other_roll_bots: Vec<u64>,
    /// If the bot should force enable auto-roll from messages.
    force_roll: bool,
}

#[async_trait]
trait HydrogenComponentListener {
    async fn execute(
        &self,
        hydrogen: HydrogenContext,
        context: Context,
        interaction: ComponentInteraction,
    );
}

/// A key for the shard manager runners in the TypeMap.
pub struct ShardManagerRunners;

impl TypeMapKey for ShardManagerRunners {
    type Value = Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>;
}

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let timer = Instant::now();
        debug!("(ready): processing...");

        let manager = HydrogenManager::new(
            ctx.cache.clone(),
            ctx.http.clone(),
            self.context.i18n.clone(),
        );
        *self.context.manager.write().await = Some(manager.clone());
        debug!("(ready): HydrogenManager initialized");

        if !register_commands(
            Some(&self.context.i18n),
            &ctx.http,
            &self.context.commands_id,
        )
        .await
        {
            warn!("(ready): cannot register commands, retrying without translations...");

            if !register_commands(None, &ctx.http, &self.context.commands_id).await {
                error!("(ready): cannot register commands");
                exit(1);
            }
        }

        for i in 0..self.lavalink_nodes.len() {
            if let Some(node) = self.lavalink_nodes.get(i) {
                if let Err(e) = manager.connect_lavalink(node.clone()).await {
                    error!("(ready): cannot connect to the lavalink node {}: {}", i, e);
                }
            }
        }

        if manager.lavalink_node_count().await == 0 {
            error!("(ready): no lavalink nodes connected.");
            exit(1);
        }

        info!(
            "(ready): connected to {} lavalink nodes",
            manager.lavalink_node_count().await
        );

        info!(
            "(ready): client connected to '{}' in {}ms",
            ready.user.name,
            timer.elapsed().as_millis()
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let timer = Instant::now();
        debug!("(interaction_create): processing...");

        match interaction {
            Interaction::Command(command) => {
                handle_command(&self.context, &ctx, &command).await;

                info!(
                    "(interaction_create): command '{}' executed in {}ms",
                    command.data.name,
                    timer.elapsed().as_millis()
                );
            }
            Interaction::Component(component) => {
                handle_component(&self.context, &ctx, &component).await;

                info!(
                    "(interaction_create): component '{}' executed in {}ms",
                    component.data.custom_id,
                    timer.elapsed().as_millis()
                );
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, _: Context, old: Option<VoiceState>, new: VoiceState) {
        let timer = Instant::now();
        debug!("(voice_state_update): processing...");

        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            match manager.update_voice_state(old, new).await {
                Ok(updated) => {
                    if updated {
                        info!(
                            "(voice_state_update): processed in {}ms...",
                            timer.elapsed().as_millis()
                        );
                    } else {
                        debug!("(voice_state_update): ignored");
                    }
                }
                Err(e) => {
                    warn!("(voice_state_update): cannot update the HydrogenManager's player voice state: {}", e);
                }
            }
        }
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        let timer = Instant::now();
        debug!("(voice_server_update): processing...");

        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            match manager.update_voice_server(voice_server).await {
                Ok(updated) => {
                    if updated {
                        info!(
                            "(voice_server_update): processed in {}ms...",
                            timer.elapsed().as_millis()
                        );
                    } else {
                        debug!("(voice_server_update): ignored");
                    }
                }
                Err(e) => {
                    warn!("(voice_server_update): cannot update HydrogenManager's player voice server: {}", e);
                }
            }
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        // Start the execution timer.
        let timer = Instant::now();
        debug!("(message): processing...");

        // Ignore messages from bots.
        if message.author.bot {
            debug!("(message): message from bot, ignored");
            return;
        }

        // Ignore messages from other roll bots.
        if !self.force_roll {
            if let Some(guild_id) = message.guild_id {
                let mut other_roll_bot = None;
                for id in &self.other_roll_bots {
                    if let Ok(member) = ctx.http.get_member(guild_id, UserId::new(*id)).await {
                        other_roll_bot = Some(member);
                        break;
                    }
                }

                if let Some(member) = other_roll_bot {
                    warn!(
                        "(message): other roll bot detected, ignored: {} ({})",
                        &member.user.name, &member.user.id
                    );
                    return;
                }
            }
        }

        // Send message to the roll parser.
        if let Some(params) = self.context.roll_parser.evaluate(&message.content) {
            match params.roll() {
                Ok(result) => {
                    if let Err(e) = message.reply_ping(ctx, result.to_string()).await {
                        warn!("(message): cannot send roll result: {}", e);
                    }
                }
                Err(e) => {
                    warn!(
                        "(message): cannot roll for user {}: {}",
                        message.author.id, e
                    );
                }
            };

            info!("(message): processed in {}ms", timer.elapsed().as_millis());
        } else {
            debug!("(message): ignored");
        }
    }
}

#[cfg(not(feature = "builtin-language"))]
/// Create a new i18n instance.
#[inline]
fn new_i18n() -> I18n {
    I18n::new()
}

#[cfg(feature = "builtin-language")]
/// Create a new i18n instance with default language if can be parsed.
#[inline]
fn new_i18n() -> I18n {
    if let Ok(default_language) = hydrogen_i18n::serde_json::from_str(HYDROGEN_DEFAULT_LANGUAGE) {
        I18n::new_with_default(default_language)
    } else {
        I18n::new()
    }
}

/// Executable entrypoint.
#[tokio::main]
async fn main() {
    // Initialize logger.
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Initialize i18n with default language if can be parsed.
    let mut i18n = new_i18n();

    // Load configuration from file or environment.
    let mut config = load_configuration().or_from_env();

    if config.public_instance.unwrap_or_default() {
        warn!("you are running this instance as a public instance");
    }

    // Load language files.
    if let Some(language_path) = config.language_path {
        if let Err(e) =
            i18n.from_dir_with_links(language_path, false, config.default_language.is_none())
        {
            warn!("cannot load language files: {}", e);
        } else {
            i18n.cleanup_links();
        }
    }

    // Set a new default language if the environment variable is set.
    if let Some(default_language) = config.default_language {
        if !i18n.set_default(&default_language, true) {
            error!("cannot set default language to '{}'", default_language);
        }
        // TODO: deduplicate loaded language when hydrogen_i18n supports it.
    }

    // Initialize time parsers.
    let time_parsers = Arc::new(match TimeParser::new() {
        Ok(v) => v,
        Err(e) => {
            error!("cannot initialize time parsers: {}", e);
            exit(1)
        }
    });

    let roll_parser = Arc::new(match RollParser::new() {
        Ok(v) => v,
        Err(e) => {
            error!("cannot initialize roll parser: {}", e);
            exit(1);
        }
    });

    // Get lavalink nodes.
    let lavalink_nodes = config
        .lavalink
        .take()
        .unwrap()
        .into_iter()
        .map(LavalinkNodeInfo::from)
        .collect();

    let mut other_roll_bots = Vec::from(OTHER_ROLL_BOTS);
    if !config.public_instance.unwrap_or_default() {
        other_roll_bots.push(HYDROGEN_PUBLIC_INSTANCE_ID);
    }

    // Initialize HydrogenHandler.
    let app = HydrogenHandler {
        context: HydrogenContext {
            manager: Arc::new(RwLock::new(None)),
            commands_id: Arc::new(RwLock::new(HashMap::new())),
            i18n: Arc::new(i18n),
            components_responses: Arc::new(DashMap::new()),
            public_instance: config.public_instance.unwrap_or_default(),
            time_parsers,
            roll_parser,
        },
        lavalink_nodes: Arc::new(lavalink_nodes),
        other_roll_bots,
        force_roll: config.force_roll.unwrap_or_default(),
    };

    let mut client = Client::builder(
        &config.discord_token.unwrap(),
        GatewayIntents::GUILDS
            | GatewayIntents::GUILD_VOICE_STATES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(app)
    .register_songbird()
    .await
    .expect("cannot initialize client");

    client
        .data
        .write()
        .await
        .insert::<ShardManagerRunners>(client.shard_manager.runners.clone());

    client.start().await.expect("cannot start client");
}
