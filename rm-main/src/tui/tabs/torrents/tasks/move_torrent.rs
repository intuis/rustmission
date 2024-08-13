use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use transmission_rpc::types::Id;

use crate::{
    transmission::TorrentAction,
    tui::{
        app,
        components::{Component, ComponentAction, InputManager},
    },
};

pub struct MoveBar {
    torrents_to_move: Vec<Id>,
    ctx: app::Ctx,
    input_mgr: InputManager,
}

impl MoveBar {
    pub fn new(ctx: app::Ctx, torrents_to_move: Vec<Id>, existing_location: String) -> Self {
        let prompt = "New directory: ".to_string();

        Self {
            torrents_to_move,
            input_mgr: InputManager::new_with_value(prompt, existing_location),
            ctx,
        }
    }

    fn handle_input(&mut self, input: KeyEvent) -> ComponentAction {
        if input.code == KeyCode::Enter {
            let new_location = self.input_mgr.text();
            let torrents_to_move = self.torrents_to_move.clone();

            let torrent_action = TorrentAction::Move(torrents_to_move, new_location.clone());
            self.ctx.send_torrent_action(torrent_action);

            let task = StatusTask::new_move(new_location);
            self.ctx.send_update_action(UpdateAction::TaskSet(task));

            ComponentAction::Quit
        } else if input.code == KeyCode::Esc {
            ComponentAction::Quit
        } else if self.input_mgr.handle_key(input).is_some() {
            self.ctx.send_action(Action::Render);
            ComponentAction::Nothing
        } else {
            ComponentAction::Nothing
        }
    }
}

impl Component for MoveBar {
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
