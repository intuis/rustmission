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
    #[serde(default)]
    pub icons: Icons,
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
    #[serde(default)]
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

#[derive(Deserialize)]
pub struct Icons {
    #[serde(default = "default_upload")]
    pub upload: String,
    #[serde(default = "default_download")]
    pub download: String,
    #[serde(default = "default_arrow_left")]
    pub arrow_left: String,
    #[serde(default = "default_arrow_right")]
    pub arrow_right: String,
    #[serde(default = "default_arrow_up")]
    pub arrow_up: String,
    #[serde(default = "default_arrow_down")]
    pub arrow_down: String,
    #[serde(default = "default_triangle_right")]
    pub triangle_right: String,
    #[serde(default = "default_triangle_down")]
    pub triangle_down: String,
    #[serde(default = "default_file")]
    pub file: String,
    #[serde(default = "default_disk")]
    pub disk: String,
    #[serde(default = "default_help")]
    pub help: String,
    #[serde(default = "default_success")]
    pub success: String,
    #[serde(default = "default_failure")]
    pub failure: String,
    #[serde(default = "default_searching")]
    pub searching: String,
    #[serde(default = "default_verifying")]
    pub verifying: String,
    #[serde(default = "default_loading")]
    pub loading: String,
    #[serde(default = "default_pause")]
    pub pause: String,
    #[serde(default = "default_idle")]
    pub idle: String,
    #[serde(default = "default_magnifying_glass")]
    pub magnifying_glass: String,
    #[serde(default = "default_provider_disabled")]
    pub provider_disabled: String,
    #[serde(default = "default_provider_category_general")]
    pub provider_category_general: String,
    #[serde(default = "default_provider_category_anime")]
    pub provider_category_anime: String,
}

fn default_upload() -> String {
    "".into()
}

fn default_download() -> String {
    "".into()
}

fn default_arrow_left() -> String {
    "".into()
}

fn default_arrow_right() -> String {
    "".into()
}

fn default_arrow_up() -> String {
    "".into()
}

fn default_arrow_down() -> String {
    "".into()
}

fn default_triangle_right() -> String {
    "▶".into()
}

fn default_triangle_down() -> String {
    "▼".into()
}

fn default_file() -> String {
    "".into()
}

fn default_disk() -> String {
    "󰋊".into()
}

fn default_help() -> String {
    "".into()
}

fn default_success() -> String {
    "".into()
}

fn default_failure() -> String {
    "".into()
}

fn default_searching() -> String {
    "".into()
}

fn default_verifying() -> String {
    "󰑓".into()
}

fn default_loading() -> String {
    "󱥸".into()
}

fn default_pause() -> String {
    "󰏤".into()
}

fn default_idle() -> String {
    "󱗼".into()
}

fn default_magnifying_glass() -> String {
    "".into()
}

fn default_provider_disabled() -> String {
    "󰪎".into()
}

fn default_provider_category_general() -> String {
    "".into()
}

fn default_provider_category_anime() -> String {
    "󰎁".into()
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            upload: default_upload(),
            download: default_download(),
            arrow_left: default_arrow_left(),
            arrow_right: default_arrow_right(),
            arrow_up: default_arrow_up(),
            arrow_down: default_arrow_down(),
            triangle_right: default_triangle_right(),
            triangle_down: default_triangle_down(),
            file: default_file(),
            disk: default_disk(),
            help: default_help(),
            success: default_success(),
            failure: default_failure(),
            searching: default_searching(),
            verifying: default_verifying(),
            loading: default_loading(),
            pause: default_pause(),
            idle: default_idle(),
            magnifying_glass: default_magnifying_glass(),
            provider_disabled: default_provider_disabled(),
            provider_category_general: default_provider_category_general(),
            provider_category_anime: default_provider_category_anime(),
        }
    }
}

impl MainConfig {
    pub(crate) const FILENAME: &'static str = "config.toml";
    pub const DEFAULT_CONFIG: &'static str = include_str!("../defaults/config.toml");

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
