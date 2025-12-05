use std::sync::{LazyLock, Mutex};

use rm_shared::action::{Action, UpdateAction};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::transmission::TorrentAction;

pub static CTX: LazyLock<Ctx> = LazyLock::new(|| CTX_RAW.0.clone());

pub(super) static CTX_RAW: LazyLock<(
    Ctx,
    Mutex<
        Option<(
            UnboundedReceiver<Action>,
            UnboundedReceiver<UpdateAction>,
            UnboundedReceiver<TorrentAction>,
        )>,
    >,
)> = LazyLock::new(|| {
    let (ctx, act_rx, upd_rx, tor_rx) = Ctx::new();
    (ctx, Mutex::new(Some((act_rx, upd_rx, tor_rx))))
});

#[derive(Clone)]
pub struct Ctx {
    pub(super) action_tx: UnboundedSender<Action>,
    pub(super) update_tx: UnboundedSender<UpdateAction>,
    pub(super) trans_tx: UnboundedSender<TorrentAction>,
}

impl Ctx {
    fn new() -> (
        Self,
        UnboundedReceiver<Action>,
        UnboundedReceiver<UpdateAction>,
        UnboundedReceiver<TorrentAction>,
    ) {
        let (action_tx, action_rx) = unbounded_channel();
        let (update_tx, update_rx) = unbounded_channel();
        let (trans_tx, trans_rx) = unbounded_channel();

        (
            Self {
                action_tx,
                update_tx,
                trans_tx,
            },
            action_rx,
            update_rx,
            trans_rx,
        )
    }

    pub(crate) fn send_action(&self, action: Action) {
        self.action_tx.send(action).unwrap();
    }

    pub(crate) fn send_torrent_action(&self, action: TorrentAction) {
        self.trans_tx.send(action).unwrap();
    }

    pub(crate) fn send_update_action(&self, action: UpdateAction) {
        self.update_tx.send(action).unwrap();
    }
}
