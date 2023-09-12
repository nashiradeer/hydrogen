use serde::Deserialize;

use crate::Exception;

/// Response sent by Lavalink when the connection is established.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    /// If a session was resumed.
    pub resumed: bool,
    /// The Lavalink session ID of this connection. Not to be confused with a Discord voice session id.
    pub session_id: String,
}

/// Response sent every x seconds by Lavalink about the player state of a guild.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdate {
    /// The guild id of the player.
    pub guild_id: String,
    /// The player state.
    pub state: PlayerState,
}

/// Player state in the Lavalink server.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    /// Unix timestamp in milliseconds.
    pub time: u64,
    /// The position of the track in milliseconds.
    pub position: Option<u32>,
    /// If Lavalink is connected to the voice gateway.
    pub connected: bool,
    /// The ping of the node to the Discord voice server in milliseconds. (-1 if not connected)
    pub ping: i16,
}

/// Information about the track that was started.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStartEvent {
    /// The guild id.
    pub guild_id: String,
    /// The base64 encoded track that started playing.
    pub encoded_track: String,
}

/// Information about the track that was finished.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackEndEvent {
    /// The guild id.
    pub guild_id: String,
    // The base64 encoded track that ended playing.
    pub encoded_track: String,
    /// The reason the track ended.
    pub reason: TrackEndReason,
}

/// The reason why a track was finished.
#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackEndReason {
    /// The track finished playing. (May start next? Yes)
    Finished,
    /// The track failed to load. (May start next? Yes)
    LoadFailed,
    /// The track was stopped. (May start next? No)
    Stopped,
    /// The track was replaced. (May start next? No)
    Replaced,
    /// The track was cleaned up. (May start next? No)
    Cleanup,
}

/// Emitted when an exception/error occurs while playing a track.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackExceptionEvent {
    /// The guild id.
    pub guild_id: String,
    /// The base64 encoded track that threw the exception.
    pub encoded_track: String,
    /// The occurred exception.
    pub exception: Exception,
}

/// Emitted when a song is stuck.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStuckEvent {
    /// The guild id.
    pub guild_id: String,
    /// The base64 encoded track that threw the exception.
    pub encoded_track: String,
    /// The threshold in milliseconds that was exceeded.
    pub threshold_ms: u32,
}

/// Emitted as soon as any connection between the Lavalink server and Discord is closed, which can be normal or abnormal.
///
/// 4xxx codes are generally bad according to the Discord documentation.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketClosedEvent {
    /// The guild id.
    pub guild_id: String,
    /// The Discord close event code.
    pub code: u16,
    /// The close reason.
    pub reason: String,
    /// If the connection was closed by Discord.
    pub by_remote: bool,
}
