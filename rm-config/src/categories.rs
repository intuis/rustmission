use std::collections::HashMap;

use intuitils::config::IntuiConfig;
use ratatui::style::Color;
use serde::Deserialize;

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

impl IntuiConfig for CategoriesConfig {
    fn app_name() -> &'static str {
        "rustmission"
    }

    fn filename() -> &'static str {
        "categories.toml"
    }

    fn default_config() -> &'static str {
        include_str!("../defaults/categories.toml")
    }

    fn should_exit_if_not_found() -> bool {
        false
    }

    fn message_if_not_found() -> Option<String> {
        None
    }

    fn post_init(&mut self) {
        self.populate_hashmap();
        self.set_lengths();
    }
}

#[derive(Deserialize, Clone)]
pub struct Category {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default = "default_color_category")]
    pub color: Color,
    pub default_dir: Option<String>,
}

fn default_color_category() -> Color {
    Color::White
}

impl CategoriesConfig {
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
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
