use std::{io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::Result;
use ratatui::style::Color;
use serde::{Deserialize, Deserializer};
use transmission_rpc::types::TorrentGetField;
use url::Url;

use crate::utils::{self};

#[derive(Deserialize)]
pub struct MainConfig {
    pub general: General,
    pub connection: Connection,
    pub torrents_tab: TorrentsTab,
}

#[derive(Deserialize)]
pub struct General {
    #[serde(default)]
    pub auto_hide: bool,
    #[serde(default = "default_accent_color")]
    pub accent_color: Color,
    #[serde(default = "default_beginner_mode")]
    pub beginner_mode: bool,
    #[serde(default)]
    pub headers_hide: bool,
}

fn default_accent_color() -> Color {
    Color::LightMagenta
}

fn default_beginner_mode() -> bool {
    true
}

#[derive(Deserialize)]
pub struct Connection {
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Url,
    #[serde(default = "default_refresh")]
    pub torrents_refresh: u64,
    #[serde(default = "default_refresh")]
    pub stats_refresh: u64,
    #[serde(default = "default_refresh")]
    pub free_space_refresh: u64,
}

fn default_refresh() -> u64 {
    5
}

#[derive(Deserialize)]
pub struct TorrentsTab {
    #[serde(deserialize_with = "get_field_deserializer")]
    pub headers: Vec<TorrentGetField>,
}

fn get_field_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<TorrentGetField>, D::Error> {
    let vals: Vec<String> = Vec::deserialize(deserializer)?;

    let mut headers = vec![];

    for val in vals {
        match val.to_lowercase().as_str() {
            "name" => headers.push(TorrentGetField::Name),
            "downloaddir" => headers.push(TorrentGetField::DownloadDir),
            _ => todo!(),
        }
    }

    Ok(headers)
}

impl MainConfig {
    pub(crate) const FILENAME: &'static str = "config.toml";
    const DEFAULT_CONFIG: &'static str = include_str!("../defaults/config.toml");

    pub(crate) fn init() -> Result<Self> {
        match utils::fetch_config::<Self>(Self::FILENAME) {
            Ok(config) => return Ok(config),
            Err(e) => match e {
                utils::ConfigFetchingError::Io(e) if e.kind() == ErrorKind::NotFound => {
                    utils::put_config::<Self>(Self::DEFAULT_CONFIG, Self::FILENAME)?;
                    println!("Update {:?} and start rustmission again", Self::path());
                    std::process::exit(0);
                }
                _ => anyhow::bail!(e),
            },
        };
    }

    pub(crate) fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}
