use ratatui::prelude::*;
use ratatui::widgets::{Row, Table, TableState};
use transmission_rpc::types::Torrent;

use super::Component;

pub struct TorrentsTab {
    pub torrents: Vec<Torrent>,
    table_state: TableState,
}

impl TorrentsTab {
    pub fn new() -> Self {
        TorrentsTab {
            torrents: vec![],
            table_state: TableState::default(),
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let header = Row::new(vec!["Name", "Size", "Progress", "ETA"]);
        let widths = [
            Constraint::Length(60),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let rows: Vec<_> = self
            .torrents
            .iter()
            .map(|t| {
                let percent = t.percent_done.unwrap();
                let status = if percent == 1f32 {
                    "DONE".to_string()
                } else {
                    (percent * 100f32).to_string()
                };

                Row::new(vec![
                    t.name.clone().unwrap(),
                    bytes_to_human(t.size_when_done.clone().unwrap()),
                    status,
                    t.eta.unwrap().to_string(),
                ])
            })
            .collect();

        let table = Table::new(rows, widths).header(header);
        f.render_stateful_widget(table, rect, &mut self.table_state)
    }
}

fn bytes_to_human(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let (value, unit) = if bytes < KB as i64 {
        (bytes as f64, "B")
    } else if bytes < MB as i64 {
        (bytes as f64 / KB, "KB")
    } else if bytes < GB as i64 {
        (bytes as f64 / MB, "MB")
    } else if bytes < TB as i64 {
        (bytes as f64 / GB, "GB")
    } else {
        (bytes as f64 / TB, "TB")
    };

    format!("{:.2} {}", value, unit)
}
