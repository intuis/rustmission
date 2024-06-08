use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    action::Action,
    ui::{centered_rect, components::Component},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPopup {
    // TODO: make sure that title always has padding
    title: String,
    message: String,
}

impl ErrorPopup {
    pub fn new(title: &'static str, message: String) -> Self {
        Self {
            title: title.to_owned(),
            message,
        }
    }
}

impl Component for ErrorPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if action == Action::Confirm {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, _rect: Rect) {
        let centered_rect = centered_rect(f.size(), 50, 50);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));
        let button_rect = Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
            .split(text_rect)[1];

        let button = Paragraph::new("[ OK ]").bold().right_aligned();

        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title_style(Style::new().red())
            .title(format!(" {} ", self.title));

        let error_message = Paragraph::new(&*self.message).wrap(Wrap { trim: false });

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(error_message, text_rect);
        f.render_widget(button, button_rect);
    }
}
