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
#[serde(rename_all = "PascalCase")]
pub enum EventType {
    /// Emitted when a track starts playing.
    TrackStartEvent,
    /// Emitted when a track ends.
    TrackEndEvent,
    /// Emitted when a track throws an exception.
    TrackExceptionEvent,
    /// Emitted when a track gets stuck while playing.
    TrackStuckEvent,
    /// Emitted when the websocket connection to Discord voice servers is closed.
    WebSocketClosedEvent,
}

/// Object used internally by the WebSocket message parser to detect the type of event in the case of `event` operation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventOP {
    #[serde(rename = "type")]
    /// The type of event.
    pub event_type: EventType,
}
