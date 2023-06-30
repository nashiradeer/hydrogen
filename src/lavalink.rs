use std::{sync::Arc, result, fmt::Display};

use async_tungstenite::tokio::connect_async;
use base64::{prelude::BASE64_STANDARD, Engine};
use futures::StreamExt;
use http::{Request, HeaderMap, header::InvalidHeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tokio::{spawn, sync::RwLock};
use tungstenite::Message;

#[async_trait]
pub trait HydrogenLavalinkHandler {
    async fn lavalink_ready(&self, _message: LavalinkSocketReady) {}
}

#[derive(Debug)]
pub enum HydrogenLavalinkError {
    Http(http::Error),
    WebSocket(tungstenite::Error),
    Reqwest(reqwest::Error),
    InvalidHeaderValue(InvalidHeaderValue),
    NotInitialized
}

impl Display for HydrogenLavalinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HydrogenLavalinkError::Http(e) => write!(f, "{}", e),
            HydrogenLavalinkError::WebSocket(e) => write!(f, "{}", e),
            HydrogenLavalinkError::Reqwest(e) => write!(f, "{}", e),
            HydrogenLavalinkError::InvalidHeaderValue(e) => write!(f, "{}", e),
            HydrogenLavalinkError::NotInitialized => write!(f, "lavalink session not initialized")
        }
    }
}

pub type Result<T> = result::Result<T, HydrogenLavalinkError>;

pub struct HydrogenLavalink {
    http_client: Client,
    http_uri: String,
    websocket_uri: String,
    password: String,
    session_id: Arc<RwLock<Option<String>>>
}

impl HydrogenLavalink {
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
                headers.insert("Authorization", password.parse().or_else(|e| Err(HydrogenLavalinkError::InvalidHeaderValue(e)))?);
                headers
            }).user_agent("hydrogen/0.0.1").build().or_else(|e| Err(HydrogenLavalinkError::Reqwest(e)))?;

        Ok(Self {
            password: password.to_owned(),
            session_id: Arc::new(RwLock::new(Option::<String>::None)),
            http_client,
            http_uri,
            websocket_uri
        })
    }

    pub async fn init<H: HydrogenLavalinkHandler + Sync + Send + 'static>(&self, user_id: &str, handler: H) -> Result<()> {
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
            .body(()).or_else(|e| Err(HydrogenLavalinkError::Http(e)))?;

        let mut socket = connect_async(request).await.or_else(|e| Err(HydrogenLavalinkError::WebSocket(e)))?.0;

        let session_id = self.session_id.clone();
        spawn(async move {
            while let Some(Ok(message)) = socket.next().await {
                if let Message::Text(message_str) = message {
                    if let Ok(op) = serde_json::from_str::<LavalinkSocketOp>(&message_str) {
                        match op.op {
                            LavalinkSocketOpType::Ready => {
                                if let Ok(ready) = serde_json::from_str::<LavalinkSocketReady>(&message_str) {
                                    *session_id.write().await = Some(ready.session_id.clone());
                                    handler.lavalink_ready(ready).await;
                                }
                            },
                            _ => ()
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn update_player(&self, guild_id: &str, no_replace: bool, player: LavalinkRestUpdatePlayer) -> Result<LavalinkRestPlayer> {
        self.http_client.patch(format!(
            "{}/sessions/{}/players/{}?noReplace={}",
            self.http_uri,
            self.session_id.read().await.clone().ok_or(HydrogenLavalinkError::NotInitialized)?,
            guild_id,
            no_replace.to_string()
        )).json(&player).send().await.or_else(|e| Err(HydrogenLavalinkError::Reqwest(e)))?
            .json::<LavalinkRestPlayer>().await.map_err(|e| HydrogenLavalinkError::Reqwest(e))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LavalinkSocketOpType {
    Ready,
    PlayerUpdate,
    Stats,
    Event
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LavalinkSocketOp {
    pub op: LavalinkSocketOpType
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkSocketReady {
    pub resumed: bool,
    pub session_id: String
}


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkRestVoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: String,

    #[serde(skip_serializing)]
    pub connected: bool,
    #[serde(skip_serializing)]
    pub ping: i32
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkRestUpdatePlayer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_track: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<LavalinkRestVoiceState>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkRestPlayer {
    pub guild_id: String,
    pub track: Option<LavalinkRestTrack>,
    pub volume: i32,
    pub paused: bool,
    pub voice: LavalinkRestVoiceState
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkRestTrack {
    pub encoded: String,
    pub track: String,
    pub info: LavalinkRestTrackInfo
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkRestTrackInfo {
    pub identifier: String,
    pub is_seekable: bool,
    pub author: String,
    pub length: i32,
    pub is_stream: bool,
    pub position: i32,
    pub title: String,
    pub uri: Option<String>,
    pub source_name: String
}

pub fn generate_key() -> String {
    let r: [u8; 16] = rand::random();
    BASE64_STANDARD.encode(&r)
}