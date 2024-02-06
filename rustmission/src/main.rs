pub mod app;
mod components;
mod transmission;
pub mod tui;

use anyhow::Result;
use app::App;
use rm_config::Config;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Config::init()?;

    let mut app = App::new(&config).await;
    app.run().await?;

    Ok(())
}
