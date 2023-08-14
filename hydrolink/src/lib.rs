use std::{
    fmt::{self, Display, Formatter},
    result,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use const_format::concatcp;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use reqwest::{
    header::{HeaderMap, InvalidHeaderValue},
    Client,
};
use tokio::{net::TcpStream, select, spawn, sync::oneshot, time::sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        self,
        http::{self, Request},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};

mod common;
mod filters;
mod internal;
mod parser;
mod rest;
mod websocket;

use parser::*;

pub use common::*;
pub use filters::*;
pub use rest::*;
pub use websocket::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const CLIENT_NAME: &str = concatcp!("hydrolink/", VERSION);

/// Event handler used by the Websocket message parser.
#[async_trait]
pub trait Handler {
    /// Triggered when the Websocket connection is established and a session ID for the REST client is received.
    async fn ready(&self, _lavalink: Lavalink, _resumed: bool) {}
    /// Triggered when the Websocket disconnects from the Lavalink server, this event actually triggers as soon as the message parser loop is finished.
    async fn disconnect(&self, _lavalink: Lavalink) {}
    /// Triggered every x seconds with the current state of the player.
    async fn player_update(&self, _lavalink: Lavalink, _player_update: PlayerUpdate) {}
    /// Triggered every minute with stats from the Lavalink server.
    async fn stats(&self, _lavalink: Lavalink, _stats: Stats) {}
    /// Event triggered when a new track is started.
    async fn track_start_event(&self, _lavalink: Lavalink, _message: TrackStartEvent) {}
    /// Event triggered when a track ends.
    async fn track_end_event(&self, _lavalink: Lavalink, _message: TrackEndEvent) {}
    /// Event triggered when an exception/error occurs while playing a track.
    async fn track_exception_event(&self, _lavalink: Lavalink, _message: TrackExceptionEvent) {}
    /// Triggered when track is stuck.
    async fn track_stuck_event(&self, _lavalink: Lavalink, _message: TrackStuckEvent) {}
    /// Triggered when the connection between the Lavalink server and Discord is closed, either for normal or abnormal reasons.
    async fn websocket_closed_event(&self, _lavalink: Lavalink, _message: WebSocketClosedEvent) {}
}

/// Enum that groups all the errors that can occur.
#[derive(Debug)]
pub enum Error {
    /// Generic HTTP errors produced using `http` crate.
    Http(http::Error),
    /// Websocket errors generated by `tungstenite` crate
    WebSocket(tungstenite::Error),
    /// REST client errors generated by `reqwest` crate
    Reqwest(reqwest::Error),
    /// Error that can be generated when building REST and Websocket client headers.
    InvalidHeaderValue(InvalidHeaderValue),
    /// Lavalink server error response because of a REST call, and the parsing error, if the call has any response.
    RestError(ErrorResponse, Option<serde_json::Error>),
    /// Error generated by an attempt to parse the response of a request in the REST API, the first value is the error generated by `serde_json` in the attempt to parse the input in the proposed type, while the second is the error generated in the attempt to parse the input in an `ErrorResponse`.
    InvalidResponse(Option<serde_json::Error>, serde_json::Error),
    /// Error generated by trying to use the REST API without a valid connection established by Websocket.
    NotConnected,
    /// Error generated by trying to resume an already connected Lavalink session.
    AlreadyConnected,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => e.fmt(f),
            Self::WebSocket(e) => e.fmt(f),
            Self::Reqwest(e) => e.fmt(f),
            Self::InvalidHeaderValue(e) => e.fmt(f),
            Self::RestError(e1, _) => write!(f, "rest error: {}", e1.message),
            Self::NotConnected => write!(f, "lavalink websocket hasn't connected"),
            Self::AlreadyConnected => write!(f, "lavalink is already connected"),
            Self::InvalidResponse(e1, e2) => match e1 {
                Some(e) => e.fmt(f),
                None => e2.fmt(f),
            },
        }
    }
}
/// Just a `Result` with the error type set to `hydrolink::Error`.
pub type Result<T> = result::Result<T, Error>;

/// Configuration used by the Lavalink client to connect to the server.
#[derive(Clone)]
pub struct LavalinkConfig {
    /// Lavalink server IP address.
    pub host: String,
    /// Lavalink server password.
    pub password: String,
    /// Enables the use of client-server TLS connections.
    pub tls: bool,
    /// Sets the maximum wait time for receiving the session ID.
    pub connection_timeout: u64,
    /// Sets the time between tries to resume the connection.
    pub resume_tries: u8,
    /// Sets the cooldown time between tries.
    pub resume_cooldown: u64,
}

impl LavalinkConfig {
    /// Initializes a new configuration with the required parameters.
    pub fn new(host: &str, password: &str) -> Self {
        Self {
            host: host.to_owned(),
            password: password.to_owned(),
            tls: false,
            connection_timeout: 5000,
            resume_tries: 3,
            resume_cooldown: 2000,
        }
    }

    #[cfg(not(feature = "lavalink-v4"))]
    // Builds a URI that can be used to access the Lavalink server's Websocket using the parameters in this configuration.
    pub fn build_websocket_uri(&self) -> String {
        format!(
            "{}://{}/v3/websocket",
            match self.tls {
                true => "wss",
                false => "ws",
            },
            self.host
        )
    }

    #[cfg(feature = "lavalink-v4")]
    // Builds a URI that can be used to access the Lavalink server's Websocket using the parameters in this configuration.
    pub fn build_websocket_uri(&self) -> String {
        format!(
            "{}://{}/v4/websocket",
            match self.tls {
                true => "wss",
                false => "ws",
            },
            self.host
        )
    }

    #[cfg(not(feature = "lavalink-v4"))]
    /// Builds a URI that can be used to make REST calls to the Lavalink server.
    pub fn build_rest_uri(&self, api_call: &str) -> String {
        format!(
            "{}://{}/v3{}",
            match self.tls {
                true => "https",
                false => "http",
            },
            self.host,
            api_call,
        )
    }

    #[cfg(feature = "lavalink-v4")]
    /// Builds a URI that can be used to make REST calls to the Lavalink server.
    pub fn build_rest_uri(&self, api_call: &str) -> String {
        format!(
            "{}://{}/v4{}",
            match self.tls {
                true => "https",
                false => "http",
            },
            self.host,
            api_call,
        )
    }
}

/// Lavalink client used to send calls and receive messages from the Lavalink server.
#[derive(Clone)]
pub struct Lavalink {
    /// HTTP client with headers (authorization and user agent) predefined and ready to use.
    http_client: Client,
    /// Configuration used by this client to connect to the Lavalink server.
    config: Arc<LavalinkConfig>,
    /// Session ID of this connection, if the Websocket message parser received one.
    session_id: Arc<RwLock<Option<String>>>,
    /// A write-only Websocket connection to the Lavalink server.
    connection: Arc<
        tokio::sync::Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    >,
    /// Resume key that can be used to resume this connection.
    resume_key: Arc<Mutex<Option<String>>>,
    /// Event handler that will be used by the Websocket message parser.
    handler: Arc<Box<dyn Handler + Sync + Send>>,
    /// User ID used to connect to Websocket.
    user_id: u64,
}

impl Lavalink {
    /// Create a new instance of this struct without connecting.
    pub fn new<H: Handler + Sync + Send + 'static>(
        config: LavalinkConfig,
        user_id: u64,
        handler: H,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Authorization",
                    config.password.parse().map_err(Error::InvalidHeaderValue)?,
                );
                headers
            })
            .user_agent(CLIENT_NAME)
            .build()
            .map_err(Error::Reqwest)?;

        Ok(Self {
            config: Arc::new(config),
            handler: Arc::new(Box::new(handler)),
            resume_key: Arc::new(Mutex::new(None)),
            session_id: Arc::new(RwLock::new(None)),
            connection: Arc::new(tokio::sync::Mutex::new(None)),
            http_client,
            user_id,
        })
    }

    /// Initializes the connection to the Lavalink server.
    pub async fn connect(&self) -> Result<()> {
        if self.session_id.read().unwrap().is_some() {
            return Err(Error::AlreadyConnected);
        }

        let websocket_uri = self.config.build_websocket_uri();

        let request = Request::builder()
            .header("Host", websocket_uri.clone())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Authorization", self.config.password.clone())
            .header("User-Id", self.user_id)
            .header("Client-Name", CLIENT_NAME)
            .uri(websocket_uri.clone())
            .body(())
            .map_err(Error::Http)?;

        debug!(
            "connecting to the Lavalink websocket in '{}'...",
            websocket_uri
        );

        let (sink, stream) = connect_async(request)
            .await
            .map_err(Error::WebSocket)?
            .0
            .split();

        debug!("Lavalink websocket connected.");

        *self.connection.lock().await = Some(sink);

        let (sender, mut receiver) = oneshot::channel();

        let websocket_lavalink = self.clone();
        spawn(async move {
            debug!("starting the websocket message parser.");
            websocket_message_parser(websocket_lavalink, Some(sender), stream).await;
        });

        debug!("waiting the session confirmation...");
        select! {
            _ = sleep(Duration::from_millis(self.config.connection_timeout)) => {
                warn!("session confirmation timeout, closing connection...");

                if let Err(e) = self.connection.lock().await.as_mut().unwrap().close().await {
                    error!("websocket connection can't be closed: {}", e);
                }

                Err(Error::NotConnected)
            }
            msg = &mut receiver => match msg {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("session confirmation channel has been dropped: {}", e);

                    if let Err(e) = self.connection.lock().await.as_mut().unwrap().close().await {
                        error!("websocket connection can't be closed: {}", e);
                    }

                    Err(Error::NotConnected)
                }
            }
        }
    }

    /// Check if the Websocket is connected.
    pub fn is_connected(&self) -> bool {
        self.session_id.read().unwrap().is_some()
    }

    /// Check if the connection is resumable.
    pub fn is_resumable(&self) -> bool {
        self.resume_key.lock().unwrap().is_some()
    }

    /// Updates or creates the player for this guild if it doesn't already exist.
    pub async fn update_player(
        &self,
        guild_id: u64,
        no_replace: bool,
        player: UpdatePlayer,
    ) -> Result<Player> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!(
            "/sessions/{}/players/{}?noReplace={}",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
            guild_id,
            no_replace
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players/{}?noReplace={}&trace=true",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
            guild_id,
            no_replace.to_string()
        );

        debug!("calling '{}'...", path);

        let response = self
            .http_client
            .patch(self.config.build_rest_uri(&path))
            .json(&player)
            .send()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?;

        info!("parsing the response from '{}'...", path);

        parse_response(&response)
    }

    /// This function is used to resolve audio tracks for use with the `update_player` function.
    pub async fn track_load(&self, identifier: &str) -> Result<TrackLoading> {
        track_load(self.config.as_ref().clone(), identifier).await
    }

    /// Returns the player for this guild in this session.
    pub async fn get_player(&self, guild_id: u64) -> Result<Player> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!(
            "/sessions/{}/players/{}",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
            guild_id
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players/{}?trace=true",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
            guild_id
        );

        debug!("calling '{}'...", path);

        let response = self
            .http_client
            .get(self.config.build_rest_uri(&path))
            .send()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?;

        info!("parsing the response from '{}'...", path);

        parse_response(&response)
    }

    /// Returns all players in this session.
    pub async fn get_players(&self) -> Result<Vec<Player>> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!(
            "/sessions/{}/players",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players?trace=true",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
        );

        debug!("calling '{}'...", path);

        let response = self
            .http_client
            .get(self.config.build_rest_uri(&path))
            .send()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?;

        info!("parsing the response from '{}'...", path);

        parse_response(&response)
    }

    /// Destroys the player for this guild in this session.
    pub async fn destroy_player(&self, guild_id: u64) -> Result<()> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!(
            "/sessions/{}/players/{}",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
            guild_id
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players/{}?trace=true",
            self.session_id
                .read()
                .unwrap()
                .clone()
                .ok_or(Error::NotConnected)?,
            guild_id
        );

        debug!("calling '{}'...", path);

        let response = self
            .http_client
            .delete(self.config.build_rest_uri(&path))
            .send()
            .await
            .map_err(Error::Reqwest)?;

        info!("parsing the response from '{}'...", path);

        if !response.status().is_success() {
            warn!("response haven't a success status code.");

            return Err(
                serde_json::from_slice(&response.bytes().await.map_err(Error::Reqwest)?)
                    .map(|v| Error::RestError(v, None))
                    .map_err(|e| Error::InvalidResponse(None, e))?,
            );
        }

        Ok(())
    }
}

/// This function is used to resolve audio tracks for use with the `update_player` function.
pub async fn track_load(config: LavalinkConfig, identifier: &str) -> Result<TrackLoading> {
    #[cfg(not(feature = "lavalink-trace"))]
    let path = format!("/loadtracks?identifier={}", identifier,);

    #[cfg(feature = "lavalink-trace")]
    let path = format!("/loadtracks?identifier={}&trace=true", identifier,);

    debug!("calling '{}'...", path);

    let http_client = Client::builder()
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(
                "Authorization",
                config.password.parse().map_err(Error::InvalidHeaderValue)?,
            );
            headers
        })
        .user_agent(CLIENT_NAME)
        .build()
        .map_err(Error::Reqwest)?;

    let response = http_client
        .get(config.build_rest_uri(&path))
        .send()
        .await
        .map_err(Error::Reqwest)?
        .bytes()
        .await
        .map_err(Error::Reqwest)?;

    info!("parsing the response from '{}'...", path);

    parse_response(&response)
}

/// Generates a new random key from 16 Base64 encoded bytes.
fn generate_key() -> String {
    let r: [u8; 16] = rand::random();
    BASE64_STANDARD.encode(r)
}
