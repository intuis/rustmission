pub mod components;
pub mod popup;
pub mod tabs;

use crate::ui::tabs::torrents::TorrentsTab;

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui_macros::constraints;
use tui_input::InputRequest;

use crate::{
    action::Action,
    app::{self},
};

use self::{
    components::{tabs::CurrentTab, Component, TabComponent},
    popup::{HelpPopup, Popup},
};

pub struct MainWindow {
    tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: tabs::search::SearchTab,
    popup: Popup,
}

impl MainWindow {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(ctx.clone()),
            search_tab: tabs::search::SearchTab::new(ctx.clone()),
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
            self.popup.handle_actions(action)
        } else {
            match self.tabs.current_tab {
                CurrentTab::Torrents => self.torrents_tab.handle_actions(action),
                CurrentTab::Search => self.search_tab.handle_actions(action),
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_bar, main_window] = Layout::vertical(constraints![==1, ==100%]).areas(rect);

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
    let popup_layout = Layout::vertical(
        constraints![==((100 - percent_y) / 2)%, ==percent_y%, ==((100 - percent_y) / 2)%],
    )
    .split(r);

    Layout::horizontal(
        constraints![==((100 - percent_x) / 2)%, ==percent_x%, ==((100 - percent_x) / 2)%],
    )
    .split(popup_layout[1])[1]
}
