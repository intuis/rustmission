use std::{sync::Arc, time::Duration};

use tokio::sync::{
    mpsc::Receiver,
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};
use transmission_rpc::{
    types::{TorrentAddArgs, TorrentGetField},
    TransClient,
};

use crate::app::Action;

pub async fn spawn_tasks(client: Arc<Mutex<TransClient>>, sender: UnboundedSender<Action>) {
    let stats_task = stats_fetch(Arc::clone(&client), sender.clone());
    let torrent_task = torrent_fetch(client, sender);
    tokio::spawn(stats_task);
    tokio::spawn(torrent_task);
}

pub async fn stats_fetch(client: Arc<Mutex<TransClient>>, sender: UnboundedSender<Action>) {
    loop {
        let stats = Box::pin(client.lock().await.session_stats().await.unwrap().arguments);
        sender.send(Action::StatsUpdate(stats)).unwrap();
        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

pub async fn torrent_fetch(client: Arc<Mutex<TransClient>>, sender: UnboundedSender<Action>) {
    loop {
        // TODO: talk to rustmission-rpc's authors to tell them that torrent_get shouldnt
        // take an ownership of this vec, or check the documentation (maybe there's a function that
        // takes a reference who knows)
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

        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

pub async fn action_handler(
    client: Arc<Mutex<TransClient>>,
    mut sender: UnboundedReceiver<Action>,
) {
    while let Some(action) = sender.recv().await {
        match action {
            Action::TorrentAdd(url) => {
                let args = TorrentAddArgs {
                    filename: Some(*url),
                    ..Default::default()
                };
                client.lock().await.torrent_add(args).await.unwrap();
            }
            _ => {}
        }
    }
}
