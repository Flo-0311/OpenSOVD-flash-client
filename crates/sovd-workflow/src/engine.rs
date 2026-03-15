use std::sync::Arc;

use sovd_client::SovdClient;
use sovd_core::{
    ComponentList, DataValue, DiagnosticTroubleCode, JobType,
    SoftwarePackage, SovdError, SovdResult,
};
use sovd_observe::EventRecorder;
use sovd_plugin::{
    FlashDecision, PhaseDecision, PluginManager, ReportOutput,
};
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::controller::JobController;

/// High-level workflow engine that orchestrates diagnostics and flashing.
///
/// This is the main entry point for all user-initiated operations.
/// It delegates work to the `JobController`, SOVD client, and **plugins**.
///
/// Plugin hooks are called at the following points:
/// - **`SecurityPlugin`** — `authenticate()` during `connect()` to obtain auth tokens
/// - **`BackendPlugin`** — `pre_flash()` / `post_flash()` around the flash lifecycle
/// - **`WorkflowPlugin`** — `on_phase_change()` at start/completion, `on_job_complete()`
/// - **`ReportingPlugin`** — `generate_report()` after a flash job completes
pub struct WorkflowEngine {
    client: SovdClient,
    controller: JobController,
    plugin_manager: PluginManager,
    recorder: Arc<EventRecorder>,
}

impl WorkflowEngine {
    #[must_use] 
    pub fn new(client: SovdClient, recorder: Arc<EventRecorder>) -> Self {
        let controller = JobController::new(recorder.clone());
        Self {
            client,
            controller,
            plugin_manager: PluginManager::new(),
            recorder,
        }
    }

    /// Get a reference to the SOVD client.
    #[must_use] 
    pub fn client(&self) -> &SovdClient {
        &self.client
    }

    /// Get a mutable reference to the SOVD client.
    pub fn client_mut(&mut self) -> &mut SovdClient {
        &mut self.client
    }

    /// Get a reference to the plugin manager.
    #[must_use] 
    pub fn plugins(&self) -> &PluginManager {
        &self.plugin_manager
    }

    /// Get a mutable reference to the plugin manager.
    pub fn plugins_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }

    /// Get a reference to the job controller.
    #[must_use] 
    pub fn jobs(&self) -> &JobController {
        &self.controller
    }

    // --- Security plugin integration (L2) ---

    /// Authenticate via registered security plugins and set the token on the client.
    ///
    /// If multiple security plugins are registered, the first successful
    /// authentication token is used.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if all security plugins fail to authenticate.
    async fn authenticate_via_plugins(&mut self) -> SovdResult<()> {
        let plugins = self.plugin_manager.security_plugins();
        if plugins.is_empty() {
            return Ok(());
        }

        info!(count = plugins.len(), "Authenticating via security plugins");
        for plugin in plugins {
            match plugin.authenticate().await {
                Ok(token) => {
                    info!(
                        plugin = %plugin.manifest().name,
                        "Authentication token obtained"
                    );
                    self.client.set_auth_token(token);
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        plugin = %plugin.manifest().name,
                        error = %e,
                        "Security plugin authentication failed, trying next"
                    );
                }
            }
        }

        Err(SovdError::Plugin(
            "All security plugins failed to authenticate".into(),
        ))
    }

    /// Connect to the SOVD server and discover capabilities.
    ///
    /// If security plugins are registered, they are called first to
    /// obtain an authentication token via `SecurityPlugin::authenticate()`.
    ///
    /// # Errors
    /// Returns `SovdError` if authentication fails, the server is unreachable,
    /// or capabilities cannot be discovered.
    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> SovdResult<()> {
        // L2: Authenticate via security plugins before connecting
        self.authenticate_via_plugins().await?;

        info!("Connecting to SOVD server...");
        let caps = self.client.connect().await?;
        let sovd_version = caps.sovd_version.clone();
        let cap_count = caps.capabilities.len();

        let resolver = self.client.resolver()?;
        let summary = resolver.summary();
        info!(%summary, "Connected to SOVD server");

        self.recorder
            .record_event(
                "connected",
                &serde_json::json!({
                    "sovd_version": sovd_version,
                    "capabilities": cap_count,
                }),
            )
            .await;

        Ok(())
    }

    /// List all ECU components.
    ///
    /// # Errors
    /// Returns `SovdError` if the HTTP request fails.
    #[instrument(skip(self))]
    pub async fn list_components(&self) -> SovdResult<ComponentList> {
        self.client.list_components().await
    }

    /// Read a diagnostic data value from a component.
    ///
    /// # Errors
    /// Returns `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn read_data(
        &self,
        component_id: &str,
        data_id: &str,
    ) -> SovdResult<DataValue> {
        self.client.read_data(component_id, data_id).await
    }

    /// Write a diagnostic data value (DID) to a component.
    ///
    /// # Errors
    /// Returns `SovdError` if the HTTP request fails.
    #[instrument(skip(self, value))]
    pub async fn write_data(
        &self,
        component_id: &str,
        data_id: &str,
        value: &serde_json::Value,
    ) -> SovdResult<()> {
        self.client.write_data(component_id, data_id, value).await
    }

    /// Read DTCs from a component.
    ///
    /// # Errors
    /// Returns `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn read_dtcs(
        &self,
        component_id: &str,
    ) -> SovdResult<Vec<DiagnosticTroubleCode>> {
        self.client.read_dtcs(component_id).await
    }

    /// Clear DTCs on a component.
    ///
    /// # Errors
    /// Returns `SovdError` if the HTTP request fails.
    #[instrument(skip(self))]
    pub async fn clear_dtcs(&self, component_id: &str) -> SovdResult<()> {
        self.client.clear_dtcs(component_id).await
    }

    // --- Backend plugin hooks ---

    /// Call `BackendPlugin::pre_flash()` on all registered backend plugins.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if any plugin returns `FlashDecision::Abort`.
    async fn run_backend_pre_flash(
        &self,
        job: &sovd_core::Job,
    ) -> SovdResult<()> {
        for plugin in self.plugin_manager.backend_plugins() {
            let decision = plugin.pre_flash(job).await.map_err(|e| {
                SovdError::Plugin(format!(
                    "Backend plugin '{}' pre_flash failed: {e}",
                    plugin.manifest().name
                ))
            })?;

            match decision {
                FlashDecision::Proceed => {
                    info!(
                        plugin = %plugin.manifest().name,
                        "Backend plugin approved flash"
                    );
                }
                FlashDecision::Abort(reason) => {
                    warn!(
                        plugin = %plugin.manifest().name,
                        reason = %reason,
                        "Backend plugin aborted flash"
                    );
                    return Err(SovdError::Plugin(format!(
                        "Flash aborted by plugin '{}': {reason}",
                        plugin.manifest().name
                    )));
                }
                FlashDecision::RequireApproval(reason) => {
                    // For now, log and continue. A full implementation
                    // would pause and wait for external approval.
                    warn!(
                        plugin = %plugin.manifest().name,
                        reason = %reason,
                        "Backend plugin requests approval (auto-approved in current implementation)"
                    );
                }
            }
        }
        Ok(())
    }

    /// Call `BackendPlugin::post_flash()` on all registered backend plugins.
    async fn run_backend_post_flash(&self, job: &sovd_core::Job) {
        for plugin in self.plugin_manager.backend_plugins() {
            if let Err(e) = plugin.post_flash(job).await {
                warn!(
                    plugin = %plugin.manifest().name,
                    error = %e,
                    "Backend plugin post_flash failed"
                );
            }
        }
    }

    // --- Workflow plugin hooks ---

    /// Call `WorkflowPlugin::on_phase_change()` on all registered workflow plugins.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if any plugin denies the phase change.
    async fn run_workflow_phase_change(
        &self,
        job: &sovd_core::Job,
        from: &str,
        to: &str,
    ) -> SovdResult<()> {
        for plugin in self.plugin_manager.workflow_plugins() {
            let decision = plugin.on_phase_change(job, from, to).await.map_err(|e| {
                SovdError::Plugin(format!(
                    "Workflow plugin '{}' on_phase_change failed: {e}",
                    plugin.manifest().name
                ))
            })?;

            match decision {
                PhaseDecision::Allow => {}
                PhaseDecision::Deny(reason) => {
                    warn!(
                        plugin = %plugin.manifest().name,
                        from = %from,
                        to = %to,
                        reason = %reason,
                        "Workflow plugin denied phase change"
                    );
                    return Err(SovdError::Plugin(format!(
                        "Phase change denied by plugin '{}': {reason}",
                        plugin.manifest().name
                    )));
                }
                PhaseDecision::WaitForApproval(reason) => {
                    warn!(
                        plugin = %plugin.manifest().name,
                        reason = %reason,
                        "Workflow plugin requests approval (auto-approved in current implementation)"
                    );
                }
            }
        }
        Ok(())
    }

    /// Call `WorkflowPlugin::on_job_complete()` on all registered workflow plugins.
    async fn run_workflow_job_complete(&self, job: &sovd_core::Job) {
        for plugin in self.plugin_manager.workflow_plugins() {
            if let Err(e) = plugin.on_job_complete(job).await {
                warn!(
                    plugin = %plugin.manifest().name,
                    error = %e,
                    "Workflow plugin on_job_complete failed"
                );
            }
        }
    }

    // --- Reporting plugin hooks ---

    /// Call `ReportingPlugin::generate_report()` on all registered reporting plugins.
    ///
    /// Returns all successfully generated reports.
    async fn run_reporting_plugins(
        &self,
        job: &sovd_core::Job,
    ) -> Vec<ReportOutput> {
        let mut reports = Vec::new();
        for plugin in self.plugin_manager.reporting_plugins() {
            match plugin.generate_report(job).await {
                Ok(report) => {
                    info!(
                        plugin = %plugin.manifest().name,
                        format = ?report.format,
                        filename = %report.filename,
                        "Report generated by plugin"
                    );
                    reports.push(report);
                }
                Err(e) => {
                    warn!(
                        plugin = %plugin.manifest().name,
                        error = %e,
                        "Reporting plugin failed"
                    );
                }
            }
        }
        reports
    }

    // --- Main flash workflow ---

    /// Execute a flash workflow for a single component.
    ///
    /// The full plugin lifecycle is:
    /// 1. `BackendPlugin::pre_flash()` — approve/abort
    /// 2. `WorkflowPlugin::on_phase_change()` — gate "pending" → "running"
    /// 3. Core flash execution (5 phases via `JobController`)
    /// 4. `WorkflowPlugin::on_phase_change()` — notify "running" → "completed"
    /// 5. `ReportingPlugin::generate_report()`
    /// 6. `WorkflowPlugin::on_job_complete()`
    /// 7. `BackendPlugin::post_flash()`
    ///
    /// # Errors
    /// Returns `SovdError::CapabilityNotAvailable` if flashing is unsupported,
    /// `SovdError::Plugin` if a plugin aborts/denies, or other `SovdError` on failure.
    #[instrument(skip(self, package))]
    pub async fn flash(
        &self,
        component_id: &str,
        package: SoftwarePackage,
    ) -> SovdResult<Uuid> {
        // Verify flashing is supported
        if let Some(caps) = self.client.capabilities() {
            if !caps.supports_flashing() {
                return Err(SovdError::CapabilityNotAvailable(
                    "Flashing not supported by this SOVD server".into(),
                ));
            }
        }

        info!(
            component = %component_id,
            package = %package.name,
            version = %package.version,
            "Starting flash workflow"
        );

        // Create the job
        let job_id = self
            .controller
            .create_job(JobType::Flash, component_id.to_string())
            .await?;

        let job = self.controller.get_job(&job_id).await?;

        // L1: Backend plugin pre-flash hooks
        if let Err(e) = self.run_backend_pre_flash(&job).await {
            // Record the abort and fail the job
            self.recorder
                .record_event(
                    "flash_aborted_by_plugin",
                    &serde_json::json!({
                        "job_id": job_id.to_string(),
                        "error": e.to_string(),
                    }),
                )
                .await;
            return Err(e);
        }

        // L1: Workflow plugin phase gate: pending → running
        self.run_workflow_phase_change(&job, "pending", "running")
            .await?;

        // Core flash execution (all 5 phases)
        let flash_result = self
            .controller
            .execute_flash(&self.client, &job_id, &package)
            .await;

        // Get the final job state for plugin hooks
        let final_job = self.controller.get_job(&job_id).await
            .unwrap_or(job);

        match &flash_result {
            Ok(()) => {
                // L1: Workflow plugin phase gate: running → completed
                if let Err(e) = self
                    .run_workflow_phase_change(&final_job, "running", "completed")
                    .await
                {
                    warn!(error = %e, "Workflow plugin denied completion notification");
                }

                // L1: Reporting plugins
                let _reports = self.run_reporting_plugins(&final_job).await;

                // L1: Workflow plugin on_job_complete
                self.run_workflow_job_complete(&final_job).await;
            }
            Err(e) => {
                // Notify plugins about failure
                if let Err(pe) = self
                    .run_workflow_phase_change(&final_job, "running", "failed")
                    .await
                {
                    warn!(error = %pe, "Workflow plugin denied failure notification");
                }
                warn!(job_id = %job_id, error = %e, "Flash execution failed");
            }
        }

        // L1: Backend plugin post-flash hooks (always called)
        self.run_backend_post_flash(&final_job).await;

        flash_result.map(|()| job_id)
    }

    /// Check server health.
    ///
    /// # Errors
    /// Returns `SovdError` on unexpected internal errors; connection failures return `Ok(false)`.
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> SovdResult<bool> {
        self.client.health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> WorkflowEngine {
        let client = SovdClient::new("http://localhost:8080").unwrap();
        let recorder = Arc::new(EventRecorder::new());
        WorkflowEngine::new(client, recorder)
    }

    #[test]
    fn engine_construction() {
        let engine = make_engine();
        assert_eq!(engine.client().base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn engine_client_accessor() {
        let engine = make_engine();
        assert!(engine.client().capabilities().is_none());
    }

    #[test]
    fn engine_client_mut_accessor() {
        let mut engine = make_engine();
        let _client = engine.client_mut();
    }

    #[test]
    fn engine_plugins_accessor() {
        let engine = make_engine();
        assert_eq!(engine.plugins().registry().count(), 0);
    }

    #[test]
    fn engine_plugins_mut_accessor() {
        let mut engine = make_engine();
        assert_eq!(engine.plugins_mut().registry().count(), 0);
    }

    #[test]
    fn engine_jobs_accessor() {
        let engine = make_engine();
        let _jobs = engine.jobs();
    }

    #[tokio::test]
    async fn engine_jobs_empty_initially() {
        let engine = make_engine();
        let jobs = engine.jobs().list_jobs().await;
        assert!(jobs.is_empty());
    }

    // --- L1: Plugin hook tests ---

    use async_trait::async_trait;
    use sovd_plugin::{
        BackendPlugin, FlashDecision, PhaseDecision, Plugin, PluginManifest, PluginType,
        ReportFormat, ReportOutput, ReportingPlugin, SecurityPlugin, WorkflowPlugin,
    };

    struct MockSecPlugin {
        manifest: PluginManifest,
    }

    impl MockSecPlugin {
        fn new() -> Self {
            Self {
                manifest: PluginManifest {
                    name: "mock-sec".into(),
                    version: "1.0".into(),
                    description: "test".into(),
                    plugin_type: PluginType::Security,
                },
            }
        }
    }

    #[async_trait]
    impl Plugin for MockSecPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl SecurityPlugin for MockSecPlugin {
        async fn authenticate(&self) -> SovdResult<String> {
            Ok("test-token-123".into())
        }
        async fn authorize(&self, _op: &str, _ctx: &serde_json::Value) -> SovdResult<bool> {
            Ok(true)
        }
    }

    struct MockBackend {
        manifest: PluginManifest,
        decision: FlashDecision,
    }

    impl MockBackend {
        fn new(decision: FlashDecision) -> Self {
            Self {
                manifest: PluginManifest {
                    name: "mock-backend".into(),
                    version: "1.0".into(),
                    description: "test".into(),
                    plugin_type: PluginType::BackendIntegration,
                },
                decision,
            }
        }
    }

    #[async_trait]
    impl Plugin for MockBackend {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl BackendPlugin for MockBackend {
        async fn pre_flash(&self, _job: &sovd_core::Job) -> SovdResult<FlashDecision> {
            Ok(self.decision.clone())
        }
        async fn post_flash(&self, _job: &sovd_core::Job) -> SovdResult<()> {
            Ok(())
        }
        async fn resolve_package(&self, _id: &str) -> SovdResult<Option<serde_json::Value>> {
            Ok(None)
        }
    }

    struct MockWfPlugin {
        manifest: PluginManifest,
        decision: PhaseDecision,
    }

    impl MockWfPlugin {
        fn new(decision: PhaseDecision) -> Self {
            Self {
                manifest: PluginManifest {
                    name: "mock-wf".into(),
                    version: "1.0".into(),
                    description: "test".into(),
                    plugin_type: PluginType::Workflow,
                },
                decision,
            }
        }
    }

    #[async_trait]
    impl Plugin for MockWfPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl WorkflowPlugin for MockWfPlugin {
        async fn on_phase_change(
            &self,
            _job: &sovd_core::Job,
            _from: &str,
            _to: &str,
        ) -> SovdResult<PhaseDecision> {
            Ok(self.decision.clone())
        }
    }

    struct MockReporter {
        manifest: PluginManifest,
    }

    impl MockReporter {
        fn new() -> Self {
            Self {
                manifest: PluginManifest {
                    name: "mock-reporter".into(),
                    version: "1.0".into(),
                    description: "test".into(),
                    plugin_type: PluginType::Reporting,
                },
            }
        }
    }

    #[async_trait]
    impl Plugin for MockReporter {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl ReportingPlugin for MockReporter {
        async fn generate_report(
            &self,
            _job: &sovd_core::Job,
        ) -> SovdResult<ReportOutput> {
            Ok(ReportOutput {
                format: ReportFormat::Json,
                content: b"{\"status\":\"ok\"}".to_vec(),
                filename: "test-report.json".into(),
            })
        }
    }

    fn make_test_job() -> sovd_core::Job {
        sovd_core::Job::new(JobType::Flash, "ecu_01".into())
    }

    #[test]
    fn engine_typed_plugin_registration() {
        let mut engine = make_engine();
        engine.plugins_mut().register_security(Arc::new(MockSecPlugin::new()));
        engine.plugins_mut().register_backend(Arc::new(MockBackend::new(FlashDecision::Proceed)));
        engine.plugins_mut().register_workflow(Arc::new(MockWfPlugin::new(PhaseDecision::Allow)));
        engine.plugins_mut().register_reporting(Arc::new(MockReporter::new()));

        assert_eq!(engine.plugins().security_plugins().len(), 1);
        assert_eq!(engine.plugins().backend_plugins().len(), 1);
        assert_eq!(engine.plugins().workflow_plugins().len(), 1);
        assert_eq!(engine.plugins().reporting_plugins().len(), 1);
    }

    #[tokio::test]
    async fn engine_backend_pre_flash_proceed() {
        let mut engine = make_engine();
        engine.plugins_mut().register_backend(Arc::new(MockBackend::new(FlashDecision::Proceed)));
        let job = make_test_job();
        let result = engine.run_backend_pre_flash(&job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn engine_backend_pre_flash_abort() {
        let mut engine = make_engine();
        engine.plugins_mut().register_backend(Arc::new(MockBackend::new(FlashDecision::Abort("not allowed".into()))));
        let job = make_test_job();
        let result = engine.run_backend_pre_flash(&job).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not allowed"));
    }

    #[tokio::test]
    async fn engine_backend_pre_flash_require_approval() {
        let mut engine = make_engine();
        engine.plugins_mut().register_backend(Arc::new(MockBackend::new(FlashDecision::RequireApproval("needs sign-off".into()))));
        let job = make_test_job();
        // RequireApproval is auto-approved in current implementation
        let result = engine.run_backend_pre_flash(&job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn engine_backend_post_flash_runs() {
        let mut engine = make_engine();
        engine.plugins_mut().register_backend(Arc::new(MockBackend::new(FlashDecision::Proceed)));
        let job = make_test_job();
        // Should not panic
        engine.run_backend_post_flash(&job).await;
    }

    #[tokio::test]
    async fn engine_workflow_phase_change_allow() {
        let mut engine = make_engine();
        engine.plugins_mut().register_workflow(Arc::new(MockWfPlugin::new(PhaseDecision::Allow)));
        let job = make_test_job();
        let result = engine.run_workflow_phase_change(&job, "pending", "running").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn engine_workflow_phase_change_deny() {
        let mut engine = make_engine();
        engine.plugins_mut().register_workflow(Arc::new(MockWfPlugin::new(PhaseDecision::Deny("blocked".into()))));
        let job = make_test_job();
        let result = engine.run_workflow_phase_change(&job, "pending", "running").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked"));
    }

    #[tokio::test]
    async fn engine_workflow_job_complete_runs() {
        let mut engine = make_engine();
        engine.plugins_mut().register_workflow(Arc::new(MockWfPlugin::new(PhaseDecision::Allow)));
        let job = make_test_job();
        engine.run_workflow_job_complete(&job).await;
    }

    #[tokio::test]
    async fn engine_reporting_generates_reports() {
        let mut engine = make_engine();
        engine.plugins_mut().register_reporting(Arc::new(MockReporter::new()));
        let job = make_test_job();
        let reports = engine.run_reporting_plugins(&job).await;
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].format, ReportFormat::Json);
        assert_eq!(reports[0].filename, "test-report.json");
    }

    #[tokio::test]
    async fn engine_no_plugins_hooks_are_noop() {
        let engine = make_engine();
        let job = make_test_job();
        // All hooks should succeed with no plugins registered
        assert!(engine.run_backend_pre_flash(&job).await.is_ok());
        engine.run_backend_post_flash(&job).await;
        assert!(engine.run_workflow_phase_change(&job, "a", "b").await.is_ok());
        engine.run_workflow_job_complete(&job).await;
        let reports = engine.run_reporting_plugins(&job).await;
        assert!(reports.is_empty());
    }

    #[tokio::test]
    async fn engine_authenticate_via_plugins_sets_token() {
        let mut engine = make_engine();
        engine.plugins_mut().register_security(Arc::new(MockSecPlugin::new()));
        let result = engine.authenticate_via_plugins().await;
        assert!(result.is_ok());
        // Token was set on the client (we can't read it back, but no panic = success)
    }

    #[tokio::test]
    async fn engine_authenticate_noop_without_plugins() {
        let mut engine = make_engine();
        let result = engine.authenticate_via_plugins().await;
        assert!(result.is_ok());
    }
}
