use std::sync::{Arc, Mutex};

use ratatui::prelude::*;

use crate::{action::Action, app, ui::components::Component};

use super::{
    tasks::{
        add_magnet::AddMagnetBar,
        default::DefaultBar,
        delete_torrent::{self, DeleteBar},
        filter::FilterBar,
    },
    TableManager,
};

pub struct TaskManager {
    ctx: app::Ctx,
    current_task: CurrentTask,
    table_manager: Arc<Mutex<TableManager>>,
}

impl TaskManager {
    pub const fn new(table_manager: Arc<Mutex<TableManager>>, ctx: app::Ctx) -> Self {
        Self {
            ctx,
            current_task: CurrentTask::Default(DefaultBar::new()),
            table_manager,
        }
    }
}

enum CurrentTask {
    AddMagnetBar(AddMagnetBar),
    DeleteBar(DeleteBar),
    FilterBar(FilterBar),
    Default(DefaultBar),
}

impl Component for TaskManager {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => match magnet_bar.handle_actions(action) {
                Some(A::Quit) => self.finish_task(),
                Some(A::Render) => Some(A::Render),
                _ => None,
            },

            CurrentTask::DeleteBar(delete_bar) => match delete_bar.handle_actions(action) {
                Some(A::Quit) => self.finish_task(),
                Some(A::Render) => Some(A::Render),
                _ => None,
            },

            CurrentTask::FilterBar(filter_bar) => match filter_bar.handle_actions(action) {
                Some(A::Quit) => self.finish_task(),
                Some(A::Render) => Some(A::Render),
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

    fn finish_task(&mut self) -> Option<Action> {
        match self.current_task {
            CurrentTask::AddMagnetBar(_) => {
                self.current_task = CurrentTask::Default(DefaultBar::new());
                Some(Action::SwitchToNormalMode)
            }
            CurrentTask::DeleteBar(_) => {
                self.current_task = CurrentTask::Default(DefaultBar::new());
                Some(Action::SwitchToNormalMode)
            }
            CurrentTask::FilterBar(_) => {
                self.current_task = CurrentTask::Default(DefaultBar::new());
                Some(Action::SwitchToNormalMode)
            }
            CurrentTask::Default(_) => None,
        }
    }
}
