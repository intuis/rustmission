use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct Connection {
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Url,
    #[serde(default = "default_refresh")]
    pub torrents_refresh: u64,
    #[serde(default = "default_refresh")]
    pub stats_refresh: u64,
    #[serde(default = "default_refresh")]
    pub free_space_refresh: u64,
}

fn default_refresh() -> u64 {
    5
}
