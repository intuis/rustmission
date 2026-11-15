use color_eyre::Result;

use crate::transmission;

pub(super) async fn add_torrent(torrent: String) -> Result<()> {
    let mut transclient = transmission::utils::new_client();
    let args = transmission::utils::torrent_add_args(&torrent)?;

    if let Err(e) = transclient.torrent_add(args).await {
        eprintln!("error while adding a torrent: {e}");
        if e.to_string().contains("expected value at line") {
            eprintln!("Check whether your arguments are valid.");
        }

        std::process::exit(1);
    };
    Ok(())
}
