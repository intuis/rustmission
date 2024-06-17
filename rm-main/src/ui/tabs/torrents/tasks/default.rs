use crate::ui::components::Component;

use ratatui::prelude::*;

pub struct DefaultBar;

impl DefaultBar {
    pub const fn new() -> Self {
        Self
    }
}

impl Component for DefaultBar {
    fn render(&mut self, f: &mut ratatui::Frame<'_>, rect: Rect) {
        f.render_widget("F1 - help", rect)
    }
}
