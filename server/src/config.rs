use std::{error::Error, fmt::Display, fs::File, io::Read, str::FromStr, string::FromUtf8Error};

use serde::Deserialize;
use tracing::Level;

#[derive(Deserialize)]
pub struct Config {
    pub database: String,
    #[serde(deserialize_with = "deserialize_log_level")]
    pub log_level: tracing::Level,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut file = File::open("./config.toml")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let file = String::from_utf8(buf)?;

        Ok(toml::from_str(&file)?)
    }

    pub fn get_db_url(&self) -> String {
        format!("sqlite:{}.db", self.database.trim_end_matches('.'))
    }
}

fn deserialize_log_level<'d, D>(deserializer: D) -> Result<tracing::Level, D::Error>
where
    D: serde::Deserializer<'d>,
{
    let value = String::deserialize(deserializer)?;
    Level::from_str(&value).map_err(serde::de::Error::custom)
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    UTF8(String),
    Deserialize(String),
}

impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) => write!(f, "Problem reading config file. Inner error: {s}"),
            Self::UTF8(s) => write!(f, "Config file was not valid UTF-8. Inner error: {s}"),
            Self::Deserialize(s) => {
                write!(f, "Config file was not valid TOML format. Inner error: {s}")
            }
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<FromUtf8Error> for ConfigError {
    fn from(value: FromUtf8Error) -> Self {
        Self::UTF8(value.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::Deserialize(value.to_string())
    }
}
