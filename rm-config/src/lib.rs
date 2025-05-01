pub mod categories;
pub mod keymap;
pub mod main_config;

use std::{path::PathBuf, sync::LazyLock};

use categories::CategoriesConfig;
use color_eyre::Result;
use intuitils::config::IntuiConfig;
use keymap::KeymapConfig;
use main_config::MainConfig;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::init().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1);
    })
});

pub struct Config {
    pub general: main_config::General,
    pub connection: main_config::Connection,
    pub torrents_tab: main_config::TorrentsTab,
    pub search_tab: main_config::SearchTab,
    pub icons: main_config::Icons,
    pub keybindings: KeymapConfig,
    pub categories: CategoriesConfig,
    pub directories: Directories,
}

pub struct Directories {
    pub main_path: &'static PathBuf,
    pub keymap_path: &'static PathBuf,
    pub categories_path: &'static PathBuf,
}

impl Config {
    fn init() -> Result<Self> {
        let main_config = MainConfig::init()?;
        let keybindings = KeymapConfig::init()?;
        let categories = CategoriesConfig::init()?;

        let directories = Directories {
            main_path: MainConfig::path(),
            keymap_path: KeymapConfig::path(),
            categories_path: CategoriesConfig::path(),
        };

        Ok(Self {
            general: main_config.general,
            connection: main_config.connection,
            torrents_tab: main_config.torrents_tab,
            search_tab: main_config.search_tab,
            icons: main_config.icons,
            keybindings,
            categories,
            directories,
        })
    }
}
