use ratatui::{
    style::{Style, Stylize},
    widgets::Row,
};
use transmission_rpc::types::{Id, Torrent, TorrentStatus};

use crate::ui::bytes_to_human_format;

#[derive(Clone)]
pub struct RustmissionTorrent {
    pub torrent_name: String,
    pub size_when_done: String,
    pub progress: String,
    pub eta_secs: String,
    pub download_speed: String,
    pub upload_speed: String,
    pub status: TorrentStatus,
    pub style: Style,
    pub id: Id,
}

impl RustmissionTorrent {
    pub fn to_row(&self) -> ratatui::widgets::Row {
        Row::new([
            self.torrent_name.as_str(),
            self.size_when_done.as_str(),
            self.progress.as_str(),
            self.eta_secs.as_str(),
            self.download_speed.as_str(),
            self.upload_speed.as_str(),
        ])
        .style(self.style)
    }
}

impl From<&Torrent> for RustmissionTorrent {
    fn from(t: &Torrent) -> Self {
        let id = t.id().unwrap();

        let torrent_name = t.name.clone().unwrap();

        let size_when_done = bytes_to_human_format(t.size_when_done.expect("field requested"));

        let progress = match t.percent_done.expect("field requested") {
            done if done == 1f32 => String::default(),
            percent => format!("{:.2}%", percent * 100f32),
        };

        let eta_secs = match t.eta.expect("field requested") {
            -2 => "∞".to_string(),
            -1 => String::default(),
            eta_secs => eta_secs.to_string(),
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

        RustmissionTorrent {
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
