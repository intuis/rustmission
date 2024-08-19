use crossterm::event::{Event, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};
use rm_config::CONFIG;
use tui_input::{backend::crossterm::to_input_request, Input, InputResponse};

use crate::tui::components::Component;

pub struct InputManager {
    input: Input,
    prompt: String,
}

impl InputManager {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            input: Input::default(),
        }
    }

    pub fn new_with_value(prompt: String, value: String) -> Self {
        Self {
            prompt,
            input: Input::default().with_value(value),
        }
    }

    pub fn text(&self) -> String {
        self.input.to_string()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputResponse {
        let event = Event::Key(key);

        if let Some(req) = to_input_request(&event) {
            self.input.handle(req)
        } else {
            None
        }
    }
}

impl Component for InputManager {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(Clear, rect);

        let spans = vec![
            Span::styled(
                self.prompt.as_str(),
                Style::default().fg(CONFIG.general.accent_color),
            ),
            Span::raw(self.text()),
        ];

        let input = self.input.to_string();
        let prefix_len = self.prompt.len() + self.text().len() - input.len();

        let paragraph = Paragraph::new(Line::from(spans));
        f.render_widget(paragraph, rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        let cursor_position = Position {
            x: rect.x + u16::try_from(cursor_offset).unwrap(),
            y: rect.y,
        };
        f.set_cursor_position(cursor_position);
    }
}
