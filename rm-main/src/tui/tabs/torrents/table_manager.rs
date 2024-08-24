use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{prelude::*, widgets::Row};
use rm_config::CONFIG;
use rm_shared::header::Header;
use std::collections::HashMap;

use crate::tui::components::GenericTable;

use super::rustmission_torrent::RustmissionTorrent;

pub struct TableManager {
    pub table: GenericTable<RustmissionTorrent>,
    pub widths: Vec<Constraint>,
    pub filter: Option<Filter>,
    pub torrents_displaying_no: u16,
    headers: Vec<&'static str>,
}

pub struct Filter {
    pub pattern: String,
    indexes: Vec<u16>,
    highlight_indices: Vec<Vec<usize>>,
}

impl TableManager {
    pub fn new() -> Self {
        let table = GenericTable::new(vec![]);
        let widths = Self::default_widths(&CONFIG.torrents_tab.headers);
        let mut headers = vec![];
        for header in &CONFIG.torrents_tab.headers {
            headers.push(header.header_name());
        }

        Self {
            table,
            widths,
            filter: None,
            torrents_displaying_no: 0,
            headers,
        }
    }

    pub fn update_rows_number(&mut self) {
        if let Some(filter) = &self.filter {
            self.table.overwrite_len(filter.indexes.len());
        } else {
            self.table.items.len();
        }
    }

    pub fn rows(&self) -> Vec<Row<'_>> {
        if let Some(filter) = &self.filter {
            let highlight_style = Style::default().fg(CONFIG.general.accent_color);
            let headers = &CONFIG.torrents_tab.headers;
            let mut rows = vec![];
            for (i, which_torrent) in filter.indexes.iter().enumerate() {
                let row = self.table.items[*which_torrent as usize].to_row_with_higlighted_indices(
                    &filter.highlight_indices[i],
                    highlight_style,
                    headers,
                );
                rows.push(row);
            }

            self.table.overwrite_len(rows.len());
            rows
        } else {
            self.table
                .items
                .iter()
                .map(|t| t.to_row(&CONFIG.torrents_tab.headers))
                .collect()
        }
    }

    pub const fn headers(&self) -> &Vec<&'static str> {
        &self.headers
    }

    pub fn current_torrent(&mut self) -> Option<&mut RustmissionTorrent> {
        let selected_idx = self.table.state.borrow().selected()?;

        if let Some(filter) = &self.filter {
            if filter.indexes.is_empty() {
                None
            } else {
                self.table
                    .items
                    .get_mut(filter.indexes[selected_idx] as usize)
            }
        } else {
            self.table.items.get_mut(selected_idx)
        }
    }

    pub fn set_new_rows(&mut self, rows: Vec<RustmissionTorrent>) {
        self.table.set_items(rows);
        self.widths = self.header_widths(&self.table.items);
        self.update_rows_number();
    }

    pub fn set_filter(&mut self, filter: String) {
        let matcher = SkimMatcherV2::default();
        let mut indexes: Vec<u16> = vec![];
        let mut highlight_indices = vec![];
        for (i, torrent) in self.table.items.iter().enumerate() {
            if let Some((_, indices)) = matcher.fuzzy_indices(&torrent.torrent_name, &filter) {
                indexes.push(i as u16);
                highlight_indices.push(indices);
            }
        }

        let filter = Filter {
            pattern: filter,
            indexes,
            highlight_indices,
        };

        self.filter = Some(filter);
    }

    fn default_widths(headers: &Vec<Header>) -> Vec<Constraint> {
        let mut constraints = vec![];

        for header in headers {
            if *header == Header::Category {
                constraints.push(Constraint::Length(u16::from(
                    CONFIG.categories.max_name_len,
                )))
            } else if *header == Header::CategoryIcon {
                constraints.push(Constraint::Length(u16::from(
                    CONFIG.categories.max_icon_len,
                )))
            } else {
                constraints.push(header.default_constraint())
            }
        }
        constraints
    }

    fn header_widths(&self, rows: &[RustmissionTorrent]) -> Vec<Constraint> {
        let headers = &CONFIG.torrents_tab.headers;

        if !CONFIG.general.auto_hide {
            return Self::default_widths(headers);
        }

        let mut map = HashMap::new();

        for header in headers {
            map.insert(header, header.default_constraint());
        }

        let hidable_headers = [
            Header::Progress,
            Header::UploadRate,
            Header::DownloadRate,
            Header::Eta,
        ];

        for hidable_header in &hidable_headers {
            map.entry(hidable_header)
                .and_modify(|c| *c = Constraint::Length(0));
        }

        for row in rows {
            if !row.download_speed.is_empty() {
                map.entry(&Header::DownloadRate)
                    .and_modify(|c| *c = Header::DownloadRate.default_constraint());
            }
            if !row.upload_speed.is_empty() {
                map.entry(&Header::UploadRate)
                    .and_modify(|c| *c = Header::UploadRate.default_constraint());
            }
            if !row.progress.is_empty() {
                map.entry(&Header::Progress)
                    .and_modify(|c| *c = Header::Progress.default_constraint());
            }

            if !row.eta_secs.is_empty() {
                map.entry(&Header::Eta)
                    .and_modify(|c| *c = Header::Eta.default_constraint());
            }
        }

        let mut constraints = vec![];

        for header in headers {
            constraints.push(map.remove(header).expect("this header exists"))
        }

        constraints
    }
}
