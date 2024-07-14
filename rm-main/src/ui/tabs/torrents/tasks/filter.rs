use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::{
    app,
    ui::{
        components::{Component, ComponentAction},
        tabs::torrents::{input_manager::InputManager, TableManager},
        to_input_request,
    },
};
use rm_shared::action::Action;

pub struct FilterBar {
    ctx: app::Ctx,
    input: InputManager,
    table_manager: Arc<Mutex<TableManager>>,
}

impl FilterBar {
    pub fn new(ctx: app::Ctx, table_manager: Arc<Mutex<TableManager>>) -> Self {
        let current_filter = table_manager.lock().unwrap().filter.lock().unwrap().clone();
        let input = InputManager::new_with_value(
            ctx.clone(),
            "Search: ".to_string(),
            current_filter.unwrap_or_default(),
        );
        Self {
            ctx,
            input,
            table_manager,
        }
    }
}

impl Component for FilterBar {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => {
                if matches!(input.code, KeyCode::Enter | KeyCode::Esc) {
                    if self.input.text().is_empty() {
                        *self.table_manager.lock().unwrap().filter.lock().unwrap() = None;
                    }
                    return ComponentAction::Quit;
                }

                if let Some(req) = to_input_request(input) {
                    self.input.handle(req);
                    let table_manager_lock = self.table_manager.lock().unwrap();
                    table_manager_lock
                        .filter
                        .lock()
                        .unwrap()
                        .replace(self.input.text());
                    table_manager_lock.table.state.borrow_mut().select(Some(0));

                    self.ctx.send_action(Action::Render);
                    return ComponentAction::Nothing;
                }

                ComponentAction::Nothing
            }
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input.render(f, rect);
    }
}
