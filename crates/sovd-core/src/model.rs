pub mod capability;
pub mod component;
pub mod datatypes;
pub mod job;

pub use capability::*;
pub use component::*;
pub use datatypes::*;
pub use job::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exports_capability_types() {
        let _cap = CapabilityCategory::Flashing;
        let _set = CapabilitySet::default();
    }

    #[test]
    fn re_exports_component_types() {
        let _ct = ComponentType::NativeSovd;
        let _list = ComponentList::default();
    }

    #[test]
    fn re_exports_job_types() {
        let _jt = JobType::Flash;
        let _js = JobState::Pending;
        let _jp = JobPhase::PreCheck;
    }

    #[test]
    fn re_exports_datatype_types() {
        let _status = DtcStatus::Active;
        let _sev = DtcSeverity::Info;
    }
}
