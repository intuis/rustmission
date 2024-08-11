use std::{error::Error, sync::Arc};

use crossterm::event::KeyEvent;
use magnetease::{MagneteaseError, MagneteaseResult};
use transmission_rpc::types::{FreeSpace, SessionStats, Torrent};

use crate::status_task::StatusTask;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // General
    HardQuit,
    Quit,
    Close,
    Render,
    Up,
    Down,
    Left,
    Right,
    ScrollDownPage,
    ScrollUpPage,
    Home,
    End,
    Confirm,
    Select,
    ShowHelp,
    Search,
    ChangeFocus,
    ChangeTab(u8),
    XdgOpen,
    Input(KeyEvent),
    // Torrents Tab
    ShowStats,
    ShowFiles,
    Pause,
    DeleteWithoutFiles,
    DeleteWithFiles,
    AddMagnet,
    MoveTorrent,
    // Search Tab
    ShowProvidersInfo,
}

pub enum UpdateAction {
    // General
    SwitchToInputMode,
    SwitchToNormalMode,
    Error(Box<ErrorMessage>),
    // Torrents Tab
    TaskClear,
    TaskSuccess,
    TaskFailure,
    TaskSet(StatusTask),
    TaskSetSuccess(StatusTask),
    SessionStats(Arc<SessionStats>),
    FreeSpace(Arc<FreeSpace>),
    UpdateTorrents(Vec<Torrent>),
    UpdateCurrentTorrent(Box<Torrent>),
    SearchFilterApply(String),
    SearchFilterClear,
    // Search Tab
    SearchStarted,
    ProviderResult(MagneteaseResult),
    ProviderError(MagneteaseError),
    SearchFinished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMessage {
    pub title: String,
    pub description: String,
    pub source: String,
}

impl ErrorMessage {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        error: Box<dyn Error>,
    ) -> Self {
        Self {
            title: title.into(),
            description: message.into(),
            source: error.to_string(),
        }
    }
}

impl Action {
    pub fn is_render(&self) -> bool {
        *self == Self::Render
    }

    pub fn is_hard_quit(&self) -> bool {
        *self == Self::HardQuit
    }

    pub fn is_quit(&self) -> bool {
        *self == Self::HardQuit || *self == Self::Quit
    }

    pub fn is_soft_quit(&self) -> bool {
        self.is_quit() || *self == Self::Close
    }
}
