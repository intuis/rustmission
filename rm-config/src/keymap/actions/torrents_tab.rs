use intuitils::user_action::UserAction;
use rm_shared::action::Action;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TorrentsAction {
    AddMagnet,
    MoveTorrent,
    Pause,
    Delete,
    ShowFiles,
    ShowStats,
    ChangeCategory,
}

impl UserAction for TorrentsAction {
    fn desc(&self) -> &'static str {
        match self {
            TorrentsAction::AddMagnet => "add a magnet",
            TorrentsAction::MoveTorrent => "move torrent download directory",
            TorrentsAction::Pause => "pause/unpause",
            TorrentsAction::Delete => "delete",
            TorrentsAction::ShowFiles => "show files",
            TorrentsAction::ShowStats => "show statistics",
            TorrentsAction::ChangeCategory => "change category",
        }
    }
}

impl From<TorrentsAction> for Action {
    fn from(value: TorrentsAction) -> Self {
        match value {
            TorrentsAction::AddMagnet => Action::AddMagnet,
            TorrentsAction::MoveTorrent => Action::MoveTorrent,
            TorrentsAction::Pause => Action::Pause,
            TorrentsAction::Delete => Action::Delete,
            TorrentsAction::ShowFiles => Action::ShowFiles,
            TorrentsAction::ShowStats => Action::ShowStats,
            TorrentsAction::ChangeCategory => Action::ChangeCategory,
        }
    }
}
