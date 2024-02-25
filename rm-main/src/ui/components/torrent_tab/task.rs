use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};
use tui_input::Input;

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{components::Component, to_input_request},
};

pub struct Task {
    ctx: app::Ctx,
    current_task: CurrentTask,
}

impl Task {
    pub const fn new(ctx: app::Ctx) -> Self {
        Self {
            ctx,
            current_task: CurrentTask::None,
        }
    }

    #[must_use]
    fn handle_events_to_self(&mut self, action: &Action) -> Option<Action> {
        match action {
            Action::AddMagnet => {
                self.current_task = CurrentTask::AddMagnetBar(AddMagnetBar::new(self.ctx.clone()));
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
            CurrentTask::None => None,
        }
    }
}

enum CurrentTask {
    AddMagnetBar(AddMagnetBar),
    None,
}

struct AddMagnetBar {
    input: Input,
    ctx: app::Ctx,
}

impl AddMagnetBar {
    fn new(ctx: app::Ctx) -> Self {
        Self {
            input: Input::default(),
            ctx,
        }
    }
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

            CurrentTask::None => self.handle_events_to_self(&action),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => magnet_bar.render(f, rect),
            CurrentTask::None => (),
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
                            self.input.to_string(),
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
        f.render_widget(Clear, rect);

        let input = self.input.to_string();

        let paragraph_text = format!("Add (Magnet URL / Torrent path): {input}");
        let prefix_len = paragraph_text.len() - input.len();

        let paragraph = Paragraph::new(paragraph_text);
        f.render_widget(paragraph, rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(rect.x + u16::try_from(cursor_offset).unwrap(), rect.y);
    }
}
