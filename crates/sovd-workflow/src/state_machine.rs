use sovd_core::{Job, JobPhase, JobState, SovdError, SovdResult};
use tracing::{debug, info, warn};

/// State machine governing valid job state transitions.
///
/// Ensures that jobs follow a well-defined lifecycle and
/// prevents invalid transitions.
pub struct StateMachine;

impl StateMachine {
    /// Check if a state transition is valid.
    #[must_use] 
    pub fn is_valid_transition(from: &JobState, to: &JobState) -> bool {
        matches!(
            (from, to),
            (JobState::Pending | JobState::Paused, JobState::Running) |
(JobState::Pending | JobState::Running | JobState::Paused,
JobState::Cancelled) |
(JobState::Running, JobState::Paused | JobState::Completed | JobState::Failed)
        )
    }

    /// Attempt a state transition. Returns an error if the transition is invalid.
    ///
    /// # Errors
    /// Returns `SovdError::Job` if the transition from the current state to `to` is not valid.
    pub fn transition(job: &mut Job, to: JobState) -> SovdResult<()> {
        if !Self::is_valid_transition(&job.state, &to) {
            return Err(SovdError::Job(format!(
                "Invalid state transition: {:?} -> {:?} for job {}",
                job.state, to, job.id
            )));
        }

        info!(
            job_id = %job.id,
            from = ?job.state,
            to = ?to,
            "Job state transition"
        );

        match to {
            JobState::Completed => job.complete(),
            JobState::Failed => job.fail("Transitioned to failed".into()),
            _ => {
                job.state = to;
                job.updated_at = chrono::Utc::now();
            }
        }

        Ok(())
    }

    /// Check if a phase transition is valid.
    #[must_use] 
    pub fn is_valid_phase_transition(from: &JobPhase, to: &JobPhase) -> bool {
        to > from
    }

    /// Attempt a phase transition.
    ///
    /// # Errors
    /// Returns `SovdError::Workflow` if the job is already at the final phase.
    pub fn advance_phase(job: &mut Job) -> SovdResult<JobPhase> {
        let prev = job.phase.clone();
        job.advance_phase();
        let next = job.phase.clone();

        if prev == next {
            warn!(
                job_id = %job.id,
                phase = ?prev,
                "Already at final phase"
            );
        } else {
            debug!(
                job_id = %job.id,
                from = ?prev,
                to = ?next,
                "Phase advanced"
            );
        }

        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sovd_core::JobType;

    #[test]
    fn valid_transitions() {
        assert!(StateMachine::is_valid_transition(
            &JobState::Pending,
            &JobState::Running
        ));
        assert!(StateMachine::is_valid_transition(
            &JobState::Running,
            &JobState::Completed
        ));
        assert!(StateMachine::is_valid_transition(
            &JobState::Running,
            &JobState::Failed
        ));
        assert!(StateMachine::is_valid_transition(
            &JobState::Running,
            &JobState::Paused
        ));
        assert!(StateMachine::is_valid_transition(
            &JobState::Paused,
            &JobState::Running
        ));
    }

    #[test]
    fn valid_cancellation_transitions() {
        assert!(StateMachine::is_valid_transition(
            &JobState::Pending,
            &JobState::Cancelled
        ));
        assert!(StateMachine::is_valid_transition(
            &JobState::Running,
            &JobState::Cancelled
        ));
        assert!(StateMachine::is_valid_transition(
            &JobState::Paused,
            &JobState::Cancelled
        ));
    }

    #[test]
    fn invalid_transitions() {
        assert!(!StateMachine::is_valid_transition(
            &JobState::Completed,
            &JobState::Running
        ));
        assert!(!StateMachine::is_valid_transition(
            &JobState::Failed,
            &JobState::Running
        ));
        assert!(!StateMachine::is_valid_transition(
            &JobState::Pending,
            &JobState::Completed
        ));
    }

    #[test]
    fn terminal_states_cannot_transition() {
        let terminal = [JobState::Completed, JobState::Failed, JobState::Cancelled];
        let targets = [
            JobState::Pending,
            JobState::Running,
            JobState::Paused,
            JobState::Completed,
            JobState::Failed,
            JobState::Cancelled,
        ];
        for from in &terminal {
            for to in &targets {
                assert!(
                    !StateMachine::is_valid_transition(from, to),
                    "Should not allow transition from {from:?} to {to:?}"
                );
            }
        }
    }

    #[test]
    fn transition_pending_to_running() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        assert_eq!(job.state, JobState::Pending);
        StateMachine::transition(&mut job, JobState::Running).unwrap();
        assert_eq!(job.state, JobState::Running);
    }

    #[test]
    fn transition_running_to_completed() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.state = JobState::Running;
        StateMachine::transition(&mut job, JobState::Completed).unwrap();
        assert_eq!(job.state, JobState::Completed);
        assert_eq!(job.progress_percent, Some(100));
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn transition_running_to_failed() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.state = JobState::Running;
        StateMachine::transition(&mut job, JobState::Failed).unwrap();
        assert_eq!(job.state, JobState::Failed);
        assert!(job.error.is_some());
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn transition_invalid_returns_error() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        let result = StateMachine::transition(&mut job, JobState::Completed);
        assert!(result.is_err());
        assert_eq!(job.state, JobState::Pending); // unchanged
    }

    #[test]
    fn transition_pause_and_resume() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        StateMachine::transition(&mut job, JobState::Running).unwrap();
        StateMachine::transition(&mut job, JobState::Paused).unwrap();
        assert_eq!(job.state, JobState::Paused);
        StateMachine::transition(&mut job, JobState::Running).unwrap();
        assert_eq!(job.state, JobState::Running);
    }

    #[test]
    fn valid_phase_transitions() {
        assert!(StateMachine::is_valid_phase_transition(
            &JobPhase::PreCheck,
            &JobPhase::Deployment
        ));
        assert!(StateMachine::is_valid_phase_transition(
            &JobPhase::Deployment,
            &JobPhase::Monitoring
        ));
        assert!(StateMachine::is_valid_phase_transition(
            &JobPhase::Monitoring,
            &JobPhase::Verification
        ));
        assert!(StateMachine::is_valid_phase_transition(
            &JobPhase::Verification,
            &JobPhase::Reporting
        ));
    }

    #[test]
    fn invalid_phase_transitions() {
        assert!(!StateMachine::is_valid_phase_transition(
            &JobPhase::Deployment,
            &JobPhase::PreCheck
        ));
        assert!(!StateMachine::is_valid_phase_transition(
            &JobPhase::Reporting,
            &JobPhase::Monitoring
        ));
        assert!(!StateMachine::is_valid_phase_transition(
            &JobPhase::Reporting,
            &JobPhase::Reporting
        ));
    }

    #[test]
    fn advance_phase_returns_next() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        let next = StateMachine::advance_phase(&mut job).unwrap();
        assert_eq!(next, JobPhase::Deployment);
        assert_eq!(job.phase, JobPhase::Deployment);
    }

    #[test]
    fn advance_phase_at_final_stays() {
        let mut job = Job::new(JobType::Flash, "ECU_01".into());
        job.phase = JobPhase::Reporting;
        let next = StateMachine::advance_phase(&mut job).unwrap();
        assert_eq!(next, JobPhase::Reporting);
    }
}
