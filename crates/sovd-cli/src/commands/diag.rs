use anyhow::Result;

use crate::output::{self, OutputFormat};
use super::create_client;

pub async fn read_data(
    server: &str,
    token: Option<&str>,
    component: &str,
    data_id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let data = client.read_data(component, data_id).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        OutputFormat::Text => {
            println!("Data: {}", data.id);
            if let Some(name) = &data.name {
                println!("Name: {name}");
            }
            println!("Value: {}", data.value);
            if let Some(unit) = &data.unit {
                println!("Unit: {unit}");
            }
        }
    }

    Ok(())
}

pub async fn read_dtcs(
    server: &str,
    token: Option<&str>,
    component: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let dtcs = client.read_dtcs(component).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&dtcs)?);
        }
        OutputFormat::Text => {
            println!("DTCs for component '{component}' ({} found)", dtcs.len());
            println!();

            let rows: Vec<Vec<String>> = dtcs
                .iter()
                .map(|d| {
                    vec![
                        d.code.clone(),
                        d.status.to_string(),
                        d.severity
                            .as_ref()
                            .map_or("-".into(), |s| s.to_string()),
                        d.description.clone().unwrap_or("-".into()),
                    ]
                })
                .collect();

            output::print_table(&["Code", "Status", "Severity", "Description"], &rows, format);
        }
    }

    Ok(())
}

pub async fn clear_dtcs(
    server: &str,
    token: Option<&str>,
    component: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    client.clear_dtcs(component).await?;

    output::print_status(true, &format!("DTCs cleared on component '{component}'"), format);
    Ok(())
}

pub async fn write_data(
    server: &str,
    token: Option<&str>,
    component: &str,
    data_id: &str,
    value: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;

    let json_value: serde_json::Value = serde_json::from_str(value)
        .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));

    client.write_data(component, data_id, &json_value).await?;

    output::print_status(
        true,
        &format!("Data '{data_id}' written to component '{component}'"),
        format,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_diag() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn create_client_invalid_url() {
        let client = create_client("not a url", None);
        assert!(client.is_err());
    }
}
