use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::{
    action::Action,
    ui::{
        components::Component,
        tabs::torrents::{input_manager::InputManager, TableManager},
        to_input_request,
    },
};

pub struct FilterBar {
    input: InputManager,
    table_manager: Arc<Mutex<TableManager>>,
}

impl FilterBar {
    pub fn new(table_manager: Arc<Mutex<TableManager>>) -> Self {
        let current_filter = table_manager.lock().unwrap().filter.lock().unwrap().clone();
        let input = InputManager::new_with_value(
            "Search: ".to_string(),
            current_filter.unwrap_or("".to_string()),
        );
        Self {
            input,
            table_manager,
        }
    }
}

impl Component for FilterBar {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Enter {
                    return Some(Action::Quit);
                }
                if input.code == KeyCode::Esc {
                    return Some(Action::Quit);
                }

                if let Some(req) = to_input_request(input.code) {
                    self.input.handle(req);
                    let table_manager_lock = self.table_manager.lock().unwrap();

                    table_manager_lock
                        .filter
                        .lock()
                        .unwrap()
                        .replace(self.input.text());
                    table_manager_lock
                        .table
                        .borrow()
                        .state
                        .borrow_mut()
                        .select(Some(0));
                    return Some(Action::Render);
                }

                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input.render(f, rect);
    }
}
