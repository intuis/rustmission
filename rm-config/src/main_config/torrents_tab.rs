use rm_shared::header::Header;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TorrentsTab {
    #[serde(default = "default_headers")]
    pub headers: Vec<Header>,
    #[serde(default = "default_sort")]
    pub default_sort: Header,
    #[serde(default = "default_true")]
    pub default_sort_reverse: bool,
    #[serde(default = "default_true")]
    pub category_icon_insert_into_name: bool,
}

fn default_true() -> bool {
    true
}

fn default_sort() -> Header {
    Header::AddedDate
}

fn default_headers() -> Vec<Header> {
    vec![
        Header::Name,
        Header::SizeWhenDone,
        Header::Progress,
        Header::Eta,
        Header::DownloadRate,
        Header::UploadRate,
    ]
}

impl Default for TorrentsTab {
    fn default() -> Self {
        Self {
            headers: default_headers(),
            default_sort: default_sort(),
            default_sort_reverse: default_true(),
            category_icon_insert_into_name: default_true(),
        }
    }
}
