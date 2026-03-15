use async_trait::async_trait;
use serde_json::Value;
use sovd_core::{Job, SovdResult};

/// Metadata describing a plugin.
#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginType {
    Security,
    BackendIntegration,
    Workflow,
    Reporting,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Security => write!(f, "Security"),
            PluginType::BackendIntegration => write!(f, "Backend Integration"),
            PluginType::Workflow => write!(f, "Workflow"),
            PluginType::Reporting => write!(f, "Reporting"),
        }
    }
}

// --- Plugin Trait Definitions (SPI) ---

/// Base trait that all plugins must implement.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Return the plugin manifest.
    fn manifest(&self) -> &PluginManifest;

    /// Called when the plugin is loaded.
    async fn on_load(&mut self) -> SovdResult<()> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    async fn on_unload(&mut self) -> SovdResult<()> {
        Ok(())
    }
}

/// Security plugin: authentication, authorization, certificates.
#[async_trait]
pub trait SecurityPlugin: Plugin {
    /// Provide an authentication token for the SOVD server.
    async fn authenticate(&self) -> SovdResult<String>;

    /// Check if an operation is authorized.
    async fn authorize(&self, operation: &str, context: &Value) -> SovdResult<bool>;

    /// Verify a signature or certificate (optional).
    async fn verify_signature(&self, _data: &[u8], _signature: &[u8]) -> SovdResult<bool> {
        Ok(true)
    }
}

/// Backend integration plugin: OEM backends, CI systems, legacy adapters.
#[async_trait]
pub trait BackendPlugin: Plugin {
    /// Called before a flash job starts. Can modify the job or abort.
    async fn pre_flash(&self, job: &Job) -> SovdResult<FlashDecision>;

    /// Called after a flash job completes.
    async fn post_flash(&self, job: &Job) -> SovdResult<()>;

    /// Retrieve software packages from an external backend.
    async fn resolve_package(&self, component_id: &str) -> SovdResult<Option<Value>>;
}

/// Decision returned by a backend plugin before flashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashDecision {
    Proceed,
    Abort(String),
    RequireApproval(String),
}

/// Workflow plugin: custom approval flows, OEM-specific UX, compliance.
#[async_trait]
pub trait WorkflowPlugin: Plugin {
    /// Called at each job phase transition. Can block or approve.
    async fn on_phase_change(
        &self,
        job: &Job,
        from: &str,
        to: &str,
    ) -> SovdResult<PhaseDecision>;

    /// Called when a job completes to allow custom post-processing.
    async fn on_job_complete(&self, job: &Job) -> SovdResult<()> {
        let _ = job;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhaseDecision {
    Allow,
    Deny(String),
    WaitForApproval(String),
}

/// Reporting plugin: produces audit output, test reports, etc.
#[async_trait]
pub trait ReportingPlugin: Plugin {
    /// Generate a report for a completed job.
    async fn generate_report(&self, job: &Job) -> SovdResult<ReportOutput>;
}

#[derive(Debug, Clone)]
pub struct ReportOutput {
    pub format: ReportFormat,
    pub content: Vec<u8>,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Html,
    Pdf,
    Xml,
    Text,
}

// --- FFI function signatures for dynamic loading ---

/// The function signature that dynamic plugins must export.
/// `extern "C" fn create_plugin() -> *mut dyn Plugin`
#[allow(improper_ctypes_definitions)]
pub type CreatePluginFn = unsafe extern "C" fn() -> *mut dyn Plugin;

/// The symbol name that dynamic plugins must export.
pub const PLUGIN_CREATE_SYMBOL: &[u8] = b"_create_plugin";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_manifest_creation() {
        let manifest = PluginManifest {
            name: "test-plugin".into(),
            version: "1.0.0".into(),
            description: "A test plugin".into(),
            plugin_type: PluginType::Security,
        };
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.plugin_type, PluginType::Security);
    }

    #[test]
    fn plugin_type_equality() {
        assert_eq!(PluginType::Security, PluginType::Security);
        assert_ne!(PluginType::Security, PluginType::Workflow);
        assert_ne!(PluginType::BackendIntegration, PluginType::Reporting);
    }

    #[test]
    fn all_plugin_types_exist() {
        let types = [
            PluginType::Security,
            PluginType::BackendIntegration,
            PluginType::Workflow,
            PluginType::Reporting,
        ];
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn flash_decision_variants() {
        let proceed = FlashDecision::Proceed;
        assert_eq!(proceed, FlashDecision::Proceed);

        let abort = FlashDecision::Abort("reason".into());
        assert_eq!(abort, FlashDecision::Abort("reason".into()));

        let approval = FlashDecision::RequireApproval("needs sign-off".into());
        assert_eq!(
            approval,
            FlashDecision::RequireApproval("needs sign-off".into())
        );
    }

    #[test]
    fn phase_decision_variants() {
        assert_eq!(PhaseDecision::Allow, PhaseDecision::Allow);
        assert_eq!(
            PhaseDecision::Deny("blocked".into()),
            PhaseDecision::Deny("blocked".into())
        );
        assert_eq!(
            PhaseDecision::WaitForApproval("pending".into()),
            PhaseDecision::WaitForApproval("pending".into())
        );
    }

    #[test]
    fn report_output_creation() {
        let output = ReportOutput {
            format: ReportFormat::Json,
            content: b"{}".to_vec(),
            filename: "report.json".into(),
        };
        assert_eq!(output.format, ReportFormat::Json);
        assert_eq!(output.filename, "report.json");
        assert_eq!(output.content, b"{}");
    }

    #[test]
    fn report_format_equality() {
        assert_eq!(ReportFormat::Json, ReportFormat::Json);
        assert_ne!(ReportFormat::Json, ReportFormat::Html);
        assert_ne!(ReportFormat::Pdf, ReportFormat::Xml);
        assert_ne!(ReportFormat::Text, ReportFormat::Html);
    }

    #[test]
    fn all_report_formats_exist() {
        let formats = [
            ReportFormat::Json,
            ReportFormat::Html,
            ReportFormat::Pdf,
            ReportFormat::Xml,
            ReportFormat::Text,
        ];
        assert_eq!(formats.len(), 5);
    }

    #[test]
    fn plugin_create_symbol_value() {
        assert_eq!(PLUGIN_CREATE_SYMBOL, b"_create_plugin");
    }

    #[test]
    fn plugin_manifest_clone() {
        let manifest = PluginManifest {
            name: "clone-test".into(),
            version: "0.1.0".into(),
            description: "Clone test".into(),
            plugin_type: PluginType::BackendIntegration,
        };
        let cloned = manifest.clone();
        assert_eq!(cloned.name, manifest.name);
        assert_eq!(cloned.plugin_type, manifest.plugin_type);
    }
}
