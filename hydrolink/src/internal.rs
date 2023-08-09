use serde::Deserialize;

/// Types of operations that can be emitted by the Lavalink server.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OPType {
    /// Emitted when you successfully connect to the Lavalink node.
    Ready,
    /// Emitted every x seconds with the latest player state.
    PlayerUpdate,
    /// Emitted when the node sends stats once per minute.
    Stats,
    /// Emitted when a player or voice event is emitted.
    Event,
}

/// Object used internally by the WebSocket message parser to detect the type of operation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketMessage {
    /// The op type.
    pub op: OPType,
}

/// Types of events that can be emitted by the Lavalink server.
#[derive(Deserialize)]
pub enum EventType {
    /// Emitted when a track starts playing.
    #[serde(rename = "TrackStartEvent")]
    TrackStart,
    /// Emitted when a track ends.
    #[serde(rename = "TrackEndEvent")]
    TrackEnd,
    /// Emitted when a track throws an exception.
    #[serde(rename = "TrackExceptionEvent")]
    TrackException,
    /// Emitted when a track gets stuck while playing.
    #[serde(rename = "TrackStuckEvent")]
    TrackStuck,
    /// Emitted when the websocket connection to Discord voice servers is closed.
    #[serde(rename = "WebSocketClosedEvent")]
    WebSocketClosed,
}

/// Object used internally by the WebSocket message parser to detect the type of event in the case of `event` operation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "type")]
    /// The type of event.
    pub event_type: EventType,
}
