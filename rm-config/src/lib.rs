mod keymap;
mod utils;

use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

use anyhow::{bail, Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use rm_shared::action::Action;
use serde::{Deserialize, Serialize};
use toml::Table;
use xdg::BaseDirectories;

use crate::utils::put_config;
use keymap::Keymap;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub connection: Connection,
    pub general: General,
    #[serde(skip)]
    pub keymap: Option<HashMap<(KeyCode, KeyModifiers), Action>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    #[serde(default)]
    pub auto_hide: bool,
    #[serde(default = "default_accent_color")]
    pub accent_color: Color,
    #[serde(default = "default_beginner_mode")]
    pub beginner_mode: bool,
}

fn default_accent_color() -> Color {
    Color::LightMagenta
}

fn default_beginner_mode() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: String,
}

const DEFAULT_CONFIG: &str = include_str!("../defaults/config.toml");
static XDG_DIRS: OnceLock<BaseDirectories> = OnceLock::new();
static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
pub const MAIN_CONFIG_FILENAME: &str = "config.toml";
pub const KEYMAP_CONFIG_FILENAME: &str = "keymap.toml";

pub fn xdg_dirs() -> &'static BaseDirectories {
    XDG_DIRS.get_or_init(|| xdg::BaseDirectories::with_prefix("rustmission").unwrap())
}

pub fn get_config_path(filename: &str) -> &'static PathBuf {
    CONFIG_PATH.get_or_init(|| xdg_dirs().place_config_file(filename).unwrap())
}

impl Config {
    pub fn init() -> Result<Self> {
        let Ok(table) = utils::fetch_config_table(MAIN_CONFIG_FILENAME) else {
            put_config(DEFAULT_CONFIG, MAIN_CONFIG_FILENAME)?;
            // TODO: check if the user really changed the config.
            println!(
                "Update {:?} and start rustmission again",
                get_config_path(MAIN_CONFIG_FILENAME)
            );
            std::process::exit(0);
        };

        Self::table_config_verify(&table)?;

        let mut config = Self::table_to_config(&table)?;
        config.keymap = Some(Keymap::init().unwrap().to_hashmap());
        Ok(config)
    }

    fn table_to_config(table: &Table) -> Result<Self> {
        let config_string = table.to_string();
        let config: Self = toml::from_str(&config_string)?;
        Ok(config)
    }

    fn table_config_verify(table: &Table) -> Result<()> {
        let Some(connection_table) = table.get("connection").unwrap().as_table() else {
            bail!("expected connection table")
        };

        let url = connection_table
            .get("url")
            .and_then(|url| url.as_str())
            .with_context(|| {
                format!(
                    "no url given in: {}",
                    get_config_path(MAIN_CONFIG_FILENAME).to_str().unwrap()
                )
            })?;

        url::Url::parse(url).with_context(|| {
            format!(
                "invalid url '{url}' in {}",
                get_config_path(MAIN_CONFIG_FILENAME).to_str().unwrap()
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invalid_config() -> Table {
        toml::toml! {
            [connection]
            username = "username"
            password = "password"
            auto_hide = "dfgoij"
            url = "bad_url"
        }
    }

    fn valid_config() -> Table {
        toml::toml! {
            [connection]
            username = "username"
            password = "password"
            url = "http://192.168.1.1/transmission/rpc"
        }
    }

    #[test]
    fn validates_properly() {
        let valid_config = valid_config();
        assert!(Config::table_config_verify(&valid_config).is_ok());
    }

    #[test]
    fn invalidates_properly() {
        let invalid_config = invalid_config();
        assert!(Config::table_config_verify(&invalid_config).is_err());
    }
}
