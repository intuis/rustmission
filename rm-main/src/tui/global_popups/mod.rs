mod error;
mod help;

use ratatui::prelude::*;

pub use error::ErrorPopup;
pub use help::HelpPopup;

use rm_shared::action::Action;

use super::{
    components::{Component, ComponentAction},
    ctx::CTX,
};

pub(super) struct GlobalPopupManager {
    pub error_popup: Option<ErrorPopup>,
    pub help_popup: Option<HelpPopup>,
}

impl GlobalPopupManager {
    pub fn new() -> Self {
        Self {
            error_popup: None,
            help_popup: None,
        }
    }

    pub const fn needs_action(&self) -> bool {
        self.error_popup.is_some() || self.help_popup.is_some()
    }

    pub fn toggle_help(&mut self) {
        if self.help_popup.is_some() {
            self.help_popup = None;
        } else {
            self.help_popup = Some(HelpPopup::new());
        }
    }

    fn handle_popups(&mut self, action: Action) {
        if let Some(popup) = &mut self.error_popup {
            if popup.handle_actions(action).is_quit() {
                self.error_popup = None;
                CTX.send_action(Action::Render);
            }
        } else if let Some(popup) = &mut self.help_popup {
            if popup.handle_actions(action).is_quit() {
                self.help_popup = None;
                CTX.send_action(Action::Render);
            }
        }
    }
}

impl Component for GlobalPopupManager {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;
        if action == A::ShowHelp {
            self.toggle_help();
            CTX.send_action(Action::Render);
            return ComponentAction::Nothing;
        }

        self.handle_popups(action);
        ComponentAction::Nothing
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(popup) = &mut self.error_popup {
            popup.render(f, rect)
        } else if let Some(popup) = &mut self.help_popup {
            popup.render(f, rect);
        }
    }
}
