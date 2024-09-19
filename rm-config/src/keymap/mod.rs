pub mod actions;

use std::{io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result};
use intuitils::config::keybindings::KeybindsHolder;
use serde::Deserialize;

use crate::utils::{self, ConfigFetchingError};
use rm_shared::action::Action;

pub use self::actions::{
    general::GeneralAction, search_tab::SearchAction, torrents_tab::TorrentsAction,
};

#[derive(Deserialize, Clone)]
pub struct KeymapConfig {
    pub general: KeybindsHolder<GeneralAction, Action>,
    pub torrents_tab: KeybindsHolder<TorrentsAction, Action>,
    pub search_tab: KeybindsHolder<SearchAction, Action>,
}

impl KeymapConfig {
    pub const FILENAME: &'static str = "keymap.toml";
    pub const DEFAULT_CONFIG: &'static str = include_str!("../../defaults/keymap.toml");

    pub fn init() -> Result<Self> {
        match utils::fetch_config::<Self>(Self::FILENAME) {
            Ok(keymap_config) => Ok(keymap_config),
            Err(e) => match e {
                ConfigFetchingError::Io(e) if e.kind() == ErrorKind::NotFound => {
                    let keymap_config =
                        utils::put_config::<Self>(Self::DEFAULT_CONFIG, Self::FILENAME)?;
                    Ok(keymap_config)
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

    pub fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}
