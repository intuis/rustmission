mod connection;
mod general;
mod icons;
mod search_tab;
mod torrents_tab;

pub use connection::Connection;
pub use general::General;
pub use icons::Icons;
use intuitils::config::IntuiConfig;
pub use search_tab::SearchTab;
pub use torrents_tab::TorrentsTab;

use serde::Deserialize;

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

impl IntuiConfig for MainConfig {
    fn app_name() -> &'static str {
        "rustmission"
    }

    fn filename() -> &'static str {
        "config.toml"
    }

    fn default_config() -> &'static str {
        include_str!("../../defaults/config.toml")
    }

    fn should_exit_if_not_found() -> bool {
        true
    }

    fn message_if_not_found() -> Option<String> {
        Some(format!(
            "Update {:?} (especially connection url) and start rustmission again",
            Self::path()
        ))
    }
}
