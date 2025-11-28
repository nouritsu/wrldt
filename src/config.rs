use chrono_tz::Tz;
use config::{Config as Conf, ConfigError as ConfError};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, str::FromStr as _};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub update_interval: u64,
    pub main_timezone: Tz,
    pub timezones: Vec<Tz>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config parse error: {0}")]
    ConfigParseError(#[from] ConfError),
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("error serializing config: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
    #[error("unknown timezone: {0}")]
    InvalidTimezone(#[from] chrono_tz::ParseError),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval: 100,
            main_timezone: Tz::default(),
            timezones: vec![Tz::default()],
        }
    }
}

impl TryFrom<Conf> for Config {
    type Error = ConfigError;

    fn try_from(value: Conf) -> Result<Self, Self::Error> {
        let update_interval = value.get::<u64>("update_interval")?;
        let main_timezone = {
            let tz = value.get::<String>("main_timezone")?;
            Tz::from_str(&tz).map_err(ConfigError::InvalidTimezone)?
        };
        let timezones = {
            let tz_strs = value.get::<Vec<String>>("timezones")?;
            let tzs_result: Result<Vec<Tz>, Self::Error> = tz_strs
                .into_iter()
                .map(|tz| Tz::from_str(&tz).map_err(ConfigError::InvalidTimezone))
                .collect();

            tzs_result.and_then(|mut tzs| {
                if !tzs.contains(&main_timezone) {
                    tzs.push(main_timezone);
                }
                Ok(tzs)
            })?
        };

        Ok(Self {
            update_interval,
            main_timezone,
            timezones,
        })
    }
}

impl Config {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let settings = Conf::builder()
            .add_source(config::File::with_name(&path.to_string_lossy()))
            .build()?;
        let config: Config = settings.try_into()?;
        Ok(config)
    }

    pub fn save_default(path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let config_toml = toml::to_string(&Config::default())?;
        fs::write(path, config_toml)?;
        Ok(())
    }
}
