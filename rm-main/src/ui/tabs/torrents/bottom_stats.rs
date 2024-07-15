use std::sync::Arc;

use ratatui::{
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Frame,
};
use rm_shared::utils::bytes_to_human_format;
use transmission_rpc::types::{FreeSpace, SessionStats};

use crate::ui::components::Component;

use super::table_manager::TableManager;

#[derive(Default)]
pub(super) struct BottomStats {
    // TODO: get rid of the Option (requires changes in transmission-rpc so SessionStats impls Default
    // TODO: ^ The same thing with FreeSpace
    pub(super) stats: Option<Arc<SessionStats>>,
    pub(super) free_space: Option<Arc<FreeSpace>>,
    torrent_count: u16,
    torrent_currently_selected: u16,
}

impl BottomStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_stats(&mut self, stats: Arc<SessionStats>) {
        self.stats = Some(stats);
    }

    pub fn set_free_space(&mut self, free_space: Arc<FreeSpace>) {
        self.free_space = Some(free_space);
    }

    pub fn update_selected_indicator(&mut self, table_manager: &TableManager) {
        self.torrent_count = table_manager.torrents_displaying_no;
        self.torrent_currently_selected = u16::try_from(table_manager.table.get_len()).unwrap() + 1;
    }
}
impl Component for BottomStats {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(stats) = &self.stats {
            let download = bytes_to_human_format(stats.download_speed);
            let upload = bytes_to_human_format(stats.upload_speed);

            let mut text = format!(" {download} |  {upload}");

            if let Some(free_space) = &self.free_space {
                let free_space = bytes_to_human_format(free_space.size_bytes);
                text = format!("󰋊 {free_space} | {text}")
            }

            if self.torrent_count > 0 {
                text = format!(
                    " {}/{} | {text}",
                    self.torrent_currently_selected, self.torrent_count
                );
            } else {
                // dont display index if nothing is selected
                text = format!(" {} | {text}", self.torrent_count);
            }

            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, rect);
        }
    }
}
