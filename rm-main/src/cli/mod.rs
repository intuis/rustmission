mod add_torrent;
mod fetch_rss;

use clap::{Parser, Subcommand};
use color_eyre::Result;

use add_torrent::add_torrent;
use fetch_rss::fetch_rss;
use intuitils::config::IntuiConfig;

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
    PrintDefaultConfig {},
    PrintDefaultKeymap {},
    PrintDefaultCategories {},
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::AddTorrent { torrent } => {
            match add_torrent(torrent.clone()).await {
                Ok(_) => tracing::info!("Torrent added: {torrent}"),
                Err(e) => {
                    tracing::error!("Cannot add torrent: {torrent}, {e}");
                    return Err(e);
                }
            }
        },
        Commands::FetchRss { url, filter } => fetch_rss(&url, filter.as_deref()).await?,
        Commands::PrintDefaultConfig {} => {
            tracing::info!("Printing config");
            println!("{}", rm_config::main_config::MainConfig::default_config())
        }
        Commands::PrintDefaultKeymap {} => {
            tracing::info!("Printing key mapping");
            println!("{}", rm_config::keymap::KeymapConfig::default_config())
        }
        Commands::PrintDefaultCategories {} => {
            tracing::info!("Printing categories");
            println!(
                "{}",
                rm_config::categories::CategoriesConfig::default_config()
            )
        }
    }
    Ok(())
}
