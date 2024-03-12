//! Configuration parsing and management.

use std::{
    env, error,
    fmt::{self, Display, Formatter},
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::Deserialize;
use tracing::{debug, warn};

#[cfg(windows)]
mod windows {
    //! Windows-specific configuration parsing and management.

    use std::{
        env,
        path::{Path, PathBuf},
    };

    /// The default configuration file path.
    pub fn default_config_file() -> PathBuf {
        Path::new(&env::var("APPDATA").unwrap_or("C:\\ProgramData".to_owned()))
            .join("Hydrogen\\Config.toml")
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
mod unix {
    //! Unix-specific configuration parsing and management.

    use std::{
        env,
        path::{Path, PathBuf},
    };

    /// The default configuration file path.
    pub fn default_config_file() -> PathBuf {
        Path::new(&env::var("XDG_CONFIG_HOME").unwrap_or("/etc".to_owned()))
            .join("hydrogen/config.toml")
    }
}

#[cfg(unix)]
pub use unix::*;

use crate::lavalink::LavalinkNodeInfo;

/// The command line arguments.
#[derive(Debug, Parser, PartialEq, Eq, Clone)]
#[command(name = "Hydrogen", version, about, long_about = None)]
pub struct Args {
    /// Configuration file path.
    #[arg(short, long, help = "The configuration file path.", long_help = None)]
    pub config_file: Option<PathBuf>,
}

/// Errors that can occur while parsing the configuration file.
#[derive(Debug)]
pub enum LoadFileError {
    /// An I/O error occurred while reading the file.
    Io(io::Error),

    /// A TOML error occurred while parsing the file.
    Toml(toml::de::Error),
}

impl Display for LoadFileError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Toml(err) => write!(f, "TOML error: {}", err),
        }
    }
}

impl error::Error for LoadFileError {}

/// Get the default Lavalink address.
fn default_lavalink_address() -> String {
    "127.0.0.1:2333".to_owned()
}

/// The default password for Lavalink.
fn default_lavalink_password() -> String {
    "youshallnotpass".to_owned()
}

/// Configuration for a single Lavalink node.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LavalinkConfig {
    /// The address of the Lavalink server.
    #[serde(default = "default_lavalink_address")]
    pub address: String,
    /// The password of the Lavalink server.
    #[serde(default = "default_lavalink_password")]
    pub password: String,
    /// Whether to use TLS to connect to the Lavalink server.
    #[serde(default)]
    pub tls: bool,
}

impl From<&str> for LavalinkConfig {
    fn from(s: &str) -> Self {
        // Get the components from the string.
        let mut components = s.split(',');

        // Get the address.
        let address = components
            .next()
            .map(|s| s.to_owned())
            .unwrap_or(default_lavalink_address());

        // Get the password.
        let password = components
            .next()
            .map(|s| s.to_owned())
            .unwrap_or(default_lavalink_password());

        // Check if TLS is enabled.
        let tls = components
            .next()
            .map(|s| matches!(s.to_lowercase().as_str(), "true" | "yes" | "1" | "enabled"))
            .unwrap_or(false);

        Self {
            address,
            password,
            tls,
        }
    }
}

impl From<LavalinkConfig> for LavalinkNodeInfo {
    fn from(config: LavalinkConfig) -> Self {
        Self {
            host: config.address,
            password: config.password,
            tls: config.tls,
        }
    }
}

/// The configuration of the server.
#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// The default language of the server.
    pub default_language: Option<String>,
    /// The path to the language files.
    pub language_path: Option<PathBuf>,
    /// The Lavalink configuration.
    pub lavalink: Option<Vec<LavalinkConfig>>,
    /// The token of the Discord bot.
    pub discord_token: Option<String>,
}

impl Config {
    /// Parse the configuration from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadFileError> {
        let file_content = read_to_string(path).map_err(LoadFileError::Io)?;
        toml::de::from_str(&file_content).map_err(LoadFileError::Toml)
    }

    /// Overwrite configuration values that are empty with the ones from the environment.
    pub fn or_from_env(self) -> Self {
        // Get the default language from the environment.
        let default_language = self
            .default_language
            .or_else(|| env::var("HYDROGEN_DEFAULT_LANGUAGE").ok());

        // Get the language path from the environment.
        let language_path = self
            .language_path
            .or_else(|| env::var("HYDROGEN_LANGUAGE_PATH").ok().map(PathBuf::from));

        // Get the Lavalink configuration from the environment.
        let lavalink = self.lavalink.or_else(|| {
            env::var("HYDROGEN_LAVALINK")
                .ok()
                .map(|s| s.split(';').map(LavalinkConfig::from).collect())
        });

        // Get the Discord token from the environment.
        let discord_token = self
            .discord_token
            .or_else(|| env::var("HYDROGEN_DISCORD_TOKEN").ok());

        Self {
            default_language,
            language_path,
            lavalink,
            discord_token,
        }
    }
}

/// Try to load the configuration file.
pub fn load_configuration() -> Config {
    debug!("searching for the configuration file...");
    let args = Args::parse();

    let config_file = args
        .config_file
        .or(env::var("HYDROGEN_CONFIG_FILE").ok().map(PathBuf::from))
        .unwrap_or(default_config_file());

    debug!("loading the configuration file: {:?}", config_file);
    match Config::from_file(config_file) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to load the configuration file: {}", e);
            Config::default()
        }
    }
}
