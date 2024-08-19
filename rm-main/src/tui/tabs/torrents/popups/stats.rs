use std::sync::Arc;

use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{block::Title, Block, BorderType, Clear, Paragraph},
};
use rm_config::CONFIG;
use transmission_rpc::types::SessionStats;

use rm_shared::{action::Action, utils::bytes_to_human_format};

use crate::tui::{
    components::{popup_close_button_highlight, Component, ComponentAction},
    main_window::centered_rect,
};

pub struct StatisticsPopup {
    stats: Arc<SessionStats>,
}

impl StatisticsPopup {
    pub const fn new(stats: Arc<SessionStats>) -> Self {
        Self { stats }
    }
}

impl Component for StatisticsPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;
        match action {
            _ if action.is_soft_quit() => ComponentAction::Quit,
            A::Confirm => ComponentAction::Quit,
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
            .title(Title::from(" Statistics ".set_style(title_style)))
            .title(popup_close_button_highlight());

        let uploaded_bytes = self.stats.cumulative_stats.uploaded_bytes;
        let downloaded_bytes = self.stats.cumulative_stats.downloaded_bytes;
        let uploaded = bytes_to_human_format(uploaded_bytes);
        let downloaded = bytes_to_human_format(downloaded_bytes);
        let ratio = uploaded_bytes as f64 / downloaded_bytes as f64;
        let text = format!("Uploaded: {uploaded}\nDownloaded: {downloaded}\nRatio: {ratio:.2}");
        let paragraph = Paragraph::new(text);

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
    }
}
