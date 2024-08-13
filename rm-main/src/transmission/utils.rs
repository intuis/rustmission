use rm_config::CONFIG;
use transmission_rpc::{types::BasicAuth, TransClient};

pub fn new_client() -> TransClient {
    let user = CONFIG
        .connection
        .username
        .as_ref()
        .unwrap_or(&"".to_string())
        .clone();
    let password = CONFIG
        .connection
        .password
        .as_ref()
        .unwrap_or(&"".to_string())
        .clone();

    let auth = BasicAuth { user, password };

    TransClient::with_auth(CONFIG.connection.url.clone(), auth)
}
