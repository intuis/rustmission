use crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::transmission::TorrentAction;
use crate::tui::components::{Component, ComponentAction, InputManager};
use crate::tui::ctx::CTX;
use rm_shared::action::{Action, UpdateAction};
use rm_shared::status_task::StatusTask;

use super::TorrentSelection;

pub struct Reannounce {
    torrents_to_reannounce: TorrentSelection,
    input_mgr: InputManager,
}

impl Reannounce {
    pub fn new(to_reannounce: TorrentSelection) -> Self {
        let prompt = String::from("Reannounce selected torrent(s) to trackers? (y/N) ");

        Self {
            torrents_to_reannounce: to_reannounce,
            input_mgr: InputManager::new(prompt),
        }
    }

    fn reannounce(&self) {
        CTX.send_torrent_action(TorrentAction::Reannounce(self.torrents_to_reannounce.ids()));

        let task = StatusTask::new_reannounce(self.torrents_to_reannounce.describe());

        CTX.send_update_action(UpdateAction::StatusTaskSet(task));
        CTX.send_update_action(UpdateAction::ReannounceDispatched);
    }
}

impl Component for Reannounce {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Esc {
                    return ComponentAction::Quit;
                } else if input.code == KeyCode::Enter {
                    let text = self.input_mgr.text().trim().to_lowercase();
                    if text == "y" || text == "yes" {
                        self.reannounce();
                    }
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
