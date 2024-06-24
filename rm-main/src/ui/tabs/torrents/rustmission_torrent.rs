use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Row,
};
use transmission_rpc::types::{Id, Torrent, TorrentStatus};

use crate::utils::{bytes_to_human_format, seconds_to_human_format};

#[derive(Clone)]
pub struct RustmissionTorrent {
    pub torrent_name: String,
    pub size_when_done: String,
    pub progress: String,
    pub eta_secs: String,
    pub download_speed: String,
    pub upload_speed: String,
    status: TorrentStatus,
    pub style: Style,
    pub id: Id,
}

impl RustmissionTorrent {
    pub fn to_row(&self) -> ratatui::widgets::Row {
        Row::new([
            self.status_icon(),
            self.torrent_name.as_str(),
            self.size_when_done.as_str(),
            self.progress.as_str(),
            self.eta_secs.as_str(),
            self.download_speed.as_str(),
            self.upload_speed.as_str(),
        ])
        .style(self.style)
    }

    pub fn to_row_with_higlighted_indices(
        &self,
        highlighted_indices: Vec<usize>,
        highlight_style: Style,
    ) -> ratatui::widgets::Row {
        let mut torrent_name_line = Line::default();

        for (index, char) in self.torrent_name.char_indices() {
            if highlighted_indices.contains(&index) {
                torrent_name_line.push_span(Span::styled(char.to_string(), highlight_style));
            } else {
                torrent_name_line.push_span(Span::styled(char.to_string(), self.style))
            }
        }

        Row::new([
            Line::from(self.status_icon()),
            torrent_name_line,
            Line::from(self.size_when_done.as_str()),
            Line::from(self.progress.as_str()),
            Line::from(self.eta_secs.as_str()),
            Line::from(self.download_speed.as_str()),
            Line::from(self.upload_speed.as_str()),
        ])
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

    fn status_icon(&self) -> &str {
        // ▼
        match self.status() {
            TorrentStatus::Stopped => "⏸",
            TorrentStatus::Verifying => "↺",
            TorrentStatus::Seeding => "▲",
            TorrentStatus::Downloading => "▼",
            TorrentStatus::QueuedToSeed => "",
            TorrentStatus::QueuedToVerify => "",
            TorrentStatus::QueuedToDownload => "",
        }
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
        }
    }
}
