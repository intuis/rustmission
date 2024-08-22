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
    autocompletions: Vec<String>,
}

impl InputManager {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            input: Input::default(),
            autocompletions: vec![],
        }
    }

    pub fn new_with_value(prompt: String, value: String) -> Self {
        Self {
            prompt,
            input: Input::default().with_value(value),
            autocompletions: vec![],
        }
    }

    pub fn autocompletions(mut self, autocompletions: Vec<String>) -> Self {
        self.autocompletions = autocompletions;
        self
    }

    pub fn get_autocompletion(&self) -> Option<&str> {
        let mut autocompletion = None;
        for possible_autocompletion in &self.autocompletions {
            if possible_autocompletion.starts_with(&self.input.to_string()) {
                autocompletion = Some(possible_autocompletion);
            }
        }
        autocompletion.map(|x| x.as_str())
    }

    pub fn apply_autocompletion(&mut self) {
        let completion = self.get_autocompletion().map(|str| str.to_string());
        if let Some(completion) = completion {
            self.set_text(completion);
        }
    }

    pub fn visual_cursor(&self) -> usize {
        self.input.visual_cursor()
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

    pub fn set_prompt(&mut self, new_prompt: impl Into<String>) {
        self.prompt = new_prompt.into();
    }

    pub fn set_text(&mut self, new_text: impl Into<String>) {
        self.input = self.input.clone().with_value(new_text.into());
    }
}

impl Component for InputManager {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(Clear, rect);

        let input = self.input.to_string();
        let spans = vec![
            Span::styled(
                self.prompt.as_str(),
                Style::default().fg(CONFIG.general.accent_color),
            ),
            Span::styled(self.text(), Style::default().fg(Color::White)),
        ];

        let paragraph = Paragraph::new(Line::from(spans));
        f.render_widget(paragraph, rect);

        let prefix_len =
            u16::try_from(self.prompt.len() + self.text().len() - input.len()).unwrap();
        if let Some(completion) = self.get_autocompletion() {
            let already_typed = u16::try_from(input.chars().count()).unwrap();
            let span = Span::from(&completion[already_typed as usize..]).dark_gray();
            let completion_rect = rect.inner(Margin {
                horizontal: prefix_len + already_typed,
                vertical: 0,
            });
            f.render_widget(span, completion_rect);
        }

        let cursor_offset = u16::try_from(self.input.visual_cursor()).unwrap() + prefix_len;
        let cursor_position = Position {
            x: rect.x + u16::try_from(cursor_offset).unwrap(),
            y: rect.y,
        };
        f.set_cursor_position(cursor_position);
    }
}
