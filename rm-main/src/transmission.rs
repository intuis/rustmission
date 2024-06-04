use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc::UnboundedReceiver;
use transmission_rpc::types::{FreeSpace, SessionStats, TorrentAddArgs, TorrentGetField};

use transmission_rpc::types::TorrentAction as RPCAction;

use crate::ui::tabs::torrents::rustmission_torrent::RustmissionTorrent;
use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{popup::ErrorPopup, tabs::torrents::table_manager::TableManager},
};

pub async fn stats_fetch(ctx: app::Ctx, stats: Arc<std::sync::Mutex<Option<SessionStats>>>) {
    loop {
        let new_stats = ctx
            .client
            .lock()
            .await
            .session_stats()
            .await
            .unwrap()
            .arguments;
        *stats.lock().unwrap() = Some(new_stats);
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

pub async fn free_space_fetch(ctx: app::Ctx, free_space: Arc<std::sync::Mutex<Option<FreeSpace>>>) {
    let download_dir = ctx
        .client
        .lock()
        .await
        .session_get()
        .await
        .unwrap()
        .arguments
        .download_dir
        .leak();

    loop {
        let new_free_space = ctx
            .client
            .lock()
            .await
            .free_space(download_dir.to_string())
            .await
            .unwrap()
            .arguments;
        *free_space.lock().unwrap() = Some(new_free_space);
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

pub async fn torrent_fetch(ctx: app::Ctx, table_manager: Arc<std::sync::Mutex<TableManager>>) {
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
        ];
        let rpc_response = ctx
            .client
            .lock()
            .await
            .torrent_get(Some(fields), None)
            .await
            .unwrap();

        let new_torrents = rpc_response.arguments.torrents;

        {
            let mut table_manager_lock = table_manager.lock().unwrap();
            table_manager_lock
                .set_new_rows(new_torrents.iter().map(RustmissionTorrent::from).collect());
        }
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

pub async fn action_handler(ctx: app::Ctx, mut trans_rx: UnboundedReceiver<TorrentAction>) {
    while let Some(action) = trans_rx.recv().await {
        match action {
            TorrentAction::Add(ref url) => {
                let args = TorrentAddArgs {
                    filename: Some(url.clone()),
                    ..Default::default()
                };

                if let Err(e) = ctx.client.lock().await.torrent_add(args).await {
                    let error_title = "Failed to add a torrent";
                    let msg = "Failed to add torrent with URL/Path:\n\"".to_owned()
                        + url
                        + "\"\n"
                        + &e.to_string();
                    let error_popup = Box::new(ErrorPopup::new(error_title, msg));
                    ctx.send_action(Action::Error(error_popup));
                }
            }
            TorrentAction::Stop(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_action(RPCAction::Stop, ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::Start(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_action(RPCAction::Start, ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::DeleteWithFiles(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_remove(ids, true)
                    .await
                    .unwrap();
            }
            TorrentAction::DeleteWithoutFiles(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_remove(ids, false)
                    .await
                    .unwrap();
            }
            TorrentAction::GetTorrentInfo(id, sender) => {
                let new_torrent_info = ctx
                    .client
                    .lock()
                    .await
                    .torrent_get(None, Some(vec![id]))
                    .await
                    .unwrap()
                    .arguments
                    .torrents
                    .pop()
                    .unwrap();
                *sender.lock().unwrap() = Some(new_torrent_info);
            }
            TorrentAction::SetArgs(args, ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_set(args, ids)
                    .await
                    .unwrap();
            }
        }
    }
}
