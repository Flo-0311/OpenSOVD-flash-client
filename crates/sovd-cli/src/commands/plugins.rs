use std::path::Path;

use anyhow::Result;
use sovd_plugin::PluginManager;

use crate::output::{self, OutputFormat};

#[allow(clippy::unused_async)]
pub async fn list(format: &OutputFormat) -> Result<()> {
    let manager = PluginManager::new();
    let count = manager.registry().count();
    output::print_status(true, &format!("{count} plugins loaded"), format);

    for plugin in manager.registry().list() {
        let m = plugin.manifest();
        println!("  {} v{} [{:?}] - {}", m.name, m.version, m.plugin_type, m.description);
    }

    Ok(())
}

#[allow(clippy::unused_async)]
pub async fn load(dir: &str, format: &OutputFormat) -> Result<()> {
    let mut manager = PluginManager::new();
    let path = Path::new(dir);

    let loaded = unsafe { manager.load_from_directory(path)? };
    output::print_status(true, &format!("{loaded} plugins loaded from {dir}"), format);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_plugin_manager_is_empty() {
        let manager = PluginManager::new();
        assert_eq!(manager.registry().count(), 0);
    }

    #[test]
    fn list_on_empty_registry() {
        let manager = PluginManager::new();
        assert!(manager.registry().list().is_empty());
    }
}
