use serde::Deserialize;

/// Lavalink server statistics.
#[derive(Clone, Deserialize)]
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
#[derive(Clone, Deserialize)]
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
#[derive(Clone, Deserialize)]
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
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameStats {
    /// The amount of frames sent to Discord.
    pub sent: i32,
    /// The amount of frames that were nulled.
    pub nulled: i32,
    /// The amount of frames that were deficit.
    pub deficit: i32,
}

/// An exception/error produced by the Lavalink server.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    /// The message of the exception.
    pub message: Option<String>,
    /// The severity of the exception.
    pub severity: Severity,
    /// The cause of the exception.
    pub cause: String,
}

/// The severity level of the exception.
#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// The cause is known and expected, indicates that there is nothing wrong with the library itself.
    Common,
    /// The cause might not be exactly known, but is possibly caused by outside factors. For example when an outside service responds in a format that we do not expect.
    Suspicious,
    /// If the probable cause is an issue with the library or when there is no way to tell what the cause might be. This is the default level and other levels are used in cases where the thrower has more in-depth knowledge about the error.
    Fault,
}
