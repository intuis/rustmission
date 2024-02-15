use crate::action::Action;

use super::Component;
use ratatui::{layout::Flex, prelude::*, widgets::Tabs};

#[derive(Clone, Copy)]
pub enum CurrentTab {
    Torrents = 0,
    Search,
    Settings,
}

pub struct TabComponent {
    tabs_list: [&'static str; 3],
    pub current_tab: CurrentTab,
}

impl TabComponent {
    pub fn new() -> Self {
        Self {
            tabs_list: ["Torrents", "Search", "Settings"],
            current_tab: CurrentTab::Torrents,
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
            .highlight_style(Style::default().light_magenta())
            .select(self.current_tab as usize)
            .divider(symbols::DOT);

        f.render_widget(tabs, center_rect);
    }

    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Action::ChangeTab(tab) = action {
            match tab {
                1 => self.current_tab = CurrentTab::Torrents,
                2 => self.current_tab = CurrentTab::Search,
                _ => (),
            }
        }
        None
    }
}
