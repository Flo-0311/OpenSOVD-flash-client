use async_trait::async_trait;
use reqwest::{Client, Response};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use sovd_core::{
    CapabilityCategory, CapabilitySet, Component, ComponentList, DataValue,
    DiagnosticTroubleCode, FlashService, SoftwarePackage, SovdError, SovdResult,
};
use tracing::{debug, instrument, warn};
use url::Url;

use crate::discovery::CapabilityResolver;

/// Centralized SOVD API path construction (F-02).
///
/// All REST paths are defined here so that a version bump (e.g. v1→v2)
/// requires a single-line change.  User-supplied path segments (component
/// IDs, data IDs, …) are percent-encoded to prevent path-traversal.
pub mod api_paths {
    use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

    /// Encode set for URL path segments: encodes control chars plus
    /// characters that would break path structure or enable traversal.
    const PATH_SEGMENT_ENCODE: &AsciiSet = &CONTROLS
        .add(b'/')
        .add(b'\\')
        .add(b'?')
        .add(b'#')
        .add(b'%')
        .add(b' ');

    pub const VERSION: &str = "v1";

    /// Percent-encode a single path segment so that `/`, `\`, `?`, `#`, `%`
    /// cannot break the URL structure or enable path-traversal.
    fn enc(segment: &str) -> String {
        utf8_percent_encode(segment, PATH_SEGMENT_ENCODE).to_string()
    }

    #[must_use] pub fn capabilities() -> String { format!("/sovd/{VERSION}/capabilities") }
    #[must_use] pub fn health() -> String { format!("/sovd/{VERSION}/health") }
    #[must_use] pub fn components() -> String { format!("/sovd/{VERSION}/components") }
    #[must_use] pub fn component(id: &str) -> String { format!("/sovd/{VERSION}/components/{}", enc(id)) }
    #[must_use] pub fn data(component: &str, data_id: &str) -> String { format!("/sovd/{VERSION}/components/{}/data/{}", enc(component), enc(data_id)) }
    #[must_use] pub fn dtcs(component: &str) -> String { format!("/sovd/{VERSION}/components/{}/dtcs", enc(component)) }
    #[must_use] pub fn flash(component: &str) -> String { format!("/sovd/{VERSION}/components/{}/flash", enc(component)) }
    #[must_use] pub fn flash_status(component: &str, job_id: &str) -> String { format!("/sovd/{VERSION}/components/{}/flash/{}", enc(component), enc(job_id)) }
    #[must_use] pub fn config(component: &str, config_id: &str) -> String { format!("/sovd/{VERSION}/components/{}/config/{}", enc(component), enc(config_id)) }
    #[must_use] pub fn monitoring(component: &str) -> String { format!("/sovd/{VERSION}/components/{}/monitoring", enc(component)) }
    #[must_use] pub fn monitoring_param(component: &str, param_id: &str) -> String { format!("/sovd/{VERSION}/components/{}/monitoring/{}", enc(component), enc(param_id)) }
    #[must_use] pub fn logs(component: &str) -> String { format!("/sovd/{VERSION}/components/{}/logs", enc(component)) }
    #[must_use] pub fn logs_subscribe(component: &str) -> String { format!("/sovd/{VERSION}/components/{}/logs/subscribe", enc(component)) }
    #[must_use] pub fn bulk() -> String { format!("/sovd/{VERSION}/bulk") }
}

/// Main SOVD REST API client.
///
/// Handles all communication with the SOVD server/gateway.
/// Does not embed any security logic — authentication is handled
/// via plugins or external token providers.
/// Configuration for automatic retry with exponential backoff (L14).
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds.
    pub initial_backoff_ms: u64,
    /// Backoff multiplier per retry.
    pub backoff_multiplier: f64,
    /// Maximum backoff duration in milliseconds.
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 200,
            backoff_multiplier: 2.0,
            max_backoff_ms: 5000,
        }
    }
}

pub struct SovdClient {
    base_url: Url,
    http: Client,
    capabilities: Option<CapabilitySet>,
    /// Authentication bearer token, wrapped in `SecretString` for zeroize-on-drop (F-11).
    auth_token: Option<SecretString>,
    retry_config: RetryConfig,
    /// Configured HTTP timeout in seconds (used for error reporting).
    timeout_secs: u64,
}

impl std::fmt::Debug for SovdClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SovdClient")
            .field("base_url", &self.base_url)
            .field("capabilities", &self.capabilities.as_ref().map(|c| c.capabilities.len()))
            .field("has_auth_token", &self.auth_token.is_some())
            .field("retry_config", &self.retry_config)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

impl SovdClient {
    /// Create a new SOVD client pointing at the given base URL.
    ///
    /// # Errors
    /// Returns `SovdError::Config` if the URL is invalid, or `SovdError::Http` if the HTTP client cannot be built.
    pub fn new(base_url: &str) -> SovdResult<Self> {
        let base_url = Url::parse(base_url)
            .map_err(|e| SovdError::Config(format!("Invalid base URL: {e}")))?;

        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| SovdError::Http(e.to_string()))?;

        Ok(Self {
            base_url,
            http,
            capabilities: None,
            auth_token: None,
            retry_config: RetryConfig::default(),
            timeout_secs: 30,
        })
    }

    /// Set an authentication bearer token (wrapped in `SecretString` for zeroize-on-drop).
    #[must_use] 
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(SecretString::from(token));
        self
    }

    /// Update the authentication token at runtime.
    ///
    /// This is used by the workflow engine to inject tokens obtained
    /// from a [`SecurityPlugin`] without rebuilding the client.
    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(SecretString::from(token));
    }

    /// Set a custom HTTP client (e.g. with custom TLS config).
    ///
    /// If the client has a custom timeout, also call [`with_timeout_secs`](Self::with_timeout_secs)
    /// so that timeout error messages report the correct value.
    #[must_use] 
    pub fn with_http_client(mut self, client: Client) -> Self {
        self.http = client;
        self
    }

    /// Override the reported timeout value (used in error messages).
    #[must_use]
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Configure retry behaviour for transient failures (L14).
    #[must_use]
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Get the current retry configuration.
    #[must_use]
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Get the base URL.
    #[must_use] 
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Get discovered capabilities (if any).
    #[must_use] 
    pub fn capabilities(&self) -> Option<&CapabilitySet> {
        self.capabilities.as_ref()
    }

    /// Create a `CapabilityResolver` from the discovered capabilities.
    ///
    /// # Errors
    /// Returns `SovdError::CapabilityNotAvailable` if `connect()` has not been called yet.
    pub fn resolver(&self) -> SovdResult<CapabilityResolver> {
        match &self.capabilities {
            Some(caps) => Ok(CapabilityResolver::new(caps.clone())),
            None => Err(SovdError::CapabilityNotAvailable(
                "Capabilities not yet discovered. Call connect() first.".into(),
            )),
        }
    }

    // --- Internal helpers ---

    /// Capability guard (F-03 / ADR-0002).
    ///
    /// If capabilities have been discovered via `connect()`, verifies
    /// that the given category is supported. If `connect()` has not
    /// been called, the guard is a no-op (callers may operate without
    /// prior discovery).
    fn require_category(&self, category: &CapabilityCategory) -> SovdResult<()> {
        if let Some(caps) = &self.capabilities {
            if caps.by_category(category).is_empty() {
                return Err(SovdError::CapabilityNotAvailable(format!(
                    "{category} not supported by this SOVD server"
                )));
            }
        }
        Ok(())
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.as_str().trim_end_matches('/'), path)
    }

    fn add_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth_token {
            Some(token) => req.bearer_auth(token.expose_secret()),
            None => req,
        }
    }

    async fn check_response(&self, resp: Response) -> SovdResult<Response> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(SovdError::Api {
                status: status.as_u16(),
                message: body,
            })
        }
    }

    /// Check if an error is transient and worth retrying.
    fn is_retryable(err: &SovdError) -> bool {
        matches!(
            err,
            SovdError::ConnectionRefused(_)
                | SovdError::Timeout(_)
                | SovdError::Http(_)
        ) || matches!(err, SovdError::Api { status, .. } if *status >= 500 || *status == 429)
    }

    /// Compute backoff duration for a given attempt.
    fn backoff_duration(&self, attempt: u32) -> std::time::Duration {
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ms = (self.retry_config.initial_backoff_ms as f64
            * self.retry_config.backoff_multiplier.powi(i32::try_from(attempt).unwrap_or(0)))
            as u64;
        std::time::Duration::from_millis(ms.min(self.retry_config.max_backoff_ms))
    }

    fn classify_reqwest_error(e: &reqwest::Error, url: &str, timeout_secs: u64) -> SovdError {
        if e.is_connect() {
            SovdError::ConnectionRefused(url.to_string())
        } else if e.is_timeout() {
            SovdError::Timeout(timeout_secs)
        } else {
            SovdError::Http(e.to_string())
        }
    }

    /// Generic retry wrapper (F-05: single retry loop for all HTTP methods).
    ///
    /// `method_label` is used for logging (e.g. "GET", "POST").
    /// `build_request` constructs the `RequestBuilder` for each attempt.
    /// `handle_ok` converts a successful `Response` into `T`.
    #[allow(clippy::cast_possible_truncation)]
    async fn execute_with_retry<T, F, H, Fut>(
        &self,
        method_label: &str,
        url: &str,
        build_request: F,
        handle_ok: H,
    ) -> SovdResult<T>
    where
        F: Fn() -> reqwest::RequestBuilder,
        H: Fn(Response) -> Fut,
        Fut: std::future::Future<Output = SovdResult<T>>,
    {
        let mut last_err = SovdError::Http("no attempt made".into());

        for attempt in 0..=self.retry_config.max_retries {
            if attempt > 0 {
                let delay = self.backoff_duration(attempt - 1);
                warn!(attempt, delay_ms = delay.as_millis() as u64, url = %url, method = %method_label, "Retrying");
                tokio::time::sleep(delay).await;
            }
            debug!(url = %url, attempt, method = %method_label);
            let req = self.add_auth(build_request());
            match req.send().await {
                Ok(resp) => match self.check_response(resp).await {
                    Ok(resp) => return handle_ok(resp).await,
                    Err(e) if Self::is_retryable(&e) && attempt < self.retry_config.max_retries => {
                        last_err = e;
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => {
                    let err = Self::classify_reqwest_error(&e, url, self.timeout_secs);
                    if Self::is_retryable(&err) && attempt < self.retry_config.max_retries {
                        last_err = err;
                    } else {
                        return Err(err);
                    }
                }
            }
        }
        Err(last_err)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> SovdResult<T> {
        let url = self.url(path);
        self.execute_with_retry(
            "GET",
            &url,
            || self.http.get(&url),
            |resp| async { resp.json::<T>().await.map_err(|e| SovdError::Serialization(e.to_string())) },
        )
        .await
    }

    async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> SovdResult<T> {
        let url = self.url(path);
        let body_json = serde_json::to_value(body)
            .map_err(|e| SovdError::Serialization(e.to_string()))?;
        self.execute_with_retry(
            "POST",
            &url,
            || self.http.post(&url).json(&body_json),
            |resp| async { resp.json::<T>().await.map_err(|e| SovdError::Serialization(e.to_string())) },
        )
        .await
    }

    async fn put<B: serde::Serialize>(&self, path: &str, body: &B) -> SovdResult<()> {
        let url = self.url(path);
        let body_json = serde_json::to_value(body)
            .map_err(|e| SovdError::Serialization(e.to_string()))?;
        self.execute_with_retry(
            "PUT",
            &url,
            || self.http.put(&url).json(&body_json),
            |_resp| async { Ok(()) },
        )
        .await
    }

    async fn delete(&self, path: &str) -> SovdResult<()> {
        let url = self.url(path);
        self.execute_with_retry(
            "DELETE",
            &url,
            || self.http.delete(&url),
            |_resp| async { Ok(()) },
        )
        .await
    }

    // --- Public SOVD API operations ---

    /// Connect to the SOVD server and discover capabilities.
    ///
    /// # Errors
    /// Returns `SovdError::Http` or `SovdError::ConnectionRefused` if the server is unreachable.
    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> SovdResult<&CapabilitySet> {
        debug!("Discovering SOVD capabilities...");
        let caps: CapabilitySet = self.get(&api_paths::capabilities()).await?;
        debug!(
            count = caps.capabilities.len(),
            version = ?caps.sovd_version,
            "Capabilities discovered"
        );
        self.capabilities = Some(caps);
        Ok(self.capabilities.as_ref().unwrap())
    }

    /// List all known components (ECUs).
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn list_components(&self) -> SovdResult<ComponentList> {
        self.get(&api_paths::components()).await
    }

    /// Get a specific component by ID.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn get_component(&self, component_id: &str) -> SovdResult<Component> {
        self.get(&api_paths::component(component_id)).await
    }

    /// Read a data value (DID) from a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn read_data(
        &self,
        component_id: &str,
        data_id: &str,
    ) -> SovdResult<DataValue> {
        self.require_category(&CapabilityCategory::Diagnostics)?;
        self.get(&api_paths::data(component_id, data_id)).await
    }

    /// Write a data value (DID) to a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails.
    #[instrument(skip(self, value))]
    pub async fn write_data(
        &self,
        component_id: &str,
        data_id: &str,
        value: &serde_json::Value,
    ) -> SovdResult<()> {
        self.require_category(&CapabilityCategory::Diagnostics)?;
        self.put(&api_paths::data(component_id, data_id), value).await
    }

    /// Read DTCs from a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn read_dtcs(
        &self,
        component_id: &str,
    ) -> SovdResult<Vec<DiagnosticTroubleCode>> {
        self.require_category(&CapabilityCategory::FaultManagement)?;
        self.get(&api_paths::dtcs(component_id)).await
    }

    /// Clear DTCs on a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails.
    #[instrument(skip(self))]
    pub async fn clear_dtcs(&self, component_id: &str) -> SovdResult<()> {
        self.require_category(&CapabilityCategory::FaultManagement)?;
        self.delete(&api_paths::dtcs(component_id)).await
    }

    /// Start a flash job on a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self, package))]
    pub async fn start_flash(
        &self,
        component_id: &str,
        package: &SoftwarePackage,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Flashing)?;
        self.post(&api_paths::flash(component_id), package).await
    }

    /// Get the status of a running flash job.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn get_flash_status(
        &self,
        component_id: &str,
        job_id: &str,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Flashing)?;
        self.get(&api_paths::flash_status(component_id, job_id)).await
    }

    // --- Configuration / Coding (L4) ---

    /// Read ECU configuration/coding data from a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn read_config(
        &self,
        component_id: &str,
        config_id: &str,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Configuration)?;
        self.get(&api_paths::config(component_id, config_id)).await
    }

    /// Write ECU configuration/coding data to a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails.
    #[instrument(skip(self, value))]
    pub async fn write_config(
        &self,
        component_id: &str,
        config_id: &str,
        value: &serde_json::Value,
    ) -> SovdResult<()> {
        self.require_category(&CapabilityCategory::Configuration)?;
        self.put(&api_paths::config(component_id, config_id), value).await
    }

    // --- Monitoring / Live Data (L5) ---

    /// Get live monitoring data from a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn get_live_data(
        &self,
        component_id: &str,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Monitoring)?;
        self.get(&api_paths::monitoring(component_id)).await
    }

    /// Get a specific monitoring parameter from a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn get_monitoring_parameter(
        &self,
        component_id: &str,
        parameter_id: &str,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Monitoring)?;
        self.get(&api_paths::monitoring_param(component_id, parameter_id)).await
    }

    // --- Logging (L6) ---

    /// Retrieve logs from a component.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn get_logs(
        &self,
        component_id: &str,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Logging)?;
        self.get(&api_paths::logs(component_id)).await
    }

    /// Subscribe to log events from a component (returns subscription endpoint info).
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn subscribe_logs(
        &self,
        component_id: &str,
        filter: &serde_json::Value,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Logging)?;
        self.post(&api_paths::logs_subscribe(component_id), filter).await
    }

    // --- Bulk Operations ---

    /// Execute a bulk operation across multiple components.
    ///
    /// # Errors
    /// Returns a `SovdError` if the HTTP request fails or the response cannot be deserialized.
    #[instrument(skip(self, request))]
    pub async fn bulk_operation(
        &self,
        request: &serde_json::Value,
    ) -> SovdResult<serde_json::Value> {
        self.require_category(&CapabilityCategory::Bulk)?;
        self.post(&api_paths::bulk(), request).await
    }

    /// Health check / ping the SOVD server.
    ///
    /// Returns `Ok(true)` if the server responds successfully, `Ok(false)` if the
    /// server is unreachable or returns an HTTP error. Propagates unexpected
    /// internal errors (e.g. serialization failures).
    #[instrument(skip(self))]
    pub async fn health(&self) -> SovdResult<bool> {
        match self.get::<serde_json::Value>(&api_paths::health()).await {
            Ok(_) => Ok(true),
            Err(SovdError::ConnectionRefused(_) | SovdError::Timeout(_)) => Ok(false),
            Err(SovdError::Api { status, .. }) => {
                warn!(status, "Health check returned non-success status");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }
}

/// F-01: Implement the `FlashService` trait so that `sovd-workflow` can depend
/// on the abstraction rather than the concrete `SovdClient`.
#[async_trait]
impl FlashService for SovdClient {
    async fn get_component(&self, component_id: &str) -> SovdResult<Component> {
        SovdClient::get_component(self, component_id).await
    }

    async fn start_flash(
        &self,
        component_id: &str,
        package: &SoftwarePackage,
    ) -> SovdResult<serde_json::Value> {
        SovdClient::start_flash(self, component_id, package).await
    }

    async fn get_flash_status(
        &self,
        component_id: &str,
        job_id: &str,
    ) -> SovdResult<serde_json::Value> {
        SovdClient::get_flash_status(self, component_id, job_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation_valid_url() {
        let client = SovdClient::new("http://localhost:8080");
        assert!(client.is_ok());
    }

    #[test]
    fn client_creation_invalid_url() {
        let client = SovdClient::new("not a url");
        assert!(client.is_err());
    }

    #[test]
    fn client_base_url() {
        let client = SovdClient::new("http://localhost:8080").unwrap();
        assert_eq!(client.base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn client_capabilities_none_before_connect() {
        let client = SovdClient::new("http://localhost:8080").unwrap();
        assert!(client.capabilities().is_none());
    }

    #[test]
    fn client_resolver_error_before_connect() {
        let client = SovdClient::new("http://localhost:8080").unwrap();
        let result = client.resolver();
        assert!(result.is_err());
    }

    #[test]
    fn client_with_auth_token() {
        let client = SovdClient::new("http://localhost:8080")
            .unwrap()
            .with_auth_token("test-token".into());
        assert!(client.base_url().as_str().contains("localhost"));
    }

    #[test]
    fn client_url_building() {
        let client = SovdClient::new("http://localhost:8080").unwrap();
        let url = client.url("/sovd/v1/components");
        assert_eq!(url, "http://localhost:8080/sovd/v1/components");
    }

    #[test]
    fn client_url_building_trailing_slash() {
        let client = SovdClient::new("http://localhost:8080/").unwrap();
        let url = client.url("/sovd/v1/health");
        assert_eq!(url, "http://localhost:8080/sovd/v1/health");
    }

    #[test]
    fn client_with_custom_http_client() {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap();
        let client = SovdClient::new("http://localhost:8080")
            .unwrap()
            .with_http_client(http);
        assert!(client.base_url().as_str().contains("localhost"));
    }

    #[test]
    fn client_set_auth_token_at_runtime() {
        let mut client = SovdClient::new("http://localhost:8080").unwrap();
        // Initially no token
        client.set_auth_token("runtime-token".into());
        // Token is set — we can only verify it doesn't panic.
        // Actual header injection is tested via integration tests.
        assert!(client.base_url().as_str().contains("localhost"));
    }

    #[test]
    fn client_write_data_url() {
        let client = SovdClient::new("http://localhost:8080").unwrap();
        let url = client.url("/sovd/v1/components/ecu_01/data/sw_version");
        assert_eq!(
            url,
            "http://localhost:8080/sovd/v1/components/ecu_01/data/sw_version"
        );
    }

    #[test]
    fn api_paths_encode_slash_in_component_id() {
        let path = api_paths::data("../admin", "secret");
        assert!(!path.contains("/../"), "path traversal must be encoded");
        assert!(path.contains("%2F") || path.contains("%2f"), "slash must be percent-encoded");
    }

    #[test]
    fn api_paths_preserve_normal_ids() {
        let path = api_paths::data("ecu_01", "sw_version");
        assert_eq!(path, "/sovd/v1/components/ecu_01/data/sw_version");
    }

    #[test]
    fn api_paths_encode_space_and_percent() {
        let path = api_paths::component("ecu 01%x");
        assert!(path.contains("%20"), "space must be encoded");
        assert!(path.contains("%25"), "percent must be encoded");
    }

    #[test]
    fn debug_impl_redacts_token() {
        let client = SovdClient::new("http://localhost:8080")
            .unwrap()
            .with_auth_token("super-secret".into());
        let debug = format!("{client:?}");
        assert!(!debug.contains("super-secret"), "token must not appear in Debug output");
        assert!(debug.contains("has_auth_token: true"));
    }

    #[test]
    fn with_timeout_secs_updates_value() {
        let client = SovdClient::new("http://localhost:8080")
            .unwrap()
            .with_timeout_secs(60);
        assert_eq!(client.timeout_secs, 60);
    }
}
