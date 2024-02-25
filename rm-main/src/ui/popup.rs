use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};
use ratatui_macros::constraints;

use crate::action::Action;

use super::{centered_rect, components::Component};

#[derive(Default)]
pub(super) struct Popup {
    pub error_popup: Option<ErrorPopup>,
    pub help_popup: Option<HelpPopup>,
}

impl Popup {
    pub fn needs_action(&self) -> bool {
        self.error_popup.is_some() || self.help_popup.is_some()
    }
}

impl Component for Popup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Some(popup) = &mut self.error_popup {
            if let Some(Action::Quit) = popup.handle_actions(action) {
                self.error_popup = None;
                return Some(Action::Render);
            }
            None
        } else if let Some(popup) = &mut self.help_popup {
            if let Some(Action::Quit) = popup.handle_actions(action) {
                self.help_popup = None;
                return Some(Action::Render);
            }
            None
        } else {
            None
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(popup) = &mut self.error_popup {
            popup.render(f, rect)
        } else if let Some(popup) = &mut self.help_popup {
            popup.render(f, rect);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ErrorPopup {
    // TODO: make sure that title always has padding
    title: String,
    message: String,
}

impl ErrorPopup {
    pub(crate) fn new(title: &'static str, message: String) -> Self {
        Self {
            title: title.to_owned(),
            message,
        }
    }
}

impl Component for ErrorPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Confirm = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, _rect: Rect) {
        let centered_rect = centered_rect(f.size(), 50, 50);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));
        let button_rect = { Layout::vertical(constraints![==100%, ==1]).split(text_rect)[1] };

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

pub(super) struct HelpPopup;

impl Component for HelpPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Quit = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let centered_rect = centered_rect(rect, 75, 75);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));

        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title_style(Style::new().light_magenta())
            .title(" Help ");

        let mut lines = vec![];

        lines.push(
            Line::from(vec![Span::styled(
                "Global Keybindings",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        lines.push(Line::from(vec![
            Span::styled("?", Style::default().bold()),
            " - show/hide help".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("1", Style::default().bold()),
            " - switch to torrents tab".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("2", Style::default().bold()),
            " - switch to search tab".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("/", Style::default().bold()),
            " - search".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("q", Style::default().bold()),
            " - quit Rustmission".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("TAB", Style::default().bold()),
            " - switch focus".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Enter", Style::default().bold()),
            " - confirm".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("j / ↓", Style::default().bold()),
            " - move down".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("k / ↑", Style::default().bold()),
            " - move up".into(),
        ]));

        lines.push(
            Line::from(vec![Span::styled(
                "Torrents Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        lines.push(Line::from(vec![
            Span::styled("t", Style::default().bold()),
            " - show statistics popup".into(),
        ]));

        lines.push(Line::from(vec![
            Span::styled("m", Style::default().bold()),
            " - add a magnet url/torrent path".into(),
        ]));

        let help_text = Text::from(lines);
        let help_paragraph = Paragraph::new(help_text);

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(help_paragraph, text_rect);
    }
}