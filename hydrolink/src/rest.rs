use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub timestamp: i64,
    pub status: i32,
    pub error: String,
    pub trace: Option<String>,
    pub message: String,
    pub path: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: String,

    #[serde(skip_serializing)]
    pub connected: bool,
    #[serde(skip_serializing)]
    pub ping: i32,
}

impl VoiceState {
    pub fn new(token: &str, endpoint: &str, session_id: &str) -> Self {
        Self {
            token: token.to_owned(),
            endpoint: endpoint.to_owned(),
            session_id: session_id.to_owned(),
            connected: Default::default(),
            ping: Default::default(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_track: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceState>,
}

impl UpdatePlayer {
    pub fn new() -> Self {
        Self {
            encoded_track: None,
            identifier: None,
            end_time: None,
            paused: None,
            position: None,
            voice: None,
            volume: None,
        }
    }

    pub fn encoded_track(&mut self, encoded_track: &str) -> &mut Self {
        if self.identifier == None {
            self.encoded_track = Some(Some(encoded_track.to_owned()));
        }

        self
    }

    pub fn voice_state(&mut self, voice_state: VoiceState) -> &mut Self {
        self.voice = Some(voice_state);

        self
    }

    pub fn position(&mut self, position: i32) -> &mut Self {
        self.position = Some(position);

        self
    }

    pub fn paused(&mut self, paused: bool) -> &mut Self {
        self.paused = Some(paused);

        self
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub guild_id: String,
    pub track: Option<Track>,
    pub volume: i32,
    pub paused: bool,
    pub voice: VoiceState,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub encoded: String,
    pub track: String,
    pub info: TrackInfo,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub identifier: String,
    pub is_seekable: bool,
    pub author: String,
    pub length: i32,
    pub is_stream: bool,
    pub position: i32,
    pub title: String,
    pub uri: Option<String>,
    pub source_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackLoading {
    pub playlist_info: PlaylistInfo,
    pub tracks: Vec<Track>,
    pub exception: Option<Exception>,
    pub load_type: LoadResultType,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoadResultType {
    TrackLoaded,
    PlaylistLoaded,
    SearchResult,
    NoMatches,
    LoadFailed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub name: Option<String>,
    pub selected_track: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    pub message: Option<String>,
    pub severity: Severity,
    pub cause: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Common,
    Suspicious,
    Fault,
}
