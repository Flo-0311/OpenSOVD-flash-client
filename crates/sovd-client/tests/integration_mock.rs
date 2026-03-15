//! Integration tests against a mock SOVD server (L12).
//!
//! Uses `wiremock` to spin up a local HTTP server that simulates
//! the SOVD REST API, then exercises `SovdClient` end-to-end.

use sovd_client::SovdClient;
use sovd_core::{ComponentStatus, ComponentType};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helper: build a client pointing at the mock server
// ---------------------------------------------------------------------------

fn client_for(server: &MockServer) -> SovdClient {
    SovdClient::new(&server.uri()).expect("valid mock URL")
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_returns_true_when_server_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"status":"ok"})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    assert!(client.health().await.unwrap());
}

#[tokio::test]
async fn health_returns_false_when_server_error() {
    let server = MockServer::start().await;
    // No mock mounted → 404
    let client = client_for(&server);
    // 404 is not a connection error, but the health() method treats non-success as false
    let result = client.health().await.unwrap();
    assert!(!result);
}

// ---------------------------------------------------------------------------
// Capabilities / connect()
// ---------------------------------------------------------------------------

fn capabilities_json() -> serde_json::Value {
    serde_json::json!({
        "capabilities": [
            {
                "id": "flash_sw",
                "category": "flashing",
                "name": "Flash Software",
                "description": "Flash ECU software",
                "href": "/sovd/v1/components/{id}/flash",
                "methods": ["POST"],
                "parameters": []
            },
            {
                "id": "diag_read",
                "category": "diagnostics",
                "name": "Read DID",
                "description": null,
                "href": "/sovd/v1/components/{id}/data/{did}",
                "methods": ["GET"],
                "parameters": []
            }
        ],
        "server_version": "1.0.0",
        "sovd_version": "1.0"
    })
}

#[tokio::test]
async fn connect_discovers_capabilities() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/capabilities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(capabilities_json()))
        .mount(&server)
        .await;

    let mut client = client_for(&server);
    let caps = client.connect().await.unwrap();
    assert_eq!(caps.capabilities.len(), 2);
    assert!(caps.supports_flashing());
    assert!(caps.supports_diagnostics());
}

#[tokio::test]
async fn resolver_available_after_connect() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/capabilities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(capabilities_json()))
        .mount(&server)
        .await;

    let mut client = client_for(&server);
    client.connect().await.unwrap();

    let resolver = client.resolver().unwrap();
    assert_eq!(resolver.sovd_version(), Some("1.0"));
    assert!(resolver.supports_flashing());
    let summary = resolver.summary();
    assert_eq!(summary.flashing, 1);
    assert_eq!(summary.diagnostics, 1);
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

fn component_list_json() -> serde_json::Value {
    serde_json::json!({
        "components": [
            {
                "id": "ecu_01",
                "name": "Engine ECU",
                "category": "powertrain",
                "href": "/sovd/v1/components/ecu_01",
                "component_type": "native_sovd",
                "status": "available",
                "software_version": "2.1.0",
                "hardware_version": "HW-A",
                "capabilities": ["flash", "diag"],
                "adapter_info": null
            },
            {
                "id": "ecu_02",
                "name": "Body ECU",
                "category": "body",
                "href": "/sovd/v1/components/ecu_02",
                "component_type": "classic_uds",
                "status": "available",
                "software_version": "1.0.0",
                "hardware_version": null,
                "capabilities": ["diag"],
                "adapter_info": null
            }
        ]
    })
}

#[tokio::test]
async fn list_components() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components"))
        .respond_with(ResponseTemplate::new(200).set_body_json(component_list_json()))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let list = client.list_components().await.unwrap();
    assert_eq!(list.components.len(), 2);
    assert_eq!(list.native_sovd().len(), 1);
    assert_eq!(list.classic_uds().len(), 1);
}

#[tokio::test]
async fn get_component_by_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ecu_01",
            "name": "Engine ECU",
            "category": "powertrain",
            "href": "/sovd/v1/components/ecu_01",
            "component_type": "native_sovd",
            "status": "available",
            "software_version": "2.1.0",
            "hardware_version": "HW-A",
            "capabilities": ["flash", "diag"],
            "adapter_info": null
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let comp = client.get_component("ecu_01").await.unwrap();
    assert_eq!(comp.id, "ecu_01");
    assert_eq!(comp.component_type, ComponentType::NativeSovd);
    assert_eq!(comp.status, ComponentStatus::Available);
    assert_eq!(comp.software_version, Some("2.1.0".into()));
}

// ---------------------------------------------------------------------------
// Diagnostic Data (DID read / write)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_data_returns_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/data/sw_version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "sw_version",
            "name": "Software Version",
            "value": "2.1.0",
            "unit": null,
            "timestamp": null
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let val = client.read_data("ecu_01", "sw_version").await.unwrap();
    assert_eq!(val.id, "sw_version");
}

#[tokio::test]
async fn write_data_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/sovd/v1/components/ecu_01/data/coding_01"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let result = client
        .write_data("ecu_01", "coding_01", &serde_json::json!("new_value"))
        .await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// DTCs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_dtcs_returns_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/dtcs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "dtc_001",
                "code": "P0300",
                "description": "Random/Multiple Cylinder Misfire Detected",
                "status": "active",
                "severity": "warning",
                "component_id": "ecu_01"
            }
        ])))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let dtcs = client.read_dtcs("ecu_01").await.unwrap();
    assert_eq!(dtcs.len(), 1);
    assert_eq!(dtcs[0].id, "dtc_001");
    assert_eq!(dtcs[0].code, "P0300");
}

#[tokio::test]
async fn clear_dtcs_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/sovd/v1/components/ecu_01/dtcs"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server);
    assert!(client.clear_dtcs("ecu_01").await.is_ok());
}

// ---------------------------------------------------------------------------
// Configuration / Coding (L4)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_config_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/config/variant_coding"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"variant": "sport", "region": "EU"})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let cfg = client
        .read_config("ecu_01", "variant_coding")
        .await
        .unwrap();
    assert_eq!(cfg["variant"], "sport");
}

#[tokio::test]
async fn write_config_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/sovd/v1/components/ecu_01/config/variant_coding"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let result = client
        .write_config(
            "ecu_01",
            "variant_coding",
            &serde_json::json!({"variant": "comfort"}),
        )
        .await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Monitoring / Live Data (L5)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_live_data() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/monitoring"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"rpm": 3500, "temp_c": 92})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let data = client.get_live_data("ecu_01").await.unwrap();
    assert_eq!(data["rpm"], 3500);
}

#[tokio::test]
async fn get_monitoring_parameter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/monitoring/rpm"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"value": 3500, "unit": "rpm"})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let data = client
        .get_monitoring_parameter("ecu_01", "rpm")
        .await
        .unwrap();
    assert_eq!(data["value"], 3500);
}

// ---------------------------------------------------------------------------
// Logging (L6)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_logs_returns_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/logs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"timestamp": "2025-01-01T00:00:00Z", "level": "info", "message": "Boot OK"}
        ])))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let logs = client.get_logs("ecu_01").await.unwrap();
    assert!(logs.is_array());
    assert_eq!(logs.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn subscribe_logs_returns_subscription() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sovd/v1/components/ecu_01/logs/subscribe"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"subscription_id": "sub-123"})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let result = client
        .subscribe_logs("ecu_01", &serde_json::json!({"level": "debug"}))
        .await
        .unwrap();
    assert_eq!(result["subscription_id"], "sub-123");
}

// ---------------------------------------------------------------------------
// Flash
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_flash_returns_job_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sovd/v1/components/ecu_01/flash"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"job_id": "abc-123", "state": "pending"})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let pkg = sovd_core::SoftwarePackage {
        id: "fw-1.0".into(),
        name: "firmware".into(),
        version: "1.0".into(),
        target_component: "ecu_01".into(),
        size_bytes: None,
        checksum: None,
        checksum_algorithm: None,
        metadata: None,
    };
    let result = client.start_flash("ecu_01", &pkg).await.unwrap();
    assert_eq!(result["job_id"], "abc-123");
}

#[tokio::test]
async fn get_flash_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/ecu_01/flash/job-42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"state": "running", "progress": 55}),
        ))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let status = client
        .get_flash_status("ecu_01", "job-42")
        .await
        .unwrap();
    assert_eq!(status["state"], "running");
    assert_eq!(status["progress"], 55);
}

// ---------------------------------------------------------------------------
// Bulk
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bulk_operation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sovd/v1/bulk"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"results": [], "success": true})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let result = client
        .bulk_operation(&serde_json::json!({"operations": []}))
        .await
        .unwrap();
    assert_eq!(result["success"], true);
}

// ---------------------------------------------------------------------------
// Retry / Resilience (L14)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_succeeds_after_transient_failure() {
    use sovd_client::client::RetryConfig;

    let server = MockServer::start().await;

    // First call: 503, second call: 200
    Mock::given(method("GET"))
        .and(path("/sovd/v1/health"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/sovd/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"status":"ok"})))
        .mount(&server)
        .await;

    let client = SovdClient::new(&server.uri())
        .unwrap()
        .with_retry_config(RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            backoff_multiplier: 1.0,
            max_backoff_ms: 100,
        });

    assert!(client.health().await.unwrap());
}

#[tokio::test]
async fn retry_exhausted_returns_error() {
    use sovd_client::client::RetryConfig;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components"))
        .respond_with(ResponseTemplate::new(503).set_body_json(serde_json::json!({"error":"down"})))
        .mount(&server)
        .await;

    let client = SovdClient::new(&server.uri())
        .unwrap()
        .with_retry_config(RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            backoff_multiplier: 1.0,
            max_backoff_ms: 50,
        });

    let result = client.list_components().await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// API error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_404_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/components/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let result = client.get_component("nonexistent").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("404"));
}

// ---------------------------------------------------------------------------
// Auth token is sent
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_token_is_sent_as_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sovd/v1/health"))
        .and(wiremock::matchers::header("Authorization", "Bearer my-secret-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .mount(&server)
        .await;

    let client = SovdClient::new(&server.uri())
        .unwrap()
        .with_auth_token("my-secret-token".into());

    assert!(client.health().await.unwrap());
}
