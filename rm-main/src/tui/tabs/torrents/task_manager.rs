use ratatui::prelude::*;
use throbber_widgets_tui::ThrobberState;
use tokio::time::Instant;

use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use transmission_rpc::types::Id;

use crate::tui::{
    app,
    components::{Component, ComponentAction},
};

use super::{
    rustmission_torrent::RustmissionTorrent,
    tasks::{self, CurrentTaskState},
};

pub struct TaskManager {
    ctx: app::Ctx,
    current_task: CurrentTask,
}

impl TaskManager {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            current_task: CurrentTask::Default(tasks::Default::new()),
            ctx,
        }
    }
}

pub enum CurrentTask {
    AddMagnet(tasks::AddMagnet),
    Delete(tasks::Delete),
    Filter(tasks::Filter),
    Move(tasks::Move),
    ChangeCategory(tasks::ChangeCategory),
    Default(tasks::Default),
    Status(tasks::Status),
    Sort(tasks::Sort),
    Selection(tasks::Selection),
}

impl CurrentTask {
    fn tick(&mut self) {
        if let Self::Status(status_bar) = self {
            status_bar.tick()
        }
    }
}

impl Component for TaskManager {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match &mut self.current_task {
            CurrentTask::AddMagnet(magnet_bar) => {
                if magnet_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }
            CurrentTask::Delete(delete_bar) => {
                if delete_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }
            CurrentTask::Move(move_bar) => {
                if move_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }
            CurrentTask::Filter(filter_bar) => {
                if filter_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }
            CurrentTask::Status(status_bar) => {
                if status_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }
            CurrentTask::ChangeCategory(category_bar) => {
                if category_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }
            CurrentTask::Default(_) => (),
            CurrentTask::Sort(_) => (),
            CurrentTask::Selection(_) => (),
        };
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::StatusTaskClear => self.cancel_task(),
            UpdateAction::StatusTaskSet(task) => self.pending_task(task),
            UpdateAction::StatusTaskSetSuccess(task) => self.success_task(task),
            UpdateAction::StatusTaskSuccess => {
                if let CurrentTask::Status(status_bar) = &mut self.current_task {
                    status_bar.set_success();
                }
            }
            UpdateAction::StatusTaskFailure => {
                if let CurrentTask::Status(status_bar) = &mut self.current_task {
                    status_bar.set_failure();
                }
            }
            _ => (),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.current_task {
            CurrentTask::AddMagnet(magnet_bar) => magnet_bar.render(f, rect),
            CurrentTask::Delete(delete_bar) => delete_bar.render(f, rect),
            CurrentTask::Move(move_bar) => move_bar.render(f, rect),
            CurrentTask::Filter(filter_bar) => filter_bar.render(f, rect),
            CurrentTask::Default(default_bar) => default_bar.render(f, rect),
            CurrentTask::Status(status_bar) => status_bar.render(f, rect),
            CurrentTask::ChangeCategory(category_bar) => category_bar.render(f, rect),
            CurrentTask::Sort(sort_bar) => sort_bar.render(f, rect),
            CurrentTask::Selection(selection_bar) => selection_bar.render(f, rect),
        }
    }

    fn tick(&mut self) {
        self.current_task.tick()
    }
}

impl TaskManager {
    pub fn add_magnet(&mut self) {
        self.current_task = CurrentTask::AddMagnet(tasks::AddMagnet::new(self.ctx.clone()));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn search(&mut self, current_pattern: &Option<String>) {
        self.current_task =
            CurrentTask::Filter(tasks::Filter::new(self.ctx.clone(), current_pattern));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn delete_torrents(&mut self, torrents: Vec<Id>, name_of_first: String) {
        self.current_task = CurrentTask::Delete(tasks::Delete::new(
            self.ctx.clone(),
            torrents,
            name_of_first,
        ));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn move_torrent(&mut self, torrent: &RustmissionTorrent) {
        self.current_task = CurrentTask::Move(tasks::Move::new(
            self.ctx.clone(),
            vec![torrent.id.clone()],
            torrent.download_dir.to_string(),
        ));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn change_category(&mut self, torrent: &RustmissionTorrent) {
        self.current_task = CurrentTask::ChangeCategory(tasks::ChangeCategory::new(
            self.ctx.clone(),
            vec![torrent.id.clone()],
        ));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn default(&mut self) {
        self.current_task = CurrentTask::Default(tasks::Default::new());
    }

    pub fn select(&mut self, amount: usize) {
        self.current_task = CurrentTask::Selection(tasks::Selection::new(amount));
    }

    pub fn sort(&mut self) {
        self.current_task = CurrentTask::Sort(tasks::Sort::new());
    }

    fn success_task(&mut self, task: StatusTask) {
        self.current_task = CurrentTask::Status(tasks::Status::new(
            self.ctx.clone(),
            task,
            CurrentTaskState::Success(Instant::now()),
        ))
    }

    fn pending_task(&mut self, task: StatusTask) {
        if matches!(self.current_task, CurrentTask::Status(_)) {
            return;
        }

        let state = ThrobberState::default();
        self.current_task = CurrentTask::Status(tasks::Status::new(
            self.ctx.clone(),
            task,
            CurrentTaskState::Loading(state),
        ));
        self.ctx
            .send_update_action(UpdateAction::SwitchToNormalMode);
    }

    fn cancel_task(&mut self) {
        if matches!(self.current_task, CurrentTask::Default(_)) {
            return;
        }

        self.ctx.send_update_action(UpdateAction::CancelTorrentTask);
    }

    pub fn is_finished_status_task(&self) -> bool {
        if let CurrentTask::Status(task) = &self.current_task {
            !matches!(task.task_status, CurrentTaskState::Loading(_))
        } else {
            false
        }
    }
}
