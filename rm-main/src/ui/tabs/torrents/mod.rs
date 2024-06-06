mod bottom_stats;
mod input_manager;
pub mod popups;
pub mod rustmission_torrent;
pub mod table_manager;
pub mod task_manager;
pub mod tasks;

use std::sync::{Arc, Mutex};

use crate::ui::tabs::torrents::popups::stats::StatisticsPopup;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::prelude::*;
use ratatui::widgets::{Row, Table};
use ratatui_macros::constraints;
use transmission_rpc::types::TorrentStatus;

use crate::action::{Action, TorrentAction};
use crate::ui::components::table::GenericTable;
use crate::ui::components::Component;
use crate::{app, transmission};

use self::bottom_stats::BottomStats;
use self::popups::files::FilesPopup;
use self::popups::{CurrentPopup, PopupManager};
use self::rustmission_torrent::RustmissionTorrent;
use self::table_manager::TableManager;
use self::task_manager::TaskManager;

pub struct TorrentsTab {
    ctx: app::Ctx,
    table_manager: Arc<Mutex<TableManager>>,
    popup_manager: PopupManager,
    task_manager: TaskManager,
    stats: BottomStats,
}

impl TorrentsTab {
    pub fn new(ctx: app::Ctx) -> Self {
        let stats = BottomStats::default();
        let table = GenericTable::new(vec![]);

        let table_manager = Arc::new(Mutex::new(TableManager::new(ctx.clone(), table)));

        tokio::spawn(transmission::stats_fetch(
            ctx.clone(),
            Arc::clone(&stats.stats),
        ));

        tokio::spawn(transmission::torrent_fetch(
            ctx.clone(),
            Arc::clone(&table_manager),
        ));

        tokio::spawn(transmission::free_space_fetch(
            ctx.clone(),
            Arc::clone(&stats.free_space),
        ));

        Self {
            stats,
            task_manager: TaskManager::new(table_manager.clone(), ctx.clone()),
            table_manager,
            popup_manager: PopupManager::new(),
            ctx,
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [torrents_list_rect, stats_rect] =
            Layout::vertical(constraints![>=10, ==1]).areas(rect);

        self.render_table(f, torrents_list_rect);

        self.stats.render(f, stats_rect);

        self.task_manager.render(f, stats_rect);

        self.popup_manager.render(f, f.size());
    }

    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        if self.popup_manager.is_showing_popup() {
            return self.popup_manager.handle_actions(action);
        }

        match action {
            A::Up => self.previous_torrent(),
            A::Down => self.next_torrent(),
            A::ShowStats => self.show_statistics_popup(),
            A::ShowFiles => self.show_files_popup(),
            A::Pause => self.pause_current_torrent(),
            other => self.task_manager.handle_actions(other),
        }
    }
}

impl<'a> TorrentsTab {
    fn render_table(&mut self, f: &mut Frame, rect: Rect) {
        let table_manager_lock = &mut *self.table_manager.lock().unwrap();

        let torrent_rows: Vec<_> = if let Some(filter) = &*table_manager_lock.filter.lock().unwrap()
        {
            let torrent_rows =
                Self::filtered_torrents_rows(&table_manager_lock.table.items, filter);
            table_manager_lock.table.overwrite_len(torrent_rows.len());
            torrent_rows
        } else {
            table_manager_lock
                .table
                .items
                .iter()
                .map(RustmissionTorrent::to_row)
                .collect()
        };

        let highlight_table_style = Style::default().on_black().bold().fg(self
            .ctx
            .config
            .general
            .accent_color
            .as_ratatui());

        let table_widget = Table::new(torrent_rows, table_manager_lock.widths)
            .header(Row::new(
                table_manager_lock.header().iter().map(|s| s.as_str()),
            ))
            .highlight_style(highlight_table_style);

        f.render_stateful_widget(
            table_widget,
            rect,
            &mut table_manager_lock.table.state.borrow_mut(),
        );
    }

    fn filtered_torrents_rows(
        torrents: &'a [RustmissionTorrent],
        filter: &str,
    ) -> Vec<ratatui::widgets::Row<'a>> {
        let matcher = SkimMatcherV2::default();
        torrents
            .iter()
            .filter(|t| matcher.fuzzy_match(&t.torrent_name, filter).is_some())
            .map(RustmissionTorrent::to_row)
            .collect()
    }

    fn show_files_popup(&mut self) -> Option<Action> {
        if let Some(highlighted_torrent) = self.table_manager.lock().unwrap().current_torrent() {
            let popup = FilesPopup::new(self.ctx.clone(), highlighted_torrent.id.clone());
            self.popup_manager.show_popup(CurrentPopup::Files(popup));
            Some(Action::Render)
        } else {
            None
        }
    }

    fn show_statistics_popup(&mut self) -> Option<Action> {
        if let Some(stats) = &*self.stats.stats.lock().unwrap() {
            let popup = StatisticsPopup::new(self.ctx.clone(), stats.clone());
            self.popup_manager.show_popup(CurrentPopup::Stats(popup));
            Some(Action::Render)
        } else {
            None
        }
    }

    fn previous_torrent(&self) -> Option<Action> {
        self.table_manager.lock().unwrap().table.previous();
        Some(Action::Render)
    }

    fn next_torrent(&self) -> Option<Action> {
        self.table_manager.lock().unwrap().table.next();
        Some(Action::Render)
    }

    fn pause_current_torrent(&mut self) -> Option<Action> {
        let mut table_manager = self.table_manager.lock().unwrap();
        if let Some(torrent) = table_manager.current_torrent() {
            let torrent_id = torrent.id.clone();
            match torrent.status() {
                TorrentStatus::Stopped => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Start(vec![torrent_id]));
                    torrent.update_status(TorrentStatus::Downloading);
                    return Some(Action::Render);
                }
                _ => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Stop(vec![torrent_id]));
                    torrent.update_status(TorrentStatus::Stopped);
                    return Some(Action::Render);
                }
            }
        }
        None
    }
}
