use crate::state::AppState;
use serde::{Deserialize, Serialize};
use sovd_client::SovdClient;
use sovd_workflow::WorkflowEngine;
use tauri::State;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Serde DTOs for the frontend (mirrors TypeScript types)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySummaryDto {
    pub total: usize,
    pub flashing: usize,
    pub diagnostics: usize,
    pub fault_management: usize,
    pub configuration: usize,
    pub provisioning: usize,
    pub monitoring: usize,
    pub logging: usize,
    pub bulk: usize,
    pub other: usize,
    pub sovd_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDto {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub status: String,
    pub software_version: Option<String>,
    pub hardware_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDto {
    pub id: String,
    pub job_type: String,
    pub target_component: String,
    pub state: String,
    pub phase: String,
    pub progress_percent: Option<u8>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtcDto {
    pub id: String,
    pub code: String,
    pub description: Option<String>,
    pub status: String,
    pub severity: Option<String>,
    pub component_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValueDto {
    pub id: String,
    pub name: Option<String>,
    pub value: serde_json::Value,
    pub unit: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwarePackageDto {
    pub name: String,
    pub version: String,
    pub target_component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfoDto {
    pub name: String,
    pub version: String,
    pub plugin_type: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn component_to_dto(c: &sovd_core::Component) -> ComponentDto {
    ComponentDto {
        id: c.id.clone(),
        name: c.name.clone(),
        component_type: c.component_type.to_string(),
        status: c.status.to_string(),
        software_version: c.software_version.clone(),
        hardware_version: c.hardware_version.clone(),
    }
}

fn job_to_dto(j: &sovd_core::Job) -> JobDto {
    JobDto {
        id: j.id.to_string(),
        job_type: j.job_type.to_string(),
        target_component: j.target_component.clone(),
        state: j.state.to_string(),
        phase: j.phase.to_string(),
        progress_percent: j.progress_percent,
        error_message: j.error.clone(),
        created_at: j.created_at.to_rfc3339(),
        updated_at: j.updated_at.to_rfc3339(),
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn connect_to_server(
    state: State<'_, AppState>,
    url: String,
    token: Option<String>,
) -> Result<CapabilitySummaryDto, String> {
    let mut client = SovdClient::new(&url).map_err(|e| e.to_string())?;

    if let Some(t) = token {
        client = client.with_auth_token(t);
    }

    // Connect & discover capabilities
    let caps = client.connect().await.map_err(|e| e.to_string())?;
    let sovd_version = caps.sovd_version.clone();

    let resolver = client.resolver().map_err(|e| e.to_string())?;
    let s = resolver.summary();

    let summary = CapabilitySummaryDto {
        total: s.total,
        flashing: s.flashing,
        diagnostics: s.diagnostics,
        fault_management: s.fault_management,
        configuration: s.configuration,
        provisioning: s.provisioning,
        monitoring: s.monitoring,
        logging: s.logging,
        bulk: s.bulk,
        other: s.other,
        sovd_version,
    };

    let recorder = state.recorder.clone();
    let engine = WorkflowEngine::new(client, recorder);
    *state.engine.lock().await = Some(engine);

    Ok(summary)
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    *state.engine.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn health_check(state: State<'_, AppState>) -> Result<bool, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    engine.health_check().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_components(state: State<'_, AppState>) -> Result<Vec<ComponentDto>, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    let comp_list = engine.list_components().await.map_err(|e| e.to_string())?;

    Ok(comp_list.components.iter().map(component_to_dto).collect())
}

#[tauri::command]
pub async fn get_component(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<ComponentDto, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    let c = engine
        .client()
        .get_component(&component_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(component_to_dto(&c))
}

#[tauri::command]
pub async fn start_flash(
    state: State<'_, AppState>,
    component_id: String,
    pkg: SoftwarePackageDto,
) -> Result<String, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;

    let package = sovd_core::SoftwarePackage {
        id: Uuid::new_v4().to_string(),
        name: pkg.name,
        version: pkg.version,
        target_component: component_id.clone(),
        size_bytes: None,
        checksum: None,
        checksum_algorithm: None,
        metadata: None,
    };

    let job_id = engine
        .flash(&component_id, package)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_id.to_string())
}

#[tauri::command]
pub async fn list_jobs(state: State<'_, AppState>) -> Result<Vec<JobDto>, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;

    let jobs = engine.jobs().list_jobs().await;
    Ok(jobs.iter().map(job_to_dto).collect())
}

#[tauri::command]
pub async fn get_job(state: State<'_, AppState>, job_id: String) -> Result<JobDto, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;

    let uuid = Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    let j = engine
        .jobs()
        .get_job(&uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_to_dto(&j))
}

#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;

    let uuid = Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    engine
        .jobs()
        .cancel_job(&uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_dtcs(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<Vec<DtcDto>, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    let dtcs = engine
        .read_dtcs(&component_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(dtcs
        .iter()
        .map(|d| DtcDto {
            id: d.id.clone(),
            code: d.code.clone(),
            description: d.description.clone(),
            status: d.status.to_string(),
            severity: d.severity.as_ref().map(|s| s.to_string()),
            component_id: d.component_id.clone(),
        })
        .collect())
}

#[tauri::command]
pub async fn clear_dtcs(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<(), String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    engine
        .clear_dtcs(&component_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_data(
    state: State<'_, AppState>,
    component_id: String,
    data_id: String,
) -> Result<DataValueDto, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    let val = engine
        .read_data(&component_id, &data_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DataValueDto {
        id: val.id.clone(),
        name: val.name.clone(),
        value: val.value.clone(),
        unit: val.unit.clone(),
        timestamp: val.timestamp.clone(),
    })
}

#[tauri::command]
pub async fn write_data(
    state: State<'_, AppState>,
    component_id: String,
    data_id: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    engine
        .write_data(&component_id, &data_id, &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_data(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<serde_json::Value, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    engine
        .client()
        .get_live_data(&component_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<serde_json::Value, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    engine
        .client()
        .get_logs(&component_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfoDto>, String> {
    let lock = state.engine.lock().await;
    let engine = lock.as_ref().ok_or("Not connected")?;
    let registry = engine.plugins().registry();

    Ok(registry
        .list()
        .iter()
        .map(|p| {
            let m = p.manifest();
            PluginInfoDto {
                name: m.name.clone(),
                version: m.version.clone(),
                plugin_type: m.plugin_type.to_string(),
                description: m.description.clone(),
            }
        })
        .collect())
}
