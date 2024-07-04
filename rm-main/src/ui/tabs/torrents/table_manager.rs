use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{prelude::*, widgets::Row};
use std::sync::{Arc, Mutex};
use transmission_rpc::types::TorrentGetField;

use crate::{app, ui::components::table::GenericTable};

use super::rustmission_torrent::RustmissionTorrent;

pub struct TableManager {
    ctx: app::Ctx,
    pub table: GenericTable<RustmissionTorrent>,
    pub widths: Vec<Constraint>,
    pub filter: Arc<Mutex<Option<String>>>,
    pub torrents_displaying_no: u16,
    headers: Vec<String>,
}

impl TableManager {
    pub fn new(ctx: app::Ctx, table: GenericTable<RustmissionTorrent>) -> Self {
        let widths = Self::default_widths(&ctx.config.torrents_tab.headers);
        let mut headers = vec![];
        for header in &ctx.config.torrents_tab.headers {
            match header {
                TorrentGetField::ActivityDate => todo!(),
                TorrentGetField::AddedDate => todo!(),
                TorrentGetField::DoneDate => todo!(),
                TorrentGetField::DownloadDir => headers.push("Directory".to_owned()),
                TorrentGetField::EditDate => todo!(),
                TorrentGetField::Error => todo!(),
                TorrentGetField::ErrorString => todo!(),
                TorrentGetField::Eta => todo!(),
                TorrentGetField::FileCount => todo!(),
                TorrentGetField::FileStats => todo!(),
                TorrentGetField::Files => todo!(),
                TorrentGetField::HashString => todo!(),
                TorrentGetField::Id => todo!(),
                TorrentGetField::IsFinished => todo!(),
                TorrentGetField::IsPrivate => todo!(),
                TorrentGetField::IsStalled => todo!(),
                TorrentGetField::Labels => todo!(),
                TorrentGetField::LeftUntilDone => todo!(),
                TorrentGetField::MetadataPercentComplete => todo!(),
                TorrentGetField::Name => headers.push("Name".to_owned()),
                TorrentGetField::PeersConnected => todo!(),
                TorrentGetField::PeersGettingFromUs => todo!(),
                TorrentGetField::PeersSendingToUs => todo!(),
                TorrentGetField::PercentDone => todo!(),
                TorrentGetField::Priorities => todo!(),
                TorrentGetField::QueuePosition => todo!(),
                TorrentGetField::RateDownload => todo!(),
                TorrentGetField::RateUpload => todo!(),
                TorrentGetField::RecheckProgress => todo!(),
                TorrentGetField::SecondsSeeding => todo!(),
                TorrentGetField::SeedRatioLimit => todo!(),
                TorrentGetField::SeedRatioMode => todo!(),
                TorrentGetField::SizeWhenDone => todo!(),
                TorrentGetField::Status => todo!(),
                TorrentGetField::TorrentFile => todo!(),
                TorrentGetField::TotalSize => todo!(),
                TorrentGetField::Trackers => todo!(),
                TorrentGetField::UploadRatio => todo!(),
                TorrentGetField::UploadedEver => todo!(),
                TorrentGetField::Wanted => todo!(),
                TorrentGetField::WebseedsSendingToUs => todo!(),
            }
        }

        Self {
            ctx,
            table,
            widths,
            filter: Arc::new(Mutex::new(None)),
            torrents_displaying_no: 0,
            headers,
        }
    }

    pub fn rows(&self) -> Vec<Row<'_>> {
        if let Some(filter) = &*self.filter.lock().unwrap() {
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

    pub const fn header(&self) -> &Vec<String> {
        &self.headers
    }

    pub fn current_torrent(&mut self) -> Option<&mut RustmissionTorrent> {
        let matcher = SkimMatcherV2::default();
        let index = self.table.state.borrow().selected()?;

        if let Some(filter) = &*self.filter.lock().unwrap() {
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
                rows.push(torrent.to_row_with_higlighted_indices(indices, highlight_style))
            }
        }

        rows
    }

    fn default_widths(headers: &Vec<TorrentGetField>) -> Vec<Constraint> {
        let mut constraints = vec![];

        for header in headers {
            match header {
                TorrentGetField::ActivityDate => todo!(),
                TorrentGetField::AddedDate => todo!(),
                TorrentGetField::DoneDate => todo!(),
                TorrentGetField::DownloadDir => constraints.push(Constraint::Max(70)),
                TorrentGetField::EditDate => todo!(),
                TorrentGetField::Error => todo!(),
                TorrentGetField::ErrorString => todo!(),
                TorrentGetField::Eta => todo!(),
                TorrentGetField::FileCount => todo!(),
                TorrentGetField::FileStats => todo!(),
                TorrentGetField::Files => todo!(),
                TorrentGetField::HashString => todo!(),
                TorrentGetField::Id => todo!(),
                TorrentGetField::IsFinished => todo!(),
                TorrentGetField::IsPrivate => todo!(),
                TorrentGetField::IsStalled => todo!(),
                TorrentGetField::Labels => todo!(),
                TorrentGetField::LeftUntilDone => todo!(),
                TorrentGetField::MetadataPercentComplete => todo!(),
                TorrentGetField::Name => constraints.push(Constraint::Max(70)),
                TorrentGetField::PeersConnected => todo!(),
                TorrentGetField::PeersGettingFromUs => todo!(),
                TorrentGetField::PeersSendingToUs => todo!(),
                TorrentGetField::PercentDone => todo!(),
                TorrentGetField::Priorities => todo!(),
                TorrentGetField::QueuePosition => todo!(),
                TorrentGetField::RateDownload => todo!(),
                TorrentGetField::RateUpload => todo!(),
                TorrentGetField::RecheckProgress => todo!(),
                TorrentGetField::SecondsSeeding => todo!(),
                TorrentGetField::SeedRatioLimit => todo!(),
                TorrentGetField::SeedRatioMode => todo!(),
                TorrentGetField::SizeWhenDone => todo!(),
                TorrentGetField::Status => todo!(),
                TorrentGetField::TorrentFile => todo!(),
                TorrentGetField::TotalSize => todo!(),
                TorrentGetField::Trackers => todo!(),
                TorrentGetField::UploadRatio => todo!(),
                TorrentGetField::UploadedEver => todo!(),
                TorrentGetField::Wanted => todo!(),
                TorrentGetField::WebseedsSendingToUs => todo!(),
            }
        }
        constraints
    }

    fn header_widths(&self, rows: &[RustmissionTorrent]) -> Vec<Constraint> {
        if !self.ctx.config.general.auto_hide {
            return Self::default_widths(&self.ctx.config.torrents_tab.headers);
        }

        let mut constraints = Self::default_widths(&self.ctx.config.torrents_tab.headers);
        constraints

        // let mut download_width = 0;
        // let mut upload_width = 0;
        // let mut progress_width = 0;
        // let mut eta_width = 0;

        // for row in rows {
        //     if !row.download_speed.is_empty() {
        //         download_width = 11;
        //     }
        //     if !row.upload_speed.is_empty() {
        //         upload_width = 11;
        //     }
        //     if !row.progress.is_empty() {
        //         progress_width = 11;
        //     }

        //     if !row.eta_secs.is_empty() {
        //         eta_width = 11;
        //     }
        // }

        // [
        //     Constraint::Max(70),                // Name
        //     Constraint::Length(5),              // <padding>
        //     Constraint::Length(11),             // Size
        //     Constraint::Length(progress_width), // Progress
        //     Constraint::Length(eta_width),      // ETA
        //     Constraint::Length(download_width), // Download
        //     Constraint::Length(upload_width),   // Upload
        //     Constraint::Max(70),                // Download directory
        // ]
    }
}
