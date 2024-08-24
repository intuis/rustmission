use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, BorderType, Clear},
};
use rm_config::CONFIG;
use rm_shared::action::Action;
use style::Styled;

use crate::tui::{
    app,
    components::{popup_close_button_highlight, Component, ComponentAction},
    main_window::centered_rect,
};

pub struct DetailsPopup {
    ctx: app::Ctx,
}

impl DetailsPopup {
    pub fn new(ctx: app::Ctx) -> Self {
        Self { ctx }
    }
}

impl Component for DetailsPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            _ if action.is_soft_quit() => ComponentAction::Quit,
            Action::Confirm => ComponentAction::Quit,
            Action::ShowFiles => {
                self.ctx.send_action(Action::ShowFiles);
                ComponentAction::Quit
            }
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(Margin::new(1, 1));
        let text_rect = block_rect.inner(Margin::new(3, 2));

        let title_style = Style::default().fg(CONFIG.general.accent_color);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Details ".set_style(title_style)))
            .title(popup_close_button_highlight());

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
    }
}
