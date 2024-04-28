use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::prelude::*;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use crate::{app, ui::components::table::GenericTable};

use super::rustmission_torrent::RustmissionTorrent;

pub struct TableManager {
    ctx: app::Ctx,
    pub table: RefCell<GenericTable<RustmissionTorrent>>,
    pub widths: [Constraint; 6],
    pub filter: Arc<Mutex<Option<String>>>,
    header: Vec<String>,
}

impl TableManager {
    pub fn new(ctx: app::Ctx, table: GenericTable<RustmissionTorrent>) -> Self {
        let widths = Self::default_widths();
        TableManager {
            ctx,
            table: table.into(),
            widths,
            filter: Arc::new(Mutex::new(None)),
            header: vec![
                "Name".to_owned(),
                "Size".to_owned(),
                "Progress".to_owned(),
                "ETA".to_owned(),
                "Download".to_owned(),
                "Upload".to_owned(),
            ],
        }
    }

    pub fn header(&self) -> &Vec<String> {
        &self.header
    }

    fn default_widths() -> [Constraint; 6] {
        [
            Constraint::Max(70),    // Name
            Constraint::Length(10), // Size
            Constraint::Length(10), // Progress
            Constraint::Length(10), // ETA
            Constraint::Length(10), // Download
            Constraint::Length(10), // Upload
        ]
    }

    pub fn current_torrent(&self) -> Option<RustmissionTorrent> {
        let matcher = SkimMatcherV2::default();
        let index = self.table.borrow().state.borrow().selected()?;

        if let Some(filter) = &*self.filter.lock().unwrap() {
            let table_borrow = self.table.borrow();
            let filtered_rows: Vec<_> = table_borrow
                .items
                .iter()
                .filter(|row| matcher.fuzzy_match(&row.torrent_name, filter).is_some())
                .collect();
            return filtered_rows.get(index).cloned().cloned();
        }
        self.table.borrow().items.get(index).cloned()
    }

    pub fn set_new_rows(&mut self, rows: Vec<RustmissionTorrent>) {
        self.table.borrow_mut().items = rows;
        self.widths = self.header_widths(&self.table.borrow().items);
    }

    fn header_widths(&self, rows: &[RustmissionTorrent]) -> [Constraint; 6] {
        if !self.ctx.config.general.auto_hide {
            return Self::default_widths();
        }

        let mut download_width = 0;
        let mut upload_width = 0;
        let mut progress_width = 0;
        let mut eta_width = 0;

        for row in rows {
            if !row.download_speed.is_empty() {
                download_width = 9;
            }
            if !row.upload_speed.is_empty() {
                upload_width = 9;
            }
            if !row.progress.is_empty() {
                progress_width = 9;
            }

            if !row.eta_secs.is_empty() {
                eta_width = 9;
            }
        }

        [
            Constraint::Max(70),                // Name
            Constraint::Length(9),              // Size
            Constraint::Length(progress_width), // Progress
            Constraint::Length(eta_width),      // ETA
            Constraint::Length(download_width), // Download
            Constraint::Length(upload_width),   // Upload
        ]
    }
}
