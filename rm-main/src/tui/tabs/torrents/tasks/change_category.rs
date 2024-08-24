use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_config::CONFIG;
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};
use transmission_rpc::types::Id;

use crate::tui::{
    app,
    components::{Component, ComponentAction, InputManager},
};

pub struct ChangeCategory {
    torrents_to_change: Vec<Id>,
    ctx: app::Ctx,
    input_mgr: InputManager,
}

impl ChangeCategory {
    pub fn new(ctx: app::Ctx, torrents_to_change: Vec<Id>) -> Self {
        let prompt = "New category: ".to_string();

        Self {
            torrents_to_change,
            input_mgr: InputManager::new(prompt)
                .autocompletions(CONFIG.categories.map.keys().cloned().collect()),
            ctx,
        }
    }

    fn handle_input(&mut self, input: KeyEvent) -> ComponentAction {
        if input.code == KeyCode::Enter {
            let category = self.input_mgr.text();
            self.ctx
                .send_torrent_action(crate::transmission::TorrentAction::ChangeCategory(
                    self.torrents_to_change.clone(),
                    category.clone(),
                ));

            let task = StatusTask::new_category(category);
            self.ctx
                .send_update_action(UpdateAction::StatusTaskSet(task));
            return ComponentAction::Quit;
        }

        if input.code == KeyCode::Esc {
            return ComponentAction::Quit;
        }

        if self.input_mgr.handle_key(input).is_some() {
            self.ctx.send_action(Action::Render);
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
        self.input_mgr.render(f, rect);
    }
}
