use sovd_core::{Capability, CapabilityCategory, CapabilitySet, SovdError, SovdResult};
use tracing::debug;

/// Resolves and interprets SOVD capabilities.
///
/// This is the central component for capability-driven workflows:
/// it determines what the server supports and which operations
/// are available for a given component.
pub struct CapabilityResolver {
    caps: CapabilitySet,
}

impl CapabilityResolver {
    #[must_use] 
    pub fn new(caps: CapabilitySet) -> Self {
        Self { caps }
    }

    /// Get all capabilities.
    #[must_use] 
    pub fn all(&self) -> &[Capability] {
        &self.caps.capabilities
    }

    /// Get the SOVD version reported by the server.
    #[must_use] 
    pub fn sovd_version(&self) -> Option<&str> {
        self.caps.sovd_version.as_deref()
    }

    /// Check if a specific capability is available.
    #[must_use] 
    pub fn has(&self, id: &str) -> bool {
        self.caps.has_capability(id)
    }

    /// Require a capability or return an error.
    ///
    /// # Errors
    /// Returns `SovdError::CapabilityNotAvailable` if the capability is not present.
    pub fn require(&self, id: &str) -> SovdResult<&Capability> {
        self.caps
            .by_id(id)
            .ok_or_else(|| SovdError::CapabilityNotAvailable(id.to_string()))
    }

    /// Get all flashing capabilities.
    #[must_use] 
    pub fn flash_capabilities(&self) -> Vec<&Capability> {
        self.caps.by_category(&CapabilityCategory::Flashing)
    }

    /// Get all diagnostic capabilities.
    #[must_use] 
    pub fn diagnostic_capabilities(&self) -> Vec<&Capability> {
        self.caps.by_category(&CapabilityCategory::Diagnostics)
    }

    /// Get all fault management capabilities (DTC, Fault Library).
    #[must_use] 
    pub fn fault_management_capabilities(&self) -> Vec<&Capability> {
        self.caps.by_category(&CapabilityCategory::FaultManagement)
    }

    /// Get all logging capabilities.
    #[must_use] 
    pub fn logging_capabilities(&self) -> Vec<&Capability> {
        self.caps.by_category(&CapabilityCategory::Logging)
    }

    /// Check if the server supports flashing.
    #[must_use] 
    pub fn supports_flashing(&self) -> bool {
        self.caps.supports_flashing()
    }

    /// Check if the server supports diagnostics.
    #[must_use] 
    pub fn supports_diagnostics(&self) -> bool {
        self.caps.supports_diagnostics()
    }

    /// Get a summary of available capabilities for display.
    #[must_use]
    pub fn summary(&self) -> CapabilitySummary {
        let mut summary = CapabilitySummary::default();
        for cap in &self.caps.capabilities {
            match &cap.category {
                CapabilityCategory::Flashing => summary.flashing += 1,
                CapabilityCategory::Diagnostics => summary.diagnostics += 1,
                CapabilityCategory::FaultManagement => summary.fault_management += 1,
                CapabilityCategory::Configuration => summary.configuration += 1,
                CapabilityCategory::Provisioning => summary.provisioning += 1,
                CapabilityCategory::Monitoring => summary.monitoring += 1,
                CapabilityCategory::Logging => summary.logging += 1,
                CapabilityCategory::Bulk => summary.bulk += 1,
                CapabilityCategory::Other(_) => summary.other += 1,
            }
        }
        summary.total = self.caps.capabilities.len();
        debug!(?summary, "Capability summary");
        summary
    }
}

#[derive(Debug, Default)]
pub struct CapabilitySummary {
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
}

impl std::fmt::Display for CapabilitySummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Capabilities: {} total (flashing={}, diagnostics={}, fault_mgmt={}, config={}, provisioning={}, monitoring={}, logging={}, bulk={}, other={})",
            self.total, self.flashing, self.diagnostics, self.fault_management,
            self.configuration, self.provisioning, self.monitoring, self.logging,
            self.bulk, self.other
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sovd_core::HttpMethod;

    fn make_cap(id: &str, category: CapabilityCategory) -> Capability {
        Capability {
            id: id.into(),
            category,
            name: format!("cap_{id}"),
            description: None,
            href: format!("/api/{id}"),
            methods: vec![HttpMethod::Get],
            parameters: vec![],
        }
    }

    fn sample_resolver() -> CapabilityResolver {
        let caps = CapabilitySet {
            capabilities: vec![
                make_cap("flash_start", CapabilityCategory::Flashing),
                make_cap("flash_status", CapabilityCategory::Flashing),
                make_cap("diag_read", CapabilityCategory::Diagnostics),
                make_cap("dtc_read", CapabilityCategory::FaultManagement),
                make_cap("config_write", CapabilityCategory::Configuration),
                make_cap("log_stream", CapabilityCategory::Logging),
                make_cap("monitor_live", CapabilityCategory::Monitoring),
                make_cap("prov_key", CapabilityCategory::Provisioning),
                make_cap("bulk_flash", CapabilityCategory::Bulk),
            ],
            server_version: Some("1.0.0".into()),
            sovd_version: Some("1.0".into()),
        };
        CapabilityResolver::new(caps)
    }

    #[test]
    fn all_returns_all_capabilities() {
        let r = sample_resolver();
        assert_eq!(r.all().len(), 9);
    }

    #[test]
    fn sovd_version_returns_value() {
        let r = sample_resolver();
        assert_eq!(r.sovd_version(), Some("1.0"));
    }

    #[test]
    fn sovd_version_none_when_missing() {
        let r = CapabilityResolver::new(CapabilitySet::default());
        assert!(r.sovd_version().is_none());
    }

    #[test]
    fn has_returns_true_for_existing() {
        let r = sample_resolver();
        assert!(r.has("flash_start"));
        assert!(r.has("diag_read"));
    }

    #[test]
    fn has_returns_false_for_missing() {
        let r = sample_resolver();
        assert!(!r.has("nonexistent"));
    }

    #[test]
    fn require_returns_capability() {
        let r = sample_resolver();
        let cap = r.require("flash_start").unwrap();
        assert_eq!(cap.id, "flash_start");
    }

    #[test]
    fn require_returns_error_for_missing() {
        let r = sample_resolver();
        let err = r.require("nonexistent");
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("nonexistent"));
    }

    #[test]
    fn flash_capabilities_count() {
        let r = sample_resolver();
        assert_eq!(r.flash_capabilities().len(), 2);
    }

    #[test]
    fn diagnostic_capabilities_count() {
        let r = sample_resolver();
        assert_eq!(r.diagnostic_capabilities().len(), 1);
    }

    #[test]
    fn fault_management_capabilities_count() {
        let r = sample_resolver();
        assert_eq!(r.fault_management_capabilities().len(), 1);
    }

    #[test]
    fn logging_capabilities_count() {
        let r = sample_resolver();
        assert_eq!(r.logging_capabilities().len(), 1);
    }

    #[test]
    fn supports_flashing_true() {
        let r = sample_resolver();
        assert!(r.supports_flashing());
    }

    #[test]
    fn supports_diagnostics_true() {
        let r = sample_resolver();
        assert!(r.supports_diagnostics());
    }

    #[test]
    fn supports_flashing_false_when_empty() {
        let r = CapabilityResolver::new(CapabilitySet::default());
        assert!(!r.supports_flashing());
    }

    #[test]
    fn summary_counts_correctly() {
        let r = sample_resolver();
        let s = r.summary();
        assert_eq!(s.total, 9);
        assert_eq!(s.flashing, 2);
        assert_eq!(s.diagnostics, 1);
        assert_eq!(s.fault_management, 1);
        assert_eq!(s.configuration, 1);
        assert_eq!(s.provisioning, 1);
        assert_eq!(s.monitoring, 1);
        assert_eq!(s.logging, 1);
        assert_eq!(s.bulk, 1);
        assert_eq!(s.other, 0);
    }

    #[test]
    fn summary_display_format() {
        let r = sample_resolver();
        let s = r.summary();
        let display = format!("{s}");
        assert!(display.starts_with("Capabilities: 9 total"));
        assert!(display.contains("flashing=2"));
        assert!(display.contains("diagnostics=1"));
        assert!(display.contains("fault_mgmt=1"));
    }

    #[test]
    fn empty_summary() {
        let r = CapabilityResolver::new(CapabilitySet::default());
        let s = r.summary();
        assert_eq!(s.total, 0);
        assert_eq!(s.flashing, 0);
    }
}
