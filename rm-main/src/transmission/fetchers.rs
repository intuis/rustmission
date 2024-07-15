use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::oneshot;
use transmission_rpc::types::{FreeSpace, TorrentGetField};

use crate::{
    app,
    ui::tabs::torrents::{rustmission_torrent::RustmissionTorrent, table_manager::TableManager},
};
use rm_shared::action::{Action, UpdateAction};

use super::TorrentAction;

pub async fn stats(ctx: app::Ctx) {
    loop {
        let (stats_tx, stats_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetSessionStats(stats_tx));
        let new_stats = stats_rx.await.unwrap();

        ctx.send_update_action(UpdateAction::SessionStats(new_stats));

        tokio::time::sleep(Duration::from_secs(ctx.config.connection.stats_refresh)).await;
    }
}

pub async fn free_space(ctx: app::Ctx) {
    let (sess_tx, sess_rx) = oneshot::channel();
    ctx.send_torrent_action(TorrentAction::GetSessionGet(sess_tx));
    let download_dir = sess_rx.await.unwrap().download_dir.leak();

    loop {
        let (space_tx, space_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetFreeSpace(
            download_dir.to_string(),
            space_tx,
        ));
        let free_space = space_rx.await.unwrap();

        ctx.send_update_action(UpdateAction::FreeSpace(Arc::new(free_space)));
        tokio::time::sleep(Duration::from_secs(
            ctx.config.connection.free_space_refresh,
        ))
        .await;
    }
}

pub async fn torrents(ctx: app::Ctx, table_manager: Arc<Mutex<TableManager>>) {
    loop {
        let fields = vec![
            TorrentGetField::Id,
            TorrentGetField::Name,
            TorrentGetField::IsFinished,
            TorrentGetField::IsStalled,
            TorrentGetField::PercentDone,
            TorrentGetField::UploadRatio,
            TorrentGetField::SizeWhenDone,
            TorrentGetField::Eta,
            TorrentGetField::RateUpload,
            TorrentGetField::RateDownload,
            TorrentGetField::Status,
            TorrentGetField::DownloadDir,
            TorrentGetField::UploadedEver,
            TorrentGetField::ActivityDate,
            TorrentGetField::AddedDate,
            TorrentGetField::PeersConnected,
        ];
        let (torrents_tx, torrents_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetTorrents(fields, torrents_tx));
        let new_torrents = torrents_rx.await.unwrap();

        {
            let mut table_manager_lock = table_manager.lock().unwrap();
            table_manager_lock
                .set_new_rows(new_torrents.iter().map(RustmissionTorrent::from).collect());
        }
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(ctx.config.connection.torrents_refresh)).await;
    }
}
