use serde::{Deserialize, Serialize};

use crate::{Exception, Filters};

/// Error response returned by Lavalink Server REST API.
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Clone, Default, Serialize)]
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

    /// The new filters to apply. This will override all previously applied filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Filters>,

    /// Information required for connecting to Discord.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceState>,
}

/// A Lavalink Player associated with a guild and a session.
#[derive(Clone, Deserialize)]
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
    /// The filters used by the player.
    pub filters: Filters,
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
#[derive(Clone, Deserialize)]
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
#[derive(Clone, Deserialize, PartialEq, Eq)]
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
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    /// The name of the loaded playlist.
    pub name: Option<String>,
    /// The selected track in this playlist. (-1 if no track is selected)
    pub selected_track: Option<i32>,
}

/// Request and response used by the `update_session` function.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSession {
    /// The resuming key to be able to resume this session later.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resuming_key: Option<Option<String>>,

    /// The timeout in seconds (default is 60s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

/// Lavalink server information.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    /// The version of this Lavalink server.
    pub version: Version,

    /// The millisecond unix timestamp when this Lavalink jar was built.
    pub build_time: u64,

    /// The git information of this Lavalink server.
    pub git: Git,

    /// The JVM version this Lavalink server runs on.
    pub jvm: String,

    /// The Lavaplayer version being used by this server.
    pub lavaplayer: String,

    /// The enabled source managers for this server.
    pub source_managers: Vec<String>,

    /// The enabled filters for this server.
    pub filters: Vec<String>,

    /// The enabled plugins for this server.
    pub plugins: Vec<Plugin>,
}

/// Parsed Semantic Versioning 2.0.0. See https://semver.org/ for more info.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// The full version string of this Lavalink server.
    pub semver: String,

    /// The major version of this Lavalink server.
    pub major: u8,

    /// The minor version of this Lavalink server.
    pub minor: u8,

    /// The patch version of this Lavalink server.
    pub patch: u8,

    /// The pre-release version according to semver as a `.` separated list of identifiers.
    pub pre_release: Option<String>,

    /// The build metadata according to semver as a `.` separated list of identifiers
    pub build: Option<String>,
}

/// Information about the branch and commit used to build the Lavalink server.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Git {
    /// The branch this Lavalink server was built.
    pub branch: String,

    /// The commit this Lavalink server was built.
    pub commit: String,

    /// The millisecond unix timestamp for when the commit was created.
    pub commit_time: u64,
}

/// Plugin used by Lavalink to extend its functions.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    /// The name of the plugin.
    pub name: String,

    /// The version of the plugin.
    pub version: String,
}

/// Response sent by Lavalink server about Route Planner status.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlannerStatus {
    /// The name of the RoutePlanner implementation being used by this server.
    pub class: Option<RoutePlannerType>,

    /// The status details of the RoutePlanner.
    pub details: Option<Details>,
}

/// The type/strategy used by the Lavalink server's Route Planner.
#[derive(Clone, Deserialize, PartialEq, Eq)]
pub enum RoutePlannerType {
    /// IP address used is switched on ban. Recommended for IPv4 blocks or IPv6 blocks smaller than a /64.
    #[serde(rename = "RotatingIpRoutePlanner")]
    Rotating,

    /// IP address used is switched on clock update. Use with at least 1 /64 IPv6 block.
    #[serde(rename = "NanoIpRoutePlanner")]
    Nano,

    /// IP address used is switched on clock update, rotates to a different /64 block on ban. Use with at least 2x /64 IPv6 blocks.
    #[serde(rename = "RotatingNanoIpRoutePlanner")]
    RotatingNano,

    /// IP address used is selected at random per request. Recommended for larger IP blocks.
    #[serde(rename = "BalancingIpRoutePlanner")]
    Balancing,
}

/// Details about the Route Planner used by the Lavalink server.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    /// The ip block being used.
    pub ip_block: IPBlock,

    /// The failing addresses.
    pub failing_addresses: Vec<FailingAddress>,

    /// The number of rotations. (Only in `Rotating` type)
    pub rotate_index: String,

    /// The current offset in the block. (Only in `Rotating` type)
    pub ip_index: String,

    /// The current address being used. (Only in `Rotating` type)
    pub current_address: String,

    /// The current offset in the ip block. (Only in `Nano` and `RotatingNano` types)
    pub current_address_index: String,

    /// The information in which /64 block ips are chosen. This number increases on each ban. (Only in `RotatingNano`)
    pub block_index: String,
}

/// IP block information.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPBlock {
    /// The type of the IP block.
    #[serde(rename = "type")]
    pub ip_type: IPBlockType,

    /// The size of the IP block.
    pub size: String,
}

/// IP block type/version.
#[derive(Clone, Deserialize, PartialEq, Eq)]
pub enum IPBlockType {
    /// The IPv4 block type.
    #[serde(rename = "Inet4Address")]
    Inet4,

    /// The IPv6 block type
    #[serde(rename = "Inet6Address")]
    Inet6,
}

/// Information about a failed IP address and when it happened.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailingAddress {
    /// The failing address.
    pub failing_address: String,

    /// The timestamp when the address failed.
    pub failing_timestamp: u64,

    /// The timestamp when the address failed as a pretty string.
    pub failing_time: String,
}

/// Request to unmark an IP address failed.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlannerFailedAddress {
    /// The address to unmark as failed. This address must be in the same ip block.
    pub address: String,
}
