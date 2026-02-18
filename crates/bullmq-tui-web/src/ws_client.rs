//! WebSocket client for connecting to the BullMQ proxy

use bullmq_core::{
    BullMQError, Job, JobState, QueueCounts, QueueInfo, QueueMeta, Result,
};
use gloo_net::websocket::futures::WebSocket;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_ID: AtomicU64 = AtomicU64::new(0);

fn next_request_id() -> String {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst).to_string()
}

/// Request types to the proxy
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(dead_code)]
enum WsRequest {
    DiscoverQueues { id: String },
    GetQueueInfo { id: String, queue: String },
    GetQueueCounts { id: String, queue: String },
    GetJobs {
        id: String,
        queue: String,
        state: String,
        offset: usize,
        count: usize,
    },
    GetJob {
        id: String,
        queue: String,
        #[serde(rename = "jobId")]
        job_id: String,
    },
    RetryJob {
        id: String,
        queue: String,
        #[serde(rename = "jobId")]
        job_id: String,
    },
    RemoveJob {
        id: String,
        queue: String,
        #[serde(rename = "jobId")]
        job_id: String,
    },
    PauseQueue { id: String, queue: String },
    ResumeQueue { id: String, queue: String },
    Ping { id: String },
}

/// Response types from the proxy
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(dead_code)]
enum WsResponse {
    Queues { id: String, queues: Vec<String> },
    QueueInfo {
        id: String,
        queue: String,
        info: serde_json::Value,
    },
    QueueCounts {
        id: String,
        queue: String,
        counts: serde_json::Value,
    },
    Jobs {
        id: String,
        jobs: Vec<serde_json::Value>,
        total: u64,
    },
    Job {
        id: String,
        job: Option<serde_json::Value>,
    },
    Success { id: String, message: String },
    Events {
        id: String,
        events: Vec<serde_json::Value>,
    },
    Error { id: String, error: String },
    Pong { id: String },
}

/// WebSocket-based BullMQ client for WASM
#[allow(dead_code)]
pub struct WsClient {
    url: String,
    ws: Rc<RefCell<Option<WebSocket>>>,
    pending: Rc<RefCell<HashMap<String, futures_channel::oneshot::Sender<WsResponse>>>>,
}

impl WsClient {
    /// Create a new WebSocket client
    pub fn new(proxy_url: &str) -> Self {
        Self {
            url: proxy_url.to_string(),
            ws: Rc::new(RefCell::new(None)),
            pending: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Connect to the WebSocket proxy
    pub async fn connect(&self) -> Result<()> {
        let ws = WebSocket::open(&self.url)
            .map_err(|e| BullMQError::Connection(e.to_string()))?;

        *self.ws.borrow_mut() = Some(ws);
        Ok(())
    }

    /// Send a request and wait for response
    #[allow(dead_code)]
    async fn request(&self, req: WsRequest) -> Result<WsResponse> {
        let ws_ref = self.ws.borrow();
        let _ws = ws_ref
            .as_ref()
            .ok_or_else(|| BullMQError::Connection("Not connected".to_string()))?;

        let _req_json = serde_json::to_string(&req)?;

        // For now, use a simple blocking approach
        // In a real implementation, we'd use proper async channels

        // This is a simplified implementation - production code would need
        // proper request/response correlation with channels
        Err(BullMQError::Connection("WebSocket client not fully implemented for WASM".to_string()))
    }

    /// Parse queue info from JSON
    #[allow(dead_code)]
    fn parse_queue_info(name: &str, info: serde_json::Value) -> QueueInfo {
        let is_paused = info.get("isPaused").and_then(|v| v.as_bool()).unwrap_or(false);

        let counts = info.get("counts").cloned().unwrap_or(serde_json::Value::Object(Default::default()));

        QueueInfo {
            name: name.to_string(),
            prefix: format!("bull:{}", name),
            is_paused,
            counts: QueueCounts {
                waiting: counts.get("waiting").and_then(|v| v.as_u64()).unwrap_or(0),
                active: counts.get("active").and_then(|v| v.as_u64()).unwrap_or(0),
                completed: counts.get("completed").and_then(|v| v.as_u64()).unwrap_or(0),
                failed: counts.get("failed").and_then(|v| v.as_u64()).unwrap_or(0),
                delayed: counts.get("delayed").and_then(|v| v.as_u64()).unwrap_or(0),
                prioritized: counts.get("prioritized").and_then(|v| v.as_u64()).unwrap_or(0),
                waiting_children: counts.get("waitingChildren").and_then(|v| v.as_u64()).unwrap_or(0),
                paused: 0,
            },
            meta: QueueMeta::default(),
        }
    }

    /// Parse job from JSON
    #[allow(dead_code)]
    fn parse_job(queue_name: &str, state: JobState, job: serde_json::Value) -> Option<Job> {
        let id = job.get("id")?.as_str()?.to_string();
        let name = job.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

        Some(Job {
            id,
            name,
            data: job.get("data").cloned().unwrap_or(serde_json::Value::Null),
            opts: serde_json::from_value(job.get("opts").cloned().unwrap_or_default()).unwrap_or_default(),
            progress: serde_json::from_value(job.get("progress").cloned().unwrap_or_default()).unwrap_or_default(),
            return_value: job.get("returnValue").cloned(),
            failed_reason: job.get("failedReason").and_then(|v| v.as_str()).map(|s| s.to_string()),
            stack_trace: job.get("stackTrace").and_then(|v| serde_json::from_value(v.clone()).ok()),
            attempts_made: job.get("attemptsMade").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            timestamp: job.get("timestamp").and_then(|v| v.as_i64()),
            processed_on: job.get("processedOn").and_then(|v| v.as_i64()),
            finished_on: job.get("finishedOn").and_then(|v| v.as_i64()),
            delay: job.get("delay").and_then(|v| v.as_i64()),
            priority: job.get("priority").and_then(|v| v.as_u64()).map(|v| v as u32),
            parent: None,
            queue_name: queue_name.to_string(),
            state,
        })
    }
}

// Note: Full async_trait implementation for WASM requires more complex handling
// This is a placeholder that shows the structure
