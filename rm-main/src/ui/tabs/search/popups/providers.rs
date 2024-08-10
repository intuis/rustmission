use ratatui::{
    layout::{Alignment, Margin},
    prelude::Rect,
    style::{Style, Styled, Stylize},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear,
    },
    Frame,
};
use rm_shared::action::Action;

use crate::{
    app,
    ui::{
        centered_rect,
        components::{Component, ComponentAction},
    },
};

pub struct ProvidersPopup {
    ctx: app::Ctx,
}

impl ProvidersPopup {
    pub const fn new(ctx: app::Ctx) -> Self {
        Self { ctx }
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
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(Margin::new(1, 1));

        let title_style = Style::default().fg(self.ctx.config.general.accent_color);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Providers ".set_style(title_style)))
            .title(
                Title::from(" [ CLOSE ] ".set_style(title_style.bold()))
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            );

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
    }
}
