use async_trait::async_trait;
use sovd_core::{Job, SovdResult};
use sovd_plugin::spi::{
    BackendPlugin, FlashDecision, Plugin, PluginManifest, PluginType,
};
use tracing::info;

/// Example plugin demonstrating the `OpenSOVD` plugin SPI.
///
/// This plugin logs flash events and always approves operations.
/// Use it as a template for building real OEM plugins.
pub struct ExamplePlugin {
    manifest: PluginManifest,
}

impl ExamplePlugin {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                name: "example-plugin".into(),
                version: "0.1.0".into(),
                description: "Example plugin for demonstration purposes".into(),
                plugin_type: PluginType::BackendIntegration,
            },
        }
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn on_load(&mut self) -> SovdResult<()> {
        info!("Example plugin loaded");
        Ok(())
    }

    async fn on_unload(&mut self) -> SovdResult<()> {
        info!("Example plugin unloaded");
        Ok(())
    }
}

#[async_trait]
impl BackendPlugin for ExamplePlugin {
    async fn pre_flash(&self, job: &Job) -> SovdResult<FlashDecision> {
        info!(
            job_id = %job.id,
            component = %job.target_component,
            "Example plugin: pre-flash check passed"
        );
        Ok(FlashDecision::Proceed)
    }

    async fn post_flash(&self, job: &Job) -> SovdResult<()> {
        info!(
            job_id = %job.id,
            state = ?job.state,
            "Example plugin: post-flash hook executed"
        );
        Ok(())
    }

    async fn resolve_package(
        &self,
        component_id: &str,
    ) -> SovdResult<Option<serde_json::Value>> {
        info!(component = %component_id, "Example plugin: no package resolution configured");
        Ok(None)
    }
}

/// FFI entry point for dynamic loading.
///
/// # Safety
/// Called by the plugin manager via libloading.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn _create_plugin() -> *mut dyn Plugin {
    let plugin = ExamplePlugin::new();
    let boxed: Box<dyn Plugin> = Box::new(plugin);
    Box::into_raw(boxed)
}
