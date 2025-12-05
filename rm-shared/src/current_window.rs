#[derive(Debug, Clone, Copy)]
pub enum Window {
    Torrents(TorrentWindow),
    Search(SearchWindow),
}

#[derive(Debug, Clone, Copy)]
pub enum TorrentWindow {
    General,
    FileViewer,
}

#[derive(Debug, Clone, Copy)]
pub enum SearchWindow {
    General,
}
