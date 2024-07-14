use std::sync::{Arc, Mutex};

use crate::{
    app,
    ui::components::{Component, ComponentAction},
};

use ratatui::{prelude::*, widgets::Paragraph};
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use throbber_widgets_tui::ThrobberState;
use tokio::time::{self, Instant};

pub struct StatusBar {
    task: StatusTask,
    pub task_status: CurrentTaskState,
    ctx: app::Ctx,
}

impl StatusBar {
    pub const fn new(ctx: app::Ctx, task: StatusTask, task_status: CurrentTaskState) -> Self {
        Self {
            task,
            task_status,
            ctx,
        }
    }
}

fn format_display_name(name: &str) -> String {
    if name.len() < 60 {
        name.to_string()
    } else {
        let truncated = &name[0..59];
        format!("\"{truncated}...\"")
    }
}

impl Component for StatusBar {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &self.task_status {
            CurrentTaskState::Loading(state) => {
                let status_text = match &self.task {
                    StatusTask::Add(name) => {
                        let display_name = format_display_name(name);
                        format!("Adding {display_name}")
                    }
                    StatusTask::Delete(name) => {
                        let display_name = format_display_name(name);
                        format!("Deleting {display_name}")
                    }
                    StatusTask::Move(name) => {
                        let display_name = format_display_name(name);
                        format!("Moving to {display_name}")
                    }
                };
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label(status_text)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(
                    default_throbber.clone(),
                    rect,
                    &mut state.lock().unwrap(),
                );
            }
            task_state => {
                let status_text = match task_state {
                    CurrentTaskState::Failure => match &self.task {
                        StatusTask::Add(name) => {
                            let display_name = format_display_name(name);
                            format!(" Error adding {display_name}")
                        }
                        StatusTask::Delete(name) => {
                            let display_name = format_display_name(name);
                            format!(" Error deleting {display_name}")
                        }
                        StatusTask::Move(name) => {
                            let display_name = format_display_name(name);
                            format!(" Error moving to {display_name}")
                        }
                    },
                    CurrentTaskState::Success(_) => match &self.task {
                        StatusTask::Add(name) => {
                            let display_name = format_display_name(name);
                            format!(" Added {display_name}")
                        }
                        StatusTask::Delete(name) => {
                            let display_name = format_display_name(name);
                            format!(" Deleted {display_name}")
                        }
                        StatusTask::Move(name) => {
                            let display_name = format_display_name(name);
                            format!(" Location moved to {display_name}")
                        }
                    },
                    _ => return,
                };
                let mut line = Line::default();
                match task_state {
                    CurrentTaskState::Failure => {
                        line.push_span(Span::styled("", Style::default().red()));
                    }
                    CurrentTaskState::Success(_) => {
                        line.push_span(Span::styled("", Style::default().green()));
                    }
                    _ => return,
                }
                line.push_span(Span::raw(status_text));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Tick => {
                self.tick();
                ComponentAction::Nothing
            }
            Action::TaskSuccess => {
                self.task_status.set_success();
                self.ctx.send_action(Action::Render);
                ComponentAction::Nothing
            }
            Action::Error(_) => {
                self.task_status.set_failure();
                self.ctx.send_action(Action::Render);
                ComponentAction::Nothing
            }
            _ => ComponentAction::Nothing,
        }
    }

    fn tick(&mut self) {
        match &self.task_status {
            CurrentTaskState::Loading(state) => {
                state.lock().unwrap().calc_next();
                self.ctx.send_action(Action::Render);
            }
            CurrentTaskState::Success(start) => {
                let expiration_duration = time::Duration::from_secs(5);
                if start.elapsed() >= expiration_duration {
                    self.ctx.send_update_action(UpdateAction::TaskClear);
                }
            }
            _ => (),
        }
    }
}

#[derive(Clone)]
pub enum CurrentTaskState {
    Loading(Arc<Mutex<ThrobberState>>),
    Success(Instant),
    Failure,
}

impl CurrentTaskState {
    fn set_failure(&mut self) {
        *self = Self::Failure;
    }

    fn set_success(&mut self) {
        *self = Self::Success(time::Instant::now());
    }
}
