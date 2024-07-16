use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::ui::{
    centered_rect,
    components::{Component, ComponentAction},
};
use rm_shared::action::Action;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPopup {
    // TODO: make sure that title always has padding
    title: String,
    message: String,
    error: String,
}

impl ErrorPopup {
    pub fn new(title: String, message: String, error: String) -> Self {
        Self {
            title,
            message,
            error,
        }
    }
}

impl Component for ErrorPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            _ if action.is_soft_quit() => ComponentAction::Quit,
            Action::Confirm => ComponentAction::Quit,
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, _rect: Rect) {
        let centered_rect = centered_rect(f.size(), 50, 50);
        let popup_rect = centered_rect.inner(Margin::new(1, 1));
        let text_rect = popup_rect.inner(Margin::new(3, 2));
        let button_rect = Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
            .split(text_rect)[1];

        let button = Paragraph::new("[ OK ]").bold().right_aligned();

        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title_style(Style::new().red())
            .title(format!(" {} ", self.title));

        let lines = vec![
            Line::from(self.message.as_str()),
            Line::default(),
            Line::from(self.error.as_str()).red().on_black(),
        ];

        let error_message = Paragraph::new(lines).wrap(Wrap { trim: false });

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(error_message, text_rect);
        f.render_widget(button, button_rect);
    }
}
