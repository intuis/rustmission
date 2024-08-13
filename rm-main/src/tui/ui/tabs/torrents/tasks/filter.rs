use crossterm::event::KeyCode;
use ratatui::prelude::*;

use rm_shared::action::{Action, UpdateAction};

use crate::tui::{
    app,
    ui::{
        components::{Component, ComponentAction},
        tabs::torrents::{input_manager::InputManager, table_manager::Filter},
    },
};

pub struct FilterBar {
    ctx: app::Ctx,
    input: InputManager,
}

impl FilterBar {
    pub fn new(ctx: app::Ctx, current_filter: &Option<Filter>) -> Self {
        let filter = {
            if let Some(current_filter) = current_filter {
                current_filter.pattern.clone()
            } else {
                "".to_string()
            }
        };

        let input = InputManager::new_with_value("Search: ".to_string(), filter);
        Self { ctx, input }
    }
}

impl Component for FilterBar {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => {
                if matches!(input.code, KeyCode::Enter | KeyCode::Esc) {
                    if self.input.text().is_empty() {
                        self.ctx.send_update_action(UpdateAction::SearchFilterClear);
                    }
                    ComponentAction::Quit
                } else if self.input.handle_key(input).is_some() {
                    self.ctx
                        .send_update_action(UpdateAction::SearchFilterApply(self.input.text()));
                    ComponentAction::Nothing
                } else {
                    ComponentAction::Nothing
                }
            }
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input.render(f, rect);
    }
}
