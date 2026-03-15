pub mod capabilities;
pub mod components;
pub mod config_cmd;
pub mod diag;
pub mod flash;
pub mod health;
pub mod jobs;
pub mod logs;
pub mod monitor;
pub mod plugins;

use sovd_client::SovdClient;
use sovd_core::SovdResult;

/// Create a SOVD client from CLI arguments.
pub fn create_client(server: &str, token: Option<&str>) -> SovdResult<SovdClient> {
    let client = SovdClient::new(server)?;
    Ok(match token {
        Some(t) => client.with_auth_token(t.to_string()),
        None => client,
    })
}
