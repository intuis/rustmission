use std::{collections::HashMap, sync::Arc};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use magnetease::Magnet;
use transmission_rpc::types::{FreeSpace, SessionStats};

use crate::{rustmission_torrent::RustmissionTorrent, status_task::StatusTask};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    HardQuit,
    Quit,
    Close,
    Render,
    Up,
    Down,
    Left,
    Right,
    ScrollDownPage,
    ScrollUpPage,
    Home,
    End,
    Confirm,
    Select,
    ShowHelp,
    ShowStats,
    ShowFiles,
    Search,
    Pause,
    DeleteWithoutFiles,
    DeleteWithFiles,
    ChangeFocus,
    AddMagnet,
    MoveTorrent,
    ChangeTab(u8),
    Input(KeyEvent),
    Error(Box<ErrorMessage>),
    TaskPending(StatusTask),
    TaskSuccess,
}

pub enum UpdateAction {
    SwitchToInputMode,
    SwitchToNormalMode,
    // Torrents Tab
    TaskClear,
    SessionStats(Arc<SessionStats>),
    FreeSpace(Arc<FreeSpace>),
    UpdateTorrents(Vec<RustmissionTorrent>),
    SearchFilterApply(String),
    SearchFilterClear,
    // Search Tab
    SearchStarted,
    SearchResults(Vec<Magnet>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMessage {
    pub title: String,
    pub message: String,
}

impl Action {
    pub fn is_render(&self) -> bool {
        *self == Self::Render
    }

    pub fn is_quit(&self) -> bool {
        *self == Self::HardQuit || *self == Self::Quit
    }

    pub fn is_soft_quit(&self) -> bool {
        self.is_quit() || *self == Self::Close
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input,
    Normal,
}

pub fn event_to_action(
    mode: Mode,
    event: Event,
    keymap: &HashMap<(KeyCode, KeyModifiers), Action>,
) -> Option<Action> {
    use Action as A;

    // Handle CTRL+C first
    if let Event::Key(key_event) = event {
        if key_event.modifiers == KeyModifiers::CONTROL
            && (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        {
            return Some(A::HardQuit);
        }
    }

    match event {
        Event::Key(key) if mode == Mode::Input => Some(A::Input(key)),
        Event::Key(key) => {
            if let KeyCode::Char(e) = key.code {
                if e.is_uppercase() {
                    return keymap.get(&(key.code, KeyModifiers::NONE)).cloned();
                }
            }
            keymap.get(&(key.code, key.modifiers)).cloned()
        }
        Event::Resize(_, _) => Some(A::Render),
        _ => None,
    }
}
