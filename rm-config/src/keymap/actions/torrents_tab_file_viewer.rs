use intuitils::user_action::UserAction;
use rm_shared::action::Action;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TorrentsFileViewerAction {
    ChangeFilePriority,
}

impl UserAction for TorrentsFileViewerAction {
    fn desc(&self) -> &'static str {
        match self {
            TorrentsFileViewerAction::ChangeFilePriority => "change file priority",
        }
    }
}

impl From<TorrentsFileViewerAction> for Action {
    fn from(value: TorrentsFileViewerAction) -> Self {
        match value {
            TorrentsFileViewerAction::ChangeFilePriority => Action::ChangeFilePriority,
        }
    }
}
