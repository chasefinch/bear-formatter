//! Configuration loaded from `bear-formatter.toml`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// User configuration. Per-rule options will grow here as the catalog lands.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Override the Bear database path (e.g. to format a copy).
    pub database: Option<PathBuf>,
}

impl Config {
    /// Load configuration by searching `start` and its ancestors for a
    /// `bear-formatter.toml`. Returns defaults when none is found.
    pub fn discover(start: &Path) -> Result<Self, ConfigError> {
        match find_config(start) {
            Some(path) => Self::from_path(&path),
            None => Ok(Self::default()),
        }
    }

    fn from_path(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn find_config(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|directory| directory.join("bear-formatter.toml"))
        .find(|candidate| candidate.is_file())
}

/// Errors raised while loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("could not read config at {}", .path.display())]
    Read {
        /// The offending path.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The config file was not valid TOML.
    #[error("could not parse config at {}", .path.display())]
    Parse {
        /// The offending path.
        path: PathBuf,
        /// The underlying parse error.
        #[source]
        source: toml::de::Error,
    },
}
