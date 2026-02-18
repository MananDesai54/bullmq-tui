//! Event model for real-time BullMQ updates

use serde::{Deserialize, Serialize};

/// Types of events emitted by BullMQ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    /// Job was added to the queue
    Added,
    /// Job became active (started processing)
    Active,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job progress was updated
    Progress,
    /// Job was removed
    Removed,
    /// Job was delayed
    Delayed,
    /// Queue was paused
    Paused,
    /// Queue was resumed
    Resumed,
    /// Job was stalled (worker died)
    Stalled,
    /// Job was retried
    Retried,
    /// Job waiting for children
    WaitingChildren,
    /// Unknown event type
    Unknown,
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "added" => EventType::Added,
            "active" => EventType::Active,
            "completed" => EventType::Completed,
            "failed" => EventType::Failed,
            "progress" => EventType::Progress,
            "removed" => EventType::Removed,
            "delayed" => EventType::Delayed,
            "paused" => EventType::Paused,
            "resumed" => EventType::Resumed,
            "stalled" => EventType::Stalled,
            "retried" => EventType::Retried,
            "waiting-children" | "waitingchildren" => EventType::WaitingChildren,
            _ => EventType::Unknown,
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EventType::Added => "added",
            EventType::Active => "active",
            EventType::Completed => "completed",
            EventType::Failed => "failed",
            EventType::Progress => "progress",
            EventType::Removed => "removed",
            EventType::Delayed => "delayed",
            EventType::Paused => "paused",
            EventType::Resumed => "resumed",
            EventType::Stalled => "stalled",
            EventType::Retried => "retried",
            EventType::WaitingChildren => "waiting-children",
            EventType::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

/// A BullMQ queue event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEvent {
    /// Event ID from Redis Stream
    pub id: String,

    /// Event type
    pub event_type: EventType,

    /// Queue name
    pub queue_name: String,

    /// Associated job ID (if applicable)
    pub job_id: Option<String>,

    /// Event data payload
    pub data: Option<serde_json::Value>,

    /// Previous state (for state transitions)
    pub prev_state: Option<String>,

    /// Timestamp of the event
    pub timestamp: Option<i64>,
}

impl QueueEvent {
    /// Create a new event
    pub fn new(id: String, event_type: EventType, queue_name: String) -> Self {
        Self {
            id,
            event_type,
            queue_name,
            job_id: None,
            data: None,
            prev_state: None,
            timestamp: None,
        }
    }

    /// Parse an event from Redis Stream entry
    pub fn from_stream_entry(
        id: String,
        queue_name: String,
        fields: &[(String, String)],
    ) -> Option<Self> {
        let mut event = QueueEvent {
            id,
            event_type: EventType::Unknown,
            queue_name,
            job_id: None,
            data: None,
            prev_state: None,
            timestamp: None,
        };

        for (key, value) in fields {
            match key.as_str() {
                "event" => event.event_type = EventType::from(value.as_str()),
                "jobId" => event.job_id = Some(value.clone()),
                "prev" => event.prev_state = Some(value.clone()),
                "data" => event.data = serde_json::from_str(value).ok(),
                "ts" => event.timestamp = value.parse().ok(),
                _ => {}
            }
        }

        if event.event_type == EventType::Unknown {
            None
        } else {
            Some(event)
        }
    }
}

/// Subscription handle for event streams
#[derive(Debug)]
pub struct EventSubscription {
    /// Queue name being subscribed to
    pub queue_name: String,

    /// Last event ID received (for resuming)
    pub last_id: String,

    /// Whether the subscription is active
    pub active: bool,
}

impl EventSubscription {
    /// Create a new subscription starting from the latest events
    pub fn new(queue_name: String) -> Self {
        Self {
            queue_name,
            last_id: "$".to_string(), // Start from latest
            active: true,
        }
    }

    /// Create a subscription starting from a specific ID
    pub fn from_id(queue_name: String, last_id: String) -> Self {
        Self {
            queue_name,
            last_id,
            active: true,
        }
    }
}
