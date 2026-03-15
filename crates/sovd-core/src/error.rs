use thiserror::Error;

/// Top-level error type for the `OpenSOVD` Flash Client.
#[derive(Debug, Error)]
pub enum SovdError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("SOVD API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Capability not available: {0}")]
    CapabilityNotAvailable(String),

    #[error("Job error: {0}")]
    Job(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Workflow error: {0}")]
    Workflow(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Timeout after {0}s")]
    Timeout(u64),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for SovdError {
    fn from(e: serde_json::Error) -> Self {
        SovdError::Serialization(e.to_string())
    }
}

pub type SovdResult<T> = Result<T, SovdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_http() {
        let e = SovdError::Http("connection reset".into());
        assert_eq!(e.to_string(), "HTTP error: connection reset");
    }

    #[test]
    fn error_display_api() {
        let e = SovdError::Api {
            status: 404,
            message: "Not Found".into(),
        };
        assert_eq!(e.to_string(), "SOVD API error 404: Not Found");
    }

    #[test]
    fn error_display_capability_not_available() {
        let e = SovdError::CapabilityNotAvailable("flash".into());
        assert_eq!(e.to_string(), "Capability not available: flash");
    }

    #[test]
    fn error_display_job() {
        let e = SovdError::Job("timeout".into());
        assert_eq!(e.to_string(), "Job error: timeout");
    }

    #[test]
    fn error_display_plugin() {
        let e = SovdError::Plugin("load failed".into());
        assert_eq!(e.to_string(), "Plugin error: load failed");
    }

    #[test]
    fn error_display_workflow() {
        let e = SovdError::Workflow("invalid phase".into());
        assert_eq!(e.to_string(), "Workflow error: invalid phase");
    }

    #[test]
    fn error_display_config() {
        let e = SovdError::Config("missing key".into());
        assert_eq!(e.to_string(), "Configuration error: missing key");
    }

    #[test]
    fn error_display_serialization() {
        let e = SovdError::Serialization("bad json".into());
        assert_eq!(e.to_string(), "Serialization error: bad json");
    }

    #[test]
    fn error_display_timeout() {
        let e = SovdError::Timeout(30);
        assert_eq!(e.to_string(), "Timeout after 30s");
    }

    #[test]
    fn error_display_connection_refused() {
        let e = SovdError::ConnectionRefused("localhost:8080".into());
        assert_eq!(e.to_string(), "Connection refused: localhost:8080");
    }

    #[test]
    fn error_display_other() {
        let e = SovdError::Other("something went wrong".into());
        assert_eq!(e.to_string(), "something went wrong");
    }

    #[test]
    fn from_serde_json_error() {
        let bad_json = "{ invalid }";
        let serde_err = serde_json::from_str::<serde_json::Value>(bad_json).unwrap_err();
        let sovd_err: SovdError = serde_err.into();
        match sovd_err {
            SovdError::Serialization(msg) => assert!(!msg.is_empty()),
            other => panic!("Expected Serialization, got: {other:?}"),
        }
    }

    #[test]
    fn sovd_result_ok() {
        let result: SovdResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn sovd_result_err() {
        let result: SovdResult<i32> = Err(SovdError::Other("fail".into()));
        assert!(result.is_err());
    }
}
