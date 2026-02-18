//! Job operations

#[cfg(not(target_arch = "wasm32"))]
use crate::error::{BullMQError, Result};
#[cfg(not(target_arch = "wasm32"))]
use crate::models::{Job, JobOptions, JobProgress, JobState};
#[cfg(not(target_arch = "wasm32"))]
use deadpool_redis::Pool;
#[cfg(not(target_arch = "wasm32"))]
use redis::AsyncCommands;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, trace, warn};

/// Job operations helper
#[cfg(not(target_arch = "wasm32"))]
pub struct JobOps {
    pool: Pool,
}

#[cfg(not(target_arch = "wasm32"))]
impl JobOps {
    /// Create a new JobOps instance
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Get jobs in a specific state with pagination
    pub async fn get_jobs(
        &self,
        queue_name: &str,
        state: JobState,
        start: usize,
        count: usize,
    ) -> Result<Vec<Job>> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);
        let key = format!("{}:{}", prefix, state.redis_key_suffix());

        // Get job IDs from the appropriate data structure
        let job_ids: Vec<String> = if state.is_sorted_set() {
            // For sorted sets, use ZRANGE
            redis::cmd("ZRANGE")
                .arg(&key)
                .arg(start as i64)
                .arg((start + count - 1) as i64)
                .query_async(&mut *conn)
                .await?
        } else {
            // For lists, use LRANGE
            conn.lrange(&key, start as isize, (start + count - 1) as isize)
                .await?
        };

        trace!(
            "Found {} job IDs in {}:{} (start={}, count={})",
            job_ids.len(),
            queue_name,
            state,
            start,
            count
        );

        if job_ids.is_empty() {
            return Ok(vec![]);
        }

        // Fetch job data in a pipeline for efficiency
        let mut pipe = redis::pipe();
        for job_id in &job_ids {
            let job_key = format!("{}:{}", prefix, job_id);
            pipe.hgetall(&job_key);
        }

        let job_data: Vec<HashMap<String, String>> = pipe.query_async(&mut *conn).await?;

        // Parse jobs
        let mut jobs = Vec::with_capacity(job_ids.len());
        for (job_id, data) in job_ids.into_iter().zip(job_data.into_iter()) {
            if data.is_empty() {
                trace!("Job {} has no data, skipping", job_id);
                continue;
            }

            match Self::parse_job_from_hash(&job_id, queue_name, state, data) {
                Ok(job) => jobs.push(job),
                Err(e) => {
                    warn!("Failed to parse job {}: {}", job_id, e);
                }
            }
        }

        Ok(jobs)
    }

    /// Get a single job by ID
    pub async fn get_job(&self, queue_name: &str, job_id: &str) -> Result<Option<Job>> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);
        let job_key = format!("{}:{}", prefix, job_id);

        let data: HashMap<String, String> = conn.hgetall(&job_key).await?;

        if data.is_empty() {
            return Ok(None);
        }

        // Determine the job's state by checking which list/set it's in
        let state = self.get_job_state(queue_name, job_id).await?;

        Self::parse_job_from_hash(job_id, queue_name, state, data).map(Some)
    }

    /// Determine which state a job is in by checking all lists/sets
    async fn get_job_state(&self, queue_name: &str, job_id: &str) -> Result<JobState> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);

        // Check lists first (active, wait)
        let active_key = format!("{}:active", prefix);
        let wait_key = format!("{}:wait", prefix);

        // Check if job is in active list
        let active_pos: Option<i64> = redis::cmd("LPOS")
            .arg(&active_key)
            .arg(job_id)
            .query_async(&mut *conn)
            .await
            .ok();

        if active_pos.is_some() {
            return Ok(JobState::Active);
        }

        // Check if job is in wait list
        let wait_pos: Option<i64> = redis::cmd("LPOS")
            .arg(&wait_key)
            .arg(job_id)
            .query_async(&mut *conn)
            .await
            .ok();

        if wait_pos.is_some() {
            return Ok(JobState::Waiting);
        }

        // Check sorted sets
        let states_to_check = [
            (JobState::Completed, "completed"),
            (JobState::Failed, "failed"),
            (JobState::Delayed, "delayed"),
            (JobState::Prioritized, "prioritized"),
            (JobState::WaitingChildren, "waiting-children"),
        ];

        for (state, suffix) in states_to_check {
            let key = format!("{}:{}", prefix, suffix);
            let score: Option<f64> = conn.zscore(&key, job_id).await.ok();
            if score.is_some() {
                return Ok(state);
            }
        }

        Ok(JobState::Unknown)
    }

    /// Parse a job from a Redis hash
    fn parse_job_from_hash(
        job_id: &str,
        queue_name: &str,
        state: JobState,
        data: HashMap<String, String>,
    ) -> Result<Job> {
        // Parse required fields
        let name = data.get("name").cloned().unwrap_or_else(|| "unknown".to_string());

        // Parse data field (JSON)
        let job_data: serde_json::Value = data
            .get("data")
            .map(|s| serde_json::from_str(s).unwrap_or(serde_json::Value::Null))
            .unwrap_or(serde_json::Value::Null);

        // Parse opts field (JSON)
        let opts: JobOptions = data
            .get("opts")
            .map(|s| serde_json::from_str(s).unwrap_or_default())
            .unwrap_or_default();

        // Parse progress
        let progress: JobProgress = data
            .get("progress")
            .map(|s| {
                // Try parsing as number first, then as object
                if let Ok(n) = s.parse::<f64>() {
                    JobProgress::Number(n)
                } else {
                    serde_json::from_str(s).unwrap_or_default()
                }
            })
            .unwrap_or_default();

        // Parse return value
        let return_value: Option<serde_json::Value> = data
            .get("returnvalue")
            .and_then(|s| serde_json::from_str(s).ok());

        // Parse failed reason
        let failed_reason = data.get("failedReason").cloned();

        // Parse stack trace
        let stack_trace: Option<Vec<String>> = data
            .get("stacktrace")
            .and_then(|s| serde_json::from_str(s).ok());

        // Parse numeric fields
        let attempts_made = data
            .get("attemptsMade")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let timestamp = data.get("timestamp").and_then(|s| s.parse().ok());
        let processed_on = data.get("processedOn").and_then(|s| s.parse().ok());
        let finished_on = data.get("finishedOn").and_then(|s| s.parse().ok());
        let delay = data.get("delay").and_then(|s| s.parse().ok());
        let priority = data.get("priority").and_then(|s| s.parse().ok());

        // Parse parent
        let parent = data
            .get("parent")
            .and_then(|s| serde_json::from_str(s).ok());

        Ok(Job {
            id: job_id.to_string(),
            name,
            data: job_data,
            opts,
            progress,
            return_value,
            failed_reason,
            stack_trace,
            attempts_made,
            timestamp,
            processed_on,
            finished_on,
            delay,
            priority,
            parent,
            queue_name: queue_name.to_string(),
            state,
        })
    }

    /// Retry a failed job by moving it back to the wait list
    pub async fn retry_job(&self, queue_name: &str, job_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);

        // Remove from failed set
        let failed_key = format!("{}:failed", prefix);
        let removed: i64 = conn.zrem(&failed_key, job_id).await?;

        if removed == 0 {
            return Err(BullMQError::JobNotFound(job_id.to_string()));
        }

        // Reset attempt counter in job data
        let job_key = format!("{}:{}", prefix, job_id);
        let _: () = conn.hset(&job_key, "attemptsMade", "0").await?;

        // Remove failed reason and stack trace
        let _: () = conn.hdel(&job_key, &["failedReason", "stacktrace"]).await?;

        // Add to wait list
        let wait_key = format!("{}:wait", prefix);
        let _: () = conn.lpush(&wait_key, job_id).await?;

        debug!("Retried job {} in queue {}", job_id, queue_name);
        Ok(())
    }

    /// Remove a job from the queue
    pub async fn remove_job(&self, queue_name: &str, job_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);

        // Try to remove from all possible locations
        let mut pipe = redis::pipe();

        // Remove from lists
        pipe.lrem(format!("{}:wait", prefix), 0, job_id);
        pipe.lrem(format!("{}:active", prefix), 0, job_id);
        pipe.lrem(format!("{}:paused", prefix), 0, job_id);

        // Remove from sorted sets
        pipe.zrem(format!("{}:completed", prefix), job_id);
        pipe.zrem(format!("{}:failed", prefix), job_id);
        pipe.zrem(format!("{}:delayed", prefix), job_id);
        pipe.zrem(format!("{}:prioritized", prefix), job_id);
        pipe.zrem(format!("{}:waiting-children", prefix), job_id);

        // Delete the job hash
        pipe.del(format!("{}:{}", prefix, job_id));

        let results: Vec<i64> = pipe.query_async(&mut *conn).await?;
        let total_removed: i64 = results.iter().sum();

        if total_removed == 0 {
            return Err(BullMQError::JobNotFound(job_id.to_string()));
        }

        debug!("Removed job {} from queue {}", job_id, queue_name);
        Ok(())
    }

    /// Get the total count of jobs in a state
    pub async fn count_jobs(&self, queue_name: &str, state: JobState) -> Result<u64> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);
        let key = format!("{}:{}", prefix, state.redis_key_suffix());

        let count: u64 = if state.is_sorted_set() {
            conn.zcard(&key).await?
        } else {
            conn.llen(&key).await?
        };

        Ok(count)
    }
}
