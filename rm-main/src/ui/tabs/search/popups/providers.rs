use magnetease::ProviderCategory;
use ratatui::{
    layout::{Alignment, Constraint, Margin},
    prelude::Rect,
    style::{Style, Styled, Stylize},
    text::{Line, Span, ToLine, ToSpan},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear, Row, Table,
    },
    Frame,
};
use rm_shared::action::Action;

use crate::{
    app,
    ui::{
        centered_rect,
        components::{Component, ComponentAction},
        tabs::search::{ConfiguredProvider, ProviderState},
    },
};

pub struct ProvidersPopup {
    ctx: app::Ctx,
    providers: Vec<ConfiguredProvider>,
}

impl From<&ConfiguredProvider> for Row<'_> {
    fn from(value: &ConfiguredProvider) -> Self {
        let mut name: Line = match value.provider_state {
            ProviderState::Idle => " 󱗼 ".yellow().into(),
            ProviderState::Searching => "  ".yellow().into(),
            ProviderState::Found(_) => "  ".green().into(),
            ProviderState::Error(_) => "  ".red().into(),
        };

        name.push_span(value.provider.name());

        let category = match value.provider.category() {
            ProviderCategory::General => " General",
            ProviderCategory::Anime => "󰎁 Anime",
        };

        let url = format!("({})", value.provider.display_url());

        let status: Span = match &value.provider_state {
            _ if !value.enabled => "Disabled".into(),
            ProviderState::Idle => "Idle".into(),
            ProviderState::Searching => " Searching...".yellow(),
            ProviderState::Found(_) => format!("Found").into(),
            ProviderState::Error(e) => e.to_string().into(),
        };

        let padding = "";

        let count = {
            if let ProviderState::Found(count) = value.provider_state {
                count.to_string().green()
            } else {
                "".into()
            }
        };

        Row::new(vec![
            name,
            url.into(),
            category.into(),
            status.into(),
            padding.into(),
            count.into(),
        ])
    }
}

impl ProvidersPopup {
    pub const fn new(ctx: app::Ctx, providers: Vec<ConfiguredProvider>) -> Self {
        Self { ctx, providers }
    }
}

impl Component for ProvidersPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            _ if action.is_soft_quit() => ComponentAction::Quit,
            Action::Confirm => ComponentAction::Quit,
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 80, 50);
        let block_rect = popup_rect.inner(Margin::new(1, 1));
        let table_rect = block_rect.inner(Margin::new(1, 1));

        let title_style = Style::default().fg(self.ctx.config.general.accent_color);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Providers ".set_style(title_style)))
            .title(
                Title::from(" [ CLOSE ] ".set_style(title_style.bold()))
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            );

        let widths = [
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Percentage(100),
            Constraint::Length(4),
        ];

        let rows: Vec<Row<'_>> = self
            .providers
            .iter()
            .map(|provider| provider.into())
            .collect();

        let table = Table::new(rows, widths);

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(table, table_rect);
    }
}
