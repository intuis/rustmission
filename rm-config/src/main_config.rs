use std::{io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result};
use magnetease::WhichProvider;
use ratatui::style::Color;
use rm_shared::header::Header;
use serde::Deserialize;
use url::Url;

use crate::utils::{self, ConfigFetchingError};

#[derive(Deserialize)]
pub struct MainConfig {
    pub general: General,
    pub connection: Connection,
    #[serde(default)]
    pub torrents_tab: TorrentsTab,
    #[serde(default)]
    pub search_tab: SearchTab,
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
    #[serde(default = "default_headers")]
    pub headers: Vec<Header>,
}

fn default_headers() -> Vec<Header> {
    vec![
        Header::Name,
        Header::SizeWhenDone,
        Header::Progress,
        Header::Eta,
        Header::DownloadRate,
        Header::UploadRate,
    ]
}

impl Default for TorrentsTab {
    fn default() -> Self {
        Self {
            headers: default_headers(),
        }
    }
}

#[derive(Deserialize)]
pub struct SearchTab {
    pub providers: Vec<WhichProvider>,
}

fn default_providers() -> Vec<WhichProvider> {
    vec![WhichProvider::Knaben, WhichProvider::Nyaa]
}

impl Default for SearchTab {
    fn default() -> Self {
        Self {
            providers: default_providers(),
        }
    }
}

impl MainConfig {
    pub(crate) const FILENAME: &'static str = "config.toml";
    const DEFAULT_CONFIG: &'static str = include_str!("../defaults/config.toml");

    pub(crate) fn init() -> Result<Self> {
        match utils::fetch_config::<Self>(Self::FILENAME) {
            Ok(config) => Ok(config),
            Err(e) => match e {
                ConfigFetchingError::Io(e) if e.kind() == ErrorKind::NotFound => {
                    utils::put_config::<Self>(Self::DEFAULT_CONFIG, Self::FILENAME)?;
                    println!("Update {:?} and start rustmission again", Self::path());
                    std::process::exit(0);
                }
                ConfigFetchingError::Toml(e) => Err(e).with_context(|| {
                    format!(
                        "Failed to parse config located at {:?}",
                        utils::get_config_path(Self::FILENAME)
                    )
                }),
                _ => anyhow::bail!(e),
            },
        }
    }

    pub(crate) fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}
