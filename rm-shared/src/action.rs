use std::{error::Error, sync::Arc};

use crossterm::event::KeyEvent;
use magnetease::{MagneteaseError, MagneteaseResult};
use transmission_rpc::types::{FreeSpace, SessionGet, SessionStats, Torrent};

use crate::{current_window::TorrentWindow, status_task::StatusTask};

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
    ScrollUpBy(u8),
    ScrollDownBy(u8),
    ScrollUpPage,
    ScrollDownPage,
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
    MoveToColumnLeft,
    MoveToColumnRight,
    // Torrents Tab
    ShowStats,
    ShowFiles,
    Pause,
    Delete,
    AddMagnet,
    MoveTorrent,
    ChangeCategory,
    ChangeFilePriority,
    Rename,
    // Search Tab
    ShowProvidersInfo,
}

pub enum UpdateAction {
    // General
    SwitchToInputMode,
    SwitchToNormalMode,
    Error(Box<ErrorMessage>),
    UnrecoverableError(Box<color_eyre::eyre::Report>),
    // Torrents Tab
    ChangeTorrentWindow(TorrentWindow),
    SessionStats(Arc<SessionStats>),
    SessionGet(Arc<SessionGet>),
    FreeSpace(Arc<FreeSpace>),
    UpdateTorrents(Vec<Torrent>),
    UpdateCurrentTorrent(Box<Torrent>),
    SearchFilterApply(String),
    SearchFilterClear,
    CancelTorrentTask,
    // Search Tab
    SearchStarted,
    ProviderResult(MagneteaseResult),
    ProviderError(MagneteaseError),
    SearchFinished,
    // Task Manager's Status Task
    StatusTaskClear,
    StatusTaskSuccess,
    StatusTaskFailure,
    StatusTaskSet(StatusTask),
    StatusTaskSetSuccess(StatusTask),
}

#[derive(Debug)]
pub struct ErrorMessage {
    pub title: String,
    pub description: String,
    pub source: Box<dyn Error + Send + Sync + 'static>,
}

impl ErrorMessage {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        error: Box<dyn Error + Send + Sync>,
    ) -> Self {
        Self {
            title: title.into(),
            description: message.into(),
            source: error,
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
