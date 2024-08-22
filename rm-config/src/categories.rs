use std::{collections::HashMap, io::ErrorKind, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::Deserialize;

use crate::utils::{self, ConfigFetchingError};

#[derive(Deserialize)]
pub struct CategoriesConfig {
    #[serde(default)]
    categories: Vec<Category>,
}

#[derive(Deserialize)]
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
            Ok(config) => Ok(config),
            Err(e) => match e {
                ConfigFetchingError::Io(e) if e.kind() == ErrorKind::NotFound => {
                    let categories_config =
                        utils::put_config::<Self>(Self::DEFAULT_CONFIG, Self::FILENAME)?;
                    Ok(categories_config)
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
    pub fn to_hashmap(self) -> HashMap<String, Category> {
        let mut hashmap = HashMap::new();
        for category in self.categories {
            hashmap.insert(category.name.clone(), category);
        }

        hashmap
    }
}
