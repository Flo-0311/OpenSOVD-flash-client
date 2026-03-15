use chrono::Utc;
use serde::Serialize;
use sovd_core::{Job, SovdResult};
use std::path::Path;
use tracing::info;

use crate::events::EventRecorder;

/// Generates audit-ready reports from job execution data.
pub struct ReportGenerator;

#[derive(Debug, Serialize)]
pub struct FlashReport {
    pub generated_at: String,
    pub job_id: String,
    pub job_type: String,
    pub target_component: String,
    pub state: String,
    pub phase: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub error: Option<String>,
    pub events: serde_json::Value,
}

impl ReportGenerator {
    /// Generate a flash report for a completed job.
    ///
    /// # Errors
    /// Returns `SovdError` if event export fails.
    pub async fn generate(job: &Job, recorder: &EventRecorder) -> SovdResult<FlashReport> {
        #[allow(clippy::cast_precision_loss)]
        let duration = job.completed_at.map(|completed| {
            (completed - job.created_at).num_milliseconds() as f64 / 1000.0
        });

        let report = FlashReport {
            generated_at: Utc::now().to_rfc3339(),
            job_id: job.id.to_string(),
            job_type: format!("{:?}", job.job_type),
            target_component: job.target_component.clone(),
            state: format!("{:?}", job.state),
            phase: format!("{:?}", job.phase),
            created_at: job.created_at.to_rfc3339(),
            completed_at: job.completed_at.map(|t| t.to_rfc3339()),
            duration_seconds: duration,
            error: job.error.clone(),
            events: recorder.export_json().await,
        };

        info!(
            job_id = %job.id,
            duration = ?duration,
            "Report generated"
        );

        Ok(report)
    }

    /// Write a report to a JSON file.
    ///
    /// # Errors
    /// Returns `SovdError::Serialization` if the report cannot be serialized,
    /// or `SovdError::Other` if the file cannot be written.
    pub fn write_json(report: &FlashReport, path: &Path) -> SovdResult<()> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| sovd_core::SovdError::Serialization(e.to_string()))?;

        std::fs::write(path, json)
            .map_err(|e| sovd_core::SovdError::Other(format!("Failed to write report: {e}")))?;

        info!(path = %path.display(), "Report written to file");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sovd_core::{JobType, JobState, JobPhase};

    fn make_pending_job() -> Job {
        Job::new(JobType::Flash, "ECU_01".into())
    }

    fn make_completed_job() -> Job {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.state = JobState::Running;
        job.phase = JobPhase::Reporting;
        job.complete();
        job
    }

    fn make_failed_job() -> Job {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.state = JobState::Running;
        job.fail("Flash timeout".into());
        job
    }

    #[tokio::test]
    async fn generate_report_for_completed_job() {
        let recorder = EventRecorder::new();
        recorder.record_event("job_created", &serde_json::json!({})).await;
        recorder.record_event("job_completed", &serde_json::json!({})).await;

        let job = make_completed_job();
        let report = ReportGenerator::generate(&job, &recorder).await.unwrap();

        assert_eq!(report.job_id, job.id.to_string());
        assert_eq!(report.target_component, "ECU_01");
        assert!(report.completed_at.is_some());
        assert!(report.duration_seconds.is_some());
        assert!(report.error.is_none());
        assert!(!report.generated_at.is_empty());
    }

    #[tokio::test]
    async fn generate_report_for_failed_job() {
        let recorder = EventRecorder::new();
        let job = make_failed_job();
        let report = ReportGenerator::generate(&job, &recorder).await.unwrap();

        assert_eq!(report.error, Some("Flash timeout".into()));
        assert!(report.completed_at.is_some());
    }

    #[tokio::test]
    async fn generate_report_for_pending_job() {
        let recorder = EventRecorder::new();
        let job = make_pending_job();
        let report = ReportGenerator::generate(&job, &recorder).await.unwrap();

        assert!(report.completed_at.is_none());
        assert!(report.duration_seconds.is_none());
        assert!(report.error.is_none());
    }

    #[tokio::test]
    async fn report_includes_events() {
        let recorder = EventRecorder::new();
        recorder.record_event("phase_started", &serde_json::json!({"phase": "pre_check"})).await;
        recorder.record_event("phase_completed", &serde_json::json!({"phase": "pre_check"})).await;

        let job = make_completed_job();
        let report = ReportGenerator::generate(&job, &recorder).await.unwrap();

        assert!(report.events.is_array());
        assert_eq!(report.events.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn flash_report_serialization() {
        let recorder = EventRecorder::new();
        let job = make_completed_job();
        let report = ReportGenerator::generate(&job, &recorder).await.unwrap();

        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("job_id"));
        assert!(json.contains("ECU_01"));
        assert!(json.contains("generated_at"));
    }

    #[test]
    fn write_json_to_file() {
        let report = FlashReport {
            generated_at: "2025-01-01T00:00:00Z".into(),
            job_id: "test-uuid".into(),
            job_type: "Flash".into(),
            target_component: "ECU_01".into(),
            state: "Completed".into(),
            phase: "Reporting".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            completed_at: Some("2025-01-01T00:01:00Z".into()),
            duration_seconds: Some(60.0),
            error: None,
            events: serde_json::json!([]),
        };

        let dir = std::env::temp_dir().join("sovd_test_report");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("test_report.json");

        ReportGenerator::write_json(&report, &path).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("ECU_01"));
        assert!(content.contains("test-uuid"));

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
