use crossterm::event::KeyCode;
use ratatui::prelude::*;
use transmission_rpc::types::Id;

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{components::Component, tabs::torrents::task_manager::InputManager, to_input_request},
};

pub struct DeleteBar {
    to_delete: Vec<Id>,
    ctx: app::Ctx,
    input_mgr: InputManager,
}

impl DeleteBar {
    pub fn new(ctx: app::Ctx, to_delete: Vec<Id>) -> Self {
        Self {
            to_delete,
            ctx,
            input_mgr: InputManager::new("Are you sure want to delete selected?: ".to_string()),
        }
    }
}

impl Component for DeleteBar {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Enter {
                    let text = self.input_mgr.text().to_lowercase();
                    if text == "y" || text == "yes" {
                        self.ctx
                            .send_torrent_action(TorrentAction::TorrentDelete(Box::new(
                                self.to_delete.clone(),
                            )));
                        return Some(Action::Quit);
                    } else if text == "n" || text == "no" {
                        return Some(Action::Quit);
                    } else {
                    }
                }

                if let Some(req) = to_input_request(input.code) {
                    self.input_mgr.handle(req);
                    return Some(Action::Render);
                }

                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input_mgr.render(f, rect)
    }
}
