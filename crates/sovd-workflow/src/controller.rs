use std::collections::HashMap;
use std::sync::Arc;

use sovd_core::{FlashService, Job, JobPhase, JobState, JobType, SoftwarePackage, SovdError, SovdResult};
use sovd_observe::EventRecorder;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::state_machine::StateMachine;

/// Manages the lifecycle of all jobs.
///
/// The `JobController` is the central coordinator that:
/// - Creates and tracks jobs
/// - Delegates phase execution to the SOVD client
/// - Records events for observability
pub struct JobController {
    jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
    recorder: Arc<EventRecorder>,
}

impl JobController {
    #[must_use] 
    pub fn new(recorder: Arc<EventRecorder>) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            recorder,
        }
    }

    /// Create a new job and register it.
    ///
    /// # Errors
    /// Returns `SovdError::Job` if the job cannot be created.
    pub async fn create_job(
        &self,
        job_type: JobType,
        target_component: String,
    ) -> SovdResult<Uuid> {
        let job = Job::new(job_type, target_component);
        let id = job.id;

        info!(job_id = %id, job_type = ?job.job_type, target = %job.target_component, "Job created");
        self.recorder
            .record_event("job_created", &serde_json::to_value(&job).unwrap_or_default())
            .await;

        self.jobs.write().await.insert(id, job);
        Ok(id)
    }

    /// Get a snapshot of a job.
    ///
    /// # Errors
    /// Returns `SovdError::Job` if the job is not found.
    pub async fn get_job(&self, id: &Uuid) -> SovdResult<Job> {
        self.jobs
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| SovdError::Job(format!("Job {id} not found")))
    }

    /// List all jobs.
    pub async fn list_jobs(&self) -> Vec<Job> {
        self.jobs.read().await.values().cloned().collect()
    }

    /// Execute a flash job end-to-end through all phases.
    ///
    /// # Errors
    /// Returns `SovdError` if any phase fails, the component is unavailable, or flash times out.
    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self, client, package))]
    pub async fn execute_flash(
        &self,
        client: &dyn FlashService,
        job_id: &Uuid,
        package: &SoftwarePackage,
    ) -> SovdResult<()> {
        // Phase 1: Pre-Check
        self.run_phase(job_id, JobPhase::PreCheck, || async {
            info!(job_id = %job_id, "Pre-check: verifying component availability");
            let component = client.get_component(&package.target_component).await?;
            if component.status != sovd_core::ComponentStatus::Available {
                return Err(SovdError::Job(format!(
                    "Component {} is not available (status: {:?})",
                    component.id, component.status
                )));
            }
            Ok(())
        })
        .await?;

        // Phase 2: Deployment
        self.run_phase(job_id, JobPhase::Deployment, || async {
            info!(job_id = %job_id, "Deploying software package");
            let _result = client
                .start_flash(&package.target_component, package)
                .await?;
            Ok(())
        })
        .await?;

        // Phase 3: Monitoring
        self.run_phase(job_id, JobPhase::Monitoring, || async {
            info!(job_id = %job_id, "Monitoring flash progress");
            // Poll status until complete
            let mut attempts = 0;
            loop {
                let status = client
                    .get_flash_status(&package.target_component, &job_id.to_string())
                    .await;

                match status {
                    Ok(val) => {
                        let state = val
                            .get("state")
                            .and_then(|s| s.as_str())
                            .unwrap_or("unknown");

                        if state == "completed" || state == "verified" {
                            break;
                        }
                        if state == "failed" {
                            let msg = val
                                .get("error")
                                .and_then(|e| e.as_str())
                                .unwrap_or("Unknown flash error");
                            return Err(SovdError::Job(msg.to_string()));
                        }

                        // Update progress
                        if let Some(progress) = val.get("progress").and_then(serde_json::Value::as_u64) {
                            let mut jobs = self.jobs.write().await;
                            if let Some(job) = jobs.get_mut(job_id) {
                                #[allow(clippy::cast_possible_truncation)]
                                {
                                    job.progress_percent = Some(progress as u8);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(job_id = %job_id, error = %e, "Status poll failed");
                    }
                }

                attempts += 1;
                if attempts > 300 {
                    return Err(SovdError::Timeout(300));
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Ok(())
        })
        .await?;

        // Phase 4: Verification
        self.run_phase(job_id, JobPhase::Verification, || async {
            info!(job_id = %job_id, "Verifying flash result");
            let component = client.get_component(&package.target_component).await?;
            if let Some(ref sw_version) = component.software_version {
                if sw_version == &package.version {
                    info!(job_id = %job_id, version = %sw_version, "Version verified");
                } else {
                    warn!(
                        job_id = %job_id,
                        expected = %package.version,
                        actual = %sw_version,
                        "Version mismatch after flash"
                    );
                }
            }
            Ok(())
        })
        .await?;

        // Phase 5: Reporting
        self.run_phase(job_id, JobPhase::Reporting, || async {
            info!(job_id = %job_id, "Generating report");
            Ok(())
        })
        .await?;

        // Mark completed
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.complete();
                self.recorder
                    .record_event(
                        "job_completed",
                        &serde_json::to_value(&*job).unwrap_or_default(),
                    )
                    .await;
            }
        }

        info!(job_id = %job_id, "Flash job completed successfully");
        Ok(())
    }

    /// Run a single phase, advancing the job's phase and handling errors.
    async fn run_phase<F, Fut>(
        &self,
        job_id: &Uuid,
        expected_phase: JobPhase,
        phase_fn: F,
    ) -> SovdResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = SovdResult<()>>,
    {
        // Ensure job is running
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                if job.state == JobState::Pending {
                    StateMachine::transition(job, JobState::Running)?;
                }
                job.phase = expected_phase.clone();
                job.updated_at = chrono::Utc::now();
            }
        }

        self.recorder
            .record_event(
                "phase_started",
                &serde_json::json!({
                    "job_id": job_id.to_string(),
                    "phase": expected_phase,
                }),
            )
            .await;

        match phase_fn().await {
            Ok(()) => {
                self.recorder
                    .record_event(
                        "phase_completed",
                        &serde_json::json!({
                            "job_id": job_id.to_string(),
                            "phase": expected_phase,
                        }),
                    )
                    .await;
                Ok(())
            }
            Err(e) => {
                error!(job_id = %job_id, phase = ?expected_phase, error = %e, "Phase failed");
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(job_id) {
                    job.fail(e.to_string());
                }
                self.recorder
                    .record_event(
                        "phase_failed",
                        &serde_json::json!({
                            "job_id": job_id.to_string(),
                            "phase": expected_phase,
                            "error": e.to_string(),
                        }),
                    )
                    .await;
                Err(e)
            }
        }
    }

    /// Cancel a running job.
    ///
    /// # Errors
    /// Returns `SovdError::Job` if the job is not found, or `SovdError::Workflow` if the transition is invalid.
    pub async fn cancel_job(&self, job_id: &Uuid) -> SovdResult<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(job_id)
            .ok_or_else(|| SovdError::Job(format!("Job {job_id} not found")))?;

        StateMachine::transition(job, JobState::Cancelled)?;

        self.recorder
            .record_event(
                "job_cancelled",
                &serde_json::json!({ "job_id": job_id.to_string() }),
            )
            .await;

        info!(job_id = %job_id, "Job cancelled");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_controller() -> JobController {
        let recorder = Arc::new(EventRecorder::new());
        JobController::new(recorder)
    }

    #[tokio::test]
    async fn create_job_returns_uuid() {
        let ctrl = make_controller();
        let id = ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        assert!(!id.is_nil());
    }

    #[tokio::test]
    async fn get_job_after_create() {
        let ctrl = make_controller();
        let id = ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        let job = ctrl.get_job(&id).await.unwrap();
        assert_eq!(job.id, id);
        assert_eq!(job.job_type, JobType::Flash);
        assert_eq!(job.target_component, "ECU_01");
        assert_eq!(job.state, JobState::Pending);
    }

    #[tokio::test]
    async fn get_job_not_found() {
        let ctrl = make_controller();
        let fake_id = Uuid::new_v4();
        let result = ctrl.get_job(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_jobs_empty() {
        let ctrl = make_controller();
        let jobs = ctrl.list_jobs().await;
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn list_jobs_after_create() {
        let ctrl = make_controller();
        ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        ctrl.create_job(JobType::DtcRead, "ECU_02".into()).await.unwrap();
        let jobs = ctrl.list_jobs().await;
        assert_eq!(jobs.len(), 2);
    }

    #[tokio::test]
    async fn cancel_pending_job() {
        let ctrl = make_controller();
        let id = ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        ctrl.cancel_job(&id).await.unwrap();
        let job = ctrl.get_job(&id).await.unwrap();
        assert_eq!(job.state, JobState::Cancelled);
    }

    #[tokio::test]
    async fn cancel_nonexistent_job() {
        let ctrl = make_controller();
        let fake_id = Uuid::new_v4();
        let result = ctrl.cancel_job(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cancel_completed_job_fails() {
        let ctrl = make_controller();
        let id = ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        // Manually complete the job
        {
            let mut jobs = ctrl.jobs.write().await;
            let job = jobs.get_mut(&id).unwrap();
            job.state = JobState::Running;
            job.complete();
        }
        let result = ctrl.cancel_job(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_job_records_event() {
        let recorder = Arc::new(EventRecorder::new());
        let ctrl = JobController::new(recorder.clone());
        ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        let events = recorder.events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "job_created");
    }

    #[tokio::test]
    async fn cancel_job_records_event() {
        let recorder = Arc::new(EventRecorder::new());
        let ctrl = JobController::new(recorder.clone());
        let id = ctrl.create_job(JobType::Flash, "ECU_01".into()).await.unwrap();
        ctrl.cancel_job(&id).await.unwrap();
        let events = recorder.events().await;
        assert_eq!(events.len(), 2); // created + cancelled
        assert_eq!(events[1].event_type, "job_cancelled");
    }
}
