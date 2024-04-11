use crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{
        components::{torrent_tab::task_manager::InputManager, Component},
        to_input_request,
    },
};

pub struct AddMagnetBar {
    // TODO: change the name to input_mgr
    input: InputManager,
    ctx: app::Ctx,
}

impl AddMagnetBar {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            input: InputManager::new("Add (Magnet URL/ Torrent path): ".to_string()),
            ctx,
        }
    }
}

impl Component for AddMagnetBar {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Enter {
                    self.ctx
                        .send_torrent_action(TorrentAction::TorrentAdd(Box::new(
                            self.input.text(),
                        )));
                    return Some(Action::Quit);
                }
                if input.code == KeyCode::Esc {
                    return Some(Action::Quit);
                }

                if let Some(req) = to_input_request(input.code) {
                    self.input.handle(req);
                    return Some(Action::Render);
                }
                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input.render(f, rect)
    }
}
