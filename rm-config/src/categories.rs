use std::{collections::HashMap, io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::Deserialize;

use crate::utils::{self, ConfigFetchingError};

#[derive(Deserialize)]
pub struct CategoriesConfig {
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(skip)]
    pub map: HashMap<String, Category>,
    #[serde(skip)]
    pub max_name_len: u8,
    #[serde(skip)]
    pub max_icon_len: u8,
}

#[derive(Deserialize, Clone)]
pub struct Category {
    pub name: String,
    pub icon: String,
    pub color: Color,
    pub default_dir: String,
}

impl CategoriesConfig {
    pub(crate) const FILENAME: &'static str = "categories.toml";
    pub const DEFAULT_CONFIG: &'static str = include_str!("../defaults/categories.toml");

    pub(crate) fn init() -> Result<Self> {
        match utils::fetch_config::<Self>(Self::FILENAME) {
            Ok(mut config) => {
                config.after_init();
                Ok(config)
            }
            Err(e) => match e {
                ConfigFetchingError::Io(e) if e.kind() == ErrorKind::NotFound => {
                    let mut config =
                        utils::put_config::<Self>(Self::DEFAULT_CONFIG, Self::FILENAME)?;
                    config.after_init();
                    Ok(config)
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

impl CategoriesConfig {
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }

    fn after_init(&mut self) {
        self.populate_hashmap();
        self.set_lengths();
    }

    fn populate_hashmap(&mut self) {
        for category in &self.categories {
            self.map.insert(category.name.clone(), category.clone());
        }
    }

    fn set_lengths(&mut self) {
        let mut max_icon_len = 0u8;
        let mut max_name_len = 0u8;

        for category in &self.categories {
            let name_len = u8::try_from(category.name.chars().count()).unwrap_or(u8::MAX);
            let icon_len = u8::try_from(category.icon.chars().count()).unwrap_or(u8::MAX);

            if name_len > max_name_len {
                max_name_len = name_len;
            }

            if icon_len > max_icon_len {
                max_icon_len = icon_len
            }
        }

        self.max_name_len = max_name_len;
        self.max_icon_len = max_icon_len;
    }
}
