use crossterm::event::{KeyCode, KeyModifiers};
use rm_config::CONFIG;
use rm_shared::{
    action::Action,
    current_window::{TorrentWindow, Window},
};

pub struct AppKeyEvent(crossterm::event::KeyEvent);

impl From<crossterm::event::KeyEvent> for AppKeyEvent {
    fn from(value: crossterm::event::KeyEvent) -> Self {
        Self(value)
    }
}

impl AppKeyEvent {
    pub fn is_ctrl_c(&self) -> bool {
        if self.0.modifiers == KeyModifiers::CONTROL
            && (self.0.code == KeyCode::Char('c') || self.0.code == KeyCode::Char('C'))
        {
            return true;
        }
        false
    }

    pub fn to_action(&self, current_window: Window) -> Option<Action> {
        let keymap = match current_window {
            Window::Torrents(torrents_tab_current_window) => match torrents_tab_current_window {
                TorrentWindow::General => &CONFIG.keybindings.torrents_tab.map,
                TorrentWindow::FileViewer => &CONFIG.keybindings.torrents_tab_file_viewer.map,
            },
            Window::Search(_) => &CONFIG.keybindings.search_tab.map,
        };

        let keybinding = self.keybinding();

        for keymap in [&CONFIG.keybindings.general.map, keymap] {
            if let Some(action) = keymap.get(&keybinding).cloned() {
                return Some(action);
            }
        }
        None
    }

    fn keybinding(&self) -> (KeyCode, KeyModifiers) {
        match self.0.code {
            KeyCode::Char(e) => {
                let modifier = if e.is_uppercase() {
                    KeyModifiers::NONE
                } else {
                    self.0.modifiers
                };
                (self.0.code, modifier)
            }
            _ => (self.0.code, self.0.modifiers),
        }
    }
}
