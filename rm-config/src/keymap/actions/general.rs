use rm_shared::action::Action;
use serde::{Deserialize, Serialize};

use super::UserAction;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeneralAction {
    ShowHelp,
    Quit,
    Close,
    SwitchToTorrents,
    SwitchToSearch,
    Left,
    Right,
    Down,
    Up,
    Search,
    SwitchFocus,
    Confirm,
    Select,
    ScrollPageDown,
    ScrollPageUp,
    GoToBeginning,
    GoToEnd,
    XdgOpen,
    MoveToColumnLeft,
    MoveToColumnRight,
}

pub enum GeneralActionMergable {
    MoveUpDown,
    MoveLeftRight,
    ScrollPageUpDown,
    MoveColumnLeftRight,
    SwitchToTorrentsSearch,
}

impl UserAction for GeneralAction {
    fn desc(&self) -> &'static str {
        match self {
            GeneralAction::ShowHelp => "toggle help",
            GeneralAction::Quit => "quit Rustmission, a popup",
            GeneralAction::Close => "close a popup, a task",
            GeneralAction::SwitchToTorrents => "switch to torrents tab",
            GeneralAction::SwitchToSearch => "switch to search tab",
            GeneralAction::Left => "switch to tab left",
            GeneralAction::Right => "switch to tab right",
            GeneralAction::Down => "move down",
            GeneralAction::Up => "move up",
            GeneralAction::Search => "search",
            GeneralAction::SwitchFocus => "switch focus",
            GeneralAction::Confirm => "confirm",
            GeneralAction::Select => "select",
            GeneralAction::ScrollPageDown => "scroll page down",
            GeneralAction::ScrollPageUp => "scroll page up",
            GeneralAction::GoToBeginning => "scroll to beginning",
            GeneralAction::GoToEnd => "scroll to end",
            GeneralAction::XdgOpen => "open with xdg-open",
            GeneralAction::MoveToColumnRight => "move to right column (sorting)",
            GeneralAction::MoveToColumnLeft => "move to left column (sorting)",
        }
    }

    fn merge_desc_with(&self, other: &GeneralAction) -> Option<&'static str> {
        match (&self, other) {
            (Self::Left, Self::Right) => Some("switch to tab left / right"),
            (Self::Right, Self::Left) => Some("switch to tab right / left"),
            (Self::Down, Self::Up) => Some("move down / up"),
            (Self::Up, Self::Down) => Some("move up / down"),
            (Self::SwitchToTorrents, Self::SwitchToSearch) => {
                Some("switch to torrents / search tab")
            }
            (Self::SwitchToSearch, Self::SwitchToTorrents) => {
                Some("switch to search / torrents tab")
            }
            (Self::MoveToColumnLeft, Self::MoveToColumnRight) => {
                Some("move to column left / right")
            }
            (Self::MoveToColumnRight, Self::MoveToColumnLeft) => {
                Some("move to column right / left")
            }
            (Self::ScrollPageDown, Self::ScrollPageUp) => Some("scroll page down / up"),
            (Self::ScrollPageUp, Self::ScrollPageDown) => Some("scroll page up / down"),
            (Self::GoToBeginning, Self::GoToEnd) => Some("go to beginning / end"),
            (Self::GoToEnd, Self::GoToBeginning) => Some("go to end / beginning"),

            _ => None,
        }
    }
}

impl From<GeneralAction> for Action {
    fn from(value: GeneralAction) -> Self {
        match value {
            GeneralAction::ShowHelp => Action::ShowHelp,
            GeneralAction::Quit => Action::Quit,
            GeneralAction::Close => Action::Close,
            GeneralAction::SwitchToTorrents => Action::ChangeTab(1),
            GeneralAction::SwitchToSearch => Action::ChangeTab(2),
            GeneralAction::Left => Action::Left,
            GeneralAction::Right => Action::Right,
            GeneralAction::Down => Action::Down,
            GeneralAction::Up => Action::Up,
            GeneralAction::Search => Action::Search,
            GeneralAction::SwitchFocus => Action::ChangeFocus,
            GeneralAction::Confirm => Action::Confirm,
            GeneralAction::Select => Action::Select,
            GeneralAction::ScrollPageDown => Action::ScrollDownPage,
            GeneralAction::ScrollPageUp => Action::ScrollUpPage,
            GeneralAction::GoToBeginning => Action::Home,
            GeneralAction::GoToEnd => Action::End,
            GeneralAction::XdgOpen => Action::XdgOpen,
            GeneralAction::MoveToColumnLeft => Action::MoveToColumnLeft,
            GeneralAction::MoveToColumnRight => Action::MoveToColumnRight,
        }
    }
}
