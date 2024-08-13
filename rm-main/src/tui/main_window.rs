use ratatui::prelude::*;

use rm_shared::action::{Action, UpdateAction};

use crate::tui::components::tabs::CurrentTab;

use super::{
    app,
    components::{tabs::TabComponent, Component, ComponentAction},
    global_popups::{ErrorPopup, GlobalPopupManager},
    tabs::{search::SearchTab, torrents::TorrentsTab},
};

pub struct MainWindow {
    pub tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: SearchTab,
    global_popup_manager: GlobalPopupManager,
}

impl MainWindow {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            tabs: TabComponent::new(ctx.clone()),
            torrents_tab: TorrentsTab::new(ctx.clone()),
            search_tab: SearchTab::new(ctx.clone()),
            global_popup_manager: GlobalPopupManager::new(ctx),
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
            A::ChangeTab(_) | A::Left | A::Right => {
                self.tabs.handle_actions(action);
            }
            _ if self.tabs.current_tab == CurrentTab::Torrents => {
                self.torrents_tab.handle_actions(action);
            }
            _ if self.tabs.current_tab == CurrentTab::Search => {
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
            action if self.tabs.current_tab == CurrentTab::Torrents => {
                self.torrents_tab.handle_update_action(action)
            }
            action if self.tabs.current_tab == CurrentTab::Search => {
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

        self.tabs.render(f, top_bar);

        match self.tabs.current_tab {
            CurrentTab::Torrents => self.torrents_tab.render(f, main_window),
            CurrentTab::Search => self.search_tab.render(f, main_window),
        }

        self.global_popup_manager.render(f, f.size());
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
