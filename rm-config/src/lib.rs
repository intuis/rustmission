use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::OnceLock,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use toml::Table;
use xdg::BaseDirectories;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connection: Connection,
    pub general: General,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    #[serde(default)]
    pub auto_hide: bool,
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

impl Config {
    pub fn init() -> Result<Self> {
        let table = match Self::table_from_home() {
            Ok(table) => table,
            Err(_) => {
                Self::put_default_conf_in_home()?;
                // TODO: check if the user really changed the config.
                println!(
                    "Update {:?} and start rustmission again",
                    Self::get_config_path()
                );
                std::process::exit(0);
            }
        };

        Self::table_config_verify(&table)?;

        Self::table_to_config(table)
    }

    fn table_from_home() -> Result<Table> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("rustmission")?;
        let config_path = xdg_dirs
            .find_config_file("config.toml")
            .ok_or_else(|| anyhow::anyhow!("config.toml not found"))?;

        let mut config_buf = String::new();
        let mut config_file = File::open(config_path).unwrap();
        config_file.read_to_string(&mut config_buf).unwrap();
        Ok(toml::from_str(&config_buf)?)
    }

    fn put_default_conf_in_home() -> Result<Table> {
        let config_path = Self::get_config_path();
        let mut config_file = File::create(config_path)?;
        config_file.write_all(DEFAULT_CONFIG.as_bytes())?;
        Ok(toml::from_str(DEFAULT_CONFIG)?)
    }

    fn table_to_config(table: Table) -> Result<Self> {
        let config_string = table.to_string();
        let config: Config = toml::from_str(&config_string)?;
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
                    Self::get_config_path().to_str().unwrap()
                )
            })?;

        url::Url::parse(url).with_context(|| {
            format!(
                "invalid url '{url}' in {}",
                Self::get_config_path().to_str().unwrap()
            )
        })?;

        Ok(())
    }

    fn get_xdg_dirs() -> &'static BaseDirectories {
        XDG_DIRS.get_or_init(|| xdg::BaseDirectories::with_prefix("rustmission").unwrap())
    }

    fn get_config_path() -> &'static PathBuf {
        CONFIG_PATH.get_or_init(|| {
            Self::get_xdg_dirs()
                .place_config_file("config.toml")
                .unwrap()
        })
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
