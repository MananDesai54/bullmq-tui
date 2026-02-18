//! WebSocket-to-Redis proxy for BullMQ TUI web client
//!
//! This proxy allows WASM clients (which cannot open TCP sockets)
//! to communicate with Redis via WebSocket.

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use bullmq_core::{BullMQClient, BullMQEventClient, JobState, RedisClient};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, warn};

/// Application state shared across handlers
struct AppState {
    client: Arc<RedisClient>,
}

/// Request types from the web client
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WsRequest {
    /// Discover all queues
    DiscoverQueues { id: String },

    /// Get queue info
    GetQueueInfo { id: String, queue: String },

    /// Get queue counts
    GetQueueCounts { id: String, queue: String },

    /// Get jobs for a queue and state
    GetJobs {
        id: String,
        queue: String,
        state: String,
        offset: usize,
        count: usize,
    },

    /// Get a single job
    GetJob {
        id: String,
        queue: String,
        job_id: String,
    },

    /// Retry a job
    RetryJob {
        id: String,
        queue: String,
        job_id: String,
    },

    /// Remove a job
    RemoveJob {
        id: String,
        queue: String,
        job_id: String,
    },

    /// Pause a queue
    PauseQueue { id: String, queue: String },

    /// Resume a queue
    ResumeQueue { id: String, queue: String },

    /// Subscribe to events
    SubscribeEvents {
        id: String,
        queue: String,
        last_id: Option<String>,
    },

    /// Ping (keep-alive)
    Ping { id: String },
}

/// Response types to the web client
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WsResponse {
    /// Queues discovered
    Queues { id: String, queues: Vec<String> },

    /// Queue info
    QueueInfo {
        id: String,
        queue: String,
        info: serde_json::Value,
    },

    /// Queue counts
    QueueCounts {
        id: String,
        queue: String,
        counts: serde_json::Value,
    },

    /// Jobs list
    Jobs {
        id: String,
        jobs: Vec<serde_json::Value>,
        total: u64,
    },

    /// Single job
    Job {
        id: String,
        job: Option<serde_json::Value>,
    },

    /// Action success
    Success { id: String, message: String },

    /// Events
    Events {
        id: String,
        events: Vec<serde_json::Value>,
    },

    /// Error response
    Error { id: String, error: String },

    /// Pong response
    Pong { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,bullmq_proxy=debug".to_string()),
        )
        .init();

    // Get Redis URL from environment or use default
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    info!("Connecting to Redis at {}", redis_url);

    // Connect to Redis
    let client = RedisClient::connect(&redis_url).await?;
    info!("Connected to Redis");

    let state = Arc::new(AppState {
        client: Arc::new(client),
    });

    // Build router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting WebSocket proxy on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// WebSocket upgrade handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    info!("New WebSocket connection");

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received: {}", text);

                let response = match serde_json::from_str::<WsRequest>(&text) {
                    Ok(request) => handle_request(request, &state).await,
                    Err(e) => WsResponse::Error {
                        id: "unknown".to_string(),
                        error: format!("Invalid request: {}", e),
                    },
                };

                let response_text = serde_json::to_string(&response).unwrap_or_else(|e| {
                    format!(r#"{{"type":"error","id":"unknown","error":"{}"}}"#, e)
                });

                if sender.send(Message::Text(response_text.into())).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    info!("WebSocket connection ended");
}

/// Handle a request and return a response
async fn handle_request(request: WsRequest, state: &AppState) -> WsResponse {
    match request {
        WsRequest::DiscoverQueues { id } => match state.client.discover_queues().await {
            Ok(queues) => WsResponse::Queues { id, queues },
            Err(e) => WsResponse::Error {
                id,
                error: e.to_string(),
            },
        },

        WsRequest::GetQueueInfo { id, queue } => match state.client.get_queue_info(&queue).await {
            Ok(info) => {
                let info_json = serde_json::json!({
                    "name": info.name,
                    "prefix": info.prefix,
                    "isPaused": info.is_paused,
                    "counts": {
                        "waiting": info.counts.waiting,
                        "active": info.counts.active,
                        "completed": info.counts.completed,
                        "failed": info.counts.failed,
                        "delayed": info.counts.delayed,
                        "prioritized": info.counts.prioritized,
                        "waitingChildren": info.counts.waiting_children,
                    }
                });
                WsResponse::QueueInfo {
                    id,
                    queue,
                    info: info_json,
                }
            }
            Err(e) => WsResponse::Error {
                id,
                error: e.to_string(),
            },
        },

        WsRequest::GetQueueCounts { id, queue } => {
            match state.client.get_queue_counts(&queue).await {
                Ok(counts) => {
                    let counts_json = serde_json::json!({
                        "waiting": counts.waiting,
                        "active": counts.active,
                        "completed": counts.completed,
                        "failed": counts.failed,
                        "delayed": counts.delayed,
                        "prioritized": counts.prioritized,
                        "waitingChildren": counts.waiting_children,
                    });
                    WsResponse::QueueCounts {
                        id,
                        queue,
                        counts: counts_json,
                    }
                }
                Err(e) => WsResponse::Error {
                    id,
                    error: e.to_string(),
                },
            }
        }

        WsRequest::GetJobs {
            id,
            queue,
            state: state_str,
            offset,
            count,
        } => {
            let job_state = parse_job_state(&state_str);
            match state.client.get_jobs(&queue, job_state, offset, count).await {
                Ok(jobs) => {
                    let total = state
                        .client
                        .get_queue_counts(&queue)
                        .await
                        .map(|c| c.get(job_state))
                        .unwrap_or(jobs.len() as u64);

                    let jobs_json: Vec<serde_json::Value> = jobs
                        .into_iter()
                        .map(|j| serde_json::to_value(j).unwrap_or(serde_json::Value::Null))
                        .collect();

                    WsResponse::Jobs {
                        id,
                        jobs: jobs_json,
                        total,
                    }
                }
                Err(e) => WsResponse::Error {
                    id,
                    error: e.to_string(),
                },
            }
        }

        WsRequest::GetJob { id, queue, job_id } => {
            match state.client.get_job(&queue, &job_id).await {
                Ok(job) => {
                    let job_json = job.map(|j| serde_json::to_value(j).unwrap_or(serde_json::Value::Null));
                    WsResponse::Job { id, job: job_json }
                }
                Err(e) => WsResponse::Error {
                    id,
                    error: e.to_string(),
                },
            }
        }

        WsRequest::RetryJob { id, queue, job_id } => {
            match state.client.retry_job(&queue, &job_id).await {
                Ok(()) => WsResponse::Success {
                    id,
                    message: format!("Job {} retried", job_id),
                },
                Err(e) => WsResponse::Error {
                    id,
                    error: e.to_string(),
                },
            }
        }

        WsRequest::RemoveJob { id, queue, job_id } => {
            match state.client.remove_job(&queue, &job_id).await {
                Ok(()) => WsResponse::Success {
                    id,
                    message: format!("Job {} removed", job_id),
                },
                Err(e) => WsResponse::Error {
                    id,
                    error: e.to_string(),
                },
            }
        }

        WsRequest::PauseQueue { id, queue } => match state.client.pause_queue(&queue).await {
            Ok(()) => WsResponse::Success {
                id,
                message: format!("Queue {} paused", queue),
            },
            Err(e) => WsResponse::Error {
                id,
                error: e.to_string(),
            },
        },

        WsRequest::ResumeQueue { id, queue } => match state.client.resume_queue(&queue).await {
            Ok(()) => WsResponse::Success {
                id,
                message: format!("Queue {} resumed", queue),
            },
            Err(e) => WsResponse::Error {
                id,
                error: e.to_string(),
            },
        },

        WsRequest::SubscribeEvents { id, queue, last_id } => {
            match state
                .client
                .subscribe_events(&queue, last_id.as_deref())
                .await
            {
                Ok(events) => {
                    let events_json: Vec<serde_json::Value> = events
                        .into_iter()
                        .map(|e| serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
                        .collect();
                    WsResponse::Events {
                        id,
                        events: events_json,
                    }
                }
                Err(e) => WsResponse::Error {
                    id,
                    error: e.to_string(),
                },
            }
        }

        WsRequest::Ping { id } => WsResponse::Pong { id },
    }
}

/// Parse a job state string
fn parse_job_state(s: &str) -> JobState {
    match s.to_lowercase().as_str() {
        "waiting" => JobState::Waiting,
        "active" => JobState::Active,
        "completed" => JobState::Completed,
        "failed" => JobState::Failed,
        "delayed" => JobState::Delayed,
        "prioritized" => JobState::Prioritized,
        "waiting-children" | "waitingchildren" => JobState::WaitingChildren,
        "paused" => JobState::Paused,
        _ => JobState::Waiting,
    }
}
