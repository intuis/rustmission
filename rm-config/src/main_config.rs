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

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Header {
    Name,
    SizeWhenDone,
    Progress,
    Eta,
    DownloadRate,
    UploadRate,
    DownloadDir,
    Padding,
    UploadRatio,
    UploadedEver,
    Id,
    ActivityDate,
    AddedDate,
    PeersConnected,
    SmallStatus,
}

impl Header {
    pub fn default_constraint(&self) -> Constraint {
        match self {
            Self::Name => Constraint::Max(70),
            Self::SizeWhenDone => Constraint::Length(12),
            Self::Progress => Constraint::Length(12),
            Self::Eta => Constraint::Length(12),
            Self::DownloadRate => Constraint::Length(12),
            Self::UploadRate => Constraint::Length(12),
            Self::DownloadDir => Constraint::Max(70),
            Self::Padding => Constraint::Length(2),
            Self::UploadRatio => Constraint::Length(6),
            Self::UploadedEver => Constraint::Length(12),
            Self::Id => Constraint::Length(4),
            Self::ActivityDate => Constraint::Length(14),
            Self::AddedDate => Constraint::Length(12),
            Self::PeersConnected => Constraint::Length(6),
            Self::SmallStatus => Constraint::Length(1),
        }
    }

    pub fn header_name(&self) -> &'static str {
        match *self {
            Self::Name => "Name",
            Self::SizeWhenDone => "Size",
            Self::Progress => "Progress",
            Self::Eta => "ETA",
            Self::DownloadRate => "Download",
            Self::UploadRate => "Upload",
            Self::DownloadDir => "Directory",
            Self::Padding => "",
            Self::UploadRatio => "Ratio",
            Self::UploadedEver => "Up Ever",
            Self::Id => "Id",
            Self::ActivityDate => "Last active",
            Self::AddedDate => "Added",
            Self::PeersConnected => "Peers",
            Self::SmallStatus => "",
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
