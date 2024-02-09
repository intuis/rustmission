use std::pin::Pin;

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph, Row, Table};
use tokio::sync::mpsc::UnboundedSender;
use transmission_rpc::types::{SessionStats, Torrent};
use tui_input::{Input, InputRequest};

use crate::app::Action;

use super::table::GenericTable;
use super::Component;

pub struct TorrentsTab {
    table: GenericTable<Torrent>,
    stats: Option<Pin<Box<SessionStats>>>,
    task: Task,
    trans_tx: UnboundedSender<Action>,
}

struct Task {
    trans_tx: UnboundedSender<Action>,
    current_task: CurrentTask,
}

impl Task {
    fn new(trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            trans_tx,
            current_task: CurrentTask::default(),
        }
    }
}

enum CurrentTask {
    AddMagnetBar(AddMagnetBar),
    None,
}

impl Default for CurrentTask {
    fn default() -> Self {
        CurrentTask::None
    }
}

struct AddMagnetBar {
    input: Input,
}

impl AddMagnetBar {
    fn new() -> Self {
        Self {
            input: Input::default(),
        }
    }
}

impl Component for Task {
    #[must_use]
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => match magnet_bar.handle_events(action) {
                Some(Action::TorrentAdd(url)) => {
                    self.trans_tx.send(Action::TorrentAdd(url)).unwrap();
                    self.current_task = CurrentTask::None;
                    Some(Action::SwitchToNormalMode)
                }
                Some(Action::Quit) => {
                    self.current_task = CurrentTask::None;
                    Some(Action::SwitchToNormalMode)
                }
                other => return other,
            },
            CurrentTask::None => match action {
                Action::AddMagnet => {
                    self.current_task = CurrentTask::AddMagnetBar(AddMagnetBar::new());
                    Some(Action::SwitchToInputMode)
                }
                _ => None,
            },
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.current_task {
            CurrentTask::AddMagnetBar(magnet_bar) => magnet_bar.render(f, rect),
            CurrentTask::None => (),
        }
    }
}

fn to_input_request(keycode: KeyCode) -> Option<InputRequest> {
    use InputRequest as R;

    match keycode {
        KeyCode::Backspace => Some(R::DeletePrevChar),
        KeyCode::Delete => Some(R::DeleteNextChar),
        KeyCode::Char(char) => Some(R::InsertChar(char)),
        _ => None,
    }
}

impl Component for AddMagnetBar {
    #[must_use]
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Enter {
                    return Some(Action::TorrentAdd(Box::new(self.input.to_string())));
                }
                if input.code == KeyCode::Esc {
                    return Some(Action::Quit);
                }

                if let Some(req) = to_input_request(input.code) {
                    self.input.handle(req);
                }
                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(Clear, rect);

        let input = self.input.to_string();

        let paragraph_text = format!("Add (Magnet URL / Torrent path): {input}");
        let prefix_len = paragraph_text.len() - input.len();

        let paragraph = Paragraph::new(paragraph_text);
        f.render_widget(paragraph, rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(rect.x + cursor_offset as u16, rect.y);
    }
}

impl TorrentsTab {
    pub fn new(trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            table: GenericTable::new(vec![]),
            stats: None,
            task: Task::new(trans_tx.clone()),
            trans_tx,
        }
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

        let torrent_rows: Vec<_> = self
            .table
            .items
            .iter()
            .map(|t| {
                let torrent_name = t.name.clone().unwrap();

                let size_when_done =
                    bytes_to_human_format(t.size_when_done.expect("field requested"));

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

                Row::new(vec![
                    torrent_name,
                    size_when_done,
                    progress,
                    eta_secs,
                    download_speed,
                    upload_speed,
                ])
            })
            .collect();

        let torrents_table = Table::new(torrent_rows, header_widths)
            .header(header)
            .highlight_style(Style::default().on_black().bold());

        f.render_stateful_widget(torrents_table, torrents_list_rect, &mut self.table.state);

        if let Some(stats) = &self.stats {
            let upload = bytes_to_human_format(stats.upload_speed);
            let download = bytes_to_human_format(stats.download_speed);
            let all = stats.torrent_count;
            let text = format!("All: {all} | ▲ {download} | ⯆ {upload}");
            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, stats_rect);
        }

        self.task.render(f, stats_rect);
    }

    fn handle_events(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match action {
            A::Up => {
                self.table.previous();
                None
            }
            A::Down => {
                self.table.next();
                None
            }
            A::TorrentListUpdate(torrents) => {
                self.table.set_items(*torrents);
                None
            }
            A::StatsUpdate(stats) => {
                self.stats = Some(stats);
                None
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
