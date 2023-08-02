use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyOP {
    pub resumed: bool,
    pub session_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStartEvent {
    pub guild_id: String,
    pub encoded_track: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackEndEvent {
    pub guild_id: String,
    pub encoded_track: String,
    pub reason: TrackEndReason,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackEndReason {
    Finished,
    LoadFailed,
    Stopped,
    Replaced,
    Cleanup,
}
