use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};
use transmission_rpc::types::{
    FreeSpace, Id, SessionGet, SessionStats, Torrent, TorrentAction as RPCAction, TorrentAddArgs,
    TorrentGetField, TorrentSetArgs,
};
use transmission_rpc::TransClient;

use rm_shared::action::ErrorMessage;
use rm_shared::action::UpdateAction;

const FAILED_TO_COMMUNICATE: &str = "Failed to communicate with Transmission";

pub enum TorrentAction {
    // Add a torrent with this Magnet/URL, Directory
    Add(String, Option<String>),
    // Stop Torrents with these given IDs
    Stop(Vec<Id>),
    // Start Torrents with these given IDs
    Start(Vec<Id>),
    // Torrent ID, Directory to move to
    Move(Vec<Id>, String),
    // Delete Torrents with these given IDs (without files)
    DeleteWithoutFiles(Vec<Id>),
    // Delete Torrents with these given IDs (with files)
    DeleteWithFiles(Vec<Id>),
    // Set various properties to Torrents with these given IDs
    SetArgs(Box<TorrentSetArgs>, Option<Vec<Id>>),
    // Get info about current Transmission session
    GetSessionGet(oneshot::Sender<SessionGet>),
    // Get info about current Transmission session statistics
    GetSessionStats(oneshot::Sender<Arc<SessionStats>>),
    // Get info about available space on the disk
    GetFreeSpace(String, oneshot::Sender<FreeSpace>),
    // Get info about all Torrents with these given Fields.
    GetTorrents(Vec<TorrentGetField>, oneshot::Sender<Vec<Torrent>>),
    // Get info about specific torrents with these given IDs
    GetTorrentsById(Vec<Id>, oneshot::Sender<Vec<Torrent>>),
}

pub async fn action_handler(
    mut client: TransClient,
    mut trans_rx: UnboundedReceiver<TorrentAction>,
    action_tx: UnboundedSender<UpdateAction>,
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
                        action_tx.send(UpdateAction::TaskSuccess).unwrap();
                    }
                    Err(err) => {
                        let msg = format!("Failed to add torrent with URL/Path: \"{url}\"");
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                        action_tx.send(UpdateAction::TaskFailure).unwrap();
                    }
                }
            }
            TorrentAction::Stop(ids) => {
                match client.torrent_action(RPCAction::Stop, ids.clone()).await {
                    Ok(_) => (),
                    Err(err) => {
                        let msg = format!("Failed to stop torrents with these IDs: {:?}", ids);
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                    }
                }
            }
            TorrentAction::Start(ids) => {
                match client.torrent_action(RPCAction::Start, ids.clone()).await {
                    Ok(_) => (),
                    Err(err) => {
                        let msg = format!("Failed to start torrents with these IDs: {:?}", ids);
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                    }
                }
            }
            TorrentAction::DeleteWithFiles(ids) => {
                match client.torrent_remove(ids.clone(), true).await {
                    Ok(_) => action_tx.send(UpdateAction::TaskSuccess).unwrap(),
                    Err(err) => {
                        let msg = format!("Failed to remove torrents with these IDs: {:?}", ids);
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                        action_tx.send(UpdateAction::TaskFailure).unwrap();
                    }
                }
            }
            TorrentAction::DeleteWithoutFiles(ids) => {
                match client.torrent_remove(ids.clone(), false).await {
                    Ok(_) => action_tx.send(UpdateAction::TaskSuccess).unwrap(),
                    Err(err) => {
                        let msg = format!("Failed to remove torrents with these IDs: {:?}", ids);
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                        action_tx.send(UpdateAction::TaskFailure).unwrap();
                    }
                }
            }
            TorrentAction::SetArgs(args, ids) => {
                match client.torrent_set(*args, ids.clone()).await {
                    Ok(_) => (),
                    Err(err) => {
                        let msg = format!(
                            "Failed to set some properties to torrents with these IDs: {:?}",
                            ids
                        );
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                    }
                }
            }
            TorrentAction::GetSessionGet(sender) => match client.session_get().await {
                Ok(session_get) => {
                    sender.send(session_get.arguments).unwrap();
                }
                Err(err) => {
                    let msg = "Failed to get session data";
                    let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                    action_tx
                        .send(UpdateAction::Error(Box::new(err_message)))
                        .unwrap();
                }
            },
            TorrentAction::Move(ids, new_directory) => {
                if let Err(err) = client
                    .torrent_set_location(ids, new_directory.clone(), Some(true))
                    .await
                {
                    let msg = format!("Failed to move torrent to new directory:\n{new_directory}");
                    let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                    action_tx
                        .send(UpdateAction::Error(Box::new(err_message)))
                        .unwrap();
                }
            }
            TorrentAction::GetSessionStats(sender) => match client.session_stats().await {
                Ok(stats) => sender.send(Arc::new(stats.arguments)).unwrap(),
                Err(err) => {
                    let msg = "Failed to get session stats";
                    let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                    action_tx
                        .send(UpdateAction::Error(Box::new(err_message)))
                        .unwrap();
                }
            },
            TorrentAction::GetFreeSpace(path, sender) => match client.free_space(path).await {
                Ok(free_space) => sender.send(free_space.arguments).unwrap(),
                Err(err) => {
                    let msg = "Failed to get free space info";
                    let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                    action_tx
                        .send(UpdateAction::Error(Box::new(err_message)))
                        .unwrap();
                }
            },
            TorrentAction::GetTorrents(fields, sender) => {
                match client.torrent_get(Some(fields), None).await {
                    Ok(torrents) => sender.send(torrents.arguments.torrents).unwrap(),
                    Err(err) => {
                        let msg = "Failed to fetch torrent data";
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                    }
                }
            }
            TorrentAction::GetTorrentsById(ids, sender) => {
                match client.torrent_get(None, Some(ids.clone())).await {
                    Ok(torrents) => sender.send(torrents.arguments.torrents).unwrap(),
                    Err(err) => {
                        let msg = format!("Failed to fetch torrents with these IDs: {:?}", ids);
                        let err_message = ErrorMessage::new(FAILED_TO_COMMUNICATE, msg, err);
                        action_tx
                            .send(UpdateAction::Error(Box::new(err_message)))
                            .unwrap();
                    }
                }
            }
        }
    }
}
