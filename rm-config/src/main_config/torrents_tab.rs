use rm_shared::header::Header;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TorrentsTab {
    #[serde(default = "default_headers")]
    pub headers: Vec<Header>,
    #[serde(default = "default_true")]
    pub category_icon_insert_into_name: bool,
}

fn default_true() -> bool {
    true
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
            category_icon_insert_into_name: default_true(),
        }
    }
}
