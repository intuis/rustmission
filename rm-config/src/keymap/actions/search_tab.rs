use intuitils::user_action::UserAction;
use rm_shared::action::Action;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SearchAction {
    ShowProvidersInfo,
}

impl UserAction for SearchAction {
    fn desc(&self) -> &'static str {
        match self {
            SearchAction::ShowProvidersInfo => "show providers info",
        }
    }
}

impl From<SearchAction> for Action {
    fn from(value: SearchAction) -> Self {
        match value {
            SearchAction::ShowProvidersInfo => Action::ShowProvidersInfo,
        }
    }
}
