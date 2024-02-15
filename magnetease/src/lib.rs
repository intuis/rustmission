pub mod magnetease;
pub mod providers;

use async_trait::async_trait;
use reqwest::Client;

pub struct Magnet {
    pub title: String,
    pub url: String,
}

#[async_trait]
trait Provider {
    async fn search(&self, client: &Client, query: &str) -> Vec<Magnet>;
}
