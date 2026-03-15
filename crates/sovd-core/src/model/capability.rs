use serde::{Deserialize, Serialize};

/// Represents a capability exposed by the SOVD server.
/// Capabilities are discovered dynamically and drive available workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub category: CapabilityCategory,
    pub name: String,
    pub description: Option<String>,
    pub href: String,
    pub methods: Vec<HttpMethod>,
    pub parameters: Vec<CapabilityParameter>,
}

/// Capability categories aligned with the Eclipse `OpenSOVD` architecture
/// and ISO 17978 SOVD standard.
///
/// **ISO 17978 standard categories**: Diagnostics, FaultManagement, Flashing,
/// Configuration, Provisioning, Monitoring, Logging.
///
/// **Extension categories** (not part of ISO 17978): Bulk, Other.
// TODO(F-08): Coordinate with OpenSOVD Workstream Core (Tuesdays 11:30 CET)
// to align this enum with the canonical SOVD client SDK once available.
// See: https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    /// Diagnostic data access (DID read/write). *ISO 17978 standard.*
    Diagnostics,
    /// Fault management (DTC read/clear) — maps to `OpenSOVD` Fault Library / Diagnostic Fault Manager. *ISO 17978 standard.*
    FaultManagement,
    /// Software flashing / update — maps to `OpenSOVD` Flash Service App. *ISO 17978 standard.*
    Flashing,
    /// ECU configuration and coding. *ISO 17978 standard.*
    Configuration,
    /// Provisioning and key management. *ISO 17978 standard.*
    Provisioning,
    /// Monitoring and live data. *ISO 17978 standard.*
    Monitoring,
    /// Logging — maps to `OpenSOVD` extended vehicle logging / publish-subscribe. *ISO 17978 standard.*
    Logging,
    /// Bulk operations across multiple components. *Extension — not part of ISO 17978.*
    Bulk,
    /// Catch-all for server-specific extensions. *Extension — not part of ISO 17978.*
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for CapabilityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityCategory::Diagnostics => write!(f, "Diagnostics"),
            CapabilityCategory::FaultManagement => write!(f, "Fault Management"),
            CapabilityCategory::Flashing => write!(f, "Flashing"),
            CapabilityCategory::Configuration => write!(f, "Configuration"),
            CapabilityCategory::Provisioning => write!(f, "Provisioning"),
            CapabilityCategory::Monitoring => write!(f, "Monitoring"),
            CapabilityCategory::Logging => write!(f, "Logging"),
            CapabilityCategory::Bulk => write!(f, "Bulk"),
            CapabilityCategory::Other(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityParameter {
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub description: Option<String>,
}

/// The complete set of capabilities discovered from a SOVD server.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub capabilities: Vec<Capability>,
    pub server_version: Option<String>,
    pub sovd_version: Option<String>,
}

impl CapabilitySet {
    /// Find capabilities by category.
    #[must_use] 
    pub fn by_category(&self, category: &CapabilityCategory) -> Vec<&Capability> {
        self.capabilities
            .iter()
            .filter(|c| &c.category == category)
            .collect()
    }

    /// Find a capability by its ID.
    #[must_use] 
    pub fn by_id(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.id == id)
    }

    /// Check if a specific capability is available.
    #[must_use] 
    pub fn has_capability(&self, id: &str) -> bool {
        self.capabilities.iter().any(|c| c.id == id)
    }

    /// Check if flashing is supported.
    #[must_use] 
    pub fn supports_flashing(&self) -> bool {
        !self.by_category(&CapabilityCategory::Flashing).is_empty()
    }

    /// Check if diagnostics is supported.
    #[must_use] 
    pub fn supports_diagnostics(&self) -> bool {
        !self.by_category(&CapabilityCategory::Diagnostics).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cap(id: &str, category: CapabilityCategory) -> Capability {
        Capability {
            id: id.into(),
            category,
            name: format!("cap_{id}"),
            description: Some(format!("Description for {id}")),
            href: format!("/api/{id}"),
            methods: vec![HttpMethod::Get],
            parameters: vec![],
        }
    }

    fn sample_caps() -> CapabilitySet {
        CapabilitySet {
            capabilities: vec![
                make_cap("flash_start", CapabilityCategory::Flashing),
                make_cap("flash_status", CapabilityCategory::Flashing),
                make_cap("diag_read", CapabilityCategory::Diagnostics),
                make_cap("dtc_read", CapabilityCategory::FaultManagement),
                make_cap("config_write", CapabilityCategory::Configuration),
                make_cap("log_stream", CapabilityCategory::Logging),
            ],
            server_version: Some("1.0.0".into()),
            sovd_version: Some("1.0".into()),
        }
    }

    #[test]
    fn by_category_filters_correctly() {
        let caps = sample_caps();
        assert_eq!(caps.by_category(&CapabilityCategory::Flashing).len(), 2);
        assert_eq!(caps.by_category(&CapabilityCategory::Diagnostics).len(), 1);
        assert_eq!(caps.by_category(&CapabilityCategory::FaultManagement).len(), 1);
        assert_eq!(caps.by_category(&CapabilityCategory::Logging).len(), 1);
        assert_eq!(caps.by_category(&CapabilityCategory::Monitoring).len(), 0);
    }

    #[test]
    fn by_id_finds_existing() {
        let caps = sample_caps();
        let found = caps.by_id("flash_start");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "flash_start");
    }

    #[test]
    fn by_id_returns_none_for_missing() {
        let caps = sample_caps();
        assert!(caps.by_id("nonexistent").is_none());
    }

    #[test]
    fn has_capability_true() {
        let caps = sample_caps();
        assert!(caps.has_capability("diag_read"));
    }

    #[test]
    fn has_capability_false() {
        let caps = sample_caps();
        assert!(!caps.has_capability("nonexistent"));
    }

    #[test]
    fn supports_flashing_true() {
        let caps = sample_caps();
        assert!(caps.supports_flashing());
    }

    #[test]
    fn supports_flashing_false_when_empty() {
        let caps = CapabilitySet::default();
        assert!(!caps.supports_flashing());
    }

    #[test]
    fn supports_diagnostics_true() {
        let caps = sample_caps();
        assert!(caps.supports_diagnostics());
    }

    #[test]
    fn supports_diagnostics_false_when_empty() {
        let caps = CapabilitySet::default();
        assert!(!caps.supports_diagnostics());
    }

    #[test]
    fn default_capability_set_is_empty() {
        let caps = CapabilitySet::default();
        assert!(caps.capabilities.is_empty());
        assert!(caps.server_version.is_none());
        assert!(caps.sovd_version.is_none());
    }

    #[test]
    fn capability_serialization_roundtrip() {
        let cap = make_cap("test", CapabilityCategory::Flashing);
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test");
        assert_eq!(deserialized.category, CapabilityCategory::Flashing);
    }

    #[test]
    fn http_method_serialization() {
        let method = HttpMethod::Post;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"POST\"");
        let deserialized: HttpMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, HttpMethod::Post);
    }

    #[test]
    fn capability_parameter_roundtrip() {
        let param = CapabilityParameter {
            name: "component_id".into(),
            r#type: "string".into(),
            required: true,
            description: Some("Target component".into()),
        };
        let json = serde_json::to_string(&param).unwrap();
        let deserialized: CapabilityParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "component_id");
        assert!(deserialized.required);
    }
}
