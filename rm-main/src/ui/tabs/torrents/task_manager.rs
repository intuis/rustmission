use std::sync::{Arc, Mutex};

use ratatui::prelude::*;
use throbber_widgets_tui::ThrobberState;

use crate::{app, ui::components::{Component, ComponentAction}};
use rm_shared::{action::{Action, UpdateAction}, status_task::StatusTask};

use super::{
    tasks::{
        add_magnet::AddMagnetBar,
        default::DefaultBar,
        delete_torrent::{self, DeleteBar, TorrentInfo},
        filter::FilterBar,
        move_torrent::MoveBar,
        status::{CurrentTaskState, StatusBar},
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
        use Action as A;
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => match magnet_bar.handle_actions(action) {
                // Some(A::TaskPending(task)) => self.pending_task(task),
                ComponentAction::Quit => self.cancel_task(),
                _ => (),
            },

            CurrentTask::DeleteBar(delete_bar) => match delete_bar.handle_actions(action) {
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
                ComponentAction::Quit => self.cancel_task(),
                _ => (),
            },
            CurrentTask::MoveBar(move_bar) => match move_bar.handle_actions(action) {
                // Some(A::TaskPending(task)) => self.pending_task(task),
                ComponentAction::Quit => self.cancel_task(),
                _ => (),
            },
            CurrentTask::FilterBar(filter_bar) => match filter_bar.handle_actions(action) {
                ComponentAction::Quit => self.cancel_task(),
                _ => (),
            },

            CurrentTask::Status(status_bar) => match status_bar.handle_actions(action) {
                ComponentAction::Quit => self.cancel_task(),
                _ => (),
            },
            CurrentTask::Default(_) => self.handle_events_to_manager(&action),
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
    #[must_use]
    fn handle_events_to_manager(&mut self, action: &Action) {
        match action {
            Action::AddMagnet => {
                self.current_task = CurrentTask::AddMagnetBar(AddMagnetBar::new(self.ctx.clone()));
                self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
            }
            Action::DeleteWithFiles => self.delete_torrent(delete_torrent::Mode::WithFiles),
            Action::DeleteWithoutFiles => self.delete_torrent(delete_torrent::Mode::WithoutFiles),
            Action::MoveTorrent => self.move_torrent(),
            Action::Search => {
                self.current_task = CurrentTask::FilterBar(FilterBar::new(
                    self.ctx.clone(),
                    self.table_manager.clone(),
                ));
                self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
            }
            _ => (),
        }
    }

    fn delete_torrent(&mut self, mode: delete_torrent::Mode) {
        if let Some(torrent) = self.table_manager.lock().unwrap().current_torrent() {
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
    }

    fn move_torrent(&mut self) {
        if let Some(torrent) = self.table_manager.lock().unwrap().current_torrent() {
            self.current_task = CurrentTask::MoveBar(MoveBar::new(
                self.ctx.clone(),
                vec![torrent.id.clone()],
                torrent.download_dir.to_string(),
            ));
            self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
        }
    }

    fn pending_task(&mut self, task: StatusTask) {
        if matches!(self.current_task, CurrentTask::Status(_)) {
            return;
        }

        let state = Arc::new(Mutex::new(ThrobberState::default()));
        self.current_task =
            CurrentTask::Status(StatusBar::new(self.ctx.clone(), task, CurrentTaskState::Loading(state)));
        self.ctx.send_update_action(UpdateAction::SwitchToNormalMode);
    }

    fn cancel_task(&mut self) {
        if matches!(self.current_task, CurrentTask::Default(_)) {
            return;
        }

        self.current_task = CurrentTask::Default(DefaultBar::new(self.ctx.clone()));
        self.ctx.send_update_action(UpdateAction::SwitchToNormalMode);
    }
}
