use std::{marker::PhantomData, path::PathBuf, sync::OnceLock};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use indexmap::IndexMap;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use toml::Table;

use crate::utils;
use rm_shared::action::Action;

#[derive(Serialize, Deserialize)]
pub(crate) struct KeymapConfig {
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

#[derive(Serialize)]
struct Keybinding<T: Into<Action>> {
    on: KeyCode,
    #[serde(default)]
    modifier: KeyModifier,
    action: T,
}

impl<T: Into<Action>> Keybinding<T> {
    fn new(on: KeyCode, action: T, modifier: Option<KeyModifier>) -> Self {
        Self {
            on,
            modifier: modifier.unwrap_or(KeyModifier::None),
            action,
        }
    }
}

impl<'de, T: Into<Action> + Deserialize<'de>> Deserialize<'de> for Keybinding<T> {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            On,
            Modifier,
            Action,
        }

        struct KeybindingVisitor<T> {
            phantom: PhantomData<T>,
        }

        impl<'de, T: Into<Action> + Deserialize<'de>> Visitor<'de> for KeybindingVisitor<T> {
            type Value = Keybinding<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Keybinding")
            }

            fn visit_map<A>(self, mut map: A) -> std::prelude::v1::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut on = None;
                let mut modifier = None;
                let mut action = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::On => {
                            if on.is_some() {
                                return Err(de::Error::duplicate_field("on"));
                            }
                            let key = map.next_value::<String>()?;

                            if key.len() == 1 {
                                on = Some(KeyCode::Char(key.chars().next().unwrap()));
                            } else if key.starts_with('F') && (key.len() == 2 || key.len() == 3) {
                                let which_f = key[1..].parse::<u8>().map_err(|_| {
                                    de::Error::invalid_value(
                                        de::Unexpected::Str(&key),
                                        &"something_correct",
                                    )
                                })?;
                                on = Some(KeyCode::F(which_f));
                            } else {
                                on = {
                                    match key.to_lowercase().as_str() {
                                        "enter" => Some(KeyCode::Enter),
                                        "esc" => Some(KeyCode::Esc),
                                        "up" => Some(KeyCode::Up),
                                        "down" => Some(KeyCode::Down),
                                        "left" => Some(KeyCode::Left),
                                        "right" => Some(KeyCode::Right),
                                        "home" => Some(KeyCode::Home),
                                        "end" => Some(KeyCode::End),
                                        "pageup" => Some(KeyCode::PageUp),
                                        "pagedown" => Some(KeyCode::PageDown),
                                        "tab" => Some(KeyCode::Tab),
                                        "backspace" => Some(KeyCode::Backspace),
                                        "delete" => Some(KeyCode::Delete),

                                        _ => {
                                            return Err(de::Error::invalid_value(
                                                de::Unexpected::Str(&key),
                                                &"something correct",
                                            ))
                                        }
                                    }
                                };
                            }
                        }
                        Field::Modifier => {
                            if modifier.is_some() {
                                return Err(de::Error::duplicate_field("modifier"));
                            }
                            modifier = Some(map.next_value());
                        }
                        Field::Action => {
                            if action.is_some() {
                                return Err(de::Error::duplicate_field("action"));
                            }
                            action = Some(map.next_value());
                        }
                    }
                }
                let on = on.ok_or_else(|| de::Error::missing_field("on"))?;
                let action = action.ok_or_else(|| de::Error::missing_field("action"))??;
                Ok(Keybinding::new(on, action, modifier.transpose().unwrap()))
            }
        }

        const FIELDS: &[&str] = &["on", "modifier", "action"];
        deserializer.deserialize_struct(
            "Keybinding",
            FIELDS,
            KeybindingVisitor {
                phantom: PhantomData::default(),
            },
        )
    }
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

impl KeymapConfig {
    pub const FILENAME: &'static str = "keymap.toml";

    pub fn init() -> Result<Self> {
        let table = {
            // TODO: handle errors or there will be hell to pay
            if let Ok(table) = utils::fetch_config(Self::FILENAME) {
                table
            } else {
                todo!();
            }
        };

        Self::table_to_keymap(&table)
    }

    pub fn to_map(self) -> IndexMap<(KeyCode, KeyModifiers), Action> {
        let mut map = IndexMap::new();
        for keybinding in self.general.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            map.insert(hash_value, keybinding.action.into());
        }
        for keybinding in self.torrents_tab.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            map.insert(hash_value, keybinding.action.into());
        }
        map
    }

    fn table_to_keymap(table: &Table) -> Result<Self> {
        let config_string = table.to_string();
        let config = toml::from_str(&config_string)?;
        Ok(config)
    }

    pub fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}
