//! Queue model and related types

use super::job::JobState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Queue metadata stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueueMeta {
    /// Queue name
    #[serde(default)]
    pub name: Option<String>,

    /// Whether the queue is paused
    #[serde(default)]
    pub paused: Option<bool>,

    /// Queue options
    #[serde(default)]
    pub opts: Option<serde_json::Value>,
}

/// Job counts per state for a queue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueCounts {
    /// Number of waiting jobs
    pub waiting: u64,

    /// Number of active jobs
    pub active: u64,

    /// Number of completed jobs
    pub completed: u64,

    /// Number of failed jobs
    pub failed: u64,

    /// Number of delayed jobs
    pub delayed: u64,

    /// Number of prioritized jobs
    pub prioritized: u64,

    /// Number of jobs waiting for children
    pub waiting_children: u64,

    /// Number of paused jobs
    pub paused: u64,
}

impl QueueCounts {
    /// Get count for a specific state
    pub fn get(&self, state: JobState) -> u64 {
        match state {
            JobState::Waiting => self.waiting,
            JobState::Active => self.active,
            JobState::Completed => self.completed,
            JobState::Failed => self.failed,
            JobState::Delayed => self.delayed,
            JobState::Prioritized => self.prioritized,
            JobState::WaitingChildren => self.waiting_children,
            JobState::Paused => self.paused,
            JobState::Unknown => 0,
        }
    }

    /// Get total number of jobs across all states
    pub fn total(&self) -> u64 {
        self.waiting
            + self.active
            + self.completed
            + self.failed
            + self.delayed
            + self.prioritized
            + self.waiting_children
    }

    /// Get counts as a map
    pub fn as_map(&self) -> HashMap<JobState, u64> {
        let mut map = HashMap::new();
        map.insert(JobState::Waiting, self.waiting);
        map.insert(JobState::Active, self.active);
        map.insert(JobState::Completed, self.completed);
        map.insert(JobState::Failed, self.failed);
        map.insert(JobState::Delayed, self.delayed);
        map.insert(JobState::Prioritized, self.prioritized);
        map.insert(JobState::WaitingChildren, self.waiting_children);
        map
    }

    /// Format count for display (e.g., 1234 -> "1.2k")
    pub fn format_count(count: u64) -> String {
        if count >= 1_000_000 {
            format!("{:.1}M", count as f64 / 1_000_000.0)
        } else if count >= 1_000 {
            format!("{:.1}k", count as f64 / 1_000.0)
        } else {
            count.to_string()
        }
    }
}

/// Information about a BullMQ queue
#[derive(Debug, Clone)]
pub struct QueueInfo {
    /// Queue name (without the "bull:" prefix)
    pub name: String,

    /// Full Redis key prefix for this queue
    pub prefix: String,

    /// Whether the queue is paused
    pub is_paused: bool,

    /// Job counts per state
    pub counts: QueueCounts,

    /// Queue metadata
    pub meta: QueueMeta,
}

impl QueueInfo {
    /// Create a new QueueInfo with just the name
    pub fn new(name: String) -> Self {
        Self {
            prefix: format!("bull:{}", name),
            name,
            is_paused: false,
            counts: QueueCounts::default(),
            meta: QueueMeta::default(),
        }
    }

    /// Get the Redis key for a specific data type
    pub fn key(&self, suffix: &str) -> String {
        format!("{}:{}", self.prefix, suffix)
    }

    /// Get the job key for a specific job ID
    pub fn job_key(&self, job_id: &str) -> String {
        format!("{}:{}", self.prefix, job_id)
    }

    /// Get the key for jobs in a specific state
    pub fn state_key(&self, state: JobState) -> String {
        self.key(state.redis_key_suffix())
    }

    /// Get formatted display for counts
    pub fn counts_display(&self) -> Vec<(JobState, String)> {
        JobState::all_tabs()
            .iter()
            .map(|&state| (state, QueueCounts::format_count(self.counts.get(state))))
            .collect()
    }
}

/// Queue discovery result
#[derive(Debug, Clone)]
pub struct DiscoveredQueue {
    /// Queue name
    pub name: String,

    /// Whether metadata was found
    pub has_meta: bool,
}
