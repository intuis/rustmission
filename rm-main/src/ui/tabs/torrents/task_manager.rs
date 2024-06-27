use std::sync::{Arc, Mutex};

use ratatui::prelude::*;
use throbber_widgets_tui::ThrobberState;

use crate::{action::Action, app, ui::components::Component};

use super::{
    tasks::{
        add_magnet::AddMagnetBar,
        default::DefaultBar,
        delete_torrent::{self, DeleteBar},
        filter::FilterBar,
        status::{CurrentTaskState, StatusBar, StatusTask},
    },
    TableManager,
};

pub struct TaskManager {
    ctx: app::Ctx,
    current_task: CurrentTask,
    table_manager: Arc<Mutex<TableManager>>,
}

impl TaskManager {
    pub fn new(table_manager: Arc<Mutex<TableManager>>, ctx: app::Ctx) -> Self {
        Self {
            current_task: CurrentTask::Default(DefaultBar::new(ctx.clone())),
            ctx,
            table_manager,
        }
    }
}

enum CurrentTask {
    AddMagnetBar(AddMagnetBar),
    DeleteBar(DeleteBar),
    FilterBar(FilterBar),
    Default(DefaultBar),
    Status(StatusBar),
}

impl Component for TaskManager {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => match magnet_bar.handle_actions(action) {
                Some(A::Confirm) => self.pending_task(StatusTask::Add),
                Some(A::Quit) => self.cancel_task(),
                Some(A::Render) => Some(A::Render),
                _ => None,
            },

            CurrentTask::DeleteBar(delete_bar) => match delete_bar.handle_actions(action) {
                Some(A::Confirm) => self.pending_task(StatusTask::Delete),
                Some(A::Quit) => self.cancel_task(),
                Some(A::Render) => Some(A::Render),
                _ => None,
            },

            CurrentTask::FilterBar(filter_bar) => match filter_bar.handle_actions(action) {
                Some(A::Quit) => self.cancel_task(),
                Some(A::Render) => Some(A::Render),
                _ => None,
            },

            CurrentTask::Status(status_bar) => match status_bar.handle_actions(action) {
                Some(A::Render) => Some(A::Render),
                Some(action) => self.handle_events_to_manager(&action),
                _ => None,
            },

            CurrentTask::Default(_) => self.handle_events_to_manager(&action),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => magnet_bar.render(f, rect),
            CurrentTask::DeleteBar(delete_bar) => delete_bar.render(f, rect),
            CurrentTask::FilterBar(filter_bar) => filter_bar.render(f, rect),
            CurrentTask::Default(default_bar) => default_bar.render(f, rect),
            CurrentTask::Status(status_bar) => status_bar.render(f, rect),
        }
    }

    fn tick(&mut self) -> Option<Action> {
        match &mut self.current_task {
            CurrentTask::Status(status_bar) => status_bar.tick(),
            _ => None,
        }
    }
}

impl TaskManager {
    #[must_use]
    fn handle_events_to_manager(&mut self, action: &Action) -> Option<Action> {
        match action {
            Action::AddMagnet => {
                self.current_task = CurrentTask::AddMagnetBar(AddMagnetBar::new(self.ctx.clone()));
                Some(Action::SwitchToInputMode)
            }
            Action::DeleteWithFiles => self.delete_torrent(delete_torrent::Mode::WithFiles),
            Action::DeleteWithoutFiles => self.delete_torrent(delete_torrent::Mode::WithoutFiles),
            Action::Search => {
                self.current_task = CurrentTask::FilterBar(FilterBar::new(
                    self.ctx.clone(),
                    self.table_manager.clone(),
                ));
                Some(Action::SwitchToInputMode)
            }
            _ => None,
        }
    }

    fn delete_torrent(&mut self, mode: delete_torrent::Mode) -> Option<Action> {
        if let Some(torrent) = self.table_manager.lock().unwrap().current_torrent() {
            self.current_task = CurrentTask::DeleteBar(DeleteBar::new(
                self.ctx.clone(),
                vec![torrent.id.clone()],
                mode,
            ));
            Some(Action::SwitchToInputMode)
        } else {
            None
        }
    }

    fn pending_task(&mut self, task: StatusTask) -> Option<Action> {
        if !matches!(self.current_task, CurrentTask::Status(_)) {
            let state = Arc::new(Mutex::new(ThrobberState::default()));
            self.current_task =
                CurrentTask::Status(StatusBar::new(task, CurrentTaskState::Loading(state)));
            Some(Action::SwitchToNormalMode)
        } else {
            None
        }
    }

    fn cancel_task(&mut self) -> Option<Action> {
        if !matches!(self.current_task, CurrentTask::Default(_)) {
            self.current_task = CurrentTask::Default(DefaultBar::new(self.ctx.clone()));
            Some(Action::SwitchToNormalMode)
        } else {
            None
        }
    }
}
