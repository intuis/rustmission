use crate::tui::components::{keybinding_style, Component};
use rm_config::CONFIG;
use rm_shared::action::Action;

use ratatui::{prelude::*, text::Span};

pub struct Selection {
    selection_amount: usize,
}

impl Selection {
    pub const fn new(selection_amount: usize) -> Self {
        Self { selection_amount }
    }
}

impl Component for Selection {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let mut line = Line::default();
        let mut line_is_empty = true;

        if let Some(keys) = CONFIG.keybindings.get_keys_for_action_joined(Action::Close) {
            line_is_empty = false;
            line.push_span(Span::styled(keys, keybinding_style()));
            line.push_span(Span::raw(" - clear selection"));
        }

        if !line_is_empty {
            line.push_span(Span::raw(" | "));
        }

        line.push_span(format!("{} selected", self.selection_amount));

        f.render_widget(line, rect);
    }
}
