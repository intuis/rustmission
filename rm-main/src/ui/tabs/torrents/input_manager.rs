use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};
use tui_input::{Input, InputRequest};

use crate::{action::Action, ui::components::Component};

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

    pub fn handle(&mut self, req: InputRequest) {
        self.input.handle(req);
    }
}

impl Component for InputManager {
    fn handle_actions(&mut self, _action: Action) -> Option<Action> {
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(Clear, rect);

        let paragraph_text = format!("{}{}", self.prompt, self.text());

        let input = self.input.to_string();
        let prefix_len = paragraph_text.len() - input.len();

        let paragraph = Paragraph::new(paragraph_text);
        f.render_widget(paragraph, rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(rect.x + u16::try_from(cursor_offset).unwrap(), rect.y);
    }
}
