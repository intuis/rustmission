use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};

use crate::{
    transmission::TorrentAction,
    tui::{
        app::{self, CTX},
        components::{Component, ComponentAction, InputManager},
    },
};

use super::TorrentSelection;

pub struct Move {
    selection: TorrentSelection,
    input_mgr: InputManager,
}

impl Move {
    pub fn new(selection: TorrentSelection, existing_location: String) -> Self {
        let prompt = "New directory: ".to_string();

        Self {
            selection,
            input_mgr: InputManager::new_with_value(prompt, existing_location),
        }
    }

    fn handle_input(&mut self, input: KeyEvent) -> ComponentAction {
        if input.code == KeyCode::Enter {
            let new_location = self.input_mgr.text();

            let torrent_action = TorrentAction::Move(self.selection.ids(), new_location.clone());
            CTX.send_torrent_action(torrent_action);

            let task = StatusTask::new_move(new_location);
            CTX.send_update_action(UpdateAction::StatusTaskSet(task));

            ComponentAction::Quit
        } else if input.code == KeyCode::Esc {
            ComponentAction::Quit
        } else if self.input_mgr.handle_key(input).is_some() {
            CTX.send_action(Action::Render);
            ComponentAction::Nothing
        } else {
            ComponentAction::Nothing
        }
    }
}

impl Component for Move {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => self.handle_input(input),
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input_mgr.render(f, rect)
    }
}
