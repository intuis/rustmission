use std::sync::{Arc, Mutex};

use ratatui::prelude::*;
use throbber_widgets_tui::ThrobberState;

use crate::{
    app,
    ui::components::{Component, ComponentAction},
};
use rm_shared::{
    action::{Action, UpdateAction},
    rustmission_torrent::RustmissionTorrent,
    status_task::StatusTask,
};

use super::tasks::{
    add_magnet::AddMagnetBar,
    default::DefaultBar,
    delete_torrent::{self, DeleteBar, TorrentInfo},
    filter::FilterBar,
    move_torrent::MoveBar,
    status::{CurrentTaskState, StatusBar},
};

pub struct TaskManager {
    ctx: app::Ctx,
    current_task: CurrentTask,
}

impl TaskManager {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            current_task: CurrentTask::Default(DefaultBar::new(ctx.clone())),
            ctx,
        }
    }
}

pub enum CurrentTask {
    AddMagnetBar(AddMagnetBar),
    DeleteBar(DeleteBar),
    FilterBar(FilterBar),
    MoveBar(MoveBar),
    Default(DefaultBar),
    Status(StatusBar),
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
            CurrentTask::AddMagnetBar(magnet_bar) => {
                if magnet_bar.handle_actions(action).is_quit() {
                    // Some(A::TaskPending(task)) => self.pending_task(task),
                    self.cancel_task()
                }
            }

            CurrentTask::DeleteBar(delete_bar) => {
                if delete_bar.handle_actions(action).is_quit() {
                    // Some(A::TaskPending(task)) => {
                    //     let selected = self
                    //         .table_manager
                    //         .lock()
                    //         .unwrap()
                    //         .table
                    //         .state
                    //         .borrow()
                    //         .selected();

                    //     // select closest existing torrent
                    //     if let Some(idx) = selected {
                    //         if idx > 0 {
                    //             self.table_manager.lock().unwrap().table.previous();
                    //         }
                    //     }
                    //     self.pending_task(task)
                    // }
                    self.cancel_task()
                }
            }
            CurrentTask::MoveBar(move_bar) => {
                if move_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                    // Some(A::TaskPending(task)) => self.pending_task(task),
                }
            }
            CurrentTask::FilterBar(filter_bar) => {
                if filter_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }

            CurrentTask::Status(status_bar) => {
                if status_bar.handle_actions(action).is_quit() {
                    self.cancel_task()
                }
            }

            _ => (),
        };
        ComponentAction::Nothing
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => magnet_bar.render(f, rect),
            CurrentTask::DeleteBar(delete_bar) => delete_bar.render(f, rect),
            CurrentTask::MoveBar(move_bar) => move_bar.render(f, rect),
            CurrentTask::FilterBar(filter_bar) => filter_bar.render(f, rect),
            CurrentTask::Default(default_bar) => default_bar.render(f, rect),
            CurrentTask::Status(status_bar) => status_bar.render(f, rect),
        }
    }

    fn tick(&mut self) {
        self.current_task.tick()
    }
}

impl TaskManager {
    pub fn add_magnet(&mut self) {
        self.current_task = CurrentTask::AddMagnetBar(AddMagnetBar::new(self.ctx.clone()));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn search(&mut self, filter: Option<String>) {
        self.current_task = CurrentTask::FilterBar(FilterBar::new(self.ctx.clone(), filter));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn delete_torrent(&mut self, torrent: &RustmissionTorrent, mode: delete_torrent::Mode) {
        self.current_task = CurrentTask::DeleteBar(DeleteBar::new(
            self.ctx.clone(),
            vec![TorrentInfo {
                id: torrent.id.clone(),
                name: torrent.torrent_name.clone(),
            }],
            mode,
        ));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn move_torrent(&mut self, torrent: &RustmissionTorrent) {
        self.current_task = CurrentTask::MoveBar(MoveBar::new(
            self.ctx.clone(),
            vec![torrent.id.clone()],
            torrent.download_dir.to_string(),
        ));
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    fn pending_task(&mut self, task: StatusTask) {
        if matches!(self.current_task, CurrentTask::Status(_)) {
            return;
        }

        let state = Arc::new(Mutex::new(ThrobberState::default()));
        self.current_task = CurrentTask::Status(StatusBar::new(
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

        self.current_task = CurrentTask::Default(DefaultBar::new(self.ctx.clone()));
        self.ctx
            .send_update_action(UpdateAction::SwitchToNormalMode);
    }
}
