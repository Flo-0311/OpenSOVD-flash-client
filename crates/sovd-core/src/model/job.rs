use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a flash or diagnostic job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub state: JobState,
    pub phase: JobPhase,
    pub target_component: String,
    pub progress_percent: Option<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Flash,
    DiagnosticRead,
    DiagnosticWrite,
    DtcRead,
    DtcClear,
    SoftwareUpdate,
    BulkFlash,
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Flash => write!(f, "Flash"),
            JobType::DiagnosticRead => write!(f, "Diagnostic Read"),
            JobType::DiagnosticWrite => write!(f, "Diagnostic Write"),
            JobType::DtcRead => write!(f, "DTC Read"),
            JobType::DtcClear => write!(f, "DTC Clear"),
            JobType::SoftwareUpdate => write!(f, "Software Update"),
            JobType::BulkFlash => write!(f, "Bulk Flash"),
        }
    }
}

/// The overall state of a job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl JobState {
    #[must_use] 
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobState::Completed | JobState::Failed | JobState::Cancelled)
    }
}

impl std::fmt::Display for JobState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobState::Pending => write!(f, "Pending"),
            JobState::Running => write!(f, "Running"),
            JobState::Paused => write!(f, "Paused"),
            JobState::Completed => write!(f, "Completed"),
            JobState::Failed => write!(f, "Failed"),
            JobState::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// The phase within a flash job lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum JobPhase {
    PreCheck,
    Deployment,
    Monitoring,
    Verification,
    Reporting,
}

impl std::fmt::Display for JobPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobPhase::PreCheck => write!(f, "Pre-Check"),
            JobPhase::Deployment => write!(f, "Deployment"),
            JobPhase::Monitoring => write!(f, "Monitoring"),
            JobPhase::Verification => write!(f, "Verification"),
            JobPhase::Reporting => write!(f, "Reporting"),
        }
    }
}

impl Job {
    /// Create a new job.
    #[must_use] 
    pub fn new(job_type: JobType, target_component: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            job_type,
            state: JobState::Pending,
            phase: JobPhase::PreCheck,
            target_component,
            progress_percent: Some(0),
            created_at: now,
            updated_at: now,
            completed_at: None,
            error: None,
            metadata: serde_json::Value::Object(serde_json::Map::default()),
        }
    }

    /// Transition to the next phase.
    pub fn advance_phase(&mut self) {
        self.phase = match self.phase {
            JobPhase::PreCheck => JobPhase::Deployment,
            JobPhase::Deployment => JobPhase::Monitoring,
            JobPhase::Monitoring => JobPhase::Verification,
            JobPhase::Verification | JobPhase::Reporting => JobPhase::Reporting,
        };
        self.updated_at = Utc::now();
    }

    /// Mark the job as completed.
    pub fn complete(&mut self) {
        self.state = JobState::Completed;
        self.progress_percent = Some(100);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark the job as failed.
    pub fn fail(&mut self, error: String) {
        self.state = JobState::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_job_defaults() {
        let job = Job::new(JobType::Flash, "ECU_01".into());
        assert_eq!(job.job_type, JobType::Flash);
        assert_eq!(job.state, JobState::Pending);
        assert_eq!(job.phase, JobPhase::PreCheck);
        assert_eq!(job.target_component, "ECU_01");
        assert_eq!(job.progress_percent, Some(0));
        assert!(job.completed_at.is_none());
        assert!(job.error.is_none());
    }

    #[test]
    fn advance_phase_full_cycle() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        assert_eq!(job.phase, JobPhase::PreCheck);

        job.advance_phase();
        assert_eq!(job.phase, JobPhase::Deployment);

        job.advance_phase();
        assert_eq!(job.phase, JobPhase::Monitoring);

        job.advance_phase();
        assert_eq!(job.phase, JobPhase::Verification);

        job.advance_phase();
        assert_eq!(job.phase, JobPhase::Reporting);
    }

    #[test]
    fn advance_phase_stays_at_reporting() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.phase = JobPhase::Reporting;
        job.advance_phase();
        assert_eq!(job.phase, JobPhase::Reporting);
    }

    #[test]
    fn complete_sets_fields() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.complete();
        assert_eq!(job.state, JobState::Completed);
        assert_eq!(job.progress_percent, Some(100));
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn fail_sets_fields() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.fail("timeout".into());
        assert_eq!(job.state, JobState::Failed);
        assert_eq!(job.error, Some("timeout".into()));
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn is_terminal_states() {
        assert!(JobState::Completed.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Cancelled.is_terminal());
        assert!(!JobState::Pending.is_terminal());
        assert!(!JobState::Running.is_terminal());
        assert!(!JobState::Paused.is_terminal());
    }

    #[test]
    fn job_type_serialization() {
        let jt = JobType::Flash;
        let json = serde_json::to_string(&jt).unwrap();
        assert_eq!(json, "\"flash\"");
        let deserialized: JobType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, JobType::Flash);
    }

    #[test]
    fn job_state_serialization() {
        let state = JobState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"running\"");
        let deserialized: JobState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, JobState::Running);
    }

    #[test]
    fn job_phase_ordering() {
        assert!(JobPhase::PreCheck < JobPhase::Deployment);
        assert!(JobPhase::Deployment < JobPhase::Monitoring);
        assert!(JobPhase::Monitoring < JobPhase::Verification);
        assert!(JobPhase::Verification < JobPhase::Reporting);
    }

    #[test]
    fn job_serialization_roundtrip() {
        let job = Job::new(JobType::SoftwareUpdate, "HPC_01".into());
        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.job_type, JobType::SoftwareUpdate);
        assert_eq!(deserialized.target_component, "HPC_01");
    }

    #[test]
    fn advance_phase_updates_timestamp() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        let before = job.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        job.advance_phase();
        assert!(job.updated_at >= before);
    }

    #[test]
    fn all_job_types_exist() {
        let types = [
            JobType::Flash,
            JobType::DiagnosticRead,
            JobType::DiagnosticWrite,
            JobType::DtcRead,
            JobType::DtcClear,
            JobType::SoftwareUpdate,
            JobType::BulkFlash,
        ];
        assert_eq!(types.len(), 7);
    }
}
