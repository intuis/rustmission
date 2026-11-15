use std::path::PathBuf;

use base64::Engine;
use color_eyre::{eyre::WrapErr, Result};
use rm_config::CONFIG;
use transmission_rpc::{
    types::{BasicAuth, TorrentAddArgs},
    TransClient,
};

pub fn new_client() -> TransClient {
    let user = CONFIG
        .connection
        .username
        .as_ref()
        .unwrap_or(&"".to_string())
        .clone();
    let password = CONFIG
        .connection
        .password
        .as_ref()
        .unwrap_or(&"".to_string())
        .clone();

    let auth = BasicAuth { user, password };

    TransClient::with_auth(CONFIG.connection.url.clone(), auth)
}

pub fn torrent_add_args(input: &str) -> Result<TorrentAddArgs> {
    let input = input.trim();

    if input.starts_with("magnet:") || input.starts_with("http:") || input.starts_with("https:") {
        return Ok(TorrentAddArgs {
            filename: Some(input.to_string()),
            ..Default::default()
        });
    }

    if input.starts_with("www") {
        return Ok(TorrentAddArgs {
            filename: Some(format!("https://{input}")),
            ..Default::default()
        });
    }

    let path = expand_tilde(input);
    let bytes = std::fs::read(&path)
        .wrap_err_with(|| format!("Cannot read torrent file \"{}\"", path.display()))?;

    Ok(TorrentAddArgs {
        metainfo: Some(base64::engine::general_purpose::STANDARD.encode(bytes)),
        ..Default::default()
    })
}

fn expand_tilde(input: &str) -> PathBuf {
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(input)
}
