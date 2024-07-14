use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};
use transmission_rpc::types::{
    FreeSpace, Id, SessionGet, SessionStats, Torrent, TorrentAction as RPCAction, TorrentAddArgs,
    TorrentGetField, TorrentSetArgs,
};
use transmission_rpc::TransClient;

use rm_shared::action::Action;
use rm_shared::action::ErrorMessage;

pub enum TorrentAction {
    // Magnet/URL, Directory
    Add(String, Option<String>),
    Stop(Vec<Id>),
    Start(Vec<Id>),
    DeleteWithoutFiles(Vec<Id>),
    DeleteWithFiles(Vec<Id>),
    GetTorrentInfo(Id, Arc<Mutex<Option<Torrent>>>),
    GetSessionGet(oneshot::Sender<SessionGet>),
    SetArgs(Box<TorrentSetArgs>, Option<Vec<Id>>),
    // Torrent ID, Directory to move to
    Move(Vec<Id>, String),
    GetSessionStats(oneshot::Sender<Arc<SessionStats>>),
    GetFreeSpace(String, oneshot::Sender<FreeSpace>),
    GetTorrents(Vec<TorrentGetField>, oneshot::Sender<Vec<Torrent>>),
    GetTorrentsById(Vec<Id>, oneshot::Sender<Vec<Torrent>>),
}

pub async fn action_handler(
    mut client: TransClient,
    mut trans_rx: UnboundedReceiver<TorrentAction>,
    action_tx: UnboundedSender<Action>,
) {
    while let Some(action) = trans_rx.recv().await {
        match action {
            TorrentAction::Add(ref url, directory) => {
                let formatted = {
                    if url.starts_with("www") {
                        format!("https://{url}")
                    } else {
                        url.to_string()
                    }
                };
                let args = TorrentAddArgs {
                    filename: Some(formatted),
                    download_dir: directory,
                    ..Default::default()
                };
                match client.torrent_add(args).await {
                    Ok(_) => {
                        action_tx.send(Action::TaskSuccess).unwrap();
                    }
                    Err(e) => {
                        let error_title = "Failed to add a torrent";
                        let msg = "Failed to add torrent with URL/Path:\n\"".to_owned()
                            + url
                            + "\"\n"
                            + &e.to_string();
                        let error_message = ErrorMessage {
                            title: error_title.to_string(),
                            message: msg,
                        };
                        action_tx
                            .send(Action::Error(Box::new(error_message)))
                            .unwrap();
                    }
                }
            }
            TorrentAction::Stop(ids) => {
                client
                    .torrent_action(RPCAction::Stop, ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::Start(ids) => {
                client
                    .torrent_action(RPCAction::Start, ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::DeleteWithFiles(ids) => {
                client.torrent_remove(ids, true).await.unwrap();
                action_tx.send(Action::TaskSuccess).unwrap();
            }
            TorrentAction::DeleteWithoutFiles(ids) => {
                client.torrent_remove(ids, false).await.unwrap();
                action_tx.send(Action::TaskSuccess).unwrap();
            }
            TorrentAction::GetTorrentInfo(id, torrent_info) => {
                let new_torrent_info = client
                    .torrent_get(None, Some(vec![id]))
                    .await
                    .unwrap()
                    .arguments
                    .torrents
                    .pop()
                    .unwrap();
                *torrent_info.lock().unwrap() = Some(new_torrent_info);
            }
            TorrentAction::SetArgs(args, ids) => {
                client.torrent_set(*args, ids).await.unwrap();
            }
            TorrentAction::GetSessionGet(sender) => {
                let session_get = client.session_get().await.unwrap().arguments;
                sender.send(session_get).unwrap();
            }
            TorrentAction::Move(ids, new_directory) => {
                if let Err(e) = client
                    .torrent_set_location(ids, new_directory.clone(), Option::from(true))
                    .await
                {
                    let error_title = "Failed to move torrent";
                    let msg = "Failed to move torrent to new directory:\n\"".to_owned()
                        + new_directory.as_str()
                        + "\"\n"
                        + &e.to_string();
                    let error_message = ErrorMessage {
                        title: error_title.to_string(),
                        message: msg,
                    };
                    action_tx
                        .send(Action::Error(Box::new(error_message)))
                        .unwrap();
                }
            }
            TorrentAction::GetSessionStats(sender) => {
                let stats = client.session_stats().await.unwrap().arguments;
                sender.send(Arc::new(stats)).unwrap();
            }
            TorrentAction::GetFreeSpace(path, sender) => {
                sender
                    .send(client.free_space(path).await.unwrap().arguments)
                    .unwrap();
            }
            TorrentAction::GetTorrents(fields, sender) => {
                sender
                    .send(
                        client
                            .torrent_get(Some(fields), None)
                            .await
                            .unwrap()
                            .arguments
                            .torrents,
                    )
                    .unwrap();
            }
            TorrentAction::GetTorrentsById(ids, sender) => {
                sender
                    .send(
                        client
                            .torrent_get(None, Some(ids))
                            .await
                            .unwrap()
                            .arguments
                            .torrents,
                    )
                    .unwrap();
            }
        }
    }
}
