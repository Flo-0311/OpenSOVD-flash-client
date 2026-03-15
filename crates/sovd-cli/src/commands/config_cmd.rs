use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn read_config(
    server: &str,
    token: Option<&str>,
    component: &str,
    config_id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let data = client.read_config(component, config_id).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        OutputFormat::Text => {
            println!("Config '{config_id}' on component '{component}':");
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
    }

    Ok(())
}

pub async fn write_config(
    server: &str,
    token: Option<&str>,
    component: &str,
    config_id: &str,
    value: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;

    let json_value: serde_json::Value = serde_json::from_str(value)
        .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));

    client.write_config(component, config_id, &json_value).await?;

    output::print_status(
        true,
        &format!("Config '{config_id}' written to component '{component}'"),
        format,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_config() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }
}
