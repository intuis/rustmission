mod bottom_stats;
mod input_manager;
pub mod popups;
pub mod rustmission_torrent;
pub mod table_manager;
pub mod task_manager;
pub mod tasks;

use crate::transmission::TorrentAction;
use crate::ui::tabs::torrents::popups::stats::StatisticsPopup;

use ratatui::prelude::*;
use ratatui::widgets::{Row, Table};
use rm_shared::status_task::StatusTask;
use rustmission_torrent::RustmissionTorrent;
use transmission_rpc::types::TorrentStatus;

use crate::ui::components::{Component, ComponentAction};
use crate::{app, transmission};
use rm_shared::action::{Action, ErrorMessage, UpdateAction};

use self::bottom_stats::BottomStats;
use self::popups::files::FilesPopup;
use self::popups::{CurrentPopup, PopupManager};
use self::table_manager::TableManager;
use self::task_manager::TaskManager;

pub struct TorrentsTab {
    ctx: app::Ctx,
    table_manager: TableManager,
    popup_manager: PopupManager,
    task_manager: TaskManager,
    bottom_stats: BottomStats,
}

impl TorrentsTab {
    pub fn new(ctx: app::Ctx) -> Self {
        let table_manager = TableManager::new(ctx.clone());
        let bottom_stats = BottomStats::new();

        tokio::spawn(transmission::fetchers::stats(ctx.clone()));
        tokio::spawn(transmission::fetchers::torrents(ctx.clone()));
        tokio::spawn(transmission::fetchers::free_space(ctx.clone()));

        Self {
            bottom_stats,
            task_manager: TaskManager::new(ctx.clone()),
            table_manager,
            popup_manager: PopupManager::new(ctx.clone()),
            ctx,
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [torrents_list_rect, stats_rect] =
            Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(rect);

        self.render_table(f, torrents_list_rect);

        self.bottom_stats.render(f, stats_rect);

        self.task_manager.render(f, stats_rect);

        self.popup_manager.render(f, f.size());
    }

    #[must_use]
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;

        if self.popup_manager.is_showing_popup() {
            self.popup_manager.handle_actions(action);
            return ComponentAction::Nothing;
        }

        if action.is_quit() {
            self.ctx.send_action(Action::Quit);
        }

        match action {
            A::Up => self.previous_torrent(),
            A::Down => self.next_torrent(),
            A::ScrollUpPage => self.scroll_page_up(),
            A::ScrollDownPage => self.scroll_page_down(),
            A::Home => self.scroll_to_home(),
            A::End => self.scroll_to_end(),
            A::ShowStats => self.show_statistics_popup(),
            A::ShowFiles => self.show_files_popup(),
            A::Pause => self.pause_current_torrent(),
            A::DeleteWithFiles => {
                if let Some(torrent) = self.table_manager.current_torrent() {
                    self.task_manager
                        .delete_torrent(torrent, tasks::delete_torrent::Mode::WithFiles);
                }
            }
            A::DeleteWithoutFiles => {
                if let Some(torrent) = self.table_manager.current_torrent() {
                    self.task_manager
                        .delete_torrent(torrent, tasks::delete_torrent::Mode::WithoutFiles);
                }
            }
            A::AddMagnet => self.task_manager.add_magnet(),
            A::Search => self.task_manager.search(&self.table_manager.filter),
            A::MoveTorrent => {
                if let Some(torrent) = self.table_manager.current_torrent() {
                    self.task_manager.move_torrent(torrent);
                }
            }
            A::XdgOpen => self.open_current_torrent(),
            other => {
                self.task_manager.handle_actions(other);
            }
        };

        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::SessionStats(stats) => {
                self.bottom_stats.set_stats(stats);
            }
            UpdateAction::FreeSpace(free_space) => {
                self.bottom_stats.set_free_space(free_space);
            }
            UpdateAction::SearchFilterApply(filter) => {
                self.table_manager.set_filter(filter);
                self.table_manager.table.state.borrow_mut().select(Some(0));
                self.table_manager.update_rows_number();
                self.bottom_stats
                    .update_selected_indicator(&self.table_manager);
            }
            UpdateAction::SearchFilterClear => {
                self.table_manager.filter = None;
                self.table_manager.table.state.borrow_mut().select(Some(0));
                self.table_manager.update_rows_number();
                self.bottom_stats
                    .update_selected_indicator(&self.table_manager);
            }
            UpdateAction::UpdateTorrents(torrents) => {
                let torrents = torrents.into_iter().map(RustmissionTorrent::from).collect();
                self.table_manager.set_new_rows(torrents);
                self.bottom_stats
                    .update_selected_indicator(&self.table_manager);
            }
            UpdateAction::UpdateCurrentTorrent(_) => {
                self.popup_manager.handle_update_action(action)
            }
            other => self.task_manager.handle_update_action(other),
        }
    }

    fn tick(&mut self) {
        self.task_manager.tick();
    }
}

impl TorrentsTab {
    fn render_table(&mut self, f: &mut Frame, rect: Rect) {
        self.table_manager.torrents_displaying_no = rect.height;

        let highlight_table_style = Style::default().on_black().bold().fg(self
            .ctx
            .config
            .general
            .accent_color);

        let table_widget = {
            let table = Table::new(self.table_manager.rows(), &self.table_manager.widths)
                .highlight_style(highlight_table_style);
            if !self.ctx.config.general.headers_hide {
                table.header(Row::new(self.table_manager.headers().iter().cloned()))
            } else {
                table
            }
        };

        f.render_stateful_widget(
            table_widget,
            rect,
            &mut self.table_manager.table.state.borrow_mut(),
        );
    }

    fn show_files_popup(&mut self) {
        if let Some(highlighted_torrent) = self.table_manager.current_torrent() {
            let popup = FilesPopup::new(self.ctx.clone(), highlighted_torrent.id.clone());
            self.popup_manager.show_popup(CurrentPopup::Files(popup));
            self.ctx.send_action(Action::Render);
        }
    }

    fn show_statistics_popup(&mut self) {
        if let Some(stats) = &self.bottom_stats.stats {
            let popup = StatisticsPopup::new(self.ctx.clone(), stats.clone());
            self.popup_manager.show_popup(CurrentPopup::Stats(popup));
            self.ctx.send_action(Action::Render)
        }
    }

    fn previous_torrent(&mut self) {
        self.table_manager.table.previous();
        self.bottom_stats
            .update_selected_indicator(&self.table_manager);
        self.ctx.send_action(Action::Render);
    }

    fn next_torrent(&mut self) {
        self.table_manager.table.next();
        self.bottom_stats
            .update_selected_indicator(&self.table_manager);
        self.ctx.send_action(Action::Render);
    }

    fn scroll_page_down(&mut self) {
        let scroll_by = self.table_manager.torrents_displaying_no;
        self.table_manager.table.scroll_down_by(scroll_by as usize);
        self.bottom_stats
            .update_selected_indicator(&self.table_manager);
        self.ctx.send_action(Action::Render);
    }

    fn scroll_page_up(&mut self) {
        let scroll_by = self.table_manager.torrents_displaying_no;
        self.table_manager.table.scroll_up_by(scroll_by as usize);
        self.bottom_stats
            .update_selected_indicator(&self.table_manager);
        self.ctx.send_action(Action::Render);
    }

    fn scroll_to_home(&mut self) {
        self.table_manager.table.scroll_to_home();
        self.bottom_stats
            .update_selected_indicator(&self.table_manager);
        self.ctx.send_action(Action::Render);
    }

    fn scroll_to_end(&mut self) {
        self.table_manager.table.scroll_to_end();
        self.bottom_stats
            .update_selected_indicator(&self.table_manager);
        self.ctx.send_action(Action::Render);
    }

    fn pause_current_torrent(&mut self) {
        if let Some(torrent) = self.table_manager.current_torrent() {
            let torrent_id = torrent.id.clone();
            match torrent.status() {
                TorrentStatus::Stopped => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Start(vec![torrent_id]));
                    torrent.update_status(TorrentStatus::Downloading);
                    self.ctx.send_action(Action::Render);
                }
                _ => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Stop(vec![torrent_id]));
                    torrent.update_status(TorrentStatus::Stopped);
                    self.ctx.send_action(Action::Render);
                }
            }
        }
    }

    fn open_current_torrent(&mut self) {
        if let Some(torrent) = self.table_manager.current_torrent() {
            let torrent_location = torrent.torrent_location();
            match open::that_detached(&torrent_location) {
                Ok(()) => {
                    self.ctx
                        .send_update_action(UpdateAction::TaskSetSuccess(StatusTask::new_open(
                            torrent_location,
                        )))
                }
                Err(err) => {
                    let desc = format!(
                        "Encountered an error while trying to open \"{}\"",
                        torrent_location
                    );
                    let err_msg = ErrorMessage::new(
                        "Failed to open a torrent directory",
                        desc,
                        Box::new(err),
                    );
                    self.ctx
                        .send_update_action(UpdateAction::Error(Box::new(err_msg)));
                }
            };
        }
    }
}
