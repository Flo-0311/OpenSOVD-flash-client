use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

/// A recorded event for audit and observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Records events during workflow execution for auditing and reporting.
pub struct EventRecorder {
    events: Arc<RwLock<Vec<Event>>>,
}

impl EventRecorder {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record an event.
    pub async fn record_event(&self, event_type: &str, data: &serde_json::Value) {
        let event = Event {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            data: data.clone(),
        };
        debug!(event_type = %event_type, "Event recorded");
        self.events.write().await.push(event);
    }

    /// Get all recorded events.
    pub async fn events(&self) -> Vec<Event> {
        self.events.read().await.clone()
    }

    /// Get events filtered by type.
    pub async fn events_by_type(&self, event_type: &str) -> Vec<Event> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Clear all events.
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }

    /// Export all events as JSON.
    pub async fn export_json(&self) -> serde_json::Value {
        let events = self.events.read().await;
        serde_json::to_value(&*events).unwrap_or_default()
    }
}

impl Default for EventRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_recorder_has_no_events() {
        let rec = EventRecorder::new();
        assert!(rec.events().await.is_empty());
    }

    #[tokio::test]
    async fn default_recorder_has_no_events() {
        let rec = EventRecorder::default();
        assert!(rec.events().await.is_empty());
    }

    #[tokio::test]
    async fn record_and_retrieve_event() {
        let rec = EventRecorder::new();
        rec.record_event("test_event", &serde_json::json!({"key": "value"}))
            .await;
        let events = rec.events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test_event");
        assert_eq!(events[0].data["key"], "value");
    }

    #[tokio::test]
    async fn record_multiple_events() {
        let rec = EventRecorder::new();
        rec.record_event("a", &serde_json::json!({})).await;
        rec.record_event("b", &serde_json::json!({})).await;
        rec.record_event("c", &serde_json::json!({})).await;
        assert_eq!(rec.events().await.len(), 3);
    }

    #[tokio::test]
    async fn events_by_type_filters() {
        let rec = EventRecorder::new();
        rec.record_event("alpha", &serde_json::json!({"n": 1})).await;
        rec.record_event("beta", &serde_json::json!({"n": 2})).await;
        rec.record_event("alpha", &serde_json::json!({"n": 3})).await;

        let alphas = rec.events_by_type("alpha").await;
        assert_eq!(alphas.len(), 2);

        let betas = rec.events_by_type("beta").await;
        assert_eq!(betas.len(), 1);

        let gammas = rec.events_by_type("gamma").await;
        assert_eq!(gammas.len(), 0);
    }

    #[tokio::test]
    async fn clear_removes_all_events() {
        let rec = EventRecorder::new();
        rec.record_event("x", &serde_json::json!({})).await;
        rec.record_event("y", &serde_json::json!({})).await;
        assert_eq!(rec.events().await.len(), 2);

        rec.clear().await;
        assert!(rec.events().await.is_empty());
    }

    #[tokio::test]
    async fn export_json_returns_array() {
        let rec = EventRecorder::new();
        rec.record_event("evt", &serde_json::json!({"data": 42})).await;
        let json = rec.export_json().await;
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn export_json_empty_is_empty_array() {
        let rec = EventRecorder::new();
        let json = rec.export_json().await;
        assert!(json.is_array());
        assert!(json.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn event_has_uuid_and_timestamp() {
        let rec = EventRecorder::new();
        rec.record_event("check", &serde_json::json!({})).await;
        let events = rec.events().await;
        assert!(!events[0].id.is_nil());
        assert!(events[0].timestamp <= Utc::now());
    }

    #[tokio::test]
    async fn event_serialization_roundtrip() {
        let event = Event {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: "test".into(),
            data: serde_json::json!({"key": "val"}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.event_type, "test");
    }
}
