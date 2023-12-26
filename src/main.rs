pub mod app;
mod components;
pub mod tui;

use anyhow::Result;
use app::App;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut app = App::new();
    app.run().await?;

    Ok(())
}
