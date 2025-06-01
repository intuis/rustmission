mod cli;
pub mod transmission;
mod tui;

use anyhow::Result;
use clap::Parser;
use tui::app::App;

#[tokio::main()]
async fn main() -> Result<()> {
    // configure logger
    if let Some(writer) = rm_config::logging::get_log_file() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_level(true)
            .with_writer(writer)
            .with_ansi(false)
            .init();
    }

    let args = cli::Args::parse();

    if let Some(command) = args.command {
        cli::handle_command(command).await?;
    } else {
        tracing::info!("Starting rustmission");
        run_tui().await?;
    }

    Ok(())
}

async fn run_tui() -> Result<()> {
    let app = App::new().await?;
    app.run().await?;
    Ok(())
}
