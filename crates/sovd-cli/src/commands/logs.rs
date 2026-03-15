use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn get_logs(
    server: &str,
    token: Option<&str>,
    component: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let data = client.get_logs(component).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        OutputFormat::Text => {
            println!("Logs for component '{component}':");
            if let Some(arr) = data.as_array() {
                for entry in arr {
                    let ts = entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or("-");
                    let level = entry.get("level").and_then(|v| v.as_str()).unwrap_or("INFO");
                    let msg = entry.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    println!("  [{ts}] {level}: {msg}");
                }
                println!("\n{} log entries", arr.len());
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
    }

    Ok(())
}

pub async fn subscribe(
    server: &str,
    token: Option<&str>,
    component: &str,
    level: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;

    let filter = serde_json::json!({
        "level": level.unwrap_or("info"),
    });

    let result = client.subscribe_logs(component, &filter).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            if let Some(sub_id) = result.get("subscription_id").and_then(|v| v.as_str()) {
                output::print_status(
                    true,
                    &format!("Subscribed to logs on '{component}': {sub_id}"),
                    format,
                );
            } else {
                println!("Subscription response:");
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_logs() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }
}
