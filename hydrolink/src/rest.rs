use serde::{Deserialize, Serialize};

use crate::Exception;

/// Error response returned by Lavalink Server REST API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// The timestamp of the error in milliseconds since the epoch.
    pub timestamp: u64,
    /// The HTTP status code.
    pub status: u16,
    /// The HTTP status code message.
    pub error: String,
    /// The stack trace of the error when the feature `lavalink-trace` is enabled.
    pub trace: Option<String>,
    /// The error message.
    pub message: String,
    /// The request path.
    pub path: String,
}

/// Discord client/bot voice state, this will be used by the Lavalink server to connect to the voice chat.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
    /// The Discord voice token to authenticate with.
    pub token: String,
    /// The Discord voice endpoint to connect to.
    pub endpoint: String,
    /// The Discord voice session id to authenticate with.
    pub session_id: String,

    /// Whether the player is connected. Response only.
    #[serde(skip_serializing)]
    pub connected: bool,
    /// Roundtrip latency in milliseconds to the voice gateway (-1 if not connected). Response only.
    #[serde(skip_serializing)]
    pub ping: i32,
}

impl VoiceState {
    /// Initializes a new `VoiceState` that will be used by the Lavalink server.
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

/// Request used by the `update_player` function to update the player on the Lavalink server.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayer {
    /// The encoded track base64 to play. `Option::None` stops the current track.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_track: Option<Option<String>>,

    /// The track identifier to play.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    /// The track position in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,

    /// The track end time in milliseconds (must be > 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<Option<u32>>,

    /// The player volume from 0 to 1000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u16>,

    /// Whether the player is paused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,

    /// Information required for connecting to Discord.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceState>,
}

impl UpdatePlayer {
    /// Initializes a new, empty `UpdatePlayer`.
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

    /// Sets a new value in the `encoded_track` parameter. `Option::None` stops the current track.
    ///
    /// The `identifier` parameter needs to be none.
    pub fn encoded_track(&mut self, encoded_track: Option<String>) -> &mut Self {
        if self.identifier.is_none() {
            self.encoded_track = Some(encoded_track);
        }
        self
    }

    /// Sets a new value in the `identifier` parameter.
    ///
    /// The `encoded_track` parameter needs to be none.
    pub fn identifier(&mut self, identifier: &str) -> &mut Self {
        if self.encoded_track.is_none() {
            self.identifier = Some(identifier.to_owned());
        }
        self
    }

    /// Sets a new value in the `position` parameter.
    pub fn position(&mut self, position: u32) -> &mut Self {
        self.position = Some(position);
        self
    }

    /// Sets a new value in the `end_time` parameter.
    pub fn end_time(&mut self, end_time: Option<u32>) -> &mut Self {
        self.end_time = Some(end_time);
        self
    }

    /// Sets a new value in the `volume` parameter.
    pub fn volume(&mut self, volume: u16) -> &mut Self {
        self.volume = Some(volume);
        self
    }

    /// Sets a new value in the `voice_state` parameter.
    pub fn voice_state(&mut self, voice_state: VoiceState) -> &mut Self {
        self.voice = Some(voice_state);
        self
    }

    /// Sets a new value in the `paused` parameter.
    pub fn paused(&mut self, paused: bool) -> &mut Self {
        self.paused = Some(paused);
        self
    }
}

impl Default for UpdatePlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// A Lavalink Player associated with a guild and a session.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    /// The guild id of the player.
    pub guild_id: String,
    /// The current playing track.
    pub track: Option<Track>,
    /// The volume of the player, range 0-1000, in percentage.
    pub volume: u16,
    /// Whether the player is paused.
    pub paused: bool,
    /// The voice state of the player.
    pub voice: VoiceState,
}

/// A single audio track.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    /// The base64 encoded track data.
    pub encoded: String,
    /// The base64 encoded track data.
    pub track: String,
    /// Info about the track.
    pub info: TrackInfo,
}

/// Information about an audio track.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    /// The track identifier.
    pub identifier: String,
    /// Whether the track is seekable.
    pub is_seekable: bool,
    /// The track author.
    pub author: String,
    /// The track length in milliseconds.
    pub length: u32,
    /// Whether the track is a stream.
    pub is_stream: bool,
    /// The track position in milliseconds.
    pub position: u32,
    /// The track title.
    pub title: String,
    /// The track uri.
    pub uri: Option<String>,
    /// The track source name.
    pub source_name: String,
}

/// Response for a load track request.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackLoading {
    /// Additional info if the the load type is `LoadResultType::PlaylistLoaded`.
    pub playlist_info: PlaylistInfo,
    /// All tracks which have been loaded. (Valid for `LoadResultType::TrackLoaded`, `LoadResultType::PlaylistLoaded`, and `LoadResultType::SearchResult`)
    pub tracks: Vec<Track>,
    /// The Exception this load failed with. (Valid for `LoadResultType::LoadFailed`)
    pub exception: Option<Exception>,
    /// The type of the result.
    pub load_type: LoadResultType,
}

/// The type of result that was loaded.
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoadResultType {
    /// A track has been loaded.
    TrackLoaded,
    /// A playlist has been loaded.
    PlaylistLoaded,
    /// A search result has been loaded.
    SearchResult,
    /// There has been no matches to your identifier.
    NoMatches,
    /// Loading has failed.
    LoadFailed,
}

/// Information about the playlist.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    /// The name of the loaded playlist.
    pub name: Option<String>,
    /// The selected track in this playlist. (-1 if no track is selected)
    pub selected_track: Option<i32>,
}
