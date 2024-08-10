use magnetease::Provider;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use rm_shared::action::{Action, UpdateAction};
use throbber_widgets_tui::ThrobberState;

use crate::{app, ui::components::Component};

use super::ProviderResult;

pub struct BottomBar {
    pub search_state: SearchState,
}

impl BottomBar {
    pub fn new(ctx: app::Ctx, providers: &[&(dyn Provider + Send + Sync)]) -> Self {
        Self {
            search_state: SearchState::new(ctx, providers),
        }
    }
}

impl Component for BottomBar {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.search_state.render(f, rect);
    }

    fn tick(&mut self) {
        self.search_state.tick();
    }
}

pub struct SearchState {
    ctx: app::Ctx,
    stage: SearchStage,
    pub provider_results: Vec<ProviderResult>,
    providers_count: usize,
}

#[derive(Clone)]
enum SearchStage {
    Nothing,
    NoResults,
    Searching(ThrobberState),
    Found(usize),
}

impl SearchState {
    fn new(ctx: app::Ctx, providers: &[&(dyn Provider + Send + Sync)]) -> Self {
        Self {
            ctx,
            stage: SearchStage::Nothing,
            provider_results: vec![],
            providers_count: providers.len(),
        }
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
        match action {
            UpdateAction::SearchStarted => {
                self.searching();
                self.provider_results.drain(..);
            }
            _ => (),
        };
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.stage {
            SearchStage::Nothing => (),
            SearchStage::Searching(ref mut state) => {
                let label = format!(
                    "Searching... {:.0}%",
                    self.provider_results.len() / self.providers_count
                );
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label(label)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(default_throbber.clone(), rect, state);
            }
            SearchStage::NoResults => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().red()));
                line.push_span(Span::raw(" No results"));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
            SearchStage::Found(count) => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().green()));
                line.push_span(Span::raw(format!(" Found {count}")));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn tick(&mut self) {
        if let SearchStage::Searching(state) = &mut self.stage {
            state.calc_next();
            self.ctx.send_action(Action::Render);
        }
    }
}
