use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use sovd_core::SoftwarePackage;
use sovd_observe::{EventRecorder, ReportGenerator};
use sovd_workflow::WorkflowEngine;

use crate::output::{self, OutputFormat};
use super::create_client;

fn make_package(component: &str, package_name: &str, version: &str) -> SoftwarePackage {
    SoftwarePackage {
        id: format!("{package_name}-{version}"),
        name: package_name.to_string(),
        version: version.to_string(),
        target_component: component.to_string(),
        size_bytes: None,
        checksum: None,
        checksum_algorithm: None,
        metadata: None,
    }
}

/// Write a report file if `--report-file` was supplied (L8).
async fn maybe_write_report(
    engine: &WorkflowEngine,
    job_id: &uuid::Uuid,
    recorder: &EventRecorder,
    report_file: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let Some(path_str) = report_file else {
        return Ok(());
    };
    let job = engine.jobs().get_job(job_id).await?;
    let report = ReportGenerator::generate(&job, recorder).await?;
    ReportGenerator::write_json(&report, Path::new(path_str))?;
    output::print_status(true, &format!("Report written to {path_str}"), format);
    Ok(())
}

/// Create a progress bar for flash monitoring (L10).
fn create_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {pos}% {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("#>-"),
    );
    pb.set_message("Flashing...");
    pb
}

#[allow(clippy::too_many_arguments)]
pub async fn start(
    server: &str,
    token: Option<&str>,
    component: &str,
    package_name: &str,
    version: &str,
    report_file: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let recorder = Arc::new(EventRecorder::new());
    let mut engine = WorkflowEngine::new(client, recorder.clone());

    engine.connect().await?;

    let package = make_package(component, package_name, version);

    // L10: progress bar
    let pb = create_progress_bar();
    output::print_status(
        true,
        &format!("Starting flash: {package_name} v{version} -> {component}"),
        format,
    );

    match engine.flash(component, package).await {
        Ok(job_id) => {
            pb.finish_with_message("Done");
            output::print_status(true, &format!("Flash job completed: {job_id}"), format);
            // L8: report export
            maybe_write_report(&engine, &job_id, &recorder, report_file, format).await?;
        }
        Err(e) => {
            pb.finish_with_message("Failed");
            output::print_error(&format!("Flash failed: {e}"), format);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn status(
    server: &str,
    token: Option<&str>,
    component: &str,
    job_id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let client = create_client(server, token)?;
    let result = client.get_flash_status(component, job_id).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            let state = result
                .get("state")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            let progress = result
                .get("progress")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            println!("Job: {job_id}");
            println!("State: {state}");
            println!("Progress: {progress}%");
        }
    }

    Ok(())
}

/// Bulk-flash multiple components with the same package (L9).
#[allow(clippy::too_many_arguments)]
pub async fn bulk(
    server: &str,
    token: Option<&str>,
    components_csv: &str,
    package_name: &str,
    version: &str,
    report_file: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let components: Vec<&str> = components_csv.split(',').map(str::trim).collect();
    let total = components.len();

    output::print_status(
        true,
        &format!("Bulk flash: {package_name} v{version} -> {total} components"),
        format,
    );

    let mut succeeded = 0usize;
    let mut failed = Vec::new();

    for (idx, component) in components.iter().enumerate() {
        let label = format!("[{}/{}] {component}", idx + 1, total);
        output::print_status(true, &format!("{label}: starting..."), format);

        let client = create_client(server, token)?;
        let recorder = Arc::new(EventRecorder::new());
        let mut engine = WorkflowEngine::new(client, recorder.clone());

        if let Err(e) = engine.connect().await {
            output::print_error(&format!("{label}: connect failed: {e}"), format);
            failed.push((*component, e.to_string()));
            continue;
        }

        let package = make_package(component, package_name, version);
        let pb = create_progress_bar();

        match engine.flash(component, package).await {
            Ok(job_id) => {
                pb.finish_with_message("Done");
                output::print_status(true, &format!("{label}: completed ({job_id})"), format);
                succeeded += 1;

                if let Some(rf) = report_file {
                    let per_component = format!("{rf}.{component}.json");
                    let _ = maybe_write_report(
                        &engine,
                        &job_id,
                        &recorder,
                        Some(&per_component),
                        format,
                    )
                    .await;
                }
            }
            Err(e) => {
                pb.finish_with_message("Failed");
                output::print_error(&format!("{label}: failed: {e}"), format);
                failed.push((*component, e.to_string()));
            }
        }
    }

    println!();
    output::print_status(
        failed.is_empty(),
        &format!("Bulk flash complete: {succeeded}/{total} succeeded"),
        format,
    );
    for (comp, err) in &failed {
        output::print_error(&format!("  {comp}: {err}"), format);
    }

    if failed.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "{} of {} components failed",
            failed.len(),
            total
        ))
    }
}

/// Interactive flash wizard using dialoguer (L11).
pub async fn interactive(
    server: &str,
    token: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    use dialoguer::{Confirm, Input, Select};

    output::print_status(true, "Interactive Flash Wizard", format);
    println!();

    // Step 1: discover components
    let client = create_client(server, token)?;
    output::print_status(true, "Discovering components...", format);
    let component_list = client.list_components().await?;

    if component_list.components.is_empty() {
        output::print_error("No components found on the server.", format);
        return Ok(());
    }

    let display_names: Vec<String> = component_list
        .components
        .iter()
        .map(|c| {
            let sw = c.software_version.as_deref().unwrap_or("-");
            format!("{} ({:?}) [SW: {sw}]", c.id, c.component_type)
        })
        .collect();

    let selection = Select::new()
        .with_prompt("Select target component")
        .items(&display_names)
        .default(0)
        .interact()?;

    let component = &component_list.components[selection].id;

    // Step 2: package info
    let package_name: String = Input::new()
        .with_prompt("Software package name")
        .interact_text()?;

    let version: String = Input::new()
        .with_prompt("Software version")
        .interact_text()?;

    // Step 3: confirm
    println!();
    println!("  Component:  {component}");
    println!("  Package:    {package_name}");
    println!("  Version:    {version}");
    println!();

    let confirmed = Confirm::new()
        .with_prompt("Proceed with flash?")
        .default(false)
        .interact()?;

    if !confirmed {
        output::print_status(true, "Flash cancelled by user.", format);
        return Ok(());
    }

    // Step 4: optional report file
    let report_file: String = Input::new()
        .with_prompt("Report file path (leave empty to skip)")
        .allow_empty(true)
        .interact_text()?;
    let report_file_opt = if report_file.is_empty() {
        None
    } else {
        Some(report_file.as_str())
    };

    // Step 5: execute flash
    start(server, token, component, &package_name, &version, report_file_opt, format).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client_for_flash() {
        let client = create_client("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn software_package_construction() {
        let pkg = make_package("ECU_01", "firmware", "1.0");
        assert_eq!(pkg.name, "firmware");
        assert_eq!(pkg.target_component, "ECU_01");
        assert_eq!(pkg.id, "firmware-1.0");
    }

    #[test]
    fn progress_bar_creation() {
        let pb = create_progress_bar();
        pb.finish_with_message("test");
    }

    #[test]
    fn csv_split_components() {
        let csv = "ecu_01, ecu_02, ecu_03";
        let parts: Vec<&str> = csv.split(',').map(str::trim).collect();
        assert_eq!(parts, vec!["ecu_01", "ecu_02", "ecu_03"]);
    }
}
