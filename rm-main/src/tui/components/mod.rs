mod input_manager;
mod misc;
mod table;

pub use input_manager::InputManager;
pub use misc::{
    keybinding_style, popup_block, popup_block_with_close_highlight, popup_close_button,
    popup_close_button_highlight, popup_rects,
};
pub use table::GenericTable;

use ratatui::prelude::*;

use rm_shared::action::Action;
use rm_shared::action::UpdateAction;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ComponentAction {
    Nothing,
    Quit,
}

impl ComponentAction {
    pub fn is_quit(self) -> bool {
        self == Self::Quit
    }
}

pub trait Component {
    fn handle_actions(&mut self, _action: Action) -> ComponentAction {
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        let _action = action;
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let _f = f;
        let _rect = rect;
    }

    fn tick(&mut self) {}
}
