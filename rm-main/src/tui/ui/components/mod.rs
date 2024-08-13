pub mod table;
pub mod tabs;

use ratatui::prelude::*;
use ratatui::Frame;

use rm_shared::action::Action;
use rm_shared::action::UpdateAction;
pub use tabs::TabComponent;

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

    fn handle_update_action(&mut self, _action: UpdateAction) {}

    fn render(&mut self, _f: &mut Frame, _rect: Rect) {}

    fn tick(&mut self) {}
}
