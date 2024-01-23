use std::{collections::HashMap, env, process::exit, sync::Arc, time::Instant};

use async_trait::async_trait;
use handler::register_commands;
use hydrogen_i18n::I18n;
use lavalink::LavalinkNodeInfo;
use manager::HydrogenManager;
use parsers::TimeParser;
use serenity::{
    all::{
        Client, CommandId, ComponentInteraction, GatewayIntents, Interaction, Ready,
        VoiceServerUpdateEvent, VoiceState,
    },
    client::{Context, EventHandler},
};
use songbird::SerenityInit;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter,
};

use crate::handler::{handle_command, handle_component};

mod commands;
mod components;
mod handler;
mod lavalink;
mod manager;
mod parsers;
mod player;
mod utils;

pub const HYDROGEN_PRIMARY_COLOR: i32 = 0x5865f2;
pub const HYDROGEN_ERROR_COLOR: i32 = 0xf04747;
pub const HYDROGEN_EMPTY_CHAT_TIMEOUT: u64 = 10;
pub const HYDROGEN_QUEUE_LIMIT: usize = 1000;
pub const HYDROGEN_SEARCH_PREFIX: &str = "ytsearch:";
pub const LAVALINK_CONNECTION_TIMEOUT: u64 = 5000;

pub static HYDROGEN_LOGO_URL: &str =
    "https://raw.githubusercontent.com/nashiradeer/hydrogen/main/icon.png";
pub static HYDROGEN_BUG_URL: &str = "https://github.com/nashiradeer/hydrogen/issues";

#[cfg(feature = "builtin-language")]
/// Default language file already loaded in the binary.
pub static HYDROGEN_DEFAULT_LANGUAGE: &str = include_str!("../assets/langs/en-US.json");

#[derive(Clone)]
struct HydrogenContext {
    pub i18n: Arc<I18n>,
    pub manager: Arc<RwLock<Option<HydrogenManager>>>,

    /// Parsers used to parse different time syntaxes.
    pub time_parsers: Arc<TimeParser>,

    pub commands_id: Arc<RwLock<HashMap<String, CommandId>>>,
}

#[derive(Clone)]
struct HydrogenHandler {
    context: HydrogenContext,
    lavalink_nodes: Arc<Vec<LavalinkNodeInfo>>,
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
                panic!("cannot register commands");
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
            if let Err(e) = manager.update_voice_state(old, new).await {
                warn!("(voice_state_update): cannot update the HydrogenManager's player voice state: {}", e);
            }
        }

        info!(
            "(voice_state_update): processed in {}ms",
            timer.elapsed().as_millis()
        );
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        let timer = Instant::now();
        debug!("(voice_server_update): processing...");

        let option_manager = self.context.manager.read().await.clone();
        if let Some(manager) = option_manager {
            if let Err(e) = manager.update_voice_server(voice_server).await {
                warn!("(voice_server_update): cannot update HydrogenManager's player voice server: {}", e);
            }
        }

        info!(
            "(voice_server_update): processed in {}ms...",
            timer.elapsed().as_millis()
        );
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

    // Load a possible new default language from HYDROGEN_DEFAULT_LANGUAGE environment variable.
    let new_default_language = env::var("DEFAULT_LANGUAGE");

    // Load language files from HYDROGEN_LANGUAGE_PATH environment variable.
    if let Ok(language_path) = env::var("LANGUAGE_PATH") {
        if let Err(e) =
            i18n.from_dir_with_links(language_path, false, new_default_language.is_err())
        {
            warn!("cannot load language files: {}", e);
        } else {
            i18n.cleanup_links();
        }
    }

    // Set a new default language if the environment variable is set.
    if let Ok(default_language) = new_default_language {
        if !i18n.set_default(&default_language, true) {
            error!("cannot set default language to '{}'", default_language);
        }
        // TODO: deduplicate loaded language when hydrogen_i18n supports it.
    }

    // Initialize lavalink nodes.
    let lavalink_nodes = {
        let mut lavalink_nodes = Vec::new();
        let lavalink_env =
            env::var("LAVALINK").expect("you need to set LAVALINK environment variable");

        for single_node in lavalink_env.split(";") {
            let mut node_components = single_node.split(",");
            let Some(host) = node_components.next() else {
                break;
            };

            let password = node_components
                .next()
                .expect("lavalink node doesn't have a password set");
            let tls = node_components.next().unwrap_or("");

            lavalink_nodes.push(LavalinkNodeInfo {
                host: host.to_owned(),
                password: password.to_owned(),
                tls: tls == "true" || tls == "enabled" || tls == "on",
            });
        }

        if lavalink_nodes.len() == 0 {
            panic!("at least one lavalink node is required to work");
        }

        Arc::new(lavalink_nodes)
    };

    // Initialize time parsers.
    let time_parsers = Arc::new(match TimeParser::new() {
        Ok(v) => v,
        Err(e) => {
            error!("cannot initialize time parsers: {}", e);
            panic!("cannot initialize time parsers");
        }
    });

    // Initialize HydrogenHandler.
    let app = HydrogenHandler {
        context: HydrogenContext {
            manager: Arc::new(RwLock::new(None)),
            commands_id: Arc::new(RwLock::new(HashMap::new())),
            i18n: Arc::new(i18n),
            time_parsers,
        },
        lavalink_nodes,
    };

    Client::builder(
        env::var("DISCORD_TOKEN").expect("you need to set DISCORD_TOKEN environment variable"),
        GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
    )
    .event_handler(app)
    .register_songbird()
    .await
    .expect("cannot initialize client")
    .start()
    .await
    .expect("cannot start client");
}
