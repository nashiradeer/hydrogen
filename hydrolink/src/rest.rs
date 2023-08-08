use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Error response returned by Lavalink Server REST API.
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

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Discord client/bot voice state, this will be used by the Lavalink server to connect to the voice chat.
pub struct VoiceState {
    /// The Discord voice token to authenticate with.
    pub token: String,
    /// The Discord voice endpoint to connect to.
    pub endpoint: String,
    /// The Discord voice session id to authenticate with.
    pub session_id: String,

    #[serde(skip_serializing)]
    /// Whether the player is connected. Response only.
    pub connected: bool,
    #[serde(skip_serializing)]
    /// Roundtrip latency in milliseconds to the voice gateway (-1 if not connected). Response only.
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// Request used by the `update_player` function to update the player on the Lavalink server.
pub struct UpdatePlayer {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The encoded track base64 to play. `Option::None` stops the current track.
    pub encoded_track: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The track identifier to play.
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The track position in milliseconds.
    pub position: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The track end time in milliseconds (must be > 0).
    pub end_time: Option<Option<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The player volume from 0 to 1000.
    pub volume: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the player is paused.
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Information required for connecting to Discord.
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// A Lavalink Player associated with a guild and a session.
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

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A single audio track.
pub struct Track {
    /// The base64 encoded track data.
    pub encoded: String,
    /// The base64 encoded track data.
    pub track: String,
    /// Info about the track.
    pub info: TrackInfo,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about an audio track.
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// Response for a load track request.
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

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// The type of result that was loaded.
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about the playlist.
pub struct PlaylistInfo {
    /// The name of the loaded playlist.
    pub name: Option<String>,
    /// The selected track in this playlist. (-1 if no track is selected)
    pub selected_track: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// An exception/error produced by the Lavalink server.
pub struct Exception {
    /// The message of the exception.
    pub message: Option<String>,
    /// The severity of the exception.
    pub severity: Severity,
    /// The cause of the exception.
    pub cause: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// The severity level of the exception.
pub enum Severity {
    /// The cause is known and expected, indicates that there is nothing wrong with the library itself.
    Common,
    /// The cause might not be exactly known, but is possibly caused by outside factors. For example when an outside service responds in a format that we do not expect.
    Suspicious,
    /// If the probable cause is an issue with the library or when there is no way to tell what the cause might be. This is the default level and other levels are used in cases where the thrower has more in-depth knowledge about the error.
    Fault,
}
