use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph},
};
use ratatui_macros::constraints;
use transmission_rpc::types::SessionStats;

use crate::{
    action::Action,
    app,
    ui::{bytes_to_human_format, centered_rect, components::Component},
};

pub struct StatisticsPopup {
    stats: SessionStats,
    ctx: app::Ctx,
}

impl StatisticsPopup {
    pub fn new(ctx: app::Ctx, stats: SessionStats) -> Self {
        Self { ctx, stats }
    }
}

impl Component for StatisticsPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Confirm = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(&Margin::new(1, 1));
        let text_rect = block_rect.inner(&Margin::new(3, 2));
        let button_rect = { Layout::vertical(constraints![==100%, ==1]).split(text_rect)[1] };

        let title_style = Style::default().fg(self.ctx.config.general.accent_color.as_ratatui());
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Statistics ")
            .title_style(title_style);

        let button = Paragraph::new("[ OK ]").bold().right_aligned();

        let uploaded_bytes = self.stats.cumulative_stats.uploaded_bytes;
        let downloaded_bytes = self.stats.cumulative_stats.downloaded_bytes;
        let uploaded = bytes_to_human_format(uploaded_bytes);
        let downloaded = bytes_to_human_format(downloaded_bytes);
        let ratio = uploaded_bytes / downloaded_bytes;
        let text = format!("Uploaded: {uploaded}\nDownloaded: {downloaded}\nRatio: {ratio}");
        let paragraph = Paragraph::new(text);

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
        f.render_widget(button, button_rect);
    }
}
