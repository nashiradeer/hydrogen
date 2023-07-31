use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkErrorResponse {
    pub timestamp: i64,
    pub status: i32,
    pub error: String,
    pub trace: Option<String>,
    pub message: String,
    pub path: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkVoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: String,

    #[serde(skip_serializing)]
    pub connected: bool,
    #[serde(skip_serializing)]
    pub ping: i32,
}

impl LavalinkVoiceState {
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
pub struct LavalinkUpdatePlayer {
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
    pub voice: Option<LavalinkVoiceState>,
}

impl LavalinkUpdatePlayer {
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

    pub fn voice_state(&mut self, voice_state: LavalinkVoiceState) -> &mut Self {
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
pub struct LavalinkPlayer {
    pub guild_id: String,
    pub track: Option<LavalinkTrack>,
    pub volume: i32,
    pub paused: bool,
    pub voice: LavalinkVoiceState,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkTrack {
    pub encoded: String,
    pub track: String,
    pub info: LavalinkTrackInfo,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkTrackInfo {
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
pub struct LavalinkTrackLoading {
    pub playlist_info: LavalinkPlaylistInfo,
    pub tracks: Vec<LavalinkTrack>,
    pub exception: Option<LavalinkException>,
    pub load_type: LavalinkLoadResultType,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LavalinkLoadResultType {
    TrackLoaded,
    PlaylistLoaded,
    SearchResult,
    NoMatches,
    LoadFailed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkPlaylistInfo {
    pub name: Option<String>,
    pub selected_track: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkException {
    pub message: Option<String>,
    pub severity: LavalinkSeverity,
    pub cause: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LavalinkSeverity {
    Common,
    Suspicious,
    Fault,
}
