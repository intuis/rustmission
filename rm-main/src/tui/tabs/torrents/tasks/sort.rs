use rm_config::CONFIG;
use rm_shared::action::Action;

use ratatui::prelude::*;

use crate::tui::components::{keybinding_style, Component};

pub struct Sort {}

impl Sort {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Component for Sort {
    fn render(&mut self, f: &mut Frame<'_>, rect: Rect) {
        let mut line = Line::default();
        let mut line_is_empty = true;

        if let Some(keys) = CONFIG.keybindings.get_keys_for_action_joined(Action::Close) {
            line_is_empty = false;
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - reset & exit"));
        }

        if let Some(keys) = CONFIG.keybindings.get_keys_for_action_joined(Action::Confirm) {
            if !line_is_empty {
                line.push_span(Span::raw(" | "));
            }
            line_is_empty = false;
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - apply"));
        }

        if let Some(keys) = CONFIG.keybindings.get_keys_for_action_joined(Action::Down) {
            if !line_is_empty {
                line.push_span(Span::raw(" | "));
            }
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - reverse"));
        }

        f.render_widget(line, rect);
    }
}
