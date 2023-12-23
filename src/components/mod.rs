use ratatui::layout::Offset;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{
    prelude::Rect,
    style::{Style, Stylize},
    symbols,
    widgets::Tabs,
    Frame,
};

use crate::{app::Action, tui::Event};

pub struct Components {
    pub tabs: TabComponent,
}

impl Components {
    pub fn new() -> Self {
        Components {
            tabs: TabComponent::new(),
        }
    }
}

pub trait Component {
    fn handle_events(&mut self, _event: Event) -> Option<Action> {
        None
    }
    fn render(&self, _f: &mut Frame, _rect: Rect) {}
}

pub struct TabComponent {
    tabs_list: Vec<&'static str>,
}

impl Component for TabComponent {
    fn render(&self, f: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                // Constraint::Min(1),
                // Constraint::Length(28),
                // Constraint::Min(1),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(rect);
        let tab = Tabs::new(self.tabs_list.clone())
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(0)
            .divider(symbols::DOT);

        f.render_widget(tab, layout[0]);
    }
}

impl TabComponent {
    fn new() -> Self {
        TabComponent {
            tabs_list: vec!["Torrents", "Search", "Settings"],
        }
    }
}
