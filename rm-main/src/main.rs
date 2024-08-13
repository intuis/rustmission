mod cli;
pub mod transmission;
mod tui;

use anyhow::Result;
use clap::Parser;
use tui::app::App;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    if let Some(command) = args.command {
        cli::handle_command(command).await?;
    } else {
        run_tui().await?;
    }

    Ok(())
}

async fn run_tui() -> Result<()> {
    let mut app = App::new().await?;
    app.run().await?;
    Ok(())
}
