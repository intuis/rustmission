use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};
use transmission_rpc::types::{Id, Torrent};
use tui_input::{Input, InputRequest};

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{
        components::{table::GenericTable, Component},
        to_input_request,
    },
};

pub struct Task {
    ctx: app::Ctx,
    current_task: CurrentTask,
    table: Arc<Mutex<GenericTable<Torrent>>>,
}

impl Task {
    pub const fn new(table: Arc<Mutex<GenericTable<Torrent>>>, ctx: app::Ctx) -> Self {
        Self {
            ctx,
            current_task: CurrentTask::None,
            table,
        }
    }

    #[must_use]
    fn handle_events_to_self(&mut self, action: &Action) -> Option<Action> {
        match action {
            Action::AddMagnet => {
                self.current_task = CurrentTask::AddMagnetBar(AddMagnetBar::new(self.ctx.clone()));
                Some(Action::SwitchToInputMode)
            }
            Action::Delete => {
                self.current_task = CurrentTask::DeleteBar(DeleteBar::new(
                    self.ctx.clone(),
                    vec![self
                        .table
                        .lock()
                        .unwrap()
                        .current_item()
                        .unwrap()
                        .id()
                        .unwrap()],
                ));
                Some(Action::SwitchToInputMode)
            }
            _ => None,
        }
    }

    fn finish_task(&mut self) -> Option<Action> {
        match self.current_task {
            CurrentTask::AddMagnetBar(_) => {
                self.current_task = CurrentTask::None;
                Some(Action::SwitchToNormalMode)
            }
            CurrentTask::DeleteBar(_) => {
                self.current_task = CurrentTask::None;
                Some(Action::SwitchToNormalMode)
            }
            CurrentTask::None => None,
        }
    }
}

enum CurrentTask {
    AddMagnetBar(AddMagnetBar),
    DeleteBar(DeleteBar),
    None,
}

impl Component for Task {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => match magnet_bar.handle_actions(action) {
                Some(Action::Quit) => self.finish_task(),

                Some(Action::Render) => Some(Action::Render),

                _ => None,
            },

            CurrentTask::DeleteBar(delete_bar) => match delete_bar.handle_actions(action) {
                Some(Action::Quit) => self.finish_task(),

                Some(Action::Render) => Some(Action::Render),

                _ => None,
            },

            CurrentTask::None => self.handle_events_to_self(&action),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => magnet_bar.render(f, rect),
            CurrentTask::DeleteBar(delete_bar) => delete_bar.render(f, rect),
            CurrentTask::None => (),
        }
    }
}

struct AddMagnetBar {
    // TODO: change the name to input_mgr
    input: InputManager,
    ctx: app::Ctx,
}

impl AddMagnetBar {
    fn new(ctx: app::Ctx) -> Self {
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

struct DeleteBar {
    to_delete: Vec<Id>,
    ctx: app::Ctx,
    input_mgr: InputManager,
}

impl DeleteBar {
    fn new(ctx: app::Ctx, to_delete: Vec<Id>) -> Self {
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

struct InputManager {
    input: Input,
    prompt: String,
}

impl InputManager {
    fn new(prompt: String) -> Self {
        Self {
            prompt,
            input: Input::default(),
        }
    }

    fn text(&self) -> String {
        self.input.to_string()
    }

    fn handle(&mut self, req: InputRequest) {
        self.input.handle(req);
    }
}

impl Component for InputManager {
    fn handle_actions(&mut self, _action: Action) -> Option<Action> {
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(Clear, rect);

        let paragraph_text = format!("{}{}", self.prompt, self.text());

        let input = self.input.to_string();
        let prefix_len = paragraph_text.len() - input.len();

        let paragraph = Paragraph::new(paragraph_text);
        f.render_widget(paragraph, rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(rect.x + u16::try_from(cursor_offset).unwrap(), rect.y);
    }
}
