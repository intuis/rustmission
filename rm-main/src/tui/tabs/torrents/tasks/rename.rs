use crossterm::event::KeyCode;
use ratatui::{prelude::*, Frame};
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use transmission_rpc::types::Id;

use crate::{
    transmission::TorrentAction,
    tui::{
        app::CTX,
        components::{Component, ComponentAction, InputManager},
    },
};

pub struct Rename {
    id: Id,
    curr_name: String,
    input_mgr: InputManager,
}

impl Rename {
    pub fn new(to_rename: Id, curr_name: String) -> Self {
        let prompt = String::from("New name: ");

        Self {
            id: to_rename,
            input_mgr: InputManager::new_with_value(prompt, curr_name.clone()),
            curr_name,
        }
    }

    fn rename(&self) {
        let new_name = self.input_mgr.text();

        if self.curr_name == new_name {
            return;
        }

        let task = StatusTask::new_rename(self.curr_name.clone());

        CTX.send_update_action(UpdateAction::StatusTaskSet(task));
        CTX.send_torrent_action(TorrentAction::Rename(
            self.id.clone(),
            self.curr_name.clone(),
            self.input_mgr.text(),
        ))
    }
}

impl Component for Rename {
    fn handle_actions(&mut self, action: Action) -> crate::tui::components::ComponentAction {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Esc {
                    return ComponentAction::Quit;
                } else if input.code == KeyCode::Enter {
                    self.rename();
                    return ComponentAction::Quit;
                }

                if self.input_mgr.handle_key(input).is_some() {
                    CTX.send_action(Action::Render);
                }

                ComponentAction::Nothing
            }

            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input_mgr.render(f, rect)
    }
}
