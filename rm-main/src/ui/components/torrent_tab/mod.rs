mod task;

use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use tokio::sync::mpsc::UnboundedSender;
use transmission_rpc::types::{SessionStats, Torrent};

use crate::action::Action;

use self::task::Task;

use super::table::GenericTable;
use super::Component;

#[derive(Default)]
struct StatsComponent {
    stats: Option<SessionStats>,
}

impl StatsComponent {
    fn set_stats(&mut self, stats: SessionStats) {
        self.stats = Some(stats);
    }
}

impl Component for StatsComponent {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(stats) = &self.stats {
            let upload = bytes_to_human_format(stats.upload_speed);
            let download = bytes_to_human_format(stats.download_speed);
            let all = stats.torrent_count;
            let text = format!("All: {all} | ▲ {download} | ⯆ {upload}");
            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, rect);
        }
    }
}

pub struct TorrentsTab {
    table: GenericTable<Torrent>,
    rows: Vec<[String; 6]>,
    stats: StatsComponent,
    task: Task,
}

impl TorrentsTab {
    pub fn new(trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            table: GenericTable::new(vec![]),
            rows: vec![],
            stats: StatsComponent::default(),
            task: Task::new(trans_tx),
        }
    }

    fn torrent_to_row(t: &Torrent) -> [String; 6] {
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

        [
            torrent_name,
            size_when_done,
            progress,
            eta_secs,
            download_speed,
            upload_speed,
        ]
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [torrents_list_rect, stats_rect] =
            Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(rect);

        let header = Row::new(vec![
            "Name", "Size", "Progress", "ETA", "Download", "Upload",
        ]);

        let header_widths = [
            Constraint::Length(60), // Name
            Constraint::Length(10), // Size
            Constraint::Length(10), // Progress
            Constraint::Length(10), // ETA
            Constraint::Length(10), // Download
            Constraint::Length(10), // Upload
        ];

        let torrent_rows = self
            .rows
            .iter()
            .map(|i| i.iter().map(|i| i.as_str()))
            .map(Row::new);

        let torrents_table = Table::new(torrent_rows, header_widths)
            .header(header)
            .highlight_style(Style::default().light_magenta().on_black().bold());

        f.render_stateful_widget(torrents_table, torrents_list_rect, &mut self.table.state);

        self.stats.render(f, stats_rect);

        self.task.render(f, stats_rect);
    }

    #[must_use]
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match action {
            A::Up => {
                self.table.previous();
                Some(Action::Render)
            }
            A::Down => {
                self.table.next();
                Some(Action::Render)
            }
            A::TorrentListUpdate(torrents) => {
                self.table.set_items(*torrents);
                self.rows = self.table.items.iter().map(Self::torrent_to_row).collect();
                Some(Action::Render)
            }
            A::StatsUpdate(stats) => {
                self.stats.set_stats(*stats);
                Some(Action::Render)
            }
            other => self.task.handle_events(other),
        }
    }
}

fn bytes_to_human_format(bytes: i64) -> String {
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
