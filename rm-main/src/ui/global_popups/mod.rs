mod error;
mod help;

use ratatui::prelude::*;

pub use error::ErrorPopup;
pub use help::HelpPopup;

use crate::action::Action;

use super::components::Component;

#[derive(Default)]
pub(super) struct GlobalPopupManager {
    pub error_popup: Option<ErrorPopup>,
    pub help_popup: Option<HelpPopup>,
}

impl GlobalPopupManager {
    pub fn needs_action(&self) -> bool {
        self.error_popup.is_some() || self.help_popup.is_some()
    }
}

impl Component for GlobalPopupManager {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Some(popup) = &mut self.error_popup {
            if let Some(Action::Quit) = popup.handle_actions(action) {
                self.error_popup = None;
                return Some(Action::Render);
            }
            None
        } else if let Some(popup) = &mut self.help_popup {
            if let Some(Action::Quit) = popup.handle_actions(action) {
                self.help_popup = None;
                return Some(Action::Render);
            }
            None
        } else {
            None
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(popup) = &mut self.error_popup {
            popup.render(f, rect)
        } else if let Some(popup) = &mut self.help_popup {
            popup.render(f, rect);
        }
    }
}
