use super::Component;
use ratatui::{layout::Flex, prelude::*, widgets::Tabs};

pub struct TabComponent {
    tabs_list: Vec<&'static str>,
}

impl TabComponent {
    pub fn new() -> Self {
        Self {
            tabs_list: vec!["Torrents", "Search", "Settings"],
        }
    }
}

impl Component for TabComponent {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let center_rect = Layout::horizontal([
            Constraint::Length(30),
            Constraint::Length(30),
            Constraint::Length(30),
        ])
        .flex(Flex::Center)
        .split(rect)[1];

        let tabs = Tabs::new(self.tabs_list.clone())
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(0)
            .divider(symbols::DOT);

        f.render_widget(tabs, center_rect);
    }
}
