use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn run(server: &str, token: Option<&str>, format: &OutputFormat) -> Result<()> {
    let mut client = create_client(server, token)?;
    let caps = client.connect().await?.clone();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&caps)?);
        }
        OutputFormat::Text => {
            let resolver = client.resolver()?;
            let summary = resolver.summary();
            println!("SOVD Server Capabilities");
            println!("========================");
            if let Some(v) = resolver.sovd_version() {
                println!("SOVD Version: {v}");
            }
            println!("{summary}");
            println!();

            let rows: Vec<Vec<String>> = caps
                .capabilities
                .iter()
                .map(|c| {
                    vec![
                        c.id.clone(),
                        c.category.to_string(),
                        c.name.clone(),
                        c.description.clone().unwrap_or_default(),
                    ]
                })
                .collect();

            output::print_table(
                &["ID", "Category", "Name", "Description"],
                &rows,
                format,
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_capabilities() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn create_client_with_token_for_capabilities() {
        let client = create_client("http://localhost:9090", Some("bearer-xyz"));
        assert!(client.is_ok());
    }
}
