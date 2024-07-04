use std::{io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::Result;
use ratatui::{layout::Constraint, style::Color};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::utils::{self};

#[derive(Deserialize)]
pub struct MainConfig {
    pub general: General,
    pub connection: Connection,
    #[serde(default)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Header {
    Name,
    SizeWhenDone,
    Progress,
    Eta,
    DownloadRate,
    UploadRate,
    DownloadDir,
}

impl Header {
    pub fn default_constraint(&self) -> Constraint {
        match self {
            Header::Name => Constraint::Max(70),
            Header::SizeWhenDone => Constraint::Length(12),
            Header::Progress => Constraint::Length(12),
            Header::Eta => Constraint::Length(12),
            Header::DownloadRate => Constraint::Length(12),
            Header::UploadRate => Constraint::Length(12),
            Header::DownloadDir => Constraint::Max(70),
        }
    }

    pub fn header_name(&self) -> &'static str {
        match self {
            Header::Name => "Name",
            Header::SizeWhenDone => "Size",
            Header::Progress => "Progress",
            Header::Eta => "ETA",
            Header::DownloadRate => "Download",
            Header::UploadRate => "Upload",
            Header::DownloadDir => "Directory",
        }
    }
}

#[derive(Deserialize)]
pub struct TorrentsTab {
    pub headers: Vec<Header>,
}

impl Default for TorrentsTab {
    fn default() -> Self {
        Self {
            headers: vec![
                Header::Name,
                Header::SizeWhenDone,
                Header::Progress,
                Header::Eta,
                Header::DownloadRate,
                Header::UploadRate,
            ],
        }
    }
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
