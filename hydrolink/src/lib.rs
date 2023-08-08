use std::{
    fmt::{self, Display, Formatter},
    result,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use reqwest::{
    header::{HeaderMap, InvalidHeaderValue},
    Client,
};

mod rest;
pub use rest::*;

mod websocket;
use serde::Deserialize;
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
pub use websocket::*;

mod internal;
use internal::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Event handler used by the Websocket message parser.
#[async_trait]
pub trait Handler {
    /// Triggered when the Websocket connection is established and a session ID for the REST client is received.
    async fn ready(&self, _lavalink: Lavalink, _resumed: bool) {}
    /// Triggered when the Websocket disconnects from the Lavalink server, this event actually triggers as soon as the message parser loop is finished.
    async fn disconnect(&self, _lavalink: Lavalink) {}
    /// Event triggered when a new track is started.
    async fn track_start_event(&self, _lavalink: Lavalink, _message: TrackStartEvent) {}
    /// Event triggered when a track ends.
    async fn track_end_event(&self, _lavalink: Lavalink, _message: TrackEndEvent) {}
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
    /// Lavalink server error response because of a REST call.
    RestError(ErrorResponse),
    /// Error generated by an attempt to parse the response of a request in the REST API, the first value is the error generated by `serde_json` in the attempt to parse the input in the proposed type, while the second is the error generated in the attempt to parse the input in an `ErrorResponse`.
    InvalidResponse(serde_json::Error, serde_json::Error),
    /// Error generated by trying to use the REST API without a valid connection established by Websocket.
    NotConnected,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => e.fmt(f),
            Self::WebSocket(e) => e.fmt(f),
            Self::Reqwest(e) => e.fmt(f),
            Self::InvalidHeaderValue(e) => e.fmt(f),
            Self::InvalidResponse(e1, _) => e1.fmt(f),
            Self::RestError(e) => write!(f, "rest error: {}", e.message),
            Self::NotConnected => write!(f, "lavalink websocket hasn't connected"),
        }
    }
}
/// Just a `Result` with the error type set to `hydrolink::Error`.
pub type Result<T> = result::Result<T, Error>;

/// Configuration used by the Lavalink client to connect to the server.
#[derive(Clone, PartialEq, Eq)]
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

    /// Enables the use of TLS connections.
    pub fn enable_tls(&mut self) -> &mut Self {
        self.tls = true;
        self
    }

    /// Sets the maximum wait time for receiving the session ID.
    pub fn set_connection_timeout(&mut self, timeout: u64) -> &mut Self {
        self.connection_timeout = timeout;
        self
    }

    /// Sets the time between tries to resume the connection.
    pub fn set_resume_tries(&mut self, tries: u8) -> &mut Self {
        self.resume_tries = tries;
        self
    }

    /// Sets the cooldown time between tries.
    pub fn set_resume_cooldown(&mut self, cooldown: u64) -> &mut Self {
        self.resume_cooldown = cooldown;
        self
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

/// Websocket connection status, this enum actually represents the usability and availability status of the session ID.
#[derive(Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// It means that the Websocket message parser has come to an end because the connection through the Websocket has been closed and the session ID cannot be used anymore.
    Disconnected,
    /// It means that the Websocket message parser is working but it is waiting for the Lavalink server to send the session ID.
    Connecting,
    /// It means that the Websocket message parser is working and the session ID is usable for REST calls.
    Connected,
}

/// Lavalink client used to send calls and receive messages from the Lavalink server.
#[derive(Clone)]
pub struct Lavalink {
    /// HTTP client with headers (authorization and user agent) predefined and ready to use.
    http_client: Client,
    /// Configuration used by this client to connect to the Lavalink server.
    config: Arc<LavalinkConfig>,
    /// Session ID of this connection, if the Websocket message parser received one.
    session_id: Arc<RwLock<String>>,
    /// Connection status of this Lavalink client.
    status: Arc<RwLock<ConnectionStatus>>,
    /// A write-only Websocket connection to the Lavalink server.
    connection: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    /// Resume key that can be used to resume this connection.
    resume_key: Arc<Mutex<Option<String>>>,
    /// Event handler that will be used by the Websocket message parser.
    handler: Arc<Box<dyn Handler + Sync + Send>>,
}

impl Lavalink {
    /// Initializes the connection to the Lavalink server and this struct.
    pub async fn connect<H: Handler + Sync + Send + 'static>(
        config: LavalinkConfig,
        user_id: u64,
        handler: H,
    ) -> Result<Self> {
        let client_name = format!("hydrolink/{}", VERSION);

        let http_client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Authorization",
                    config.password.parse().map_err(Error::InvalidHeaderValue)?,
                );
                headers
            })
            .user_agent(client_name.clone())
            .build()
            .map_err(Error::Reqwest)?;

        let websocket_uri = config.build_websocket_uri();

        let request = Request::builder()
            .header("Host", websocket_uri.clone())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Authorization", config.password.clone())
            .header("User-Id", user_id)
            .header("Client-Name", client_name)
            .uri(websocket_uri)
            .body(())
            .map_err(Error::Http)?;

        let (sink, stream) = connect_async(request)
            .await
            .map_err(Error::WebSocket)?
            .0
            .split();

        let lavalink = Self {
            session_id: Arc::new(RwLock::new(String::new())),
            connection: Arc::new(Mutex::new(sink)),
            config: Arc::new(config.clone()),
            resume_key: Arc::new(Mutex::new(None)),
            status: Arc::new(RwLock::new(ConnectionStatus::Connecting)),
            handler: Arc::new(Box::new(handler)),
            http_client,
        };

        let (sender, mut receiver) = oneshot::channel();

        let lavalink_clone = lavalink.clone();
        spawn(async move {
            lavalink_clone
                .websocket_message_parser(Some(sender), stream)
                .await;
        });

        select! {
            _ = sleep(Duration::from_millis(config.connection_timeout)) => {
                let mut connection = lavalink.connection.lock().unwrap();
                _ = connection.close().await;
                Err(Error::NotConnected)
            }
            msg = &mut receiver => {
                if msg.is_err() {
                    let mut connection = lavalink.connection.lock().unwrap();
                    _ = connection.close().await;
                    return Err(Error::NotConnected);
                }

                Ok(lavalink)
            }
        }
    }

    /// Gets the status of the Websocket connection.
    pub fn connection_status(&self) -> ConnectionStatus {
        self.status.read().unwrap().clone()
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
            self.session_id.read().unwrap().clone(),
            guild_id,
            no_replace
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players/{}?noReplace={}&trace=true",
            self.session_id.read().unwrap().clone(),
            guild_id,
            no_replace.to_string()
        );

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

        parse_response(&response)
    }

    /// This function is used to resolve audio tracks for use with the `update_player` function.
    pub async fn track_load(&self, identifier: &str) -> Result<TrackLoading> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!("/loadtracks?identifier={}", identifier,);

        #[cfg(feature = "lavalink-trace")]
        let path = format!("/loadtracks?identifier={}&trace=true", identifier,);

        let response = self
            .http_client
            .get(self.config.build_rest_uri(&path))
            .send()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?;

        parse_response(&response)
    }

    /// Returns the player for this guild in this session.
    pub async fn get_player(&self, guild_id: u64) -> Result<Player> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!(
            "/sessions/{}/players/{}",
            self.session_id.read().unwrap().clone(),
            guild_id
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players/{}?trace=true",
            self.session_id.read().unwrap().clone(),
            guild_id
        );

        let response = self
            .http_client
            .get(self.config.build_rest_uri(&path))
            .send()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?;

        parse_response(&response)
    }

    /// Destroys the player for this guild in this session.
    pub async fn destroy_player(&self, guild_id: u64) -> Result<()> {
        #[cfg(not(feature = "lavalink-trace"))]
        let path = format!(
            "/sessions/{}/players/{}",
            self.session_id.read().unwrap().clone(),
            guild_id
        );

        #[cfg(feature = "lavalink-trace")]
        let path = format!(
            "/sessions/{}/players/{}?trace=true",
            self.session_id.read().unwrap().clone(),
            guild_id
        );

        self.http_client
            .delete(self.config.build_rest_uri(&path))
            .send()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?;

        Ok(())
    }

    /// Parses messages coming from Websocket, triggering the handler as they are received.
    async fn websocket_message_parser(
        &self,
        mut sender: Option<oneshot::Sender<()>>,
        mut stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        while let Some(Ok(message)) = stream.next().await {
            if let Message::Text(message_str) = message {
                if let Ok(op) = serde_json::from_str::<WebsocketMessage>(&message_str) {
                    match op.op {
                        OPType::Ready => {
                            if let Ok(ready) = serde_json::from_str::<ReadyOP>(&message_str) {
                                *self.session_id.write().unwrap() = ready.session_id.clone();
                                *self.status.write().unwrap() = ConnectionStatus::Connected;

                                if let Some(some_sender) = sender {
                                    if some_sender.send(()).is_err() {
                                        break;
                                    }

                                    sender = None;
                                }

                                self.handler.ready(self.clone(), ready.resumed).await;
                            }
                        }
                        OPType::Event => {
                            if let Ok(event) = serde_json::from_str::<EventOP>(&message_str) {
                                match event.event_type {
                                    EventType::TrackStart => {
                                        if let Ok(track_start) =
                                            serde_json::from_str::<TrackStartEvent>(&message_str)
                                        {
                                            self.handler
                                                .track_start_event(self.clone(), track_start)
                                                .await;
                                        }
                                    }
                                    EventType::TrackEnd => {
                                        if let Ok(track_end) =
                                            serde_json::from_str::<TrackEndEvent>(&message_str)
                                        {
                                            self.handler
                                                .track_end_event(self.clone(), track_end)
                                                .await;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        *self.status.write().unwrap() = ConnectionStatus::Disconnected;
        self.handler.disconnect(self.clone()).await;
    }
}

impl PartialEq for Lavalink {
    fn eq(&self, other: &Self) -> bool {
        let self_session_id = self.session_id.read().unwrap().clone();
        let self_connection_status = self.status.read().unwrap().clone();
        let self_resume_key = self.resume_key.lock().unwrap().clone();
        let other_session_id = other.session_id.read().unwrap().clone();
        let other_connection_status = other.status.read().unwrap().clone();
        let other_resume_key = other.resume_key.lock().unwrap().clone();

        self_session_id == other_session_id
            && self_connection_status == other_connection_status
            && self_resume_key == other_resume_key
            && self.config == other.config
    }
}

/// Generates a new random key from 16 Base64 encoded bytes.
fn generate_key() -> String {
    let r: [u8; 16] = rand::random();
    BASE64_STANDARD.encode(r)
}

/// Attempts to parse the byte array into the selected type, if this attempt fails, a new attempt will be made parsing the input into an `ErrorResponse` which will be returned as an `Error::RestError`, if this also fails the `Error::InvalidResponse` will be returned.
fn parse_response<'a, T: Deserialize<'a>>(response: &'a [u8]) -> Result<T> {
    serde_json::from_slice::<T>(response).map_err(|e1| {
        match serde_json::from_slice::<ErrorResponse>(response) {
            Ok(v) => Error::RestError(v),
            Err(e2) => Error::InvalidResponse(e1, e2),
        }
    })
}
