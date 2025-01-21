use ratatui::{prelude::*, style::Style};
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use throbber_widgets_tui::ThrobberState;
use tokio::time::{self, Instant};

use crate::tui::{app::CTX, components::Component};

pub struct Status {
    task: StatusTask,
    pub task_status: CurrentTaskState,
}

#[derive(Clone)]
pub enum CurrentTaskState {
    Loading(ThrobberState),
    Success(Instant),
    Failure(Instant),
}

impl Status {
    pub const fn new(task: StatusTask, task_status: CurrentTaskState) -> Self {
        Self { task, task_status }
    }

    pub fn set_failure(&mut self) {
        self.task_status = CurrentTaskState::Failure(Instant::now());
    }

    pub fn set_success(&mut self) {
        self.task_status = CurrentTaskState::Success(Instant::now());
    }
}

impl Component for Status {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.task_status {
            CurrentTaskState::Loading(ref mut state) => {
                let status_text = self.task.loading_str();
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label(status_text)
                    .style(Style::default().yellow());
                f.render_stateful_widget(default_throbber.clone(), rect, state);
            }
            CurrentTaskState::Failure(_) => {
                let line = Line::from(vec![
                    Span::styled(" ", Style::default().red()),
                    Span::raw(self.task.failure_str()),
                ]);
                f.render_widget(line, rect);
            }
            CurrentTaskState::Success(_) => {
                let line = Line::from(vec![
                    Span::styled(" ", Style::default().green()),
                    Span::raw(self.task.success_str()),
                ]);
                f.render_widget(line, rect);
            }
        }
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::StatusTaskSuccess => {
                self.set_success();
                CTX.send_action(Action::Render);
            }
            UpdateAction::Error(_) => {
                self.set_failure();
                CTX.send_action(Action::Render);
            }
            _ => (),
        }
    }

    fn tick(&mut self) {
        match &mut self.task_status {
            CurrentTaskState::Loading(state) => {
                state.calc_next();
                CTX.send_action(Action::Render);
            }
            CurrentTaskState::Success(start) => {
                let expiration_duration = time::Duration::from_secs(5);
                if start.elapsed() >= expiration_duration {
                    CTX.send_update_action(UpdateAction::StatusTaskClear);
                }
            }
            CurrentTaskState::Failure(start) => {
                let expiration_duration = time::Duration::from_secs(5);
                if start.elapsed() >= expiration_duration {
                    CTX.send_update_action(UpdateAction::StatusTaskClear);
                }
            }
        }
    }
}
