use serde::Deserialize;

use crate::Exception;

/// Response sent by Lavalink when the connection is established.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    /// If a session was resumed.
    pub resumed: bool,
    /// The Lavalink session ID of this connection. Not to be confused with a Discord voice session id.
    pub session_id: String,
}

/// Response sent every x seconds by Lavalink about the player state of a guild.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdateOP {
    /// The guild id of the player.
    pub guild_id: String,
    /// The player state.
    pub state: PlayerState,
}

/// Player state in the Lavalink server.
#[derive(Deserialize)]
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

/// Lavalink server statistics.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    /// The amount of players connected to the node.
    pub players: u64,
    /// The amount of players playing a track.
    pub playing_players: u64,
    /// The uptime of the node in milliseconds.
    pub uptime: u64,
    /// The memory stats of the node.
    pub memory: Memory,
    /// The cpu stats of the node.
    pub cpu: CPU,
    /// The frame stats of the node. `Option::None` if the node has no players or when retrieved via `Lavalink::get_stats()`.
    pub frame_stats: Option<FrameStats>,
}

/// Statistics related to Lavalink server RAM memory usage.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    /// The amount of free memory in bytes.
    pub free: i32,
    /// The amount of used memory in bytes.
    pub used: i32,
    /// The amount of allocated memory in bytes.
    pub allocated: i32,
    /// The amount of reservable memory in bytes.
    pub reservable: i32,
}

/// Statistics related to Lavalink server CPU usage.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CPU {
    /// The amount of cores the node has.
    pub cores: i32,
    /// The system load of the node.
    pub system_load: f32,
    /// The load of Lavalink on the node.
    pub lavalink_load: f32,
}

/// Statistics related to the connections between Lavalink server and Discord.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameStats {
    /// The amount of frames sent to Discord.
    pub sent: i32,
    /// The amount of frames that were nulled.
    pub nulled: i32,
    /// The amount of frames that were deficit.
    pub deficit: i32,
}

/// Information about the track that was started.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStartEvent {
    /// The guild id.
    pub guild_id: String,
    /// The base64 encoded track that started playing.
    pub encoded_track: String,
}

/// Information about the track that was finished.
#[derive(Deserialize)]
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
#[derive(Deserialize, PartialEq, Eq)]
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
#[derive(Deserialize)]
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
#[derive(Deserialize)]
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
#[derive(Deserialize)]
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
