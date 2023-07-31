use std::{
    fmt::{self, Display, Formatter},
    result,
    sync::{Arc, Mutex, RwLock},
};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, InvalidHeaderValue},
    Client,
};

mod rest;
pub use rest::*;

mod websocket;
use tokio::{select, spawn, sync::oneshot};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        self,
        http::{self, Request},
    },
};
pub use websocket::*;

mod internal;
use internal::*;

/// Event handler used by the Websocket message parser.
#[async_trait]
pub trait Handler {
    /// Triggered when the Websocket connection is established and a session ID for the REST client is received.
    async fn ready(&self, _lavalink: Lavalink, _resumed: bool) {}
    /// Triggered when the Websocket disconnects from the Lavalink server, this event actually triggers as soon as the message parser loop is finished.
    async fn disconnect(&self, _lavalink: Lavalink) {}
    /// Event triggered when a new track is started.
    async fn track_start_event(&self, _lavalink: Lavalink, _message: LavalinkTrackStartEvent) {}
    /// Event triggered when a track ends.
    async fn track_end_event(&self, _lavalink: Lavalink, _message: LavalinkTrackEndEvent) {}
}

/// Enum that groups all the errors that can occur.
#[derive(Debug)]
pub enum Error {
    /// Generic HTTP errors produced using `http` crate.
    Http(http::Error),
    WebSocket(tungstenite::Error),
    Reqwest(reqwest::Error),
    InvalidHeaderValue(InvalidHeaderValue),
    RestError(LavalinkErrorResponse),
    InvalidResponse(serde_json::Error),
    NotConnected,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => e.fmt(f),
            Self::WebSocket(e) => e.fmt(f),
            Self::Reqwest(e) => e.fmt(f),
            Self::InvalidHeaderValue(e) => e.fmt(f),
            Self::InvalidResponse(e) => e.fmt(f),
            Self::RestError(e) => write!(f, "rest error: {}", e.message),
            Self::NotConnected => write!(f, "lavalink websocket hasn't connected before timeout"),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, PartialEq, Eq)]
pub enum Connection {
    Disconnected,
    Connecting,
    Connected,
}

pub struct NodeInfo {
    pub host: String,
    pub password: String,
    pub tls: bool,
    pub timeout: u64,
}

impl NodeInfo {
    pub fn new(host: &str, password: &str) -> Self {
        Self {
            host: host.to_owned(),
            password: password.to_owned(),
            tls: false,
            timeout: 5000,
        }
    }

    pub fn enable_tls(&mut self) -> &mut Self {
        self.tls = true;
        self
    }

    pub fn timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Clone)]
pub struct Lavalink {
    http_client: Client,
    tls: bool,
    host: Arc<String>,
    session_id: Arc<RwLock<String>>,
    status: Arc<RwLock<Connection>>,
    connection: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl Lavalink {
    pub async fn connect<H: Handler + Sync + Send + 'static>(
        node: NodeInfo,
        user_id: u64,
        handler: H,
    ) -> Result<Self> {
        let websocket_uri = format!(
            "{}://{}/v3/websocket",
            match node.tls {
                true => "wss",
                false => "ws",
            },
            node.host
        );

        let http_client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Authorization",
                    node.password
                        .parse()
                        .or_else(|e| Err(Error::InvalidHeaderValue(e)))?,
                );
                headers
            })
            .user_agent("hydrogen/0.0.1")
            .build()
            .or_else(|e| Err(Error::Reqwest(e)))?;

        let request = Request::builder()
            .header("Host", websocket_uri.clone())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Authorization", node.password.clone())
            .header("User-Id", user_id)
            .header("Client-Name", "hydrogen/0.0.1")
            .uri(websocket_uri)
            .body(())
            .or_else(|e| Err(Error::Http(e)))?;

        let (mut sink, stream) = connect_async(request)
            .await
            .or_else(|e| Err(Error::WebSocket(e)))?
            .0
            .split();

        let lavalink = Self {
            session_id: Arc::new(RwLock::new(String::new())),
            host: Arc::new(node.host),
            connected: Arc::new(RwLock::new(Connection::Connecting)),
            tls: node.tls,
            // connection: Arc::new(Mutex::new(sink)),
            http_client,
        };

        let (sender, mut receiver) = oneshot::channel();

        let lavalink_clone = lavalink.clone();
        spawn(async move {
            read_socket(handler, lavalink_clone, Some(sender), stream).await;
        });

        select! {
            _ = sleep(Duration::from_millis(LAVALINK_CONNECTION_TIMEOUT)) => {
                _ = sink.close().await;
                Err(LavalinkError::NotConnected)
            }
            msg = &mut receiver => {
                if let Err(_) = msg {
                    _ = sink.close().await;
                    return Err(LavalinkError::NotConnected);
                }

                Ok(lavalink)
            }
        }
    }

    pub async fn connected(&self) -> LavalinkConnection {
        self.connected.read().await.clone()
    }

    pub async fn update_player(
        &self,
        guild_id: u64,
        no_replace: bool,
        player: &LavalinkUpdatePlayer,
    ) -> Result<LavalinkPlayer> {
        let response = self
            .http_client
            .patch(format!(
                "{}://{}/v3/sessions/{}/players/{}?noReplace={}",
                match self.tls {
                    true => "https",
                    false => "http",
                },
                self.host,
                self.session_id.read().await.clone(),
                guild_id,
                no_replace.to_string()
            ))
            .json(&player)
            .send()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?
            .bytes()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?;

        parse_response(&response)
    }

    pub async fn track_load(&self, identifier: &str) -> Result<LavalinkTrackLoading> {
        let response = self
            .http_client
            .get(format!(
                "{}://{}/v3/loadtracks?identifier={}",
                match self.tls {
                    true => "https",
                    false => "http",
                },
                self.host,
                identifier
            ))
            .send()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?
            .bytes()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?;

        parse_response(&response)
    }

    pub async fn get_player(&self, guild_id: u64) -> Result<LavalinkPlayer> {
        let response = self
            .http_client
            .get(format!(
                "{}://{}/v3/sessions/{}/players/{}",
                match self.tls {
                    true => "https",
                    false => "http",
                },
                self.host,
                self.session_id.read().await.clone(),
                guild_id
            ))
            .send()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?
            .bytes()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?;

        parse_response(&response)
    }

    pub async fn destroy_player(&self, guild_id: u64) -> Result<()> {
        self.http_client
            .delete(format!(
                "{}://{}/v3/sessions/{}/players/{}",
                match self.tls {
                    true => "https",
                    false => "http",
                },
                self.host,
                self.session_id.read().await.clone(),
                guild_id
            ))
            .send()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?
            .bytes()
            .await
            .map_err(|e| LavalinkError::Reqwest(e))?;

        Ok(())
    }

    pub async fn eq(&self, other: &Self) -> bool {
        self.session_id.read().await.clone() == other.session_id.read().await.clone()
            && self.host == other.host
            && self.connected.read().await.clone() == other.connected.read().await.clone()
    }
}

async fn read_socket<H: LavalinkHandler + Sync + Send + 'static>(
    handler: H,
    origin: Lavalink,
    mut sender: Option<oneshot::Sender<()>>,
    mut stream: SplitStream<
        WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
    >,
) {
    while let Some(Ok(message)) = stream.next().await {
        if let Message::Text(message_str) = message {
            if let Ok(op) = serde_json::from_str::<LavalinkInternalOp>(&message_str) {
                match op.op {
                    LavalinkOpType::Ready => {
                        if let Ok(ready) = serde_json::from_str::<LavalinkReadyEvent>(&message_str)
                        {
                            origin
                                .session_id
                                .write()
                                .await
                                .replace_range(.., &ready.session_id);
                            *origin.connected.write().await = LavalinkConnection::Connected;

                            if let Some(some_sender) = sender {
                                if let Err(_) = some_sender.send(()) {
                                    break;
                                }

                                sender = None;
                            }

                            handler.lavalink_ready(origin.clone(), ready.resumed).await;
                        }
                    }
                    LavalinkOpType::Event => {
                        if let Ok(event) =
                            serde_json::from_str::<LavalinkInternalEvent>(&message_str)
                        {
                            match event.event_type {
                                LavalinkEventType::TrackStartEvent => {
                                    if let Ok(track_start) =
                                        serde_json::from_str::<LavalinkTrackStartEvent>(
                                            &message_str,
                                        )
                                    {
                                        handler
                                            .lavalink_track_start(origin.clone(), track_start)
                                            .await;
                                    }
                                }
                                LavalinkEventType::TrackEndEvent => {
                                    if let Ok(track_end) =
                                        serde_json::from_str::<LavalinkTrackEndEvent>(&message_str)
                                    {
                                        handler.lavalink_track_end(origin.clone(), track_end).await;
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
    *origin.connected.write().await = LavalinkConnection::Disconnected;
    handler.lavalink_disconnect(origin).await;
}

fn generate_key() -> String {
    let r: [u8; 16] = rand::random();
    BASE64_STANDARD.encode(&r)
}

fn parse_response<'a, T: Deserialize<'a>>(response: &'a [u8]) -> Result<T> {
    serde_json::from_slice::<T>(&response).map_err(|_| {
        match serde_json::from_slice::<LavalinkErrorResponse>(&response) {
            Ok(v) => LavalinkError::RestError(v),
            Err(e) => LavalinkError::InvalidResponse(e),
        }
    })
}
