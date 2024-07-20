use std::{sync::Arc, time::Duration};

use tokio::sync::oneshot;
use transmission_rpc::types::TorrentGetField;

use crate::app;
use rm_shared::action::UpdateAction;

use super::TorrentAction;

pub async fn stats(ctx: app::Ctx) {
    loop {
        let (stats_tx, stats_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetSessionStats(stats_tx));

        match stats_rx.await.unwrap() {
            Ok(stats) => {
                ctx.send_update_action(UpdateAction::SessionStats(stats));
            }
            Err(err_message) => {
                ctx.send_update_action(UpdateAction::Error(err_message));
            }
        };

        tokio::time::sleep(Duration::from_secs(ctx.config.connection.stats_refresh)).await;
    }
}

pub async fn free_space(ctx: app::Ctx) {
    let download_dir = loop {
        let (sess_tx, sess_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetSessionGet(sess_tx));
        match sess_rx.await.unwrap() {
            Ok(sess) => {
                break sess.download_dir.leak();
            }
            Err(err_message) => {
                ctx.send_update_action(UpdateAction::Error(err_message));
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        };
    };

    loop {
        let (space_tx, space_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetFreeSpace(
            download_dir.to_string(),
            space_tx,
        ));

        match space_rx.await.unwrap() {
            Ok(free_space) => {
                ctx.send_update_action(UpdateAction::FreeSpace(Arc::new(free_space)));
            }
            Err(err_message) => {
                ctx.send_update_action(UpdateAction::Error(err_message));
            }
        }

        tokio::time::sleep(Duration::from_secs(
            ctx.config.connection.free_space_refresh,
        ))
        .await;
    }
}

pub async fn torrents(ctx: app::Ctx) {
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

        match torrents_rx.await.unwrap() {
            Ok(torrents) => {
                ctx.send_update_action(UpdateAction::UpdateTorrents(torrents));
            }
            Err(err_message) => {
                ctx.send_update_action(UpdateAction::Error(err_message));
            }
        };

        tokio::time::sleep(Duration::from_secs(ctx.config.connection.torrents_refresh)).await;
    }
}
