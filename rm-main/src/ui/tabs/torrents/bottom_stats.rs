use std::{
    borrow::Borrow,
    sync::{Arc, Mutex},
};

use ratatui::{
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Frame,
};
use transmission_rpc::types::{FreeSpace, SessionStats};

use crate::{ui::components::Component, utils::bytes_to_human_format};

use super::table_manager::TableManager;

pub(super) struct BottomStats {
    // TODO: get rid of the Option
    pub(super) stats: Arc<Mutex<Option<SessionStats>>>,
    pub(super) free_space: Arc<Mutex<Option<FreeSpace>>>,
    pub(super) table_manager: Arc<Mutex<TableManager>>,
}

impl BottomStats {
    pub fn new(
        stats: Arc<Mutex<Option<SessionStats>>>,
        free_space: Arc<Mutex<Option<FreeSpace>>>,
        table_manager: Arc<Mutex<TableManager>>,
    ) -> Self {
        Self {
            stats,
            free_space,
            table_manager,
        }
    }
}
impl Component for BottomStats {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(stats) = &*self.stats.lock().unwrap() {
            let download = bytes_to_human_format(stats.download_speed);
            let upload = bytes_to_human_format(stats.upload_speed);

            let mut text = format!(" {download} |  {upload}");

            if let Some(free_space) = &*self.free_space.lock().unwrap() {
                let free_space = bytes_to_human_format(free_space.size_bytes);
                text = format!("󰋊 {free_space} | {text}")
            }

            let table_manager = &*self.table_manager.lock().unwrap();
            let table = table_manager.table.borrow();
            let all = table.get_len();

            if let Some(current) = table.state.borrow().selected() {
                if all > 0 {
                    let current_idx = current + 1;
                    text = format!(" {current_idx}/{all} | {text}");
                } else {
                    // dont display index if no items in table
                    text = format!(" {all} | {text}");
                }
            } else {
                text = format!(" {all} | {text}");
            }

            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, rect);
        }
    }
}
