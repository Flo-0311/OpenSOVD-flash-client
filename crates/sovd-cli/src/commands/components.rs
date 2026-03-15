use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn run(server: &str, token: Option<&str>, format: &OutputFormat) -> Result<()> {
    let client = create_client(server, token)?;
    let components = client.list_components().await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&components)?);
        }
        OutputFormat::Text => {
            println!("ECU Components ({} found)", components.components.len());
            println!();

            let native = components.native_sovd().len();
            let classic = components.classic_uds().len();
            println!("  Native SOVD (HPC): {native}  |  Classic UDS (via CDA): {classic}");
            println!();

            let rows: Vec<Vec<String>> = components
                .components
                .iter()
                .map(|c| {
                    vec![
                        c.id.clone(),
                        c.name.clone(),
                        format!("{:?}", c.component_type),
                        format!("{:?}", c.status),
                        c.software_version.clone().unwrap_or("-".into()),
                        c.hardware_version.clone().unwrap_or("-".into()),
                    ]
                })
                .collect();

            output::print_table(
                &["ID", "Name", "Type", "Status", "SW Version", "HW Version"],
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
    fn create_client_for_components() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn create_client_with_token_for_components() {
        let client = create_client("http://sovd-server:9090", Some("tok"));
        assert!(client.is_ok());
    }
}
