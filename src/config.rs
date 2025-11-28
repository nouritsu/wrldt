use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, str::FromStr as _};
use thiserror::Error;

#[derive(Debug)]
pub struct Config {
    pub update_interval: u64,
    pub timezones: Vec<Tz>,
}

#[derive(Serialize, Deserialize)]
struct Config_ {
    pub update_interval: u64,
    pub timezones: Vec<String>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    TomlParseError(#[from] toml::de::Error),
    #[error("error serializing config: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
    #[error("error parsing timezone: {0}")]
    InvalidTimezone(#[from] chrono_tz::ParseError),
}

impl TryFrom<Config_> for Config {
    type Error = ConfigError;
    fn try_from(value: Config_) -> Result<Self, Self::Error> {
        let timezones: Result<Vec<Tz>, Self::Error> = value
            .timezones
            .into_iter()
            .map(|tz| Tz::from_str(&tz).map_err(ConfigError::InvalidTimezone))
            .collect();

        Ok(Self {
            update_interval: value.update_interval,
            timezones: timezones?,
        })
    }
}

impl From<Config> for Config_ {
    fn from(config: Config) -> Self {
        Self {
            update_interval: config.update_interval,
            timezones: config
                .timezones
                .into_iter()
                .map(|tz| tz.to_string())
                .collect(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval: 60,
            timezones: vec![Tz::default()],
        }
    }
}

impl Config {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config_toml: Config_ = toml::from_str(&fs::read_to_string(path)?)?;
        let config: Config = config_toml.try_into()?;
        Ok(config)
    }

    pub fn save_default(path: impl AsRef<Path>) -> Result<bool, ConfigError> {
        if fs::exists(&path)? {
            return Ok(false);
        }

        let config: Config_ = Self::default().into();
        let config_toml = toml::to_string(&config)?;
        fs::write(path, config_toml)?;
        Ok(true)
    }
}
