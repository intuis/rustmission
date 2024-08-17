use rm_shared::header::Header;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TorrentsTab {
    #[serde(default = "default_headers")]
    pub headers: Vec<Header>,
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
        }
    }
}
