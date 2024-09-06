use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_config::CONFIG;
use rm_shared::{
    action::{Action, UpdateAction},
    status_task::StatusTask,
};

use crate::tui::{
    app,
    components::{Component, ComponentAction, InputManager},
};

use super::TorrentSelection;

pub struct ChangeCategory {
    selection: TorrentSelection,
    ctx: app::Ctx,
    input_mgr: InputManager,
}

impl ChangeCategory {
    pub fn new(ctx: app::Ctx, selection: TorrentSelection) -> Self {
        let prompt = "New category: ".to_string();

        Self {
            selection,
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
                    self.selection.ids(),
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
