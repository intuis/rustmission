use std::time::Duration;

use crate::{
    transmission::TorrentAction,
    tui::{
        components::{Component, ComponentAction},
        ctx::CTX,
    },
};

use self::{files::FilesPopup, stats::StatisticsPopup, trackers::TrackersPopup};
use details::DetailsPopup;
use rm_shared::{
    action::{Action, UpdateAction},
    current_window::TorrentWindow,
};

use ratatui::prelude::*;
use tokio::sync::oneshot;
use transmission_rpc::types::Id;

pub mod details;
pub mod files;
pub mod stats;
pub mod trackers;

/// Periodically re-fetches the given torrent and sends it as `UpdateCurrentTorrent`.
pub async fn fetch_current_torrent(torrent_id: Id) {
    loop {
        let (torrent_tx, torrent_rx) = oneshot::channel();
        CTX.send_torrent_action(TorrentAction::GetTorrentsById(
            vec![torrent_id.clone()],
            torrent_tx,
        ));

        match torrent_rx.await.unwrap() {
            Ok(mut torrents) => {
                if let Some(torrent) = torrents.pop() {
                    CTX.send_update_action(UpdateAction::UpdateCurrentTorrent(Box::new(torrent)));
                }
            }
            Err(err_message) => {
                CTX.send_update_action(UpdateAction::Error(err_message));
            }
        };

        tokio::time::sleep(Duration::from_secs(6)).await;
    }
}

pub struct PopupManager {
    pub current_popup: Option<CurrentPopup>,
}

pub enum CurrentPopup {
    Stats(StatisticsPopup),
    Files(FilesPopup),
    Details(DetailsPopup),
    Trackers(TrackersPopup),
}

impl PopupManager {
    pub const fn new() -> Self {
        Self {
            current_popup: None,
        }
    }

    pub const fn is_showing_popup(&self) -> bool {
        self.current_popup.is_some()
    }

    pub fn show_popup(&mut self, popup: CurrentPopup) {
        self.current_popup = Some(popup);
    }

    pub fn close_popup(&mut self) {
        self.current_popup = None;
    }
}

impl Component for PopupManager {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        if let Some(current_popup) = &mut self.current_popup {
            let should_close = match current_popup {
                CurrentPopup::Stats(popup) => popup.handle_actions(action).is_quit(),
                CurrentPopup::Files(popup) => popup.handle_actions(action).is_quit(),
                CurrentPopup::Details(popup) => popup.handle_actions(action).is_quit(),
                CurrentPopup::Trackers(popup) => popup.handle_actions(action).is_quit(),
            };

            if should_close {
                self.close_popup();
                CTX.send_update_action(UpdateAction::ChangeTorrentWindow(TorrentWindow::General));
            }
        }
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match &mut self.current_popup {
            Some(CurrentPopup::Files(popup)) => popup.handle_update_action(action),
            Some(CurrentPopup::Trackers(popup)) => popup.handle_update_action(action),
            _ => (),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(current_popup) = &mut self.current_popup {
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    popup.render(f, rect);
                }
                CurrentPopup::Files(popup) => {
                    popup.render(f, rect);
                }
                CurrentPopup::Details(popup) => popup.render(f, rect),
                CurrentPopup::Trackers(popup) => popup.render(f, rect),
            }
        }
    }
}
