use ratatui::prelude::*;
use rm_config::CONFIG;
use rm_shared::action::Action;

use crate::tui::components::{keybinding_style, Component};

pub struct Default {}

impl Default {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Component for Default {
    fn render(&mut self, f: &mut Frame<'_>, rect: Rect) {
        let mut line = Line::default();
        let mut line_is_empty = true;

        if CONFIG.general.beginner_mode {
            if let Some(keys) = CONFIG.keybindings.get_keys_for_action(Action::ShowHelp) {
                line_is_empty = false;
                line.push_span(Span::raw(format!("{} ", CONFIG.icons.help)));
                line.push_span(Span::styled(keys, keybinding_style()));
                line.push_span(Span::raw(" - help"));
            }
            if let Some(keys) = CONFIG.keybindings.get_keys_for_action(Action::Confirm) {
                if !line_is_empty {
                    line.push_span(Span::raw(" | "));
                } else {
                    line.push_span(Span::raw(format!("{} ", CONFIG.icons.help)));
                }
                line.push_span(Span::styled(keys, keybinding_style()));
                line.push_span(Span::raw(" - details"));
            }
        }
        f.render_widget(line, rect);
    }
}
