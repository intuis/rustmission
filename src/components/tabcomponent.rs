use super::Component;
use ratatui::{prelude::*, widgets::Tabs};

pub struct TabComponent {
    tabs_list: Vec<&'static str>,
}

impl Component for TabComponent {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(rect.width / 2 - 25),
                Constraint::Percentage(100),
            ])
            .split(rect);

        let tab = Tabs::new(self.tabs_list.clone())
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(0)
            .divider(symbols::DOT);

        f.render_widget(tab, layout[1]);
    }
}

impl TabComponent {
    pub fn new() -> Self {
        Self {
            tabs_list: vec!["Torrents", "Search", "Settings"],
        }
    }
}
