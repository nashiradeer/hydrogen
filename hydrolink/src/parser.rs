use std::any::type_name;

use futures::{stream::SplitStream, StreamExt};
use serde::Deserialize;
use tokio::{net::TcpStream, sync::oneshot};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::{
    internal::{Event, EventType, OPType, WebsocketMessage},
    ConnectionStatus, Error, ErrorResponse, Lavalink, PlayerUpdate, Ready, Result, Stats,
    TrackEndEvent, TrackExceptionEvent, TrackStartEvent, TrackStuckEvent, WebSocketClosedEvent,
};

/// Parses messages coming from Websocket, triggering the handler as they are received.
pub async fn websocket_message_parser(
    lavalink: Lavalink,
    mut sender: Option<oneshot::Sender<()>>,
    mut stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    while let Some(value) = stream.next().await {
        let message = match value {
            Ok(v) => {
                debug!("parsing the websocket message: {}", v);
                v
            }
            Err(e) => {
                error!("websocket generated an error: {}", e);
                break;
            }
        };

        if let Message::Text(message_str) = message {
            let op = match serde_json::from_str::<WebsocketMessage>(&message_str) {
                Ok(v) => v,
                Err(e) => {
                    warn!("can't parse the message: {}", e);
                    continue;
                }
            };

            match op.op {
                OPType::Ready => {
                    info!("op: ready");
                    let ready = match serde_json::from_str::<Ready>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("can't parse the ready message: {}", e);
                            continue;
                        }
                    };

                    *lavalink.session_id.write().unwrap() = ready.session_id.clone();
                    *lavalink.status.write().unwrap() = ConnectionStatus::Connected;
                    debug!("updated the Lavalink session id and status.");

                    if let Some(some_sender) = sender {
                        if some_sender.send(()).is_err() {
                            error!("can't send the session confirmation...");
                            break;
                        }
                        info!("session confirmation has been sent.");

                        sender = None;
                    }

                    debug!("emitting 'ready' in the event handler...");
                    lavalink
                        .handler
                        .ready(lavalink.clone(), ready.resumed)
                        .await;
                }
                OPType::PlayerUpdate => {
                    info!("op: player update");
                    let player_update = match serde_json::from_str::<PlayerUpdate>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("can't parse the playerUpdate message: {}", e);
                            continue;
                        }
                    };

                    debug!("emitting 'player_update' in the event handler...");
                    lavalink
                        .handler
                        .player_update(lavalink.clone(), player_update)
                        .await;
                }
                OPType::Stats => {
                    info!("op: stats");
                    let stats = match serde_json::from_str::<Stats>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("can't parse the stats message: {}", e);
                            continue;
                        }
                    };

                    debug!("emitting 'stats' in the event handler...");
                    lavalink.handler.stats(lavalink.clone(), stats).await;
                }
                OPType::Event => {
                    info!("op: event");
                    let event = match serde_json::from_str::<Event>(&message_str) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("can't parse the event message: {}", e);
                            continue;
                        }
                    };

                    match event.event_type {
                        EventType::TrackStart => {
                            info!("event: track start");
                            let track_start =
                                match serde_json::from_str::<TrackStartEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("can't parse the track start event: {}", e);
                                        continue;
                                    }
                                };

                            debug!("emitting 'track_start_event' in the event handler...");
                            lavalink
                                .handler
                                .track_start_event(lavalink.clone(), track_start)
                                .await;
                        }
                        EventType::TrackEnd => {
                            info!("event: track end");
                            let track_end =
                                match serde_json::from_str::<TrackEndEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("can't parse the track end event: {}", e);
                                        continue;
                                    }
                                };

                            debug!("emitting 'track_end_event' in the event handler...");
                            lavalink
                                .handler
                                .track_end_event(lavalink.clone(), track_end)
                                .await;
                        }
                        EventType::TrackException => {
                            info!("event: track exception");
                            let track_exception =
                                match serde_json::from_str::<TrackExceptionEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("can't parse the track exception event: {}", e);
                                        continue;
                                    }
                                };

                            debug!("emitting 'track_exception_event' in the event handler...");
                            lavalink
                                .handler
                                .track_exception_event(lavalink.clone(), track_exception)
                                .await;
                        }
                        EventType::TrackStuck => {
                            info!("event: track stuck");
                            let track_stuck =
                                match serde_json::from_str::<TrackStuckEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("can't parse the track stuck event: {}", e);
                                        continue;
                                    }
                                };

                            debug!("emitting 'track_stuck_event' in the event handler...");
                            lavalink
                                .handler
                                .track_stuck_event(lavalink.clone(), track_stuck)
                                .await;
                        }
                        EventType::WebSocketClosed => {
                            info!("event: websocket closed");
                            let websocket_closed =
                                match serde_json::from_str::<WebSocketClosedEvent>(&message_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("can't parse the websocket closed event: {}", e);
                                        continue;
                                    }
                                };

                            debug!("emitting 'websocket_closed_event' in the event handler...");
                            lavalink
                                .handler
                                .websocket_closed_event(lavalink.clone(), websocket_closed)
                                .await;
                        }
                    }
                }
            }
        } else {
            warn!("the message isn't a text and will not be parsed.");
        }
    }

    info!("websocket message parser finished.");
    *lavalink.status.write().unwrap() = ConnectionStatus::Disconnected;

    debug!("emitting 'disconnect' in the event handler...");
    lavalink.handler.disconnect(lavalink.clone()).await;
}

/// Attempts to parse the byte array into the selected type, if this attempt fails, a new attempt will be made parsing the input into an `ErrorResponse` which will be returned as an `Error::RestError`, if this also fails the `Error::InvalidResponse` will be returned.
pub fn parse_response<'a, T: Deserialize<'a>>(response: &'a [u8]) -> Result<T> {
    serde_json::from_slice::<T>(response).map_err(|e1| {
        warn!("can't parse to '{}': {}", type_name::<T>(), e1);

        match serde_json::from_slice::<ErrorResponse>(response) {
            Ok(v) => Error::RestError(v, Some(e1)),
            Err(e2) => {
                error!("can't parse to ErrorResponse: {}", e2);

                Error::InvalidResponse(Some(e1), e2)
            }
        }
    })
}
