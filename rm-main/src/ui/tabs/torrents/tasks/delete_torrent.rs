use crossterm::event::KeyCode;
use ratatui::prelude::*;
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
use rm_shared::action::{Action, UpdateAction};
use rm_shared::status_task::StatusTask;

#[derive(Clone)]
pub struct TorrentInfo {
    pub id: Id,
    pub name: String,
}

pub struct DeleteBar {
    torrents_to_delete: Vec<TorrentInfo>,
    ctx: app::Ctx,
    input_mgr: InputManager,
    mode: Mode,
}

pub enum Mode {
    WithFiles,
    WithoutFiles,
}

impl DeleteBar {
    pub fn new(ctx: app::Ctx, to_delete: Vec<TorrentInfo>, mode: Mode) -> Self {
        let prompt = {
            match mode {
                Mode::WithFiles => "Really delete selected WITH files? (y/n) ".to_string(),
                Mode::WithoutFiles => "Really delete selected without files? (y/n) ".to_string(),
            }
        };

        Self {
            torrents_to_delete: to_delete,
            input_mgr: InputManager::new(ctx.clone(), prompt),
            ctx,
            mode,
        }
    }
}

impl Component for DeleteBar {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Esc {
                    return ComponentAction::Quit;
                }

                if input.code == KeyCode::Enter {
                    let text = self.input_mgr.text().to_lowercase();
                    if text == "y" || text == "yes" {
                        let torrents_to_delete: Vec<Id> = self
                            .torrents_to_delete
                            .iter()
                            .map(|x| x.id.clone())
                            .collect();
                        match self.mode {
                            Mode::WithFiles => self.ctx.send_torrent_action(
                                TorrentAction::DeleteWithFiles(torrents_to_delete),
                            ),
                            Mode::WithoutFiles => {
                                self.ctx
                                    .send_torrent_action(TorrentAction::DeleteWithoutFiles(
                                        torrents_to_delete,
                                    ))
                            }
                        }
                        self.ctx
                            .send_update_action(UpdateAction::TaskSet(StatusTask::Delete(
                                self.torrents_to_delete[0].name.clone(),
                            )));
                    } else if text == "n" || text == "no" {
                        return ComponentAction::Quit;
                    }
                }

                if let Some(req) = to_input_request(input) {
                    self.input_mgr.handle(req);
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
