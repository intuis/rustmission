use std::{fs::File, io::Read};

use anyhow::Result;
use base64::Engine;
use clap::{Parser, Subcommand};
use rm_config::Config;
use transmission_rpc::types::TorrentAddArgs;

use crate::transmission;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    AddTorrent { torrent: String },
}

pub async fn handle_command(config: &Config, command: Commands) -> Result<()> {
    match command {
        Commands::AddTorrent { torrent } => add_torrent(config, torrent).await?,
    }
    Ok(())
}

async fn add_torrent(config: &Config, torrent: String) -> Result<()> {
    let mut transclient = transmission::utils::client_from_config(&config);
    let args = {
        if torrent.starts_with("magnet:")
            || torrent.starts_with("http:")
            || torrent.starts_with("https:")
        {
            TorrentAddArgs {
                filename: Some(torrent),
                ..Default::default()
            }
        } else if torrent.starts_with("www") {
            TorrentAddArgs {
                filename: Some(format!("https://{torrent}")),
                ..Default::default()
            }
        } else {
            let mut torrent_file = File::open(torrent)?;
            let mut buf = vec![];
            torrent_file.read_to_end(&mut buf).unwrap();
            let metainfo = base64::engine::general_purpose::STANDARD.encode(buf);
            TorrentAddArgs {
                metainfo: Some(metainfo),
                ..Default::default()
            }
        }
    };

    if let Err(e) = transclient.torrent_add(args).await {
        eprintln!("error while adding a torrent: {e}");
        if e.to_string().contains("expected value at line") {
            eprintln!("Check whether your arguments are valid.");
        }

        std::process::exit(1);
    };
    Ok(())
}
