use std::path::Path;
use std::sync::Arc;

use sovd_core::{SovdError, SovdResult};
use tracing::{debug, error, info};

use crate::registry::PluginRegistry;
use crate::spi::{
    BackendPlugin, Plugin, ReportingPlugin, SecurityPlugin, WorkflowPlugin,
    PLUGIN_CREATE_SYMBOL,
};

/// Manages plugin lifecycle: discovery, loading, and unloading.
///
/// In addition to the generic [`PluginRegistry`], the manager maintains
/// **typed plugin collections** so that the workflow engine can call
/// sub-trait methods (`SecurityPlugin`, `BackendPlugin`, etc.) without
/// requiring trait-object downcasting.
pub struct PluginManager {
    registry: PluginRegistry,
    /// Holds loaded dynamic libraries to keep them alive.
    loaded_libs: Vec<libloading::Library>,
    /// Typed collections for sub-trait plugins.
    security_plugins: Vec<Arc<dyn SecurityPlugin>>,
    backend_plugins: Vec<Arc<dyn BackendPlugin>>,
    workflow_plugins: Vec<Arc<dyn WorkflowPlugin>>,
    reporting_plugins: Vec<Arc<dyn ReportingPlugin>>,
}

impl PluginManager {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
            loaded_libs: Vec::new(),
            security_plugins: Vec::new(),
            backend_plugins: Vec::new(),
            workflow_plugins: Vec::new(),
            reporting_plugins: Vec::new(),
        }
    }

    /// Get a reference to the plugin registry.
    #[must_use] 
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get a mutable reference to the plugin registry.
    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }

    /// Register a built-in (statically linked) plugin.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if registration fails.
    pub fn register_builtin(&mut self, plugin: Arc<dyn Plugin>) -> SovdResult<()> {
        self.registry.register(plugin)
    }

    // --- Typed plugin registration ---

    /// Register a security plugin (authentication, authorization, signatures).
    pub fn register_security(&mut self, plugin: Arc<dyn SecurityPlugin>) {
        info!(name = %plugin.manifest().name, "Registering security plugin");
        self.security_plugins.push(plugin);
    }

    /// Register a backend integration plugin (OEM backends, CI systems).
    pub fn register_backend(&mut self, plugin: Arc<dyn BackendPlugin>) {
        info!(name = %plugin.manifest().name, "Registering backend plugin");
        self.backend_plugins.push(plugin);
    }

    /// Register a workflow plugin (approval flows, compliance checks).
    pub fn register_workflow(&mut self, plugin: Arc<dyn WorkflowPlugin>) {
        info!(name = %plugin.manifest().name, "Registering workflow plugin");
        self.workflow_plugins.push(plugin);
    }

    /// Register a reporting plugin (custom report formats).
    pub fn register_reporting(&mut self, plugin: Arc<dyn ReportingPlugin>) {
        info!(name = %plugin.manifest().name, "Registering reporting plugin");
        self.reporting_plugins.push(plugin);
    }

    // --- Typed plugin accessors ---

    /// Get all registered security plugins.
    #[must_use]
    pub fn security_plugins(&self) -> &[Arc<dyn SecurityPlugin>] {
        &self.security_plugins
    }

    /// Get all registered backend plugins.
    #[must_use]
    pub fn backend_plugins(&self) -> &[Arc<dyn BackendPlugin>] {
        &self.backend_plugins
    }

    /// Get all registered workflow plugins.
    #[must_use]
    pub fn workflow_plugins(&self) -> &[Arc<dyn WorkflowPlugin>] {
        &self.workflow_plugins
    }

    /// Get all registered reporting plugins.
    #[must_use]
    pub fn reporting_plugins(&self) -> &[Arc<dyn ReportingPlugin>] {
        &self.reporting_plugins
    }

    /// Load a dynamic plugin from a shared library file.
    ///
    /// # Safety
    /// This loads and executes code from an external shared library.
    /// **Only load plugins from trusted sources.** There is currently no
    /// signature or checksum verification (F-12). Consider adding
    /// `SecurityPlugin::verify_signature()` before production use.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if the library cannot be loaded or is missing the create symbol.
    pub unsafe fn load_dynamic(&mut self, path: &Path) -> SovdResult<()> {
        // F-12: Audit trail — log file metadata for post-incident analysis
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        info!(
            path = %path.display(),
            file_size_bytes = file_size,
            "Loading dynamic plugin (WARNING: no signature verification)"
        );

        let lib = libloading::Library::new(path).map_err(|e| {
            SovdError::Plugin(format!("Failed to load library {}: {e}", path.display()))
        })?;

        let create_fn: libloading::Symbol<crate::spi::CreatePluginFn> =
            lib.get(PLUGIN_CREATE_SYMBOL).map_err(|e| {
                SovdError::Plugin(format!(
                    "Plugin {} missing create symbol: {e}",
                    path.display()
                ))
            })?;

        let raw_plugin = create_fn();
        let plugin: Box<dyn Plugin> = Box::from_raw(raw_plugin);
        let plugin: Arc<dyn Plugin> = Arc::from(plugin);

        self.registry.register(plugin)?;
        self.loaded_libs.push(lib); // keep library alive

        Ok(())
    }

    /// Scan a directory for plugin shared libraries and load them.
    ///
    /// # Safety
    /// See `load_dynamic`.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if any plugin in the directory fails to load.
    pub unsafe fn load_from_directory(&mut self, dir: &Path) -> SovdResult<usize> {
        if !dir.exists() {
            debug!(path = %dir.display(), "Plugin directory does not exist, skipping");
            return Ok(0);
        }

        let mut loaded = 0;
        let entries = std::fs::read_dir(dir).map_err(|e| {
            SovdError::Plugin(format!("Cannot read plugin directory: {e}"))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            let is_plugin = path.extension().is_some_and(|ext| {
                ext == "so" || ext == "dylib" || ext == "dll"
            });

            if is_plugin {
                match self.load_dynamic(&path) {
                    Ok(()) => loaded += 1,
                    Err(e) => {
                        error!(path = %path.display(), error = %e, "Failed to load plugin");
                    }
                }
            }
        }

        info!(count = loaded, dir = %dir.display(), "Plugins loaded from directory");
        Ok(loaded)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spi::{PluginManifest, PluginType};
    use async_trait::async_trait;
    use sovd_core::SovdResult;

    struct DummyPlugin {
        manifest: PluginManifest,
    }

    impl DummyPlugin {
        fn new(name: &str) -> Self {
            Self {
                manifest: PluginManifest {
                    name: name.into(),
                    version: "0.1.0".into(),
                    description: "Dummy".into(),
                    plugin_type: PluginType::BackendIntegration,
                },
            }
        }
    }

    #[async_trait]
    impl crate::spi::Plugin for DummyPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[test]
    fn new_manager_has_empty_registry() {
        let mgr = PluginManager::new();
        assert_eq!(mgr.registry().count(), 0);
    }

    #[test]
    fn default_manager_has_empty_registry() {
        let mgr = PluginManager::default();
        assert_eq!(mgr.registry().count(), 0);
    }

    #[test]
    fn register_builtin_plugin() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(DummyPlugin::new("builtin-1"));
        mgr.register_builtin(plugin).unwrap();
        assert_eq!(mgr.registry().count(), 1);
        assert!(mgr.registry().get("builtin-1").is_some());
    }

    #[test]
    fn register_multiple_builtins() {
        let mut mgr = PluginManager::new();
        mgr.register_builtin(Arc::new(DummyPlugin::new("a"))).unwrap();
        mgr.register_builtin(Arc::new(DummyPlugin::new("b"))).unwrap();
        mgr.register_builtin(Arc::new(DummyPlugin::new("c"))).unwrap();
        assert_eq!(mgr.registry().count(), 3);
    }

    #[test]
    fn registry_mut_access() {
        let mut mgr = PluginManager::new();
        mgr.register_builtin(Arc::new(DummyPlugin::new("x"))).unwrap();
        let removed = mgr.registry_mut().unregister("x");
        assert!(removed.is_some());
        assert_eq!(mgr.registry().count(), 0);
    }

    #[test]
    fn load_from_nonexistent_directory() {
        let mut mgr = PluginManager::new();
        let path = Path::new("/tmp/nonexistent_plugin_dir_sovd_test");
        let result = unsafe { mgr.load_from_directory(path) };
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // --- Typed plugin registration tests (L1) ---

    struct MockSecurityPlugin {
        manifest: PluginManifest,
    }

    #[async_trait]
    impl crate::spi::Plugin for MockSecurityPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl crate::spi::SecurityPlugin for MockSecurityPlugin {
        async fn authenticate(&self) -> SovdResult<String> {
            Ok("mock-token".into())
        }
        async fn authorize(&self, _op: &str, _ctx: &serde_json::Value) -> SovdResult<bool> {
            Ok(true)
        }
    }

    struct MockBackendPlugin {
        manifest: PluginManifest,
    }

    #[async_trait]
    impl crate::spi::Plugin for MockBackendPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl crate::spi::BackendPlugin for MockBackendPlugin {
        async fn pre_flash(&self, _job: &sovd_core::Job) -> SovdResult<crate::spi::FlashDecision> {
            Ok(crate::spi::FlashDecision::Proceed)
        }
        async fn post_flash(&self, _job: &sovd_core::Job) -> SovdResult<()> {
            Ok(())
        }
        async fn resolve_package(&self, _id: &str) -> SovdResult<Option<serde_json::Value>> {
            Ok(None)
        }
    }

    struct MockWorkflowPlugin {
        manifest: PluginManifest,
    }

    #[async_trait]
    impl crate::spi::Plugin for MockWorkflowPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl crate::spi::WorkflowPlugin for MockWorkflowPlugin {
        async fn on_phase_change(
            &self,
            _job: &sovd_core::Job,
            _from: &str,
            _to: &str,
        ) -> SovdResult<crate::spi::PhaseDecision> {
            Ok(crate::spi::PhaseDecision::Allow)
        }
    }

    struct MockReportingPlugin {
        manifest: PluginManifest,
    }

    #[async_trait]
    impl crate::spi::Plugin for MockReportingPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[async_trait]
    impl crate::spi::ReportingPlugin for MockReportingPlugin {
        async fn generate_report(
            &self,
            _job: &sovd_core::Job,
        ) -> SovdResult<crate::spi::ReportOutput> {
            Ok(crate::spi::ReportOutput {
                format: crate::spi::ReportFormat::Json,
                content: b"{}".to_vec(),
                filename: "report.json".into(),
            })
        }
    }

    fn make_manifest(name: &str, pt: PluginType) -> PluginManifest {
        PluginManifest {
            name: name.into(),
            version: "1.0.0".into(),
            description: "Mock".into(),
            plugin_type: pt,
        }
    }

    #[test]
    fn typed_collections_empty_initially() {
        let mgr = PluginManager::new();
        assert!(mgr.security_plugins().is_empty());
        assert!(mgr.backend_plugins().is_empty());
        assert!(mgr.workflow_plugins().is_empty());
        assert!(mgr.reporting_plugins().is_empty());
    }

    #[test]
    fn register_and_access_security_plugin() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(MockSecurityPlugin {
            manifest: make_manifest("sec-1", PluginType::Security),
        });
        mgr.register_security(plugin);
        assert_eq!(mgr.security_plugins().len(), 1);
        assert_eq!(mgr.security_plugins()[0].manifest().name, "sec-1");
    }

    #[test]
    fn register_and_access_backend_plugin() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(MockBackendPlugin {
            manifest: make_manifest("be-1", PluginType::BackendIntegration),
        });
        mgr.register_backend(plugin);
        assert_eq!(mgr.backend_plugins().len(), 1);
        assert_eq!(mgr.backend_plugins()[0].manifest().name, "be-1");
    }

    #[test]
    fn register_and_access_workflow_plugin() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(MockWorkflowPlugin {
            manifest: make_manifest("wf-1", PluginType::Workflow),
        });
        mgr.register_workflow(plugin);
        assert_eq!(mgr.workflow_plugins().len(), 1);
    }

    #[test]
    fn register_and_access_reporting_plugin() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(MockReportingPlugin {
            manifest: make_manifest("rpt-1", PluginType::Reporting),
        });
        mgr.register_reporting(plugin);
        assert_eq!(mgr.reporting_plugins().len(), 1);
    }

    #[test]
    fn register_multiple_typed_plugins() {
        let mut mgr = PluginManager::new();
        mgr.register_security(Arc::new(MockSecurityPlugin {
            manifest: make_manifest("sec-a", PluginType::Security),
        }));
        mgr.register_security(Arc::new(MockSecurityPlugin {
            manifest: make_manifest("sec-b", PluginType::Security),
        }));
        mgr.register_backend(Arc::new(MockBackendPlugin {
            manifest: make_manifest("be-a", PluginType::BackendIntegration),
        }));
        assert_eq!(mgr.security_plugins().len(), 2);
        assert_eq!(mgr.backend_plugins().len(), 1);
        assert!(mgr.workflow_plugins().is_empty());
        assert!(mgr.reporting_plugins().is_empty());
    }
}
