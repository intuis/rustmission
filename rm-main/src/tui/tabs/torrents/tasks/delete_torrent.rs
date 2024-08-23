use crossterm::event::KeyCode;
use ratatui::prelude::*;
use transmission_rpc::types::Id;

use crate::transmission::TorrentAction;
use crate::tui::app;
use crate::tui::components::{Component, ComponentAction, InputManager};
use crate::tui::tabs::torrents::rustmission_torrent::RustmissionTorrent;
use rm_shared::action::{Action, UpdateAction};
use rm_shared::status_task::StatusTask;

pub struct Delete {
    torrents_to_delete: Vec<RustmissionTorrent>,
    ctx: app::Ctx,
    input_mgr: InputManager,
    delete_with_files: bool,
}

impl Delete {
    pub fn new(ctx: app::Ctx, to_delete: Vec<RustmissionTorrent>, delete_with_files: bool) -> Self {
        let prompt = if delete_with_files {
            "Really delete selected WITH files? (y/n) ".to_string()
        } else {
            "Really delete selected without files? (y/n) ".to_string()
        };

        Self {
            torrents_to_delete: to_delete,
            input_mgr: InputManager::new(prompt),
            ctx,
            delete_with_files,
        }
    }

    fn delete(&self) {
        let torrents_to_delete: Vec<Id> = self
            .torrents_to_delete
            .iter()
            .map(|x| x.id.clone())
            .collect();
        if self.delete_with_files {
            self.ctx
                .send_torrent_action(TorrentAction::DelWithFiles(torrents_to_delete))
        } else {
            self.ctx
                .send_torrent_action(TorrentAction::DelWithoutFiles(torrents_to_delete))
        }

        let task = StatusTask::new_del(self.torrents_to_delete[0].torrent_name.clone());
        self.ctx
            .send_update_action(UpdateAction::StatusTaskSet(task));
    }
}

impl Component for Delete {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Esc {
                    return ComponentAction::Quit;
                } else if input.code == KeyCode::Enter {
                    let text = self.input_mgr.text().to_lowercase();
                    if text == "y" || text == "yes" {
                        self.delete();
                        return ComponentAction::Quit;
                    } else if text == "n" || text == "no" {
                        return ComponentAction::Quit;
                    }
                }

                if self.input_mgr.handle_key(input).is_some() {
                    self.ctx.send_action(Action::Render);
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
