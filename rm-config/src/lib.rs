mod keymap;
mod main_config;
mod utils;

use indexmap::IndexMap;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use keymap::KeymapConfig;
use main_config::MainConfig;

use rm_shared::action::Action;

pub struct Config {
    pub general: main_config::General,
    pub connection: main_config::Connection,
    pub keymap: IndexMap<(KeyCode, KeyModifiers), Action>,
    pub directories: Directories,
}

pub struct Directories {
    pub main_path: &'static PathBuf,
    pub keymap_path: &'static PathBuf,
}

impl Config {
    pub fn init() -> Result<Self> {
        let main_config = MainConfig::init()?;
        let keybindings = KeymapConfig::init()?;

        let directories = Directories {
            main_path: MainConfig::path(),
            keymap_path: KeymapConfig::path(),
        };

        Ok(Self {
            general: main_config.general,
            connection: main_config.connection,
            keymap: keybindings.to_map(),
            directories,
        })
    }
}
