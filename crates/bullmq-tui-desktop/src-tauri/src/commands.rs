//! Tauri IPC commands for BullMQ operations

use bullmq_core::{BullMQClient, JobState, RedisClient};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state holding the Redis client
pub struct AppState {
    client: RwLock<Option<Arc<RedisClient>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            client: RwLock::new(None),
        }
    }
}

/// Queue info response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueInfoResponse {
    pub name: String,
    pub is_paused: bool,
    pub counts: QueueCountsResponse,
}

/// Queue counts response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueCountsResponse {
    pub waiting: u64,
    pub active: u64,
    pub completed: u64,
    pub failed: u64,
    pub delayed: u64,
    pub prioritized: u64,
    pub waiting_children: u64,
}

/// Job response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobResponse {
    pub id: String,
    pub name: String,
    pub data: serde_json::Value,
    pub progress: serde_json::Value,
    pub attempts_made: u32,
    pub timestamp: Option<i64>,
    pub processed_on: Option<i64>,
    pub finished_on: Option<i64>,
    pub failed_reason: Option<String>,
    pub return_value: Option<serde_json::Value>,
}

/// Connect to Redis
#[tauri::command]
pub async fn connect(
    state: State<'_, AppState>,
    url: String,
) -> Result<String, String> {
    let client = RedisClient::connect(&url)
        .await
        .map_err(|e| e.to_string())?;

    *state.client.write().await = Some(Arc::new(client));

    Ok("Connected".to_string())
}

/// Disconnect from Redis
#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    *state.client.write().await = None;
    Ok(())
}

/// Get the client or return error
async fn get_client(state: &State<'_, AppState>) -> Result<Arc<RedisClient>, String> {
    state
        .client
        .read()
        .await
        .clone()
        .ok_or_else(|| "Not connected to Redis".to_string())
}

/// Discover all queues
#[tauri::command]
pub async fn discover_queues(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let client = get_client(&state).await?;
    client.discover_queues().await.map_err(|e| e.to_string())
}

/// Get queue info
#[tauri::command]
pub async fn get_queue_info(
    state: State<'_, AppState>,
    queue: String,
) -> Result<QueueInfoResponse, String> {
    let client = get_client(&state).await?;
    let info = client
        .get_queue_info(&queue)
        .await
        .map_err(|e| e.to_string())?;

    Ok(QueueInfoResponse {
        name: info.name,
        is_paused: info.is_paused,
        counts: QueueCountsResponse {
            waiting: info.counts.waiting,
            active: info.counts.active,
            completed: info.counts.completed,
            failed: info.counts.failed,
            delayed: info.counts.delayed,
            prioritized: info.counts.prioritized,
            waiting_children: info.counts.waiting_children,
        },
    })
}

/// Get queue counts
#[tauri::command]
pub async fn get_queue_counts(
    state: State<'_, AppState>,
    queue: String,
) -> Result<QueueCountsResponse, String> {
    let client = get_client(&state).await?;
    let counts = client
        .get_queue_counts(&queue)
        .await
        .map_err(|e| e.to_string())?;

    Ok(QueueCountsResponse {
        waiting: counts.waiting,
        active: counts.active,
        completed: counts.completed,
        failed: counts.failed,
        delayed: counts.delayed,
        prioritized: counts.prioritized,
        waiting_children: counts.waiting_children,
    })
}

/// Parse job state from string
fn parse_state(s: &str) -> JobState {
    match s.to_lowercase().as_str() {
        "waiting" => JobState::Waiting,
        "active" => JobState::Active,
        "completed" => JobState::Completed,
        "failed" => JobState::Failed,
        "delayed" => JobState::Delayed,
        "prioritized" => JobState::Prioritized,
        "waiting-children" | "waitingchildren" => JobState::WaitingChildren,
        _ => JobState::Waiting,
    }
}

/// Get jobs for a queue and state
#[tauri::command]
pub async fn get_jobs(
    state: State<'_, AppState>,
    queue: String,
    job_state: String,
    offset: usize,
    count: usize,
) -> Result<Vec<JobResponse>, String> {
    let client = get_client(&state).await?;
    let state_enum = parse_state(&job_state);

    let jobs = client
        .get_jobs(&queue, state_enum, offset, count)
        .await
        .map_err(|e| e.to_string())?;

    Ok(jobs
        .into_iter()
        .map(|j| JobResponse {
            id: j.id,
            name: j.name,
            data: j.data,
            progress: serde_json::to_value(&j.progress).unwrap_or_default(),
            attempts_made: j.attempts_made,
            timestamp: j.timestamp,
            processed_on: j.processed_on,
            finished_on: j.finished_on,
            failed_reason: j.failed_reason,
            return_value: j.return_value,
        })
        .collect())
}

/// Get a single job
#[tauri::command]
pub async fn get_job(
    state: State<'_, AppState>,
    queue: String,
    job_id: String,
) -> Result<Option<JobResponse>, String> {
    let client = get_client(&state).await?;

    let job = client
        .get_job(&queue, &job_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.map(|j| JobResponse {
        id: j.id,
        name: j.name,
        data: j.data,
        progress: serde_json::to_value(&j.progress).unwrap_or_default(),
        attempts_made: j.attempts_made,
        timestamp: j.timestamp,
        processed_on: j.processed_on,
        finished_on: j.finished_on,
        failed_reason: j.failed_reason,
        return_value: j.return_value,
    }))
}

/// Retry a failed job
#[tauri::command]
pub async fn retry_job(
    state: State<'_, AppState>,
    queue: String,
    job_id: String,
) -> Result<(), String> {
    let client = get_client(&state).await?;
    client
        .retry_job(&queue, &job_id)
        .await
        .map_err(|e| e.to_string())
}

/// Remove a job
#[tauri::command]
pub async fn remove_job(
    state: State<'_, AppState>,
    queue: String,
    job_id: String,
) -> Result<(), String> {
    let client = get_client(&state).await?;
    client
        .remove_job(&queue, &job_id)
        .await
        .map_err(|e| e.to_string())
}

/// Pause a queue
#[tauri::command]
pub async fn pause_queue(state: State<'_, AppState>, queue: String) -> Result<(), String> {
    let client = get_client(&state).await?;
    client.pause_queue(&queue).await.map_err(|e| e.to_string())
}

/// Resume a queue
#[tauri::command]
pub async fn resume_queue(state: State<'_, AppState>, queue: String) -> Result<(), String> {
    let client = get_client(&state).await?;
    client
        .resume_queue(&queue)
        .await
        .map_err(|e| e.to_string())
}
