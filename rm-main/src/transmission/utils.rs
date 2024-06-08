use rm_config::Config;
use transmission_rpc::{types::BasicAuth, TransClient};

pub fn client_from_config(config: &Config) -> TransClient {
    let user = config
        .connection
        .username
        .as_ref()
        .unwrap_or(&"".to_string())
        .clone();
    let password = config
        .connection
        .password
        .as_ref()
        .unwrap_or(&"".to_string())
        .clone();
    let url = config.connection.url.parse().unwrap();

    let auth = BasicAuth { user, password };

    TransClient::with_auth(url, auth)
}
