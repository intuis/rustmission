use std::collections::HashMap;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{utils, KEYMAP_CONFIG_FILENAME};
use rm_shared::action::Action;

#[derive(Serialize, Deserialize)]
pub struct Keymap {
    general: General<GeneralAction>,
    torrents_tab: TorrentsTab<TorrentsAction>,
}

#[derive(Serialize, Deserialize)]
struct General<T: Into<Action>> {
    keybindings: Vec<Keybinding<T>>,
}

#[derive(Serialize, Deserialize, Debug)]
enum GeneralAction {
    ShowHelp,
    Quit,
    SoftQuit,
    SwitchToTorrents,
    SwitchToSearch,
    Left,
    Right,
    Down,
    Up,
    Search,
    SwitchFocus,
    Confirm,
    PageDown,
    PageUp,
    Home,
    End,
}

impl From<GeneralAction> for Action {
    fn from(value: GeneralAction) -> Self {
        match value {
            GeneralAction::ShowHelp => Action::ShowHelp,
            GeneralAction::Quit => Action::Quit,
            GeneralAction::SoftQuit => Action::SoftQuit,
            GeneralAction::SwitchToTorrents => Action::ChangeTab(1),
            GeneralAction::SwitchToSearch => Action::ChangeTab(2),
            GeneralAction::Left => Action::Left,
            GeneralAction::Right => Action::Right,
            GeneralAction::Down => Action::Down,
            GeneralAction::Up => Action::Up,
            GeneralAction::Search => Action::Search,
            GeneralAction::SwitchFocus => Action::ChangeFocus,
            GeneralAction::Confirm => Action::Confirm,
            GeneralAction::PageDown => Action::ScrollDownPage,
            GeneralAction::PageUp => Action::ScrollUpPage,
            GeneralAction::Home => Action::Home,
            GeneralAction::End => Action::End,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TorrentsTab<T: Into<Action>> {
    keybindings: Vec<Keybinding<T>>,
}

#[derive(Serialize, Deserialize, Debug)]
enum TorrentsAction {
    AddMagnet,
    Pause,
    DeleteWithFiles,
    DeleteWithoutFiles,
    ShowFiles,
    ShowStats,
}

impl From<TorrentsAction> for Action {
    fn from(value: TorrentsAction) -> Self {
        match value {
            TorrentsAction::AddMagnet => Action::AddMagnet,
            TorrentsAction::Pause => Action::Pause,
            TorrentsAction::DeleteWithFiles => Action::DeleteWithFiles,
            TorrentsAction::DeleteWithoutFiles => Action::DeleteWithoutFiles,
            TorrentsAction::ShowFiles => Action::ShowFiles,
            TorrentsAction::ShowStats => Action::ShowStats,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Keybinding<T: Into<Action>> {
    on: KeyCode,
    #[serde(default)]
    modifier: KeyModifier,
    action: T,
}

#[derive(Serialize, Deserialize, Hash)]
enum KeyModifier {
    None,
    Ctrl,
    Shift,
}

impl From<KeyModifier> for KeyModifiers {
    fn from(value: KeyModifier) -> Self {
        match value {
            KeyModifier::None => KeyModifiers::NONE,
            KeyModifier::Ctrl => KeyModifiers::CONTROL,
            KeyModifier::Shift => KeyModifiers::SHIFT,
        }
    }
}

impl Default for KeyModifier {
    fn default() -> Self {
        Self::None
    }
}

impl Keymap {
    pub fn init() -> Result<Self> {
        let table = {
            if let Ok(table) = utils::fetch_config_table(KEYMAP_CONFIG_FILENAME) {
                table
            } else {
                todo!();
            }
        };

        Self::table_to_keymap(&table)
    }

    pub fn to_hashmap(self) -> HashMap<(KeyCode, KeyModifiers), Action> {
        let mut hashmap = HashMap::new();
        for keybinding in self.general.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            hashmap.insert(hash_value, keybinding.action.into());
        }
        for keybinding in self.torrents_tab.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            hashmap.insert(hash_value, keybinding.action.into());
        }
        hashmap
    }

    fn table_to_keymap(table: &Table) -> Result<Self> {
        let config_string = table.to_string();
        let config = toml::from_str(&config_string)?;
        Ok(config)
    }
}
