pub mod components;

use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;

use self::components::{tabcomponent::CurrentTab, Component, TabComponent, TorrentsTab};

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

#[derive(Default)]
struct Pipup {
    error_popup: Option<ErrorPopup>,
    help_popup: Option<HelpPopup>,
}

impl Pipup {
    fn needs_action(&self) -> bool {
        self.error_popup.is_some() || self.help_popup.is_some()
    }
}

impl Component for Pipup {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Some(popup) = &mut self.error_popup {
            if let Some(Action::Quit) = popup.handle_events(action) {
                self.error_popup = None;
                return Some(Action::Render);
            }
            None
        } else if let Some(popup) = &mut self.help_popup {
            if let Some(Action::Quit) = popup.handle_events(action) {
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
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Action::Confirm = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, _rect: Rect) {
        let centered_rect = centered_rect(f.size(), 50, 50);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));
        let button_rect = {
            let temp_rect = Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                .split(text_rect)[1];
            temp_rect
        };

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

struct HelpPopup;

impl Component for HelpPopup {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
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
            .title_style(Style::new().red())
            .title("Help");

        let global_headline =
            Line::styled("GLOBAL KEYBINDINGS", Style::new().bold().underlined()).centered();

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(global_headline, text_rect);
    }
}

pub struct MainWindow {
    tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: SearchTab,
    popup: Pipup,
}

struct SearchTab {}

impl SearchTab {
    fn new() -> Self {
        Self {}
    }
}

impl Component for SearchTab {}

impl MainWindow {
    pub fn new(trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(trans_tx),
            search_tab: SearchTab::new(),
            popup: Pipup::default(),
        }
    }
}

impl Component for MainWindow {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Action::Error(e_popup) = action {
            self.popup.error_popup = Some(*e_popup);
            return Some(Action::Render);
        }

        if let Action::ShowHelp = action {
            self.popup.help_popup = Some(HelpPopup);
            return Some(Action::Render);
        }

        if let Action::ChangeTab(_) = action {
            self.tabs.handle_events(action);
            return Some(Action::Render);
        }

        if self.popup.needs_action() {
            return self.popup.handle_events(action);
        } else {
            match self.tabs.current_tab {
                CurrentTab::Torrents => return self.torrents_tab.handle_events(action),
                CurrentTab::Search => self.search_tab.handle_events(action),
                CurrentTab::Settings => todo!(),
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_bar, main_window] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(rect);

        self.tabs.render(f, top_bar);

        match self.tabs.current_tab {
            CurrentTab::Torrents => self.torrents_tab.render(f, main_window),
            CurrentTab::Search => self.search_tab.render(f, main_window),
            CurrentTab::Settings => todo!(),
        }

        self.popup.render(f, f.size());
    }
}
