use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{prelude::*, widgets::Row};
use rm_shared::header::Header;
use std::collections::HashMap;

use crate::{app, ui::components::table::GenericTable};

use super::rustmission_torrent::RustmissionTorrent;

pub struct TableManager {
    ctx: app::Ctx,
    pub table: GenericTable<RustmissionTorrent>,
    pub widths: Vec<Constraint>,
    pub filter: Option<String>,
    pub torrents_displaying_no: u16,
    headers: Vec<&'static str>,
}

impl TableManager {
    pub fn new(ctx: app::Ctx) -> Self {
        let table = GenericTable::new(vec![]);
        let widths = Self::default_widths(&ctx.config.torrents_tab.headers);
        let mut headers = vec![];
        for header in &ctx.config.torrents_tab.headers {
            headers.push(header.header_name());
        }

        Self {
            ctx,
            table,
            widths,
            filter: None,
            torrents_displaying_no: 0,
            headers,
        }
    }

    pub fn rows(&self) -> Vec<Row<'_>> {
        if let Some(filter) = &self.filter {
            let rows = self.filtered_torrents_rows(&self.table.items, filter);
            self.table.overwrite_len(rows.len());
            rows
        } else {
            self.table
                .items
                .iter()
                .map(|t| t.to_row(&self.ctx.config.torrents_tab.headers))
                .collect()
        }
    }

    pub const fn headers(&self) -> &Vec<&'static str> {
        &self.headers
    }

    pub fn current_torrent(&mut self) -> Option<&mut RustmissionTorrent> {
        let matcher = SkimMatcherV2::default();
        let index = self.table.state.borrow().selected()?;

        if let Some(filter) = &self.filter {
            let mut loop_index = 0;
            for rustmission_torrent in &mut self.table.items {
                if matcher
                    .fuzzy_match(&rustmission_torrent.torrent_name, filter)
                    .is_some()
                {
                    if index == loop_index {
                        return Some(rustmission_torrent);
                    }
                    loop_index += 1;
                }
            }
            return None;
        }
        self.table.items.get_mut(index)
    }

    pub fn set_new_rows(&mut self, rows: Vec<RustmissionTorrent>) {
        self.table.items = rows;
        self.widths = self.header_widths(&self.table.items);
    }

    fn filtered_torrents_rows<'a>(
        &self,
        torrents: &'a [RustmissionTorrent],
        filter: &str,
    ) -> Vec<Row<'a>> {
        let matcher = SkimMatcherV2::default();
        let mut rows = vec![];

        let highlight_style = Style::default().fg(self.ctx.config.general.accent_color);

        for torrent in torrents {
            if let Some((_, indices)) = matcher.fuzzy_indices(&torrent.torrent_name, filter) {
                rows.push(torrent.to_row_with_higlighted_indices(
                    indices,
                    highlight_style,
                    &self.ctx.config.torrents_tab.headers,
                ))
            }
        }

        rows
    }

    fn default_widths(headers: &Vec<Header>) -> Vec<Constraint> {
        let mut constraints = vec![];

        for header in headers {
            constraints.push(header.default_constraint())
        }
        constraints
    }

    fn header_widths(&self, rows: &[RustmissionTorrent]) -> Vec<Constraint> {
        let headers = &self.ctx.config.torrents_tab.headers;

        if !self.ctx.config.general.auto_hide {
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
