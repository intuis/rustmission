use crossterm::event::KeyCode;
use ratatui::prelude::*;
use transmission_rpc::types::Id;

use crate::transmission::TorrentAction;
use crate::tui::app;
use crate::tui::components::{Component, ComponentAction, InputManager};
use rm_shared::action::{Action, UpdateAction};
use rm_shared::status_task::StatusTask;

pub struct Delete {
    delete_with_files: bool,
    torrents_to_delete: Vec<Id>,
    name_of_first: String,
    input_mgr: InputManager,
    ctx: app::Ctx,
}

impl Delete {
    pub fn new(ctx: app::Ctx, to_delete: Vec<Id>, name_of_first: String) -> Self {
        let prompt = String::from("Delete selected with files? (Y/n) ");

        Self {
            delete_with_files: false,
            torrents_to_delete: to_delete,
            name_of_first,
            input_mgr: InputManager::new(prompt),
            ctx,
        }
    }

    fn delete(&self) {
        if self.delete_with_files {
            self.ctx
                .send_torrent_action(TorrentAction::DelWithFiles(self.torrents_to_delete.clone()))
        } else {
            self.ctx.send_torrent_action(TorrentAction::DelWithoutFiles(
                self.torrents_to_delete.clone(),
            ))
        }

        let task = if self.torrents_to_delete.len() == 1 {
            StatusTask::new_del(self.name_of_first.clone())
        } else {
            StatusTask::new_del(self.torrents_to_delete.len().to_string())
        };
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
                    if text == "y" || text == "yes" || text.is_empty() {
                        self.delete_with_files = true;
                        self.delete();
                        return ComponentAction::Quit;
                    } else if text == "n" || text == "no" {
                        self.delete();
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
