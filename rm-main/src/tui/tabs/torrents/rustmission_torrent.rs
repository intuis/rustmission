use chrono::{Datelike, NaiveDateTime};
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Cell, Row},
};
use rm_config::CONFIG;
use rm_shared::{
    header::Header,
    utils::{bytes_to_human_format, seconds_to_human_format},
};
use transmission_rpc::types::{ErrorType, Id, Torrent, TorrentStatus};

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
    pub categories: Vec<String>,
    pub error: Option<String>,
}

impl RustmissionTorrent {
    pub fn to_row(&self, headers: &[Header]) -> ratatui::widgets::Row {
        headers
            .iter()
            .map(|header| self.header_to_cell(*header))
            .collect::<Row>()
            .style(self.style)
            .height(if self.error.is_some() { 2 } else { 1 })
    }

    pub fn to_row_with_higlighted_indices(
        &self,
        highlighted_indices: &Vec<usize>,
        highlight_style: Style,
        headers: &Vec<Header>,
    ) -> ratatui::widgets::Row {
        let mut torrent_name_line = Line::default();

        let char_indices: Vec<usize> = self.torrent_name.char_indices().map(|(i, _)| i).collect();
        let mut last_end = 0;
        let mut flush_line = |start: usize, end: usize| {
            let mut start = char_indices[start];
            let mut end = char_indices[end];
            torrent_name_line.push_span(Span::styled(
                &self.torrent_name[last_end..start],
                self.style,
            ));

            while !self.torrent_name.is_char_boundary(start) {
                start -= 1;
            }

            while !self.torrent_name.is_char_boundary(end + 1) {
                end += 1;
            }

            torrent_name_line.push_span(Span::styled(
                &self.torrent_name[start..=end],
                highlight_style,
            ));
            last_end = end + 1;
        };

        let mut first: Option<usize> = None;
        let mut second: Option<usize> = None;
        for indice in highlighted_indices {
            let fst = if let Some(fst) = first {
                fst
            } else {
                first = Some(*indice);
                continue;
            };

            let snd = if let Some(snd) = second {
                snd
            } else {
                if fst + 1 == *indice {
                    second = Some(*indice);
                } else {
                    flush_line(fst, fst);
                    first = Some(*indice);
                }
                continue;
            };

            if snd + 1 == *indice {
                second = Some(*indice);
            } else {
                flush_line(fst, snd);
                first = Some(*indice);
                second = None;
            }
        }

        if let (Some(first), None) = (first, second) {
            flush_line(first, first);
        } else if let (Some(first), Some(second)) = (first, second) {
            flush_line(first, second);
        }

        torrent_name_line.push_span(Span::styled(&self.torrent_name[last_end..], self.style));

        let mut cells = vec![];

        for header in headers {
            if *header == Header::Name {
                cells.push(std::mem::take(&mut torrent_name_line).into())
            } else {
                cells.push(self.header_to_cell(*header).style(self.style))
            }
        }

        Row::new(cells)
    }

    pub fn torrent_location(&self) -> String {
        format!("{}/{}", self.download_dir, self.torrent_name)
    }

    fn header_to_cell(&self, header: Header) -> Cell {
        match header {
            Header::Name => {
                if let Some(error) = &self.error {
                    Cell::from(format!("{}\n{error}", self.torrent_name))
                } else {
                    Cell::from(self.torrent_name.as_str())
                }
            }
            Header::SizeWhenDone => Cell::from(self.size_when_done.as_str()),
            Header::Progress => Cell::from(self.progress.as_str()),
            Header::Eta => Cell::from(self.eta_secs.as_str()),
            Header::DownloadRate => Cell::from(download_speed_format(&self.download_speed)),
            Header::UploadRate => Cell::from(upload_speed_format(&self.upload_speed)),
            Header::DownloadDir => Cell::from(self.download_dir.as_str()),
            Header::Padding => Cell::from(""),
            Header::Id => match &self.id {
                Id::Id(id) => Cell::from(id.to_string()),
                Id::Hash(hash) => Cell::from(hash.as_str()),
            },
            Header::UploadRatio => Cell::from(self.upload_ratio.as_str()),
            Header::UploadedEver => Cell::from(self.uploaded_ever.as_str()),
            Header::ActivityDate => time_to_line(self.activity_date).into(),
            Header::AddedDate => time_to_line(self.added_date).into(),
            Header::PeersConnected => Cell::from(self.peers_connected.to_string()),
            Header::SmallStatus => {
                if self.error.is_some() {
                    return Cell::from(CONFIG.icons.failure.as_str());
                }

                match self.status() {
                    TorrentStatus::Stopped => Cell::from(CONFIG.icons.pause.as_str()),
                    TorrentStatus::QueuedToVerify => Cell::from(CONFIG.icons.loading.as_str()),
                    TorrentStatus::Verifying => Cell::from(CONFIG.icons.verifying.as_str()),
                    TorrentStatus::QueuedToDownload => Cell::from(CONFIG.icons.loading.as_str()),
                    TorrentStatus::QueuedToSeed => Cell::from(CONFIG.icons.loading.as_str()),
                    TorrentStatus::Downloading => Cell::from(CONFIG.icons.download.as_str()),
                    TorrentStatus::Seeding => {
                        if !self.upload_speed.is_empty() {
                            Cell::from(CONFIG.icons.upload.as_str())
                        } else {
                            Cell::from(CONFIG.icons.success.as_str())
                        }
                    }
                }
            }
            Header::Category => match self.categories.first() {
                Some(category) => {
                    if let Some(config_category) = CONFIG.categories.get(category) {
                        Cell::from(category.as_str()).fg(config_category.color)
                    } else {
                        Cell::from(category.as_str())
                    }
                }
                None => Cell::default(),
            },
            Header::CategoryIcon => match self.categories.first() {
                Some(category) => {
                    if let Some(config_category) = CONFIG.categories.get(category) {
                        Cell::from(config_category.icon.as_str()).fg(config_category.color)
                    } else {
                        Cell::default()
                    }
                }
                None => Cell::default(),
            },
        }
    }

    pub const fn status(&self) -> TorrentStatus {
        self.status
    }

    pub fn update_status(&mut self, new_status: TorrentStatus) {
        if self.error.is_some() {
            self.style = Style::default().red().italic();
        } else if new_status == TorrentStatus::Stopped {
            self.style = Style::default().dark_gray().italic();
        } else {
            self.style = Style::default();
        }

        self.status = new_status;
    }
}

impl From<Torrent> for RustmissionTorrent {
    fn from(t: Torrent) -> Self {
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

        let error = {
            if t.error.expect("field requested") != ErrorType::Ok {
                Some(t.error_string.expect("field requested"))
            } else {
                None
            }
        };

        let style = {
            if error.is_some() {
                Style::default().red().italic()
            } else {
                match status {
                    TorrentStatus::Stopped => Style::default().dark_gray().italic(),
                    _ => Style::default(),
                }
            }
        };

        let categories = t.labels.unwrap();

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
            categories,
            error,
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

fn download_speed_format(download_speed: &str) -> String {
    if !download_speed.is_empty() {
        return format!("{} {}", CONFIG.icons.download, download_speed);
    }
    download_speed.to_string()
}

fn upload_speed_format(upload_speed: &str) -> String {
    if !upload_speed.is_empty() {
        return format!("{} {}", CONFIG.icons.upload, upload_speed);
    }
    upload_speed.to_string()
}
