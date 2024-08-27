use std::sync::Arc;

use ratatui::{
    prelude::*,
    widgets::{BarChart, Block, Clear, Paragraph},
};
use rm_config::CONFIG;
use transmission_rpc::types::SessionStats;

use rm_shared::{action::Action, utils::bytes_to_human_format};

use crate::tui::components::{
    popup_block_with_close_highlight, popup_rects, Component, ComponentAction,
};

pub struct StatisticsPopup {
    stats: Arc<SessionStats>,
    upload_data: Vec<(&'static str, u64)>,
    download_data: Vec<(&'static str, u64)>,
    max_up: i64,
    max_down: i64,
}

impl StatisticsPopup {
    pub fn new(stats: Arc<SessionStats>) -> Self {
        Self {
            upload_data: vec![("", stats.upload_speed as u64)],
            download_data: vec![("", stats.download_speed as u64)],
            max_up: stats.upload_speed,
            max_down: stats.download_speed,
            stats,
        }
    }

    pub fn update_stats(&mut self, stats: &SessionStats) {
        let up = stats.upload_speed;
        let down = stats.download_speed;

        if up > self.max_up {
            self.max_up = up;
        }

        if down > self.max_down {
            self.max_down = down;
        }

        self.upload_data.insert(0, ("", up as u64));
        self.download_data.insert(0, ("", down as u64));
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
        let (popup_rect, block_rect, text_rect) = popup_rects(rect, 75, 50);

        let [text_rect, _, upload_rect, download_rect] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(text_rect);

        let block = popup_block_with_close_highlight(" Statistics ");

        let upload_barchart = make_barchart("Upload", self.max_up as u64, &self.upload_data);
        let download_barchart =
            make_barchart("Download", self.max_down as u64, &self.download_data);

        let uploaded_bytes = self.stats.cumulative_stats.uploaded_bytes;
        let downloaded_bytes = self.stats.cumulative_stats.downloaded_bytes;
        let uploaded = bytes_to_human_format(uploaded_bytes);
        let downloaded = bytes_to_human_format(downloaded_bytes);
        let ratio = uploaded_bytes as f64 / downloaded_bytes as f64;
        let text = format!(
            "Total uploaded: {uploaded}\nTotal downloaded: {downloaded}\nRatio: {ratio:.2}"
        );
        let paragraph = Paragraph::new(text);

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
        f.render_widget(upload_barchart, upload_rect);
        f.render_widget(download_barchart, download_rect);
    }
}

fn make_barchart<'a>(
    name: &'static str,
    max: u64,
    data: &'a [(&'static str, u64)],
) -> BarChart<'a> {
    let avg = bytes_to_human_format(
        (data.iter().fold(0, |acc, x| acc + x.1) / u64::try_from(data.len()).unwrap()) as i64,
    );

    BarChart::default()
        .block(Block::new().title(format!(
            "{name} (avg {avg}/sec - max {})",
            bytes_to_human_format(max as i64)
        )))
        .bar_width(1)
        .bar_gap(0)
        .bar_style(Style::new().fg(CONFIG.general.accent_color))
        .data(data)
        .max(max)
}
