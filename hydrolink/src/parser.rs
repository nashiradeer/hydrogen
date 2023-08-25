use std::any::type_name;

use futures::{stream::SplitStream, StreamExt};
use serde::Deserialize;
use tokio::{net::TcpStream, sync::oneshot};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::{
    internal::{Event, EventType, OPType, WebsocketMessage},
    Error, ErrorResponse, PlayerUpdate, Ready, Result, Session, Stats, TrackEndEvent,
    TrackExceptionEvent, TrackStartEvent, TrackStuckEvent, WebSocketClosedEvent,
};

/// Parses messages coming from Websocket, triggering the handler as they are received.
pub async fn websocket_message_parser(
    lavalink: Session,
    mut sender: Option<oneshot::Sender<()>>,
    mut stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    info!("WEBSOCKET [START]: websocket message parser initialized");
    while let Some(value) = stream.next().await {
        let message = match value {
            Ok(v) => {
                debug!("WEBSOCKET [MESSAGE]: {}", v);
                v
            }
            Err(e) => {
                error!("WEBSOCKET [ERROR]: {}", e);
                break;
            }
        };

        if let Message::Text(message_str) = message {
            let op = match serde_json::from_str::<WebsocketMessage>(&message_str) {
                Ok(v) => v,
                Err(e) => {
                    warn!("WEBSOCKET [JSON/ERROR]: {}", e);
                    continue;
                }
            };

            match op.op {
                OPType::Ready => {
                    let ready = match serde_json::from_str::<Ready>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("WEBSOCKET [READY/ERROR]: {}", e);
                            continue;
                        }
                    };

                    *lavalink.session_id.write().unwrap() = Some(ready.session_id.clone());

                    if let Some(some_sender) = sender {
                        if some_sender.send(()).is_err() {
                            debug!("WEBSOCKET [READY/ERROR]: can't confirm the session ID");
                            break;
                        }

                        sender = None;
                    }

                    debug!("WEBSOCKET [READY]: {}", &message_str);
                    lavalink
                        .handler
                        .ready(lavalink.clone(), ready.resumed)
                        .await;
                }
                OPType::PlayerUpdate => {
                    let player_update = match serde_json::from_str::<PlayerUpdate>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("WEBSOCKET [PLAYER_UPDATE/ERROR]: {}", e);
                            continue;
                        }
                    };

                    debug!("WEBSOCKET [PLAYER_UPDATE]: {}", &message_str);
                    lavalink
                        .handler
                        .player_update(lavalink.clone(), player_update)
                        .await;
                }
                OPType::Stats => {
                    let stats = match serde_json::from_str::<Stats>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("WEBSOCKET [STATS/ERROR]: {}", e);
                            continue;
                        }
                    };

                    debug!("WEBSOCKET [STATS]: {}", &message_str);
                    lavalink.handler.stats(lavalink.clone(), stats).await;
                }
                OPType::Event => {
                    let event = match serde_json::from_str::<Event>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("WEBSOCKET [EVENT/ERROR]: {}", e);
                            continue;
                        }
                    };

                    match event.event_type {
                        EventType::TrackStart => {
                            let track_start =
                                match serde_json::from_str::<TrackStartEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("WEBSOCKET [TRACK_START/ERROR]: {}", e);
                                        continue;
                                    }
                                };

                            debug!("WEBSOCKET [TRACK_START]: {}", &message_str);
                            lavalink
                                .handler
                                .track_start_event(lavalink.clone(), track_start)
                                .await;
                        }
                        EventType::TrackEnd => {
                            let track_end =
                                match serde_json::from_str::<TrackEndEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("WEBSOCKET [TRACK_END/ERROR]: {}", e);
                                        continue;
                                    }
                                };

                            debug!("WEBSOCKET [TRACK_END]: {}", &message_str);
                            lavalink
                                .handler
                                .track_end_event(lavalink.clone(), track_end)
                                .await;
                        }
                        EventType::TrackException => {
                            let track_exception =
                                match serde_json::from_str::<TrackExceptionEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("WEBSOCKET [TRACK_EXCEPTION/ERROR]: {}", e);
                                        continue;
                                    }
                                };

                            debug!("WEBSOCKET [TRACK_EXCEPTION]: {}", &message_str);
                            lavalink
                                .handler
                                .track_exception_event(lavalink.clone(), track_exception)
                                .await;
                        }
                        EventType::TrackStuck => {
                            let track_stuck =
                                match serde_json::from_str::<TrackStuckEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("WEBSOCKET [TRACK_STUCK/ERROR]: {}", e);
                                        continue;
                                    }
                                };

                            debug!("WEBSOCKET [TRACK_STUCK]: {}", &message_str);
                            lavalink
                                .handler
                                .track_stuck_event(lavalink.clone(), track_stuck)
                                .await;
                        }
                        EventType::WebSocketClosed => {
                            let websocket_closed =
                                match serde_json::from_str::<WebSocketClosedEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("WEBSOCKET [WEBSOCKET_CLOSED/ERROR]: {}", e);
                                        continue;
                                    }
                                };

                            debug!("WEBSOCKET [WEBSOCKET_CLOSED]: {}", &message_str);
                            lavalink
                                .handler
                                .websocket_closed_event(lavalink.clone(), websocket_closed)
                                .await;
                        }
                    }
                }
            }
        } else {
            warn!("WEBSOCKET [UNKNOWN]: skipping message...");
        }
    }

    *lavalink.session_id.write().unwrap() = None;
    *lavalink.connection.lock().await = None;

    lavalink.handler.disconnect(lavalink.clone()).await;
    info!("WEBSOCKET [END]: session ID and connection cleaned");
}

/// Attempts to parse the byte array into the selected type, if this attempt fails, a new attempt will be made parsing the input into an `ErrorResponse` which will be returned as an `Error::RestError`, if this also fails the `Error::InvalidResponse` will be returned.
pub fn parse_response<'a, T: Deserialize<'a>>(response: &'a [u8]) -> Result<T> {
    serde_json::from_slice::<T>(response).map_err(|e1| {
        warn!("RESPONSE [{}]: {}", type_name::<T>(), e1);

        match serde_json::from_slice::<ErrorResponse>(response) {
            Ok(v) => Error::RestError(v, Some(e1)),
            Err(e2) => {
                error!("RESPONSE [ErrorResponse]: {}", e2);

                Error::InvalidResponse(Some(e1), e2)
            }
        }
    })
}
