use serde::Deserialize;

/// Response sent by Lavalink when the connection is established.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyOP {
    /// If a session was resumed.
    pub resumed: bool,
    /// The Lavalink session ID of this connection. Not to be confused with a Discord voice session id.
    pub session_id: String,
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
