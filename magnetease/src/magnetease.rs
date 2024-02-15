use reqwest::Client;

use crate::{providers::knaben::Knaben, Magnet, Provider};

pub struct Magnetease {
    client: Client,
    providers: Vec<Box<dyn Provider>>,
}

impl Magnetease {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            providers: vec![Box::new(Knaben)],
        }
    }
}

impl Magnetease {
    pub async fn search(&self, query: &str) -> Vec<Magnet> {
        let mut results = Vec::new();
        for provider in &self.providers {
            let provider_results = provider.search(&self.client, query).await;
            results.extend(provider_results);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let magnetease = Magnetease::new();
        magnetease.search("minecraft").await;
    }
}
