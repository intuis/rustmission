mod add_torrent;
mod fetch_rss;

use anyhow::Result;
use clap::{Parser, Subcommand};

use add_torrent::add_torrent;
use fetch_rss::fetch_rss;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    AddTorrent { torrent: String },
    FetchRss { url: String, filter: Option<String> },
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::AddTorrent { torrent } => add_torrent(torrent).await?,
        Commands::FetchRss { url, filter } => fetch_rss(&url, filter.as_deref()).await?,
    }
    Ok(())
}
