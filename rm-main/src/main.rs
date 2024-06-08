mod action;
pub mod app;
mod transmission;
pub mod tui;
mod ui;
mod utils;

use std::{fs::File, io::Read};

use app::App;
use rm_config::Config;
use utils::trans_client_from_config;

use anyhow::Result;
use base64::Engine;
use clap::{Parser, Subcommand};
use transmission_rpc::types::TorrentAddArgs;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    AddTorrent { torrent: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = Config::init()?;

    if let Some(command) = args.command {
        match command {
            Commands::AddTorrent { torrent } => add_torrent(&config, torrent).await?,
        }
    } else {
        run_tui(config).await?;
    }

    Ok(())
}

async fn add_torrent(config: &Config, torrent: String) -> Result<()> {
    let mut transclient = trans_client_from_config(&config);
    let args = {
        if torrent.starts_with("magnet:")
            || torrent.starts_with("http:")
            || torrent.starts_with("https:")
        {
            TorrentAddArgs {
                filename: Some(torrent),
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

async fn run_tui(config: Config) -> Result<()> {
    let mut app = App::new(config);
    app.run().await?;
    Ok(())
}
