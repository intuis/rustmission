use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedReceiver;
use transmission_rpc::types::{
    Id, Torrent, TorrentAction as RPCAction, TorrentAddArgs, TorrentSetArgs,
};

use crate::{action::Action, app, ui::global_popups::ErrorPopup};

#[derive(Debug)]
pub enum TorrentAction {
    Add(String),
    Stop(Vec<Id>),
    Start(Vec<Id>),
    DeleteWithoutFiles(Vec<Id>),
    DeleteWithFiles(Vec<Id>),
    GetTorrentInfo(Id, Arc<Mutex<Option<Torrent>>>),
    SetArgs(Box<TorrentSetArgs>, Option<Vec<Id>>),
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
                    .torrent_set(*args, ids)
                    .await
                    .unwrap();
            }
        }
    }
}
