use serde::{Deserialize, Serialize};

/// A diagnostic data identifier (DID) value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValue {
    pub id: String,
    pub name: Option<String>,
    pub value: serde_json::Value,
    pub unit: Option<String>,
    pub timestamp: Option<String>,
}

/// A diagnostic trouble code (DTC).
// TODO(F-09): Align with eclipse-opensovd/fault-lib once DTC schema stabilizes.
// The fault-lib will define canonical DTC/FID types; this struct should either
// re-export those types or be replaced by a shared `sovd-types` crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticTroubleCode {
    pub id: String,
    pub code: String,
    pub description: Option<String>,
    pub status: DtcStatus,
    pub severity: Option<DtcSeverity>,
    pub component_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DtcStatus {
    Active,
    Pending,
    Confirmed,
    Cleared,
}

impl std::fmt::Display for DtcStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DtcStatus::Active => write!(f, "Active"),
            DtcStatus::Pending => write!(f, "Pending"),
            DtcStatus::Confirmed => write!(f, "Confirmed"),
            DtcStatus::Cleared => write!(f, "Cleared"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DtcSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for DtcSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DtcSeverity::Info => write!(f, "Info"),
            DtcSeverity::Warning => write!(f, "Warning"),
            DtcSeverity::Error => write!(f, "Error"),
            DtcSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Software package metadata for flashing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwarePackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub target_component: String,
    pub size_bytes: Option<u64>,
    pub checksum: Option<String>,
    pub checksum_algorithm: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// SOVD API response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovdResponse<T> {
    pub data: Option<T>,
    pub error: Option<SovdApiError>,
}

/// SOVD API error structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovdApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_value_roundtrip() {
        let dv = DataValue {
            id: "voltage".into(),
            name: Some("Battery Voltage".into()),
            value: serde_json::json!(13.8),
            unit: Some("V".into()),
            timestamp: Some("2025-01-01T00:00:00Z".into()),
        };
        let json = serde_json::to_string(&dv).unwrap();
        let deserialized: DataValue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "voltage");
        assert_eq!(deserialized.value, serde_json::json!(13.8));
        assert_eq!(deserialized.unit.unwrap(), "V");
    }

    #[test]
    fn data_value_with_complex_value() {
        let dv = DataValue {
            id: "multi".into(),
            name: None,
            value: serde_json::json!({"rpm": 3000, "temp": 90}),
            unit: None,
            timestamp: None,
        };
        let json = serde_json::to_string(&dv).unwrap();
        let deserialized: DataValue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.value["rpm"], 3000);
    }

    #[test]
    fn dtc_roundtrip() {
        let dtc = DiagnosticTroubleCode {
            id: "dtc_001".into(),
            code: "P0123".into(),
            description: Some("Throttle position sensor".into()),
            status: DtcStatus::Active,
            severity: Some(DtcSeverity::Warning),
            component_id: Some("ECU_01".into()),
        };
        let json = serde_json::to_string(&dtc).unwrap();
        let deserialized: DiagnosticTroubleCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, "P0123");
        assert_eq!(deserialized.status, DtcStatus::Active);
        assert_eq!(deserialized.severity, Some(DtcSeverity::Warning));
    }

    #[test]
    fn dtc_status_serialization() {
        let statuses = vec![
            (DtcStatus::Active, "\"active\""),
            (DtcStatus::Pending, "\"pending\""),
            (DtcStatus::Confirmed, "\"confirmed\""),
            (DtcStatus::Cleared, "\"cleared\""),
        ];
        for (status, expected) in statuses {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn dtc_severity_serialization() {
        let severities = vec![
            (DtcSeverity::Info, "\"info\""),
            (DtcSeverity::Warning, "\"warning\""),
            (DtcSeverity::Error, "\"error\""),
            (DtcSeverity::Critical, "\"critical\""),
        ];
        for (sev, expected) in severities {
            let json = serde_json::to_string(&sev).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn software_package_roundtrip() {
        let pkg = SoftwarePackage {
            id: "pkg_001".into(),
            name: "FirmwareUpdate".into(),
            version: "2.1.0".into(),
            target_component: "ECU_01".into(),
            size_bytes: Some(1024 * 1024),
            checksum: Some("abc123".into()),
            checksum_algorithm: Some("sha256".into()),
            metadata: Some(serde_json::json!({"release": "stable"})),
        };
        let json = serde_json::to_string(&pkg).unwrap();
        let deserialized: SoftwarePackage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, "2.1.0");
        assert_eq!(deserialized.size_bytes, Some(1024 * 1024));
        assert_eq!(deserialized.checksum_algorithm.unwrap(), "sha256");
    }

    #[test]
    fn software_package_minimal() {
        let pkg = SoftwarePackage {
            id: "pkg_min".into(),
            name: "Min".into(),
            version: "1.0".into(),
            target_component: "ECU".into(),
            size_bytes: None,
            checksum: None,
            checksum_algorithm: None,
            metadata: None,
        };
        let json = serde_json::to_string(&pkg).unwrap();
        let deserialized: SoftwarePackage = serde_json::from_str(&json).unwrap();
        assert!(deserialized.size_bytes.is_none());
        assert!(deserialized.checksum.is_none());
    }

    #[test]
    fn sovd_response_with_data() {
        let resp = SovdResponse {
            data: Some(42),
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: SovdResponse<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data, Some(42));
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn sovd_response_with_error() {
        let resp: SovdResponse<String> = SovdResponse {
            data: None,
            error: Some(SovdApiError {
                code: "NOT_FOUND".into(),
                message: "Component not found".into(),
                details: None,
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: SovdResponse<String> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.data.is_none());
        let err = deserialized.error.unwrap();
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[test]
    fn sovd_api_error_with_details() {
        let err = SovdApiError {
            code: "INVALID_PARAM".into(),
            message: "Bad parameter".into(),
            details: Some(serde_json::json!({"field": "component_id"})),
        };
        let json = serde_json::to_string(&err).unwrap();
        let deserialized: SovdApiError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.details.unwrap()["field"], "component_id");
    }
}
