use sovd_observe::EventRecorder;
use sovd_workflow::WorkflowEngine;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state managed by Tauri.
///
/// The `WorkflowEngine` owns the `SovdClient`, `JobController`, and `PluginManager`.
/// We only need to store the engine (once connected) and the recorder.
pub struct AppState {
    pub engine: Mutex<Option<WorkflowEngine>>,
    pub recorder: Arc<EventRecorder>,
}

impl AppState {
    pub fn new() -> Self {
        let recorder = Arc::new(EventRecorder::new());
        Self {
            engine: Mutex::new(None),
            recorder,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
