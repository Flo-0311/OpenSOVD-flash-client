use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn live_data(
    server: &str,
    token: Option<&str>,
    component: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let data = client.get_live_data(component).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        OutputFormat::Text => {
            println!("Live data for component '{component}':");
            if let Some(obj) = data.as_object() {
                for (key, val) in obj {
                    println!("  {key}: {val}");
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
    }

    Ok(())
}

pub async fn parameter(
    server: &str,
    token: Option<&str>,
    component: &str,
    parameter_id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let data = client.get_monitoring_parameter(component, parameter_id).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        OutputFormat::Text => {
            println!("Parameter '{parameter_id}' on '{component}':");
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
    }

    Ok(())
}

pub async fn watch(
    server: &str,
    token: Option<&str>,
    component: &str,
    parameter_id: &str,
    interval_ms: u64,
    count: Option<u64>,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let mut iteration = 0u64;

    loop {
        let data = client.get_monitoring_parameter(component, parameter_id).await?;

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            OutputFormat::Text => {
                let ts = chrono::Utc::now().format("%H:%M:%S%.3f");
                println!("[{ts}] {parameter_id}: {data}");
            }
        }

        iteration += 1;
        if let Some(max) = count {
            if iteration >= max {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
    }

    output::print_status(true, &format!("Watched {iteration} samples"), format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_monitor() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }
}
