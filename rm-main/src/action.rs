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
    Left,
    Right,
    ScrollDownPage,
    ScrollUpPage,
    Home,
    End,
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
    MoveTorrent,
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
        Event::Quit => Some(A::Quit),
        Event::Error => todo!(),
        Event::Render => Some(A::Render),
        Event::Key(key) if mode == Mode::Input => Some(A::Input(key)),
        Event::Key(key) => key_event_to_action(key),
    }
}

fn key_event_to_action(key: KeyEvent) -> Option<Action> {
    use Action as A;

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(A::ScrollDownPage),
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(A::ScrollUpPage),
        (_, keycode) => keycode_to_action(keycode),
    }
}

fn keycode_to_action(key: KeyCode) -> Option<Action> {
    use Action as A;
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(A::Quit),
        KeyCode::Esc => Some(A::SoftQuit),
        KeyCode::Tab => Some(A::ChangeFocus),
        KeyCode::Home => Some(A::Home),
        KeyCode::End => Some(A::End),
        KeyCode::PageUp => Some(A::ScrollUpPage),
        KeyCode::PageDown => Some(A::ScrollDownPage),
        KeyCode::Char('j') | KeyCode::Down => Some(A::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(A::Up),
        KeyCode::Char('h') | KeyCode::Left => Some(A::Left),
        KeyCode::Char('l') | KeyCode::Right => Some(A::Right),
        KeyCode::Char('?') | KeyCode::F(1) => Some(A::ShowHelp),
        KeyCode::Char('s') => Some(A::ShowStats),
        KeyCode::Char('f') => Some(A::ShowFiles),
        KeyCode::Char('/') => Some(A::Search),
        KeyCode::Char('a') => Some(A::AddMagnet),
        KeyCode::Char('m') => Some(A::MoveTorrent),
        KeyCode::Char('p') => Some(A::Pause),
        KeyCode::Char('d') => Some(A::DeleteWithoutFiles),
        KeyCode::Char('D') => Some(A::DeleteWithFiles),
        KeyCode::Char(' ') => Some(A::Space),
        KeyCode::Char(n @ '1'..='9') => {
            Some(A::ChangeTab(n.to_digit(10).expect("This is ok") as u8))
        }
        KeyCode::Enter => Some(A::Confirm),
        _ => None,
    }
}
