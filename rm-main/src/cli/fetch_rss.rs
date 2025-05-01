use color_eyre::{eyre::bail, Result};
use regex::Regex;
use transmission_rpc::types::TorrentAddArgs;

use crate::transmission;

pub async fn fetch_rss(url: &str, filter: Option<&str>) -> Result<()> {
    let mut transclient = transmission::utils::new_client();
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;
    let re: Option<Regex> = {
        if let Some(filter_str) = filter {
            let res = Regex::new(filter_str)?;
            Some(res)
        } else {
            None
        }
    };
    let items = channel.items().iter().filter_map(|item| {
        if let (Some(title), Some(url)) = (item.title(), item.link()) {
            if let Some(re) = &re {
                if re.is_match(title) {
                    return Some((title, url));
                }
            } else {
                return Some((title, url));
            }
        }
        None
    });
    for (title, url) in items {
        println!("downloading {title}");
        let args = TorrentAddArgs {
            filename: Some(url.to_string()),
            ..Default::default()
        };
        if let Err(e) = transclient.torrent_add(args).await {
            bail!("error while adding a torrent: {e}")
        }
    }
    Ok(())
}
