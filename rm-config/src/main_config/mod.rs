mod connection;
mod general;
mod icons;
mod search_tab;
mod torrents_tab;

pub use connection::Connection;
pub use general::General;
pub use icons::Icons;
pub use search_tab::SearchTab;
pub use torrents_tab::TorrentsTab;

use std::{io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::utils::{self, ConfigFetchingError};

#[derive(Deserialize)]
pub struct MainConfig {
    #[serde(default)]
    pub general: General,
    pub connection: Connection,
    #[serde(default)]
    pub torrents_tab: TorrentsTab,
    #[serde(default)]
    pub search_tab: SearchTab,
    #[serde(default)]
    pub icons: Icons,
}

impl MainConfig {
    pub(crate) const FILENAME: &'static str = "config.toml";
    pub const DEFAULT_CONFIG: &'static str = include_str!("../../defaults/config.toml");

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
