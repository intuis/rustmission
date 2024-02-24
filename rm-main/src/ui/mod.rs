pub mod components;
pub mod popup;
mod search_tab;

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::InputRequest;

use crate::action::Action;

use self::{
    components::{tabcomponent::CurrentTab, Component, TabComponent, TorrentsTab},
    popup::{HelpPopup, Popup},
    search_tab::SearchTab,
};

pub struct MainWindow {
    tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: SearchTab,
    popup: Popup,
}

impl MainWindow {
    pub fn new(action_tx: UnboundedSender<Action>, trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(trans_tx.clone()),
            search_tab: SearchTab::new(action_tx, trans_tx),
            popup: Popup::default(),
        }
    }
}

impl Component for MainWindow {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Error(e_popup) = action {
            self.popup.error_popup = Some(*e_popup);
            return Some(Action::Render);
        }

        if let Action::ShowHelp = action {
            if self.popup.help_popup.is_some() {
                self.popup.help_popup = None;
            } else {
                self.popup.help_popup = Some(HelpPopup);
            }
            return Some(Action::Render);
        }

        if let Action::ChangeTab(_) = action {
            self.tabs.handle_actions(action);
            return Some(Action::Render);
        }

        if self.popup.needs_action() {
            return self.popup.handle_actions(action);
        } else {
            match self.tabs.current_tab {
                CurrentTab::Torrents => return self.torrents_tab.handle_actions(action),
                CurrentTab::Search => self.search_tab.handle_actions(action),
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_bar, main_window] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(rect);

        self.tabs.render(f, top_bar);

        match self.tabs.current_tab {
            CurrentTab::Torrents => self.torrents_tab.render(f, main_window),
            CurrentTab::Search => self.search_tab.render(f, main_window),
        }

        self.popup.render(f, f.size());
    }
}

const fn to_input_request(keycode: KeyCode) -> Option<InputRequest> {
    use InputRequest as R;

    match keycode {
        KeyCode::Backspace => Some(R::DeletePrevChar),
        KeyCode::Delete => Some(R::DeleteNextChar),
        KeyCode::Char(char) => Some(R::InsertChar(char)),
        _ => None,
    }
}

pub fn bytes_to_human_format(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let (value, unit) = if bytes < (KB - 25f64) as i64 {
        (bytes as f64, "B")
    } else if bytes < (MB - 25f64) as i64 {
        (bytes as f64 / KB, "KB")
    } else if bytes < (GB - 25f64) as i64 {
        (bytes as f64 / MB, "MB")
    } else if bytes < (TB - 25f64) as i64 {
        (bytes as f64 / GB, "GB")
    } else {
        (bytes as f64 / TB, "TB")
    };

    format!("{value:.1} {unit}")
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
