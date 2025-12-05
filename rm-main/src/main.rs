mod cli;
pub mod transmission;
mod tui;

use std::io::stdout;

use clap::Parser;
use color_eyre::Result;
use crossterm::{
    cursor::Show,
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
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
  
    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .issue_filter(|kind| match kind {
            color_eyre::ErrorKind::NonRecoverable(_) => true,
            color_eyre::ErrorKind::Recoverable(_) => false,
        })
        .install()?;

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
    if let Err(e) = app.run().await {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, Show, DisableMouseCapture);
        return Err(e);
    };
    Ok(())
}
