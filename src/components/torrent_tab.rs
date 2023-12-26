use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Row, Table};
use transmission_rpc::types::{SessionStats, Torrent};

use crate::app::Action;

use super::table::GenericTable;
use super::Component;

pub struct TorrentsTab {
    table: GenericTable<Torrent>,
    stats: Option<SessionStats>,
}

impl TorrentsTab {
    pub fn new() -> Self {
        Self {
            table: GenericTable::new(vec![]),
            stats: None,
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(1)])
            .split(rect);

        let header = Row::new(vec![
            "Name", "Size", "Progress", "ETA", "Download", "Upload",
        ]);
        let widths = [
            Constraint::Length(60), // Name
            Constraint::Length(10), // Size
            Constraint::Length(10), // Progress
            Constraint::Length(10), // ETA
            Constraint::Length(10), // Download
            Constraint::Length(10), // Upload
        ];

        let rows: Vec<_> = self
            .table
            .items
            .iter()
            .map(|t| {
                let progress = match t.percent_done.expect("field requested") {
                    perc if perc == 1f32 => String::default(),
                    perc => format!("{:.2}%", perc * 100f32),
                };

                let eta = match t.eta.expect("field requested") {
                    -2 => "∞".to_string(),
                    -1 => String::default(),
                    eta => eta.to_string(),
                };

                let download = match t.rate_download.expect("field requested") {
                    0 => String::default(),
                    down => bytes_to_human(down),
                };

                let upload = match t.rate_upload.expect("field requested") {
                    0 => String::default(),
                    upload => bytes_to_human(upload),
                };

                Row::new(vec![
                    t.name.clone().unwrap(),
                    bytes_to_human(t.size_when_done.unwrap()),
                    progress,
                    eta,
                    download,
                    upload,
                ])
            })
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .highlight_style(Style::default().on_black());

        if let Some(stats) = &self.stats {
            let upload = bytes_to_human(stats.upload_speed);
            let download = bytes_to_human(stats.download_speed);
            let all = stats.torrent_count;
            let text = format!("All: {all} | ▲ {download} | ⯆ {upload}");
            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, layout[1]);
        }
        f.render_stateful_widget(table, layout[0], &mut self.table.state);
    }

    fn handle_events(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Up => self.table.previous(),
            Action::Down => self.table.next(),
            Action::TorrentListUpdate(torrents) => self.table.update_items(torrents),
            Action::StatsUpdate(stats) => self.stats = Some(stats),
            _ => (),
        };
        None
    }
}

fn bytes_to_human(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let (value, unit) = if bytes < (KB - 25f64) as i64 {
        (bytes as f64, "B")
    } else if bytes < (MB - 25f64) as i64 {
        (bytes as f64 / KB, "KB")
    } else if bytes < (GB - 25f64) as i64 {
        (bytes as f64 / MB, "MB")
    } else if bytes < (TB - 25f64) as i64 {
        (bytes as f64 / GB, "GB")
    } else {
        (bytes as f64 / TB, "TB")
    };

    format!("{value:.1} {unit}")
}
