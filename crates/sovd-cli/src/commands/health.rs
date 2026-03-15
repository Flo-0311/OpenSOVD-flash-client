use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn run(server: &str, token: Option<&str>, format: &OutputFormat) -> Result<()> {
    let client = create_client(server, token)?;
    let healthy = client.health().await?;

    output::print_status(
        healthy,
        &format!("SOVD server at {server} is {}", if healthy { "reachable" } else { "unreachable" }),
        format,
    );

    if !healthy {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_health() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn create_client_for_health_with_token() {
        let client = create_client("http://localhost:8080", Some("test-token"));
        assert!(client.is_ok());
    }
}
