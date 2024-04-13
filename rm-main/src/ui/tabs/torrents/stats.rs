use std::sync::{Arc, Mutex};

use ratatui::{
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Frame,
};
use transmission_rpc::types::SessionStats;

use crate::ui::{bytes_to_human_format, components::Component};

#[derive(Default)]
pub(super) struct StatsComponent {
    // TODO: get rid of the Option
    pub(super) stats: Arc<Mutex<Option<SessionStats>>>,
}

impl Component for StatsComponent {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(stats) = &*self.stats.lock().unwrap() {
            let upload = bytes_to_human_format(stats.upload_speed);
            let download = bytes_to_human_format(stats.download_speed);
            let all = stats.torrent_count;

            let text = format!("All: {all} | ▼ {download} | ▲ {upload}");

            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, rect);
        }
    }
}
