use std::sync::Arc;

use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};
use transmission_rpc::types::SessionStats;

use rm_shared::{action::Action, utils::bytes_to_human_format};

use crate::tui::components::{
    popup_block_with_close_highlight, popup_rects, Component, ComponentAction,
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
        let (popup_rect, block_rect, text_rect) = popup_rects(rect, 50, 50);

        let block = popup_block_with_close_highlight(" Statistics ");

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
