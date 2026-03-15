use serde::{Deserialize, Serialize};

/// Represents a vehicle component (ECU) as exposed by SOVD.
///
/// # SOVD Communication Principle
///
/// The client communicates with **all** components exclusively via SOVD REST APIs.
/// The underlying protocol (native SOVD or UDS via CDA) is **transparent** to this
/// client — the SOVD Gateway / Classic Diagnostic Adapter handles all translation.
///
/// - **Native SOVD (HPC)**: SOVD Gateway forwards requests directly.
/// - **Classic UDS (ECU)**: SOVD Gateway routes through the CDA which translates
///   SOVD calls → UDS/DoIP using diagnostic descriptions (MDD/ODX).
///
/// The `component_type` field is **informational metadata** only — it does NOT
/// affect how the client communicates. The client always speaks SOVD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub href: String,
    /// Informational: how this component is reached behind the SOVD Gateway.
    /// Transparent to the client — the client always speaks SOVD.
    #[serde(default)]
    pub component_type: ComponentType,
    pub software_version: Option<String>,
    pub hardware_version: Option<String>,
    pub status: ComponentStatus,
    pub capabilities: Vec<String>,
    /// Information about the adapter layer (if exposed by the server).
    /// For classic ECUs this may reference the CDA instance translating SOVD→UDS.
    /// For native SOVD HPCs this is typically `None`.
    pub adapter_info: Option<AdapterInfo>,
}

/// The type of a component behind the SOVD Gateway.
///
/// This is **informational metadata only**. The SOVD client always uses
/// the same REST API regardless of the component type. The SOVD Gateway
/// and Classic Diagnostic Adapter (CDA) handle protocol translation
/// transparently (ISO 17978).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    /// Native SOVD-capable High Performance Computer.
    /// The SOVD Gateway forwards requests directly via SOVD.
    NativeSovd,
    /// Classic ECU communicating via UDS (ISO 14229).
    /// The SOVD Gateway routes through a Classic Diagnostic Adapter (CDA)
    /// which translates SOVD ↔ UDS/DoIP using MDD/ODX diagnostic descriptions.
    ClassicUds,
    /// Component type not specified by the server.
    #[default]
    Unknown,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::NativeSovd => write!(f, "Native SOVD"),
            ComponentType::ClassicUds => write!(f, "Classic UDS"),
            ComponentType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentStatus::Available => write!(f, "Available"),
            ComponentStatus::Busy => write!(f, "Busy"),
            ComponentStatus::Error => write!(f, "Error"),
            ComponentStatus::Offline => write!(f, "Offline"),
            ComponentStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about the adapter layer between SOVD Gateway and the ECU.
/// Exposed as metadata by the SOVD server — not used for routing by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    /// The type of adapter (e.g. "cda" for Classic Diagnostic Adapter).
    pub adapter_type: String,
    /// The diagnostic protocol used behind the adapter (e.g. "uds", "doip").
    pub diagnostic_protocol: Option<String>,
    /// Transport layer (e.g. "doip", "can", "ethernet").
    pub transport: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ComponentStatus {
    Available,
    Busy,
    Error,
    Offline,
    #[default]
    Unknown,
}


/// A list of components returned by the SOVD server.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentList {
    pub components: Vec<Component>,
}

impl ComponentList {
    /// Find a component by ID.
    #[must_use] 
    pub fn by_id(&self, id: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }

    /// Filter components by status.
    #[must_use] 
    pub fn by_status(&self, status: &ComponentStatus) -> Vec<&Component> {
        self.components
            .iter()
            .filter(|c| &c.status == status)
            .collect()
    }

    /// Filter by component type (informational — does not affect communication).
    #[must_use] 
    pub fn by_type(&self, component_type: &ComponentType) -> Vec<&Component> {
        self.components
            .iter()
            .filter(|c| &c.component_type == component_type)
            .collect()
    }

    /// Get all native SOVD HPCs.
    #[must_use] 
    pub fn native_sovd(&self) -> Vec<&Component> {
        self.by_type(&ComponentType::NativeSovd)
    }

    /// Get all classic UDS ECUs (reached via CDA).
    #[must_use] 
    pub fn classic_uds(&self) -> Vec<&Component> {
        self.by_type(&ComponentType::ClassicUds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_component(id: &str, ctype: ComponentType, status: ComponentStatus) -> Component {
        Component {
            id: id.into(),
            name: format!("ECU_{id}"),
            category: Some("powertrain".into()),
            href: format!("/sovd/v1/components/{id}"),
            component_type: ctype,
            software_version: Some("1.0.0".into()),
            hardware_version: Some("HW_A".into()),
            status,
            capabilities: vec!["flash".into(), "diag".into()],
            adapter_info: None,
        }
    }

    fn sample_list() -> ComponentList {
        ComponentList {
            components: vec![
                make_component("hpc_01", ComponentType::NativeSovd, ComponentStatus::Available),
                make_component("ecu_01", ComponentType::ClassicUds, ComponentStatus::Available),
                make_component("ecu_02", ComponentType::ClassicUds, ComponentStatus::Busy),
                make_component("ecu_03", ComponentType::Unknown, ComponentStatus::Offline),
            ],
        }
    }

    #[test]
    fn component_type_default_is_unknown() {
        assert_eq!(ComponentType::default(), ComponentType::Unknown);
    }

    #[test]
    fn component_status_default_is_unknown() {
        assert_eq!(ComponentStatus::default(), ComponentStatus::Unknown);
    }

    #[test]
    fn component_list_default_is_empty() {
        let list = ComponentList::default();
        assert!(list.components.is_empty());
    }

    #[test]
    fn by_id_finds_component() {
        let list = sample_list();
        let found = list.by_id("ecu_01");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "ECU_ecu_01");
    }

    #[test]
    fn by_id_returns_none_for_missing() {
        let list = sample_list();
        assert!(list.by_id("nonexistent").is_none());
    }

    #[test]
    fn by_status_filters_correctly() {
        let list = sample_list();
        let available = list.by_status(&ComponentStatus::Available);
        assert_eq!(available.len(), 2);
        let busy = list.by_status(&ComponentStatus::Busy);
        assert_eq!(busy.len(), 1);
        let offline = list.by_status(&ComponentStatus::Offline);
        assert_eq!(offline.len(), 1);
        let error = list.by_status(&ComponentStatus::Error);
        assert_eq!(error.len(), 0);
    }

    #[test]
    fn by_type_filters_correctly() {
        let list = sample_list();
        assert_eq!(list.by_type(&ComponentType::NativeSovd).len(), 1);
        assert_eq!(list.by_type(&ComponentType::ClassicUds).len(), 2);
        assert_eq!(list.by_type(&ComponentType::Unknown).len(), 1);
    }

    #[test]
    fn native_sovd_shorthand() {
        let list = sample_list();
        let native = list.native_sovd();
        assert_eq!(native.len(), 1);
        assert_eq!(native[0].id, "hpc_01");
    }

    #[test]
    fn classic_uds_shorthand() {
        let list = sample_list();
        let classic = list.classic_uds();
        assert_eq!(classic.len(), 2);
    }

    #[test]
    fn component_serialization_roundtrip() {
        let comp = make_component("test", ComponentType::NativeSovd, ComponentStatus::Available);
        let json = serde_json::to_string(&comp).unwrap();
        let deserialized: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test");
        assert_eq!(deserialized.component_type, ComponentType::NativeSovd);
    }

    #[test]
    fn component_type_serialization() {
        let native = ComponentType::NativeSovd;
        let json = serde_json::to_string(&native).unwrap();
        assert_eq!(json, "\"native_sovd\"");
        let classic = ComponentType::ClassicUds;
        let json = serde_json::to_string(&classic).unwrap();
        assert_eq!(json, "\"classic_uds\"");
    }

    #[test]
    fn component_type_defaults_on_missing_field() {
        let json = r#"{
            "id": "ecu_99",
            "name": "Test ECU",
            "category": null,
            "href": "/test",
            "software_version": null,
            "hardware_version": null,
            "status": "available",
            "capabilities": [],
            "adapter_info": null
        }"#;
        let comp: Component = serde_json::from_str(json).unwrap();
        assert_eq!(comp.component_type, ComponentType::Unknown);
    }

    #[test]
    fn adapter_info_roundtrip() {
        let info = AdapterInfo {
            adapter_type: "cda".into(),
            diagnostic_protocol: Some("uds".into()),
            transport: Some("doip".into()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AdapterInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.adapter_type, "cda");
        assert_eq!(deserialized.diagnostic_protocol.unwrap(), "uds");
        assert_eq!(deserialized.transport.unwrap(), "doip");
    }

    #[test]
    fn component_with_adapter_info() {
        let mut comp = make_component("ecu_cda", ComponentType::ClassicUds, ComponentStatus::Available);
        comp.adapter_info = Some(AdapterInfo {
            adapter_type: "cda".into(),
            diagnostic_protocol: Some("uds".into()),
            transport: Some("doip".into()),
        });
        let json = serde_json::to_string(&comp).unwrap();
        let deserialized: Component = serde_json::from_str(&json).unwrap();
        assert!(deserialized.adapter_info.is_some());
        assert_eq!(deserialized.adapter_info.unwrap().adapter_type, "cda");
    }
}
