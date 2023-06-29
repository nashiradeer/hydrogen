use std::{sync::Arc, result, fmt::Display};

use async_tungstenite::{tokio::{connect_async, TokioAdapter}, WebSocketStream, stream::Stream};
use futures::{StreamExt, lock::Mutex};
use http::Request;
use serde::Deserialize;
use async_trait::async_trait;
use tokio::{spawn, net::TcpStream};
use tokio_rustls::client::TlsStream;
use tungstenite::Message;

#[async_trait]
pub trait HydrogenLavalinkHandler {
    async fn lavalink_ready(&self, _message: LavalinkSocketReady) {}
}

pub enum HydrogenLavalinkError {
    Http(http::Error),
    WebSocket(tungstenite::Error)
}

impl Display for HydrogenLavalinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HydrogenLavalinkError::Http(e) => write!(f, "{}", e),
            HydrogenLavalinkError::WebSocket(e) => write!(f, "{}", e)
        }
    }
}

pub type Result<T> = result::Result<T, HydrogenLavalinkError>;

#[derive(Clone)]
pub struct HydrogenLavalink {
    uri: String,
    socket: Arc<Mutex<WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>>>,
    handler: Arc<Box<dyn HydrogenLavalinkHandler + Sync + Send>>
}

impl HydrogenLavalink {
    pub async fn new<H: HydrogenLavalinkHandler + Sync + Send + 'static>(uri: &str, password: &str, user_id: &str, handler: H) -> Result<Self> {
        let request = match Request::builder()
            .header("Authorization", password)
            .header("User-Id", user_id)
            .header("Client-Name", "hydrogen/0.0.1")
            .uri(format!("{}/v3/websocket", uri))
            .body(()) {
                Ok(v) => v,
                Err(e) => return Err(HydrogenLavalinkError::Http(e))
            };

        let socket = Arc::new(Mutex::new(match connect_async(request).await {
            Ok(v) => v.0,
            Err(e) => return Err(HydrogenLavalinkError::WebSocket(e))
        }));

        let this = Self {
            uri: uri.to_owned(),
            handler: Arc::new(Box::new(handler)),
            socket
        };

        let this_clone = this.clone();
        spawn(async move {
            while let Some(Ok(message)) = this_clone.socket.lock().await.next().await {
                if let Message::Text(message_str) = message {
                    if let Ok(op) = serde_json::from_str::<LavalinkSocketOp>(&message_str) {
                        match op.op {
                            LavalinkSocketOpType::Ready => {
                                if let Ok(ready) = serde_json::from_str::<LavalinkSocketReady>(&message_str) {
                                    this_clone.handler.lavalink_ready(ready).await;
                                }
                            },
                            _ => ()
                        }
                    }
                }
            }
        });

        Ok(this)
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