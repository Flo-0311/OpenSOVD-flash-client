use async_trait::async_trait;

use crate::{Component, SoftwarePackage, SovdResult};

/// Abstraction over flash-related SOVD operations (F-01).
///
/// `JobController` depends on this trait instead of the concrete `SovdClient`,
/// enabling:
/// - Unit testing workflows without an HTTP server (mock implementations)
/// - Swapping transport layers (e.g. in-process, gRPC) without touching workflow logic
/// - Breaking the tight coupling between `sovd-workflow` and `sovd-client`
#[async_trait]
pub trait FlashService: Send + Sync {
    /// Retrieve a component by ID to verify availability.
    async fn get_component(&self, component_id: &str) -> SovdResult<Component>;

    /// Start a flash job on a component, returning server-side status JSON.
    async fn start_flash(
        &self,
        component_id: &str,
        package: &SoftwarePackage,
    ) -> SovdResult<serde_json::Value>;

    /// Poll the status of a running flash job.
    async fn get_flash_status(
        &self,
        component_id: &str,
        job_id: &str,
    ) -> SovdResult<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify the trait is object-safe
    #[test]
    fn flash_service_is_object_safe() {
        fn _assert_object_safe(_: &dyn FlashService) {}
    }
}
