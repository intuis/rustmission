use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use rm_config::{keymap::SearchAction, CONFIG};
use rm_shared::action::{Action, UpdateAction};
use throbber_widgets_tui::ThrobberState;

use crate::tui::{
    components::{keybinding_style, Component, ComponentAction},
    ctx::CTX,
    tabs::torrents::tasks,
};

use super::{ConfiguredProvider, ProviderState};

pub struct BottomBar {
    pub search_state: SearchState,
    pub task: Option<tasks::AddMagnet>,
}

impl BottomBar {
    pub fn new(providers: &Vec<ConfiguredProvider>) -> Self {
        Self {
            search_state: SearchState::new(providers),
            task: None,
        }
    }

    pub fn add_magnet(&mut self, magnet: impl Into<String>) {
        self.task = Some(tasks::AddMagnet::new().magnet(magnet));
        CTX.send_update_action(UpdateAction::SwitchToInputMode);
    }

    pub fn requires_input(&self) -> bool {
        self.task.is_some()
    }
}

impl Component for BottomBar {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(task) = &mut self.task {
            task.render(f, rect);
        } else {
            self.search_state.render(f, rect);
        }
    }

    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        if let Some(task) = &mut self.task {
            if task.handle_actions(action).is_quit() {
                self.task = None;
                CTX.send_update_action(UpdateAction::SwitchToNormalMode);
            };
        }

        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        self.search_state.handle_update_action(action);
    }

    fn tick(&mut self) {
        self.search_state.tick();
    }
}

pub struct SearchState {
    stage: SearchStage,
    providers_finished: u8,
    providers_errored: u8,
    providers_count: u8,
}

#[derive(Clone)]
enum SearchStage {
    Nothing,
    NoResults,
    Searching(ThrobberState),
    Found(usize),
}

impl SearchState {
    fn new(providers: &Vec<ConfiguredProvider>) -> Self {
        let mut providers_count = 0u8;
        for provider in providers {
            if provider.enabled {
                providers_count += 1;
            }
        }

        Self {
            stage: SearchStage::Nothing,
            providers_errored: 0,
            providers_finished: 0,
            providers_count,
        }
    }

    pub fn update_counts(&mut self, providers: &Vec<ConfiguredProvider>) {
        let mut providers_finished = 0;
        let mut providers_errored = 0;
        for provider in providers {
            if provider.enabled {
                if matches!(provider.provider_state, ProviderState::Found(_)) {
                    providers_finished += 1;
                } else if matches!(provider.provider_state, ProviderState::Error(_)) {
                    providers_errored += 1;
                }
            }
        }

        self.providers_finished = providers_finished;
        self.providers_errored = providers_errored;
    }

    pub fn searching(&mut self) {
        self.stage = SearchStage::Searching(ThrobberState::default());
    }

    pub fn not_found(&mut self) {
        self.stage = SearchStage::NoResults;
    }

    pub fn found(&mut self, count: usize) {
        self.stage = SearchStage::Found(count);
    }
}

impl Component for SearchState {
    fn handle_update_action(&mut self, action: UpdateAction) {
        if let UpdateAction::SearchStarted = action {
            self.searching();
        };
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let append_key_info = |line: &mut Line| {
            let providers_key = CONFIG
                .keybindings
                .search_tab
                .get_keys_for_action_joined(SearchAction::ShowProvidersInfo);
            if let Some(key) = providers_key {
                line.push_span(Span::raw("Press "));
                line.push_span(Span::styled(key, keybinding_style()));
                line.push_span(Span::raw(" for details."))
            }
        };

        match &mut self.stage {
            SearchStage::Nothing => (),
            SearchStage::Searching(ref mut state) => {
                let label = format!(
                    "Searching... {}/{}",
                    self.providers_finished, self.providers_count
                );
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label(label)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(default_throbber.clone(), rect, state);
            }
            SearchStage::NoResults => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().red()));
                line.push_span(Span::raw(" No results. "));
                append_key_info(&mut line);
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
            SearchStage::Found(count) => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().green()));
                line.push_span(Span::raw(format!(" Found {count}. ")));
                append_key_info(&mut line);
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn tick(&mut self) {
        if let SearchStage::Searching(state) = &mut self.stage {
            state.calc_next();
            CTX.send_action(Action::Render);
        }
    }
}
