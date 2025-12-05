pub mod actions;

use std::collections::HashMap;

use actions::torrents_tab_file_viewer::TorrentsFileViewerAction;
use crossterm::event::KeyModifiers;
use intuitils::config::{
    keybindings::{KeyModifier, Keybinding, KeybindsHolder},
    IntuiConfig,
};
use serde::Deserialize;

use rm_shared::action::Action;

pub use self::actions::{
    general::GeneralAction, search_tab::SearchAction, torrents_tab::TorrentsAction,
};

#[derive(Deserialize, Clone)]
pub struct KeymapConfig {
    pub general: KeybindsHolder<GeneralAction, Action>,
    pub torrents_tab: KeybindsHolder<TorrentsAction, Action>,
    #[serde(default = "default_torrents_tab_file_viewer")]
    pub torrents_tab_file_viewer: KeybindsHolder<TorrentsFileViewerAction, Action>,
    pub search_tab: KeybindsHolder<SearchAction, Action>,
}

fn default_torrents_tab_file_viewer() -> KeybindsHolder<TorrentsFileViewerAction, Action> {
    // Set default bind for changing file priority
    let keycode = crossterm::event::KeyCode::Char('p');
    let action = TorrentsFileViewerAction::ChangeFilePriority;

    let mut map = HashMap::new();
    map.insert((keycode, KeyModifiers::NONE), Action::ChangeFilePriority);

    KeybindsHolder {
        keybindings: vec![Keybinding {
            on: keycode,
            modifier: KeyModifier::None,
            action,
            show_in_help: true,
        }],
        map,
    }
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
