use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{Magnet, Provider};

const API_URL: &str = "https://api.knaben.eu/v1";

pub struct Knaben;

#[async_trait]
impl Provider for Knaben {
    async fn search(&self, client: &Client, query: &str) -> Vec<Magnet> {
        let json_data = serde_json::json!({
            "search_type": "100%",
            "search_field": "title",
            "query": query,
            "order_by": "seeders",
            "order_direction": "desc",
            "size": 150
        })
        .to_string();

        let res = client
            .post(API_URL)
            .body(json_data)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap()
            .json::<KnabenAPIResult>()
            .await
            .unwrap();

        res.hits
            .into_iter()
            .filter_map(|x| Magnet::try_from(x).ok())
            .collect()
    }
}

impl TryFrom<KnabenMagnet> for Magnet {
    type Error = ();

    fn try_from(value: KnabenMagnet) -> Result<Self, Self::Error> {
        match value {
            KnabenMagnet {
                title,
                magnet_url: Some(url),
            } => Ok(Self { title, url }),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct KnabenAPIResult {
    hits: Vec<KnabenMagnet>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnabenMagnet {
    magnet_url: Option<String>,
    title: String,
}
