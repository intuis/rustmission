use std::fmt::Display;

use intui_tabs::{Tabs, TabsState};
use ratatui::prelude::*;

use rm_config::CONFIG;
use rm_shared::action::{Action, UpdateAction};

use crate::tui::app::CTX;

use super::{
    app,
    components::{Component, ComponentAction},
    global_popups::{ErrorPopup, GlobalPopupManager},
    tabs::{search::SearchTab, torrents::TorrentsTab},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CurrentTab {
    Torrents = 0,
    Search,
}

impl Default for CurrentTab {
    fn default() -> Self {
        CurrentTab::Torrents
    }
}

impl Display for CurrentTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentTab::Torrents => write!(f, "Torrents"),
            CurrentTab::Search => write!(f, "Search"),
        }
    }
}

pub struct MainWindow {
    pub tabs: intui_tabs::TabsState<CurrentTab>,
    torrents_tab: TorrentsTab,
    search_tab: SearchTab,
    global_popup_manager: GlobalPopupManager,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            tabs: TabsState::new(vec![CurrentTab::Torrents, CurrentTab::Search]),
            torrents_tab: TorrentsTab::new(),
            search_tab: SearchTab::new(),
            global_popup_manager: GlobalPopupManager::new(),
        }
    }
}

impl Component for MainWindow {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;

        match action {
            A::ShowHelp => {
                self.global_popup_manager.handle_actions(action);
            }
            _ if self.global_popup_manager.needs_action() => {
                self.global_popup_manager.handle_actions(action);
            }
            A::Left | A::ChangeTab(1) => {
                if self.tabs.current() != CurrentTab::Torrents {
                    self.tabs.set(1);
                    CTX.send_action(Action::Render);
                }
            }
            A::Right | A::ChangeTab(2) => {
                if self.tabs.current() != CurrentTab::Search {
                    self.tabs.set(2);
                    CTX.send_action(Action::Render);
                }
            }
            _ if self.tabs.current() == CurrentTab::Torrents => {
                self.torrents_tab.handle_actions(action);
            }
            _ if self.tabs.current() == CurrentTab::Search => {
                self.search_tab.handle_actions(action);
            }
            _ => unreachable!(),
        };

        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::Error(err) => {
                let error_popup = ErrorPopup::new(err.title, err.description, err.source);
                self.global_popup_manager.error_popup = Some(error_popup);
            }
            action if self.tabs.current() == CurrentTab::Torrents => {
                self.torrents_tab.handle_update_action(action)
            }
            action if self.tabs.current() == CurrentTab::Search => {
                self.search_tab.handle_update_action(action)
            }
            _ => unreachable!(),
        }
    }

    fn tick(&mut self) {
        self.search_tab.tick();
        self.torrents_tab.tick();
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_bar, main_window] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(rect);

        let tabs = Tabs::new()
            .beginner_mode(CONFIG.general.beginner_mode)
            .color(CONFIG.general.accent_color);
        f.render_stateful_widget(tabs, top_bar, &mut self.tabs);

        match self.tabs.current() {
            CurrentTab::Torrents => self.torrents_tab.render(f, main_window),
            CurrentTab::Search => self.search_tab.render(f, main_window),
        }

        self.global_popup_manager.render(f, f.area());
    }
}

pub fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
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
