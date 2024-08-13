use crate::tui::{
    app,
    components::{Component, ComponentAction},
};

use self::{files::FilesPopup, stats::StatisticsPopup};
use rm_shared::action::{Action, UpdateAction};

use ratatui::prelude::*;

pub mod files;
pub mod stats;

pub struct PopupManager {
    ctx: app::Ctx,
    current_popup: Option<CurrentPopup>,
}

pub enum CurrentPopup {
    Stats(StatisticsPopup),
    Files(FilesPopup),
}

impl PopupManager {
    pub const fn new(ctx: app::Ctx) -> Self {
        Self {
            ctx,
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
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    if popup.handle_actions(action).is_quit() {
                        self.close_popup();
                        self.ctx.send_action(Action::Render);
                    }
                }
                CurrentPopup::Files(popup) => {
                    if popup.handle_actions(action).is_quit() {
                        self.close_popup();
                        self.ctx.send_action(Action::Render);
                    }
                }
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
            }
        }
    }
}
