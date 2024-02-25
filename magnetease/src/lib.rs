pub mod magnetease;
pub mod providers;

use async_trait::async_trait;
use reqwest::Client;

#[derive(Clone)]
pub struct Magnet {
    pub title: String,
    pub url: String,
    pub seeders: u32,
    pub bytes: u64,
}

#[async_trait]
trait Provider {
    async fn search(&self, client: &Client, query: &str) -> Vec<Magnet>;
}
