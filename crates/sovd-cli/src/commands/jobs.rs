use anyhow::Result;

use crate::output::{self, OutputFormat};

#[allow(clippy::unused_async)]
pub async fn list(_server: &str, _token: Option<&str>, format: &OutputFormat) -> Result<()> {
    // In a real implementation, this would query persistent job storage
    // For now, jobs exist only in-memory during a session
    output::print_status(true, "Job listing requires an active session. Use 'flash start' to create jobs.", format);
    Ok(())
}

#[allow(clippy::unused_async)]
pub async fn cancel(_server: &str, _token: Option<&str>, job_id: &str, format: &OutputFormat) -> Result<()> {
    output::print_status(true, &format!("Cancel requested for job {job_id}"), format);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn job_id_format() {
        let job_id = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(job_id.len(), 36);
        assert!(job_id.contains('-'));
    }
}
