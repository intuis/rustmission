use std::{sync::Arc, time::Duration};

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};
use transmission_rpc::{
    types::{SessionStats, TorrentAddArgs, TorrentGetField},
    TransClient,
};

use crate::{action::Action, app, ui::popup::ErrorPopup};

pub fn spawn_fetchers(client: Arc<Mutex<TransClient>>, sender: UnboundedSender<Action>) {
    // let stats_task = stats_fetch(Arc::clone(&client), sender.clone());
    let torrent_task = torrent_fetch(client, sender);
    // tokio::spawn(stats_task);
    tokio::spawn(torrent_task);
}

pub async fn stats_fetch(
    client: Arc<Mutex<TransClient>>,
    stats: Arc<std::sync::Mutex<Option<SessionStats>>>,
) {
    loop {
        let new_stats = client.lock().await.session_stats().await.unwrap().arguments;
        *stats.lock().unwrap() = Some(new_stats);
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

pub async fn torrent_fetch(client: Arc<Mutex<TransClient>>, sender: UnboundedSender<Action>) {
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
        ];
        let rpc_response = client
            .lock()
            .await
            .torrent_get(Some(fields), None)
            .await
            .unwrap();
        let torrents = rpc_response.arguments.torrents;
        sender
            .send(Action::TorrentListUpdate(Box::new(torrents)))
            .unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

pub async fn action_handler(ctx: app::Ctx, mut trans_rx: UnboundedReceiver<Action>) {
    while let Some(action) = trans_rx.recv().await {
        if let Action::TorrentAdd(url) = action {
            let args = TorrentAddArgs {
                filename: Some(*url.clone()),
                ..Default::default()
            };

            if let Err(e) = ctx.client.lock().await.torrent_add(args).await {
                let error_title = "Failed to add a torrent";
                let msg = "Failed to add torrent with URL/Path:\n\"".to_owned()
                    + &*url
                    + "\"\n"
                    + &e.to_string();
                let error_popup = Box::new(ErrorPopup::new(error_title, msg));
                ctx.action_tx.send(Action::Error(error_popup)).unwrap();
            }
        }
    }
}
