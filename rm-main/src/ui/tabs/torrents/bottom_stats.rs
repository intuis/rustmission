use std::sync::{Arc, Mutex};

use ratatui::{
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Frame,
};
use transmission_rpc::types::{FreeSpace, SessionStats};

use crate::{ui::components::Component, utils::bytes_to_human_format};

#[derive(Default)]
pub(super) struct BottomStats {
    // TODO: get rid of the Option
    pub(super) stats: Arc<Mutex<Option<SessionStats>>>,
    pub(super) free_space: Arc<Mutex<Option<FreeSpace>>>,
}

impl Component for BottomStats {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(stats) = &*self.stats.lock().unwrap() {
            let all = stats.torrent_count;
            let download = bytes_to_human_format(stats.download_speed);
            let upload = bytes_to_human_format(stats.upload_speed);
            let mut text = format!(" {all} | ▼ {download} | ▲ {upload}");

            if let Some(free_space) = &*self.free_space.lock().unwrap() {
                let free_space = bytes_to_human_format(free_space.size_bytes);
                text = format!("󰋊 {free_space} | {text}")
            }

            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, rect);
        }
    }
}
