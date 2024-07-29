use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use transmission_rpc::types::Id;

use crate::{
    app,
    transmission::TorrentAction,
    ui::{
        components::{Component, ComponentAction},
        tabs::torrents::input_manager::InputManager,
        to_input_request,
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
            input_mgr: InputManager::new_with_value(ctx.clone(), prompt, existing_location),
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

            return ComponentAction::Quit;
        }

        if input.code == KeyCode::Esc {
            return ComponentAction::Quit;
        }

        if let Some(req) = to_input_request(input) {
            self.input_mgr.handle(req);
            self.ctx.send_action(Action::Render);
        }

        ComponentAction::Nothing
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
