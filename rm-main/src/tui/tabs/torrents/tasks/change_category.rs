use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_config::CONFIG;
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};

use crate::{
    transmission::TorrentAction,
    tui::{
        components::{Component, ComponentAction, InputManager},
        ctx::CTX,
    },
};

use super::TorrentSelection;

pub struct ChangeCategory {
    selection: TorrentSelection,
    category_input_mgr: InputManager,
    directory_input_mgr: InputManager,
    default_dir: Option<String>,
    stage: Stage,
}

enum Stage {
    Category,
    Directory,
}

impl ChangeCategory {
    pub fn new(selection: TorrentSelection) -> Self {
        let prompt = "New category: ".to_string();

        Self {
            selection,
            category_input_mgr: InputManager::new(prompt)
                .autocompletions(CONFIG.categories.map.keys().cloned().collect()),
            directory_input_mgr: InputManager::new(
                "Move to category's default dir? (Y/n): ".into(),
            ),
            stage: Stage::Category,
            default_dir: None,
        }
    }
    fn send_status_task(&self) {
        let task = StatusTask::new_category(self.category_input_mgr.text());
        CTX.send_update_action(UpdateAction::StatusTaskSet(task));
    }

    fn set_stage_directory(&mut self, directory: String) {
        self.default_dir = Some(directory);
        self.stage = Stage::Directory;
    }

    fn handle_input(&mut self, input: KeyEvent) -> ComponentAction {
        match self.stage {
            Stage::Category => self.handle_category_input(input),
            Stage::Directory => self.handle_directory_input(input),
        }
    }

    fn handle_directory_input(&mut self, input: KeyEvent) -> ComponentAction {
        if input.code == KeyCode::Enter {
            if self.directory_input_mgr.text().to_lowercase() == "y"
                || self.directory_input_mgr.text().is_empty()
            {
                CTX.send_torrent_action(TorrentAction::ChangeCategory(
                    self.selection.ids(),
                    self.category_input_mgr.text(),
                ));

                CTX.send_torrent_action(TorrentAction::Move(
                    self.selection.ids(),
                    self.default_dir
                        .take()
                        .expect("it was set in the previous stage"),
                ));

                self.send_status_task();

                return ComponentAction::Quit;
            } else if self.directory_input_mgr.text().to_lowercase() == "n" {
                CTX.send_torrent_action(TorrentAction::ChangeCategory(
                    self.selection.ids(),
                    self.category_input_mgr.text(),
                ));

                self.send_status_task();

                return ComponentAction::Quit;
            }
        }

        if input.code == KeyCode::Esc {
            return ComponentAction::Quit;
        }

        if self.directory_input_mgr.handle_key(input).is_some() {
            CTX.send_action(Action::Render);
        }

        ComponentAction::Nothing
    }

    fn handle_category_input(&mut self, input: KeyEvent) -> ComponentAction {
        if input.code == KeyCode::Enter {
            let category = self.category_input_mgr.text();

            if let Some(config_category) = CONFIG.categories.map.get(&category) {
                self.set_stage_directory(config_category.default_dir.clone());
                CTX.send_action(Action::Render);
                return ComponentAction::Nothing;
            } else {
                CTX.send_torrent_action(TorrentAction::ChangeCategory(
                    self.selection.ids(),
                    category.clone(),
                ));
                self.send_status_task();
                return ComponentAction::Quit;
            };
        }

        if input.code == KeyCode::Esc {
            return ComponentAction::Quit;
        }

        if self.category_input_mgr.handle_key(input).is_some() {
            CTX.send_action(Action::Render);
        }

        ComponentAction::Nothing
    }
}

impl Component for ChangeCategory {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            Action::Input(input) => self.handle_input(input),
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match self.stage {
            Stage::Category => self.category_input_mgr.render(f, rect),
            Stage::Directory => self.directory_input_mgr.render(f, rect),
        }
    }
}
