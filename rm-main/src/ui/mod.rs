pub mod components;
pub mod global_popups;
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
    global_popups::{GlobalPopupManager, HelpPopup},
};

pub struct MainWindow {
    ctx: app::Ctx,
    tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: tabs::search::SearchTab,
    global_popup_manager: GlobalPopupManager,
}

impl MainWindow {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            tabs: TabComponent::new(ctx.clone()),
            torrents_tab: TorrentsTab::new(ctx.clone()),
            search_tab: tabs::search::SearchTab::new(ctx.clone()),
            global_popup_manager: GlobalPopupManager::default(),
        }
    }
}

impl Component for MainWindow {
    // Rewrite this to one big match
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Error(e_popup) = action {
            self.global_popup_manager.error_popup = Some(*e_popup);
            return Some(Action::Render);
        }

        if let Action::ShowHelp = action {
            if self.global_popup_manager.help_popup.is_some() {
                self.global_popup_manager.help_popup = None;
            } else {
                self.global_popup_manager.help_popup = Some(HelpPopup::new(self.ctx.clone()));
            }
            return Some(Action::Render);
        }

        if self.global_popup_manager.needs_action() {
            self.global_popup_manager.handle_actions(action)
        } else {
            if let Action::ChangeTab(_) = action {
                self.tabs.handle_actions(action);
                return Some(Action::Render);
            }

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

        self.global_popup_manager.render(f, f.size());
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

pub fn seconds_to_human_format(seconds: i64) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = MINUTE * 60;
    const DAY: i64 = HOUR * 24;

    if seconds == 0 {
        return "0s".to_string();
    }

    let mut curr_string = String::new();

    let mut rest = seconds;
    if seconds > DAY {
        let days = rest / DAY;
        rest = seconds % DAY;

        curr_string = format!("{curr_string}{days}d");
    }

    if seconds > HOUR {
        let hours = rest / HOUR;
        rest = rest % HOUR;
        curr_string = format!("{curr_string}{hours}h");
    }

    if seconds > MINUTE {
        let minutes = rest / MINUTE;
        rest = rest % MINUTE;
        curr_string = format!("{curr_string}{minutes}m");
    }

    curr_string = format!("{curr_string}{rest}s");
    curr_string
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
