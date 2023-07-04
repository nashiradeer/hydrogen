use std::{fmt::Display, result, sync::Arc};

use async_trait::async_trait;
use async_tungstenite::tokio::connect_async;
use base64::{prelude::BASE64_STANDARD, Engine};
use futures::StreamExt;
use http::{header::InvalidHeaderValue, HeaderMap, Request};
use reqwest::Client;
use serde::Deserialize;
use tokio::{sync::RwLock, spawn};
use tungstenite::Message;

use self::{websocket::{LavalinkReadyEvent, LavalinkTrackStartEvent, LavalinkTrackEndEvent}, rest::{LavalinkUpdatePlayer, LavalinkPlayer, LavalinkErrorResponse, LavalinkTrackLoading}};

pub mod rest;
pub mod websocket;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum LavalinkOpType {
    Ready,
    PlayerUpdate,
    Stats,
    Event
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LavalinkInternalOp {
    pub op: LavalinkOpType
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
enum LavalinkEventType {
    TrackStartEvent,
    TrackEndEvent,
    TrackExceptionEvent,
    TrackStuckEvent,
    WebSocketClosedEvent
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LavalinkInternalEvent {
    #[serde(rename = "type")]
    pub event_type: LavalinkEventType
}

#[async_trait]
pub trait LavalinkHandler {
    async fn lavalink_ready(&self, _node: Lavalink, _message: LavalinkReadyEvent) {}
    async fn lavalink_disconnect(&self, _node: Lavalink) {}
    async fn lavalink_track_start(&self, _node: Lavalink, _message: LavalinkTrackStartEvent) {}
    async fn lavalink_track_end(&self, _node: Lavalink, _message: LavalinkTrackEndEvent) {}
}

#[derive(Debug)]
pub enum LavalinkError {
    Http(http::Error),
    WebSocket(tungstenite::Error),
    Reqwest(reqwest::Error),
    InvalidHeaderValue(InvalidHeaderValue),
    RestError(LavalinkErrorResponse),
    NotConnected,
    InvalidResponse(serde_json::Error)
}

impl Display for LavalinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => e.fmt(f),
            Self::WebSocket(e) => e.fmt(f),
            Self::Reqwest(e) => e.fmt(f),
            Self::InvalidHeaderValue(e) => e.fmt(f),
            Self::InvalidResponse(e) => e.fmt(f),
            Self::RestError(e) => write!(f, "rest error: {}", e.message),
            Self::NotConnected => write!(f, "lavalink isn't connected")
        }
    }
}

pub type Result<T> = result::Result<T, LavalinkError>;

#[derive(Clone)]
pub struct Lavalink {
    http_client: Client,
    http_uri: String,
    websocket_uri: String,
    password: String,
    session_id: Arc<RwLock<Option<String>>>
}

impl Lavalink {
    pub fn new(uri: &str, password: &str, tls: bool) -> Result<Self> {
        let websocket_uri = format!("{}://{}/v3/websocket", match tls {
            true => "wss",
            false => "ws"
        }, uri);

        let http_uri = format!("{}://{}/v3", match tls {
            true => "https",
            false => "http"
        }, uri);

        let http_client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert("Authorization", password.parse().or_else(|e| Err(LavalinkError::InvalidHeaderValue(e)))?);
                headers
            }).user_agent("hydrogen/0.0.1").build().or_else(|e| Err(LavalinkError::Reqwest(e)))?;

        Ok(Self {
            password: password.to_owned(),
            session_id: Arc::new(RwLock::new(Option::<String>::None)),
            http_client,
            http_uri,
            websocket_uri
        })
    }

    pub async fn connect<H: LavalinkHandler + Sync + Send + 'static>(&self, user_id: &str, handler: H) -> Result<()> {
        let request = Request::builder()
            .header("Host", self.websocket_uri.clone())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Authorization", self.password.clone())
            .header("User-Id", user_id)
            .header("Client-Name", "hydrogen/0.0.1")
            .uri(self.websocket_uri.clone())
            .body(()).or_else(|e| Err(LavalinkError::Http(e)))?;

        let mut socket = connect_async(request).await.or_else(|e| Err(LavalinkError::WebSocket(e)))?.0;

        let this = self.clone();
        spawn(async move {
            while let Some(Ok(message)) = socket.next().await {
                if let Message::Text(message_str) = message {
                    if let Ok(op) = serde_json::from_str::<LavalinkInternalOp>(&message_str) {
                        match op.op {
                            LavalinkOpType::Ready => {
                                if let Ok(ready) = serde_json::from_str::<LavalinkReadyEvent>(&message_str) {
                                    *this.session_id.write().await = Some(ready.session_id.clone());
                                    handler.lavalink_ready(this.clone(), ready).await;
                                }
                            },
                            LavalinkOpType::Event => {
                                if let Ok(event) = serde_json::from_str::<LavalinkInternalEvent>(&message_str) {
                                    match event.event_type {
                                        LavalinkEventType::TrackStartEvent => {
                                            if let Ok(track_start) = serde_json::from_str::<LavalinkTrackStartEvent>(&message_str) {
                                                handler.lavalink_track_start(this.clone(), track_start).await;
                                            }
                                        },
                                        LavalinkEventType::TrackEndEvent => {
                                            if let Ok(track_end) = serde_json::from_str::<LavalinkTrackEndEvent>(&message_str) {
                                                handler.lavalink_track_end(this.clone(), track_end).await;
                                            }
                                        },
                                        _ => ()
                                    }
                                }
                            },
                            _ => ()
                        }
                    }
                }
            }

            _ = socket.close(None).await;
            *this.session_id.write().await = None;
            handler.lavalink_disconnect(this).await;
        });

        Ok(())
    }

    fn parse_response<'a, T: Deserialize<'a>>(response: &'a [u8]) -> Result<T> {
        serde_json::from_slice::<T>(&response).map_err(|_| {
            match serde_json::from_slice::<LavalinkErrorResponse>(&response) {
                Ok(v) => LavalinkError::RestError(v),
                Err(e) => LavalinkError::InvalidResponse(e)
            }
        })
    }

    pub async fn update_player(&self, guild_id: &str, no_replace: bool, player: LavalinkUpdatePlayer) -> Result<LavalinkPlayer> {
        let response = self.http_client.patch(format!(
            "{}/sessions/{}/players/{}?noReplace={}",
            self.http_uri,
            self.session_id.read().await.clone().ok_or(LavalinkError::NotConnected)?,
            guild_id,
            no_replace.to_string()
        )).json(&player).send().await.map_err(|e| LavalinkError::Reqwest(e))?
            .bytes().await.map_err(|e| LavalinkError::Reqwest(e))?;

        Self::parse_response(&response)
    }

    pub async fn track_load(&self, identifier: &str) -> Result<LavalinkTrackLoading> {
        let response = self.http_client.get(format!(
            "{}/loadtracks?identifier={}",
            self.http_uri,
            identifier
        )).send().await.map_err(|e| LavalinkError::Reqwest(e))?
            .bytes().await.map_err(|e| LavalinkError::Reqwest(e))?;

        Self::parse_response(&response)
    }

    pub async fn get_player(&self, guild_id: &str) -> Result<LavalinkPlayer> {
        let response = self.http_client.get(format!(
            "{}/sessions/{}/players/{}",
            self.http_uri,
            self.session_id.read().await.clone().ok_or(LavalinkError::NotConnected)?,
            guild_id
        )).send().await.map_err(|e| LavalinkError::Reqwest(e))?
            .bytes().await.map_err(|e| LavalinkError::Reqwest(e))?;

        Self::parse_response(&response)
    }

    pub async fn destroy_player(&self, guild_id: &str) -> Result<()> {
        self.http_client.get(format!(
            "{}/sessions/{}/players/{}",
            self.http_uri,
            self.session_id.read().await.clone().ok_or(LavalinkError::NotConnected)?,
            guild_id
        )).send().await.map_err(|e| LavalinkError::Reqwest(e))?
            .bytes().await.map_err(|e| LavalinkError::Reqwest(e))?;

        Ok(())
    }
}

pub fn generate_key() -> String {
    let r: [u8; 16] = rand::random();
    BASE64_STANDARD.encode(&r)
}