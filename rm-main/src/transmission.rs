use std::{sync::Arc, time::Duration};

use ratatui::{
    style::{Style, Stylize},
    widgets::Row,
};
use tokio::sync::mpsc::UnboundedReceiver;
use transmission_rpc::types::{
    SessionStats, Torrent, TorrentAddArgs, TorrentGetField, TorrentStatus,
};

use transmission_rpc::types::TorrentAction as RPCAction;

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{bytes_to_human_format, popup::ErrorPopup, tabs::torrents::torrents::TableManager},
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

pub async fn torrent_fetch(
    ctx: app::Ctx,
    torrents: Arc<std::sync::Mutex<Vec<Torrent>>>,
    table_manager: Arc<std::sync::Mutex<TableManager>>,
) {
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
        table_manager.lock().unwrap().set_new_rows(
            new_torrents
                .iter()
                .map(|torrent| torrent_to_row(torrent))
                .collect(),
        );
        *torrents.lock().unwrap() = new_torrents;
        ctx.send_action(Action::Render);

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

pub struct RustmissionTorrent {
    pub torrent_name: String,
    pub size_when_done: String,
    pub progress: String,
    pub eta_secs: String,
    pub download_speed: String,
    pub upload_speed: String,
    pub status: TorrentStatus,
    pub style: Style,
}

impl RustmissionTorrent {
    pub fn to_row(&self) -> ratatui::widgets::Row {
        Row::new([
            self.torrent_name.as_str(),
            self.size_when_done.as_str(),
            self.progress.as_str(),
            self.eta_secs.as_str(),
            self.download_speed.as_str(),
            self.upload_speed.as_str(),
        ])
        .style(self.style)
    }
}

fn torrent_to_row(t: &Torrent) -> RustmissionTorrent {
    let torrent_name = t.name.clone().unwrap();

    let size_when_done = bytes_to_human_format(t.size_when_done.expect("field requested"));

    let progress = match t.percent_done.expect("field requested") {
        done if done == 1f32 => String::default(),
        percent => format!("{:.2}%", percent * 100f32),
    };

    let eta_secs = match t.eta.expect("field requested") {
        -2 => "∞".to_string(),
        -1 => String::default(),
        eta_secs => eta_secs.to_string(),
    };

    let download_speed = match t.rate_download.expect("field requested") {
        0 => String::default(),
        down => bytes_to_human_format(down),
    };

    let upload_speed = match t.rate_upload.expect("field requested") {
        0 => String::default(),
        upload => bytes_to_human_format(upload),
    };

    let status = t.status.expect("field requested");

    let style = match status {
        TorrentStatus::Stopped => Style::default().dark_gray().italic(),
        _ => Style::default(),
    };

    RustmissionTorrent {
        torrent_name,
        size_when_done,
        progress,
        eta_secs,
        download_speed,
        upload_speed,
        status,
        style,
    }
}

pub async fn action_handler(ctx: app::Ctx, mut trans_rx: UnboundedReceiver<TorrentAction>) {
    while let Some(action) = trans_rx.recv().await {
        match action {
            TorrentAction::TorrentAdd(ref url) => {
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
                    ctx.send_action(Action::Error(error_popup));
                }
            }
            TorrentAction::TorrentStop(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_action(RPCAction::Stop, *ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::TorrentStart(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_action(RPCAction::Start, *ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::TorrentDelete(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_remove(*ids, true)
                    .await
                    .unwrap();
            }
        }
    }
}
