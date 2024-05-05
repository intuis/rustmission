use std::sync::{Arc, OnceLock};

use crossterm::event::{KeyCode, KeyEvent};
use transmission_rpc::types::{Id, Torrent};

use crate::{tui::Event, ui::popup::ErrorPopup};

#[derive(Debug)]
pub(crate) enum TorrentAction {
    Add(String),
    Stop(Vec<Id>),
    Start(Vec<Id>),
    DeleteWithoutFiles(Vec<Id>),
    DeleteWithFiles(Vec<Id>),
    GetTorrentInfo(Id, Arc<OnceLock<Torrent>>),
}

#[derive(Debug, Clone)]
pub(crate) enum Action {
    Quit,
    Render,
    Up,
    Down,
    Confirm,
    ShowHelp,
    ShowStats,
    ShowInfo,
    Search,
    Pause,
    DeleteWithoutFiles,
    DeleteWithFiles,
    SwitchToInputMode,
    SwitchToNormalMode,
    ChangeFocus,
    AddMagnet,
    ChangeTab(u8),
    Input(KeyEvent),
    Error(Box<ErrorPopup>),
}

impl Action {
    pub const fn is_render(&self) -> bool {
        matches!(self, Self::Render)
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    Input,
    Normal,
}

pub fn event_to_action(mode: Mode, event: Event) -> Option<Action> {
    match event {
        Event::Quit => Some(Action::Quit),
        Event::Error => todo!(),
        Event::Render => Some(Action::Render),
        Event::Key(key) if matches!(mode, Mode::Input) => Some(Action::Input(key)),
        Event::Key(key) => keycode_to_action(key),
    }
}

fn keycode_to_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Tab => Some(Action::ChangeFocus),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        KeyCode::Char('t') => Some(Action::ShowStats),
        KeyCode::Char('i') => Some(Action::ShowInfo),
        KeyCode::Char('/') => Some(Action::Search),
        KeyCode::Char('a') => Some(Action::AddMagnet),
        KeyCode::Char('p') => Some(Action::Pause),
        KeyCode::Char('d') => Some(Action::DeleteWithoutFiles),
        KeyCode::Char('D') => Some(Action::DeleteWithFiles),
        KeyCode::Char(n @ '1'..='9') => {
            Some(Action::ChangeTab(n.to_digit(10).expect("This is ok") as u8))
        }
        KeyCode::Enter => Some(Action::Confirm),
        _ => None,
    }
}
