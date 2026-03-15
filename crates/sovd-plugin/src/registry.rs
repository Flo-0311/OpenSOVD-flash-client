use std::collections::HashMap;
use std::sync::Arc;

use sovd_core::SovdResult;
use tracing::{debug, info, warn};

use crate::spi::{Plugin, PluginType};

/// In-memory registry of loaded plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginRegistry {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin. Replaces any existing plugin with the same name.
    ///
    /// # Errors
    /// Returns `SovdError::Plugin` if registration fails.
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> SovdResult<()> {
        let manifest = plugin.manifest();
        let name = manifest.name.clone();
        info!(
            name = %name,
            version = %manifest.version,
            plugin_type = ?manifest.plugin_type,
            "Registering plugin"
        );
        if self.plugins.contains_key(&name) {
            warn!(name = %name, "Plugin already registered, replacing");
        }
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Unregister a plugin by name.
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn Plugin>> {
        debug!(name = %name, "Unregistering plugin");
        self.plugins.remove(name)
    }

    /// Get a plugin by name.
    #[must_use] 
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(name)
    }

    /// List all registered plugins.
    #[must_use] 
    pub fn list(&self) -> Vec<&Arc<dyn Plugin>> {
        self.plugins.values().collect()
    }

    /// List plugins by type.
    #[must_use] 
    pub fn by_type(&self, plugin_type: &PluginType) -> Vec<&Arc<dyn Plugin>> {
        self.plugins
            .values()
            .filter(|p| &p.manifest().plugin_type == plugin_type)
            .collect()
    }

    /// Number of registered plugins.
    #[must_use] 
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spi::{PluginManifest, PluginType};
    use async_trait::async_trait;

    struct TestPlugin {
        manifest: PluginManifest,
    }

    impl TestPlugin {
        fn new(name: &str, plugin_type: PluginType) -> Self {
            Self {
                manifest: PluginManifest {
                    name: name.into(),
                    version: "1.0.0".into(),
                    description: "Test plugin".into(),
                    plugin_type,
                },
            }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = PluginRegistry::new();
        assert_eq!(reg.count(), 0);
        assert!(reg.list().is_empty());
    }

    #[test]
    fn default_registry_is_empty() {
        let reg = PluginRegistry::default();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn register_and_get() {
        let mut reg = PluginRegistry::new();
        let plugin = Arc::new(TestPlugin::new("my-plugin", PluginType::Security));
        reg.register(plugin).unwrap();
        assert_eq!(reg.count(), 1);
        assert!(reg.get("my-plugin").is_some());
    }

    #[test]
    fn get_returns_none_for_missing() {
        let reg = PluginRegistry::new();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn register_replaces_existing() {
        let mut reg = PluginRegistry::new();
        let p1 = Arc::new(TestPlugin::new("dup", PluginType::Security));
        let p2 = Arc::new(TestPlugin::new("dup", PluginType::Workflow));
        reg.register(p1).unwrap();
        reg.register(p2).unwrap();
        assert_eq!(reg.count(), 1);
        let plugin = reg.get("dup").unwrap();
        assert_eq!(plugin.manifest().plugin_type, PluginType::Workflow);
    }

    #[test]
    fn unregister_removes_plugin() {
        let mut reg = PluginRegistry::new();
        let plugin = Arc::new(TestPlugin::new("removeme", PluginType::Reporting));
        reg.register(plugin).unwrap();
        assert_eq!(reg.count(), 1);
        let removed = reg.unregister("removeme");
        assert!(removed.is_some());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn unregister_returns_none_for_missing() {
        let mut reg = PluginRegistry::new();
        assert!(reg.unregister("nonexistent").is_none());
    }

    #[test]
    fn list_returns_all() {
        let mut reg = PluginRegistry::new();
        reg.register(Arc::new(TestPlugin::new("a", PluginType::Security)))
            .unwrap();
        reg.register(Arc::new(TestPlugin::new("b", PluginType::Workflow)))
            .unwrap();
        reg.register(Arc::new(TestPlugin::new("c", PluginType::Reporting)))
            .unwrap();
        assert_eq!(reg.list().len(), 3);
    }

    #[test]
    fn by_type_filters_correctly() {
        let mut reg = PluginRegistry::new();
        reg.register(Arc::new(TestPlugin::new("sec1", PluginType::Security)))
            .unwrap();
        reg.register(Arc::new(TestPlugin::new("sec2", PluginType::Security)))
            .unwrap();
        reg.register(Arc::new(TestPlugin::new("wf1", PluginType::Workflow)))
            .unwrap();

        let security = reg.by_type(&PluginType::Security);
        assert_eq!(security.len(), 2);

        let workflow = reg.by_type(&PluginType::Workflow);
        assert_eq!(workflow.len(), 1);

        let reporting = reg.by_type(&PluginType::Reporting);
        assert_eq!(reporting.len(), 0);
    }
}
