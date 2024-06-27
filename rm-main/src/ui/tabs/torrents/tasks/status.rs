use std::sync::{Arc, Mutex};

use crate::{action::Action, ui::components::Component};

use ratatui::{prelude::*, widgets::Paragraph};
use throbber_widgets_tui::ThrobberState;

pub struct StatusBar {
    task: StatusTask,
    task_status: CurrentTaskState,
}

impl StatusBar {
    pub const fn new(task: StatusTask, task_status: CurrentTaskState) -> Self {
        Self { task, task_status }
    }
}

impl Component for StatusBar {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &self.task_status {
            CurrentTaskState::Loading(state) => {
                let title = match self.task {
                    StatusTask::Add => "Adding torrent...",
                    StatusTask::Delete => "Deleting torrent...",
                };
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label(title)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(
                    default_throbber.clone(),
                    rect,
                    &mut state.lock().unwrap(),
                );
            }
            CurrentTaskState::Failure() => {
                let title = match self.task {
                    StatusTask::Add => " Error adding torrent",
                    StatusTask::Delete => " Error deleting torrent",
                };
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().red()));
                line.push_span(Span::raw(title));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
            CurrentTaskState::Success() => {
                let title = match self.task {
                    StatusTask::Add => " Added torrent",
                    StatusTask::Delete => " Deleted torrent",
                };
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().green()));
                line.push_span(Span::raw(title));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn handle_actions(&mut self, _action: Action) -> Option<Action> {
        match _action {
            Action::Tick => self.tick(),
            Action::Success => {
                self.task_status.success();
                Some(Action::Render)
            }
            Action::Error(_) => {
                self.task_status.failure();
                Some(Action::Render)
            }
            _ => Some(_action),
        }
    }

    fn tick(&mut self) -> Option<Action> {
        match &self.task_status {
            CurrentTaskState::Loading(state) => {
                state.lock().unwrap().calc_next();
                Some(Action::Render)
            }
            _ => None,
        }
    }
}

pub enum StatusTask {
    Add,
    Delete,
}

#[derive(Clone)]
pub enum CurrentTaskState {
    Loading(Arc<Mutex<ThrobberState>>),
    Success(),
    Failure(),
}

impl CurrentTaskState {
    fn failure(&mut self) {
        *self = Self::Failure();
    }

    fn success(&mut self) {
        *self = Self::Success();
    }
}
