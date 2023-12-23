pub mod app;
mod components;
pub mod tui;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let result = run().await;

    result?;

    Ok(())
}

async fn run() -> Result<()> {
    let mut app = App::new();
    let res = app.run().await;

    res?;

    Ok(())
}
