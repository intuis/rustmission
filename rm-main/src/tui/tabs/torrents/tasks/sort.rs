use rm_config::{keymap::GeneralAction, CONFIG};

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

        if let Some(keys) = CONFIG
            .keybindings
            .general
            .get_keys_for_action_joined(GeneralAction::Close)
        {
            line_is_empty = false;
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - reset & exit"));
        }

        if let Some(keys) = CONFIG
            .keybindings
            .general
            .get_keys_for_action_joined(GeneralAction::Confirm)
        {
            if !line_is_empty {
                line.push_span(Span::raw(" | "));
            }
            line_is_empty = false;
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - apply"));
        }

        if let Some(keys) = CONFIG
            .keybindings
            .general
            .get_keys_for_action_joined(GeneralAction::Down)
        {
            if !line_is_empty {
                line.push_span(Span::raw(" | "));
            }
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - reverse"));
        }

        f.render_widget(line, rect);
    }
}
