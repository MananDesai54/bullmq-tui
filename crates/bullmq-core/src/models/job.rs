//! Job model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Job state in BullMQ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    /// Job is waiting to be processed
    #[default]
    Waiting,
    /// Job is currently being processed
    Active,
    /// Job completed successfully
    Completed,
    /// Job failed after all retries
    Failed,
    /// Job is delayed until a specific time
    Delayed,
    /// Job is prioritized
    Prioritized,
    /// Job is waiting for child jobs to complete
    WaitingChildren,
    /// Queue is paused (affects job processing)
    Paused,
    /// Unknown state
    Unknown,
}

impl JobState {
    /// Get all displayable job states for tabs
    pub fn all_tabs() -> &'static [JobState] {
        &[
            JobState::Active,
            JobState::Waiting,
            JobState::Completed,
            JobState::Failed,
            JobState::Delayed,
            JobState::Prioritized,
            JobState::WaitingChildren,
        ]
    }

    /// Get the Redis key suffix for this state
    pub fn redis_key_suffix(&self) -> &'static str {
        match self {
            JobState::Waiting => "wait",
            JobState::Active => "active",
            JobState::Completed => "completed",
            JobState::Failed => "failed",
            JobState::Delayed => "delayed",
            JobState::Prioritized => "prioritized",
            JobState::WaitingChildren => "waiting-children",
            JobState::Paused => "paused",
            JobState::Unknown => "unknown",
        }
    }

    /// Check if this state uses a sorted set
    pub fn is_sorted_set(&self) -> bool {
        matches!(
            self,
            JobState::Completed
                | JobState::Failed
                | JobState::Delayed
                | JobState::Prioritized
                | JobState::WaitingChildren
        )
    }

    /// Get display name for the state
    pub fn display_name(&self) -> &'static str {
        match self {
            JobState::Waiting => "WAITING",
            JobState::Active => "ACTIVE",
            JobState::Completed => "COMPLETED",
            JobState::Failed => "FAILED",
            JobState::Delayed => "DELAYED",
            JobState::Prioritized => "PRIORITIZED",
            JobState::WaitingChildren => "CHILDREN",
            JobState::Paused => "PAUSED",
            JobState::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for JobState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Job options/configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobOptions {
    /// Number of retry attempts
    #[serde(default)]
    pub attempts: Option<u32>,

    /// Backoff configuration
    #[serde(default)]
    pub backoff: Option<BackoffConfig>,

    /// Delay before processing (in ms)
    #[serde(default)]
    pub delay: Option<u64>,

    /// Job priority (lower = higher priority)
    #[serde(default)]
    pub priority: Option<u32>,

    /// Time-to-live for the job (in ms)
    #[serde(default, rename = "removeOnComplete")]
    pub remove_on_complete: Option<RemoveOnConfig>,

    /// Remove job on fail
    #[serde(default, rename = "removeOnFail")]
    pub remove_on_fail: Option<RemoveOnConfig>,

    /// Repeat configuration for recurring jobs
    #[serde(default)]
    pub repeat: Option<RepeatConfig>,

    /// Job ID (custom or auto-generated)
    #[serde(default, rename = "jobId")]
    pub job_id: Option<String>,

    /// Timestamp when job was created
    #[serde(default)]
    pub timestamp: Option<i64>,

    /// Stack trace limit
    #[serde(default, rename = "stackTraceLimit")]
    pub stack_trace_limit: Option<u32>,

    /// Parent job information
    #[serde(default)]
    pub parent: Option<ParentConfig>,
}

/// Backoff configuration for retries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackoffConfig {
    /// Backoff type (fixed, exponential, etc.)
    #[serde(rename = "type")]
    pub backoff_type: String,

    /// Delay in milliseconds
    pub delay: u64,
}

/// Configuration for removing jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RemoveOnConfig {
    /// Remove immediately if true
    Bool(bool),
    /// Keep this many jobs
    Count(u32),
    /// Detailed configuration
    Config {
        /// Number of jobs to keep
        count: Option<u32>,
        /// Age in milliseconds
        age: Option<u64>,
    },
}

/// Repeat configuration for recurring jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatConfig {
    /// Cron pattern
    pub pattern: Option<String>,

    /// Repeat every N milliseconds
    pub every: Option<u64>,

    /// Maximum number of repetitions
    pub limit: Option<u32>,

    /// Timezone for cron
    pub tz: Option<String>,
}

/// Parent job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentConfig {
    /// Parent queue name
    pub queue: String,

    /// Parent job ID
    pub id: String,
}

/// A BullMQ job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Unique job ID
    pub id: String,

    /// Job name/type
    pub name: String,

    /// Job data (payload)
    #[serde(default)]
    pub data: serde_json::Value,

    /// Job options
    #[serde(default)]
    pub opts: JobOptions,

    /// Current progress (0-100 or custom value)
    #[serde(default)]
    pub progress: JobProgress,

    /// Return value after completion
    #[serde(default, rename = "returnvalue")]
    pub return_value: Option<serde_json::Value>,

    /// Error message if failed
    #[serde(default, rename = "failedReason")]
    pub failed_reason: Option<String>,

    /// Stack trace if failed
    #[serde(default, rename = "stacktrace")]
    pub stack_trace: Option<Vec<String>>,

    /// Number of attempts made
    #[serde(default, rename = "attemptsMade")]
    pub attempts_made: u32,

    /// Timestamp when job was added
    #[serde(default)]
    pub timestamp: Option<i64>,

    /// Timestamp when job was processed
    #[serde(default, rename = "processedOn")]
    pub processed_on: Option<i64>,

    /// Timestamp when job finished
    #[serde(default, rename = "finishedOn")]
    pub finished_on: Option<i64>,

    /// Delay timestamp (for delayed jobs)
    #[serde(default)]
    pub delay: Option<i64>,

    /// Priority value
    #[serde(default)]
    pub priority: Option<u32>,

    /// Parent job reference
    #[serde(default)]
    pub parent: Option<ParentConfig>,

    /// Queue name (set by client, not stored in Redis)
    #[serde(skip)]
    pub queue_name: String,

    /// Current state (set by client based on where job was found)
    #[serde(skip)]
    pub state: JobState,
}

impl Job {
    /// Get the job creation time as DateTime
    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.timestamp
            .map(|ts| DateTime::from_timestamp_millis(ts))
            .flatten()
    }

    /// Get the job processing start time as DateTime
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.processed_on
            .map(|ts| DateTime::from_timestamp_millis(ts))
            .flatten()
    }

    /// Get the job completion time as DateTime
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished_on
            .map(|ts| DateTime::from_timestamp_millis(ts))
            .flatten()
    }

    /// Get human-readable time since creation
    pub fn age_display(&self) -> String {
        self.created_at()
            .map(|dt| {
                let duration = Utc::now().signed_duration_since(dt);
                if duration.num_days() > 0 {
                    format!("{}d ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{}h ago", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{}m ago", duration.num_minutes())
                } else {
                    format!("{}s ago", duration.num_seconds())
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Get progress as a percentage (0-100)
    pub fn progress_percent(&self) -> Option<u8> {
        match &self.progress {
            JobProgress::Number(n) => Some((*n).min(100.0).max(0.0) as u8),
            JobProgress::Object(obj) => obj
                .get("percent")
                .or_else(|| obj.get("progress"))
                .and_then(|v| v.as_f64())
                .map(|n| n.min(100.0).max(0.0) as u8),
        }
    }
}

/// Job progress can be a number or an object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobProgress {
    /// Simple numeric progress (0-100)
    Number(f64),
    /// Custom progress object
    Object(HashMap<String, serde_json::Value>),
}

impl Default for JobProgress {
    fn default() -> Self {
        JobProgress::Number(0.0)
    }
}

/// Summary of a job for display in lists
#[derive(Debug, Clone)]
pub struct JobSummary {
    pub id: String,
    pub name: String,
    pub state: JobState,
    pub progress: Option<u8>,
    pub age: String,
    pub attempts: u32,
    pub has_error: bool,
}

impl From<&Job> for JobSummary {
    fn from(job: &Job) -> Self {
        JobSummary {
            id: job.id.clone(),
            name: job.name.clone(),
            state: job.state,
            progress: job.progress_percent(),
            age: job.age_display(),
            attempts: job.attempts_made,
            has_error: job.failed_reason.is_some(),
        }
    }
}
