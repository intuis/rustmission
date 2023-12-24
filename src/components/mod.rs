mod torrent_tab;

use ratatui::prelude::*;
use ratatui::{
    prelude::Rect,
    style::{Style, Stylize},
    symbols,
    widgets::Tabs,
    Frame,
};

use crate::{app::Action, tui::Event};

use self::torrent_tab::TorrentsTab;

pub struct Components {
    pub tabs: TabComponent,
    pub torrents_tab: TorrentsTab,
}

impl Components {
    pub fn new() -> Self {
        Components {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(),
        }
    }
}

pub trait Component {
    fn handle_events(&mut self, _event: Event) -> Option<Action> {
        None
    }
    fn render(&mut self, _f: &mut Frame, _rect: Rect) {}
}

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
    fn new() -> Self {
        TabComponent {
            tabs_list: vec!["Torrents", "Search", "Settings"],
        }
    }
}
