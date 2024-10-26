pub mod actions;

use intuitils::config::{keybindings::KeybindsHolder, IntuiConfig};
use serde::Deserialize;

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

impl IntuiConfig for KeymapConfig {
    fn app_name() -> &'static str {
        "rustmission"
    }

    fn filename() -> &'static str {
        "keymap.toml"
    }

    fn default_config() -> &'static str {
        include_str!("../../defaults/keymap.toml")
    }

    fn should_exit_if_not_found() -> bool {
        false
    }

    fn message_if_not_found() -> Option<String> {
        None
    }
}
