use chrono::{Datelike, NaiveDateTime};
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Row,
};
use rm_config::main_config::Header;
use transmission_rpc::types::{Id, Torrent, TorrentStatus};

use crate::utils::{
    bytes_to_human_format, download_speed_format, seconds_to_human_format, upload_speed_format,
};

#[derive(Clone)]
pub struct RustmissionTorrent {
    pub torrent_name: String,
    pub size_when_done: String,
    pub progress: String,
    pub eta_secs: String,
    pub download_speed: String,
    pub upload_speed: String,
    pub uploaded_ever: String,
    pub upload_ratio: String,
    status: TorrentStatus,
    pub style: Style,
    pub id: Id,
    pub download_dir: String,
    pub activity_date: NaiveDateTime,
    pub added_date: NaiveDateTime,
    pub peers_connected: i64,
}

impl RustmissionTorrent {
    pub fn to_row(&self, headers: &Vec<Header>) -> ratatui::widgets::Row {
        headers
            .iter()
            .map(|header| self.header_to_line(*header))
            .collect::<Row>()
            .style(self.style)
    }

    pub fn to_row_with_higlighted_indices(
        &self,
        highlighted_indices: Vec<usize>,
        highlight_style: Style,
        headers: &Vec<Header>,
    ) -> ratatui::widgets::Row {
        let mut torrent_name_line = Line::default();

        for (index, char) in self.torrent_name.char_indices() {
            if highlighted_indices.contains(&index) {
                torrent_name_line.push_span(Span::styled(char.to_string(), highlight_style));
            } else {
                torrent_name_line.push_span(Span::styled(char.to_string(), self.style))
            }
        }

        let mut cells = vec![];

        for header in headers {
            if *header == Header::Name {
                cells.push(Line::from(torrent_name_line.clone()))
            } else {
                cells.push(self.header_to_line(*header))
            }
        }

        Row::new(cells)
    }

    fn header_to_line(&self, header: Header) -> Line {
        match header {
            Header::Name => Line::from(self.torrent_name.as_str()),
            Header::SizeWhenDone => Line::from(self.size_when_done.as_str()),
            Header::Progress => Line::from(self.progress.as_str()),
            Header::Eta => Line::from(self.eta_secs.as_str()),
            Header::DownloadRate => Line::from(download_speed_format(&self.download_speed)),
            Header::UploadRate => Line::from(upload_speed_format(&self.upload_speed)),
            Header::DownloadDir => Line::from(self.download_dir.as_str()),
            Header::Padding => Line::raw(""),
            Header::Id => match &self.id {
                Id::Id(id) => Line::from(id.to_string()),
                Id::Hash(hash) => Line::from(hash.as_str()),
            },
            Header::UploadRatio => Line::from(self.upload_ratio.as_str()),
            Header::UploadedEver => Line::from(self.uploaded_ever.as_str()),
            Header::ActivityDate => time_to_line(self.activity_date),
            Header::AddedDate => time_to_line(self.added_date),
            Header::PeersConnected => Line::from(self.peers_connected.to_string()),
            Header::SmallStatus => match self.status() {
                TorrentStatus::Stopped => Line::from("󰏤"),
                TorrentStatus::QueuedToVerify => Line::from("󱥸"),
                TorrentStatus::Verifying => Line::from("󰑓"),
                TorrentStatus::QueuedToDownload => Line::from("󱥸"),
                TorrentStatus::QueuedToSeed => Line::from("󱥸"),
                TorrentStatus::Seeding => {
                    if !self.upload_speed.is_empty() {
                        Line::from("")
                    } else {
                        Line::from("󰄬")
                    }
                }
                TorrentStatus::Downloading => Line::from(""),
            },
        }
    }

    pub const fn status(&self) -> TorrentStatus {
        self.status
    }

    pub fn update_status(&mut self, new_status: TorrentStatus) {
        if new_status == TorrentStatus::Stopped {
            self.style = Style::default().dark_gray().italic();
        } else {
            self.style = Style::default();
        }

        self.status = new_status;
    }
}

impl From<&Torrent> for RustmissionTorrent {
    fn from(t: &Torrent) -> Self {
        let id = t.id().expect("id requested");

        let torrent_name = t.name.clone().expect("name requested");

        let size_when_done = bytes_to_human_format(t.size_when_done.expect("field requested"));

        let progress = match t.percent_done.expect("field requested") {
            done if done == 1f32 => String::default(),
            percent => format!("{:.2}%", percent * 100f32),
        };

        let eta_secs = match t.eta.expect("field requested") {
            -2 => "∞".to_string(),
            -1 => String::default(),
            eta_secs => seconds_to_human_format(eta_secs),
        };

        let download_speed = match t.rate_download.expect("field requested") {
            0 => String::default(),
            down => bytes_to_human_format(down),
        };

        let upload_speed = match t.rate_upload.expect("field requested") {
            0 => String::default(),
            upload => bytes_to_human_format(upload),
        };

        let status = t.status.expect("field requested");

        let style = match status {
            TorrentStatus::Stopped => Style::default().dark_gray().italic(),
            _ => Style::default(),
        };

        let download_dir = t.download_dir.clone().expect("field requested");

        let uploaded_ever = bytes_to_human_format(t.uploaded_ever.expect("field requested"));

        let upload_ratio = {
            let raw = t.upload_ratio.expect("field requested");
            format!("{:.1}", raw)
        };

        let activity_date = {
            let raw = t.activity_date.expect("field requested");
            chrono::DateTime::from_timestamp(raw, 0)
                .unwrap()
                .naive_local()
        };

        let added_date = {
            let raw = t.added_date.expect("field requested");
            chrono::DateTime::from_timestamp(raw, 0)
                .unwrap()
                .naive_local()
        };

        let peers_connected = t.peers_connected.expect("field requested");

        Self {
            torrent_name,
            size_when_done,
            progress,
            eta_secs,
            download_speed,
            upload_speed,
            status,
            style,
            id,
            download_dir,
            uploaded_ever,
            upload_ratio,
            activity_date,
            added_date,
            peers_connected,
        }
    }
}

fn time_to_line<'a>(time: NaiveDateTime) -> Line<'a> {
    let today = chrono::Local::now();
    if time.year() == today.year() && time.month() == today.month() && time.day() == today.day() {
        Line::from(time.format("Today %H:%M").to_string())
    } else {
        Line::from(time.format("%y|%m|%d %H:%M").to_string())
    }
}
