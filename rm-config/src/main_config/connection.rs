use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

#[derive(Deserialize)]
pub struct Connection {
    pub username: Option<String>,
    pub password: Option<String>,
    pub credentials_file: Option<PathBuf>,
    pub url: Url,
    #[serde(default = "default_refresh")]
    pub torrents_refresh: u64,
    #[serde(default = "default_refresh")]
    pub stats_refresh: u64,
    #[serde(default = "default_refresh")]
    pub free_space_refresh: u64,
}

fn default_refresh() -> u64 {
    5
}

impl Connection {
    // If `credentials_file` is set, read it and populate `username`/`password`
    // where they are not already provided in the main config.
    pub fn load_credentials_from_file(&mut self) -> Result<()> {
        // Don't allow both inline username/password and a credentials file.
        if self.credentials_file.is_some() && (self.username.is_some() || self.password.is_some()) {
            return Err(anyhow!(
                "cannot specify both `credentials_file` and `username`/`password`"
            ));
        }
        if let Some(ref path) = self.credentials_file {
            if !path.exists() {
                return Err(anyhow!("credentials file not found: {}", path.display()));
            }
            let content = std::fs::read_to_string(path)?;
            let creds: Credentials = toml::from_str(&content)?;
            self.username = Some(creds.username);
            self.password = Some(creds.password);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
