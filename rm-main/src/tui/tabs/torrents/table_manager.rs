use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{prelude::*, widgets::Row};
use rm_config::CONFIG;
use rm_shared::header::Header;
use std::{cmp::Ordering, collections::HashMap};
use transmission_rpc::types::Id;

use crate::tui::components::GenericTable;

use super::rustmission_torrent::RustmissionTorrent;

pub struct TableManager {
    pub table: GenericTable<RustmissionTorrent>,
    pub widths: Vec<Constraint>,
    pub filter: Option<Filter>,
    pub torrents_displaying_no: u16,
    pub sort_header: Option<usize>,
    pub sort_reverse: bool,
    pub sorting_is_being_selected: bool,
    pub selected_torrents_ids: Vec<i64>,
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

        Self {
            table,
            widths,
            filter: None,
            torrents_displaying_no: 0,
            sort_header: None,
            sort_reverse: false,
            sorting_is_being_selected: false,
            selected_torrents_ids: vec![],
        }
    }

    pub fn enter_sorting_selection(&mut self) {
        self.sorting_is_being_selected = true;
        if self.sort_header.is_none() {
            self.sort_header = Some(0);
            self.sort();
        }
    }

    pub fn reverse_sort(&mut self) {
        self.sort_reverse = !self.sort_reverse;
        self.sort();
    }

    pub fn leave_sorting(&mut self) {
        self.sorting_is_being_selected = false;
        self.sort_header = None;
        self.sort();
    }

    pub fn apply_sort(&mut self) {
        self.sorting_is_being_selected = false;
    }

    pub fn move_to_column_left(&mut self) {
        if let Some(selected) = self.sort_header {
            if selected == 0 {
                self.sort_header = Some(self.headers().len() - 1);
                self.sort();
            } else {
                self.sort_header = Some(selected - 1);
                self.sort();
            }
        } else {
            self.sort_header = Some(0);
            self.sort();
        }
    }

    pub fn move_to_column_right(&mut self) {
        if let Some(selected) = self.sort_header {
            let headers_count = self.headers().len();
            if selected < headers_count.saturating_sub(1) {
                self.sort_header = Some(selected + 1);
                self.sort();
            } else {
                self.sort_header = Some(0);
                self.sort();
            }
        }
    }

    pub fn sort(&mut self) {
        let sort_by = self
            .sort_header
            .map(|idx| CONFIG.torrents_tab.headers[idx])
            .unwrap_or(CONFIG.torrents_tab.default_sort);

        match sort_by {
            Header::Id => todo!(),
            Header::Name => self.table.items.sort_by(|x, y| {
                x.torrent_name
                    .to_lowercase()
                    .cmp(&y.torrent_name.to_lowercase())
            }),
            Header::SizeWhenDone => self
                .table
                .items
                .sort_by(|x, y| x.size_when_done.cmp(&y.size_when_done)),
            Header::Progress => self.table.items.sort_unstable_by(|x, y| {
                x.progress
                    .partial_cmp(&y.progress)
                    .unwrap_or(Ordering::Equal)
            }),
            Header::Eta => self.table.items.sort_by(|x, y| x.eta_secs.cmp(&y.eta_secs)),
            Header::DownloadRate => self
                .table
                .items
                .sort_by(|x, y| x.download_speed.cmp(&y.download_speed)),
            Header::UploadRate => self
                .table
                .items
                .sort_by(|x, y| x.upload_speed.cmp(&y.upload_speed)),
            Header::DownloadDir => self
                .table
                .items
                .sort_by(|x, y| x.download_dir.cmp(&y.download_dir)),
            Header::Padding => (),
            Header::UploadRatio => self
                .table
                .items
                .sort_by(|x, y| x.upload_ratio.cmp(&y.upload_ratio)),
            Header::UploadedEver => self
                .table
                .items
                .sort_by(|x, y| x.uploaded_ever.cmp(&y.uploaded_ever)),
            Header::ActivityDate => self
                .table
                .items
                .sort_by(|x, y| x.activity_date.cmp(&y.activity_date)),
            Header::AddedDate => self
                .table
                .items
                .sort_by(|x, y| x.added_date.cmp(&y.added_date)),
            Header::PeersConnected => self
                .table
                .items
                .sort_by(|x, y| x.peers_connected.cmp(&y.peers_connected)),
            Header::SmallStatus => (),
            Header::Category => self.table.items.sort_by(|x, y| {
                x.category
                    .as_ref()
                    .map(|cat| {
                        cat.name().cmp(
                            y.category
                                .as_ref()
                                .map(|cat| cat.name())
                                .unwrap_or_default(),
                        )
                    })
                    .unwrap_or(Ordering::Less)
            }),
            Header::CategoryIcon => (),
        }
        if self.sort_reverse
            || (self.sort_header.is_none() && CONFIG.torrents_tab.default_sort_reverse)
        {
            self.table.items.reverse();
        }
    }

    pub fn update_rows_number(&mut self) {
        if let Some(filter) = &self.filter {
            self.table.overwrite_len(filter.indexes.len());
        } else {
            self.table.items.len();
        }
    }

    pub fn select_current_torrent(&mut self) {
        let mut is_selected = true;
        if let Some(t) = self.current_torrent() {
            if let Id::Id(id) = t.id {
                match self.selected_torrents_ids.iter().position(|&x| x == id) {
                    Some(idx) => {
                        self.selected_torrents_ids.remove(idx);
                        is_selected = false;
                    }
                    None => {
                        self.selected_torrents_ids.push(id);
                    }
                }
            } else {
                unreachable!();
            }
        }

        if let Some(t) = self.current_torrent() {
            t.is_selected = is_selected;
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

    pub fn headers(&self) -> &Vec<Header> {
        &CONFIG.torrents_tab.headers
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

    pub fn set_new_rows(&mut self, mut rows: Vec<RustmissionTorrent>) {
        if !self.selected_torrents_ids.is_empty() {
            for row in &mut rows {
                if let Id::Id(id) = row.id {
                    if self.selected_torrents_ids.contains(&id) {
                        row.is_selected = true;
                    }
                }
            }
        }

        self.table.set_items(rows);
        self.widths = self.header_widths(&self.table.items);
        self.update_rows_number();
        self.sort();
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
            if !row.download_speed().is_empty() {
                map.entry(&Header::DownloadRate)
                    .and_modify(|c| *c = Header::DownloadRate.default_constraint());
            }
            if !row.upload_speed.is_empty() {
                map.entry(&Header::UploadRate)
                    .and_modify(|c| *c = Header::UploadRate.default_constraint());
            }
            if !row.progress().is_empty() {
                map.entry(&Header::Progress)
                    .and_modify(|c| *c = Header::Progress.default_constraint());
            }

            if !row.eta_secs().is_empty() {
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
