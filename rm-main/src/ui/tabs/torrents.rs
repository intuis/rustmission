mod task_manager;
mod tasks;

use std::sync::{Arc, Mutex};

use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Row, Table};
use ratatui_macros::constraints;
use transmission_rpc::types::{SessionStats, Torrent, TorrentStatus};

use crate::action::{Action, TorrentAction};
use crate::transmission::RustmissionTorrent;
use crate::ui::components::table::GenericTable;
use crate::ui::components::Component;
use crate::ui::{bytes_to_human_format, centered_rect};
use crate::{app, transmission};

use self::task_manager::TaskManager;

#[derive(Default)]
struct StatsComponent {
    // TODO: get rid of the Option
    stats: Arc<Mutex<Option<SessionStats>>>,
}

impl Component for StatsComponent {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(stats) = &*self.stats.lock().unwrap() {
            let upload = bytes_to_human_format(stats.upload_speed);
            let download = bytes_to_human_format(stats.download_speed);
            let all = stats.torrent_count;
            let text = format!("All: {all} | ▲ {download} | ⯆ {upload}");
            let paragraph = Paragraph::new(text).alignment(Alignment::Right);
            f.render_widget(paragraph, rect);
        }
    }
}

struct StatisticsPopup {
    stats: SessionStats,
}

impl StatisticsPopup {
    fn new(stats: SessionStats) -> Self {
        Self { stats }
    }
}

impl Component for StatisticsPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Confirm = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(&Margin::new(1, 1));
        let text_rect = block_rect.inner(&Margin::new(3, 2));
        let button_rect = { Layout::vertical(constraints![==100%, ==1]).split(text_rect)[1] };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Statistics ")
            .title_style(Style::default().light_magenta());

        let button = Paragraph::new("[ OK ]").bold().right_aligned();

        let uploaded_bytes = self.stats.cumulative_stats.uploaded_bytes;
        let downloaded_bytes = self.stats.cumulative_stats.downloaded_bytes;
        let uploaded = bytes_to_human_format(uploaded_bytes);
        let downloaded = bytes_to_human_format(downloaded_bytes);
        let ratio = uploaded_bytes / downloaded_bytes;
        let text = format!("Uploaded: {uploaded}\nDownloaded: {downloaded}\nRatio: {ratio}");
        let paragraph = Paragraph::new(text);

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
        f.render_widget(button, button_rect);
    }
}

pub struct TorrentsTab {
    table: Arc<Mutex<GenericTable<Torrent>>>,
    rows: Arc<Mutex<Vec<RustmissionTorrent>>>,
    stats: StatsComponent,
    task: TaskManager,
    statistics_popup: Option<StatisticsPopup>,
    ctx: app::Ctx,
}

impl TorrentsTab {
    pub fn new(ctx: app::Ctx) -> Self {
        let stats = StatsComponent::default();
        let table = Arc::new(Mutex::new(GenericTable::new(vec![])));
        let rows = Arc::new(Mutex::new(vec![]));

        tokio::spawn(transmission::stats_fetch(
            ctx.clone(),
            Arc::clone(&stats.stats),
        ));

        tokio::spawn(transmission::torrent_fetch(
            ctx.clone(),
            Arc::clone(&table.lock().unwrap().items),
            Arc::clone(&rows),
        ));

        Self {
            table: Arc::clone(&table),
            rows,
            stats,
            task: TaskManager::new(Arc::clone(&table), ctx.clone()),
            statistics_popup: None,
            ctx,
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [torrents_list_rect, stats_rect] =
            Layout::vertical(constraints![>=10, ==1]).areas(rect);

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

        let rows = self.rows.lock().unwrap();

        let torrent_rows = rows
            .iter()
            .map(crate::transmission::RustmissionTorrent::to_row);

        let torrents_table = Table::new(torrent_rows, header_widths)
            .header(header)
            .highlight_style(Style::default().light_magenta().on_black().bold());

        f.render_stateful_widget(
            torrents_table,
            torrents_list_rect,
            &mut self.table.lock().unwrap().state.borrow_mut(),
        );

        self.stats.render(f, stats_rect);

        self.task.render(f, stats_rect);

        if let Some(popup) = &mut self.statistics_popup {
            popup.render(f, f.size());
        }
    }

    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        if let Some(popup) = &mut self.statistics_popup {
            if let Some(Action::Quit) = popup.handle_actions(action) {
                self.statistics_popup = None;
                return Some(Action::Render);
            };
            return None;
        }

        match action {
            A::Up => {
                self.table.lock().unwrap().previous();
                Some(Action::Render)
            }
            A::Down => {
                self.table.lock().unwrap().next();
                Some(Action::Render)
            }
            A::ShowStats => {
                if let Some(stats) = &*self.stats.stats.lock().unwrap() {
                    self.statistics_popup = Some(StatisticsPopup::new(stats.clone()));
                    Some(Action::Render)
                } else {
                    None
                }
            }
            A::Pause => {
                if let Some(torrent) = self.table.lock().unwrap().current_item() {
                    let torrent_id = torrent.id().unwrap();
                    let torrent_status = torrent.status.unwrap();

                    match torrent_status {
                        TorrentStatus::Stopped => {
                            self.ctx
                                .send_torrent_action(TorrentAction::TorrentStart(Box::new(vec![
                                    torrent_id,
                                ])));
                        }
                        _ => {
                            self.ctx
                                .send_torrent_action(TorrentAction::TorrentStop(Box::new(vec![
                                    torrent_id,
                                ])));
                        }
                    }
                    return None;
                }
                None
            }

            other => self.task.handle_actions(other),
        }
    }
}