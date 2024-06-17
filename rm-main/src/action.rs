use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{tui::Event, ui::global_popups::ErrorPopup};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    HardQuit,
    Quit,
    SoftQuit,
    Render,
    Up,
    Down,
    Confirm,
    Space,
    ShowHelp,
    ShowStats,
    ShowFiles,
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
    pub fn is_render(&self) -> bool {
        *self == Self::Render
    }

    pub fn is_quit(&self) -> bool {
        *self == Self::HardQuit || *self == Self::Quit
    }

    pub fn is_soft_quit(&self) -> bool {
        self.is_quit() || *self == Self::SoftQuit
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input,
    Normal,
}

pub fn event_to_action(mode: Mode, event: Event) -> Option<Action> {
    // Handle CTRL+C first
    if let Event::Key(key_event) = event {
        if key_event.modifiers == KeyModifiers::CONTROL
            && (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        {
            return Some(Action::HardQuit);
        }
    }

    match event {
        Event::Quit => Some(Action::Quit),
        Event::Error => todo!(),
        Event::Render => Some(Action::Render),
        Event::Key(key) if mode == Mode::Input => Some(Action::Input(key)),
        Event::Key(key) => key_event_to_action(key),
    }
}

fn key_event_to_action(key: KeyEvent) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, keycode) => keycode_to_action(keycode),
    }
}

fn keycode_to_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::SoftQuit),
        KeyCode::Tab => Some(Action::ChangeFocus),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
        KeyCode::Char('?') | KeyCode::F(1) => Some(Action::ShowHelp),
        KeyCode::Char('s') => Some(Action::ShowStats),
        KeyCode::Char('f') => Some(Action::ShowFiles),
        KeyCode::Char('/') => Some(Action::Search),
        KeyCode::Char('a') => Some(Action::AddMagnet),
        KeyCode::Char('p') => Some(Action::Pause),
        KeyCode::Char('d') => Some(Action::DeleteWithoutFiles),
        KeyCode::Char('D') => Some(Action::DeleteWithFiles),
        KeyCode::Char(' ') => Some(Action::Space),
        KeyCode::Char(n @ '1'..='9') => {
            Some(Action::ChangeTab(n.to_digit(10).expect("This is ok") as u8))
        }
        KeyCode::Enter => Some(Action::Confirm),
        _ => None,
    }
}
