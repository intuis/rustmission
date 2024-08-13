use crate::ui::components::Component;

use ratatui::prelude::*;
use rm_config::CONFIG;

pub struct DefaultBar {}

impl DefaultBar {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Component for DefaultBar {
    fn render(&mut self, f: &mut ratatui::Frame<'_>, rect: Rect) {
        if CONFIG.general.beginner_mode {
            if let Some(keys) = CONFIG
                .keybindings
                .get_keys_for_action(rm_shared::action::Action::ShowHelp)
            {
                f.render_widget(format!("󰘥 {keys} - help"), rect)
            }
        }
    }
}
