use crate::tui::{
    components::{Component, ComponentAction},
    ctx::CTX,
};

use self::{files::FilesPopup, stats::StatisticsPopup};
use details::DetailsPopup;
use rm_shared::{
    action::{Action, UpdateAction},
    current_window::TorrentWindow,
};

use ratatui::prelude::*;

pub mod details;
pub mod files;
pub mod stats;

pub struct PopupManager {
    pub current_popup: Option<CurrentPopup>,
}

pub enum CurrentPopup {
    Stats(StatisticsPopup),
    Files(FilesPopup),
    Details(DetailsPopup),
}

impl PopupManager {
    pub const fn new() -> Self {
        Self {
            current_popup: None,
        }
    }

    pub const fn is_showing_popup(&self) -> bool {
        self.current_popup.is_some()
    }

    pub fn show_popup(&mut self, popup: CurrentPopup) {
        self.current_popup = Some(popup);
    }

    pub fn close_popup(&mut self) {
        self.current_popup = None;
    }
}

impl Component for PopupManager {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        if let Some(current_popup) = &mut self.current_popup {
            let should_close = match current_popup {
                CurrentPopup::Stats(popup) => popup.handle_actions(action).is_quit(),
                CurrentPopup::Files(popup) => popup.handle_actions(action).is_quit(),
                CurrentPopup::Details(popup) => popup.handle_actions(action).is_quit(),
            };

            if should_close {
                self.close_popup();
                CTX.send_update_action(UpdateAction::ChangeTorrentWindow(TorrentWindow::General));
            }
        }
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        if let Some(CurrentPopup::Files(popup)) = &mut self.current_popup {
            popup.handle_update_action(action);
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(current_popup) = &mut self.current_popup {
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    popup.render(f, rect);
                }
                CurrentPopup::Files(popup) => {
                    popup.render(f, rect);
                }
                CurrentPopup::Details(popup) => popup.render(f, rect),
            }
        }
    }
}
