//! BullMQ client trait and Redis implementation

use crate::error::Result;
use crate::models::{Job, JobState, QueueCounts, QueueEvent, QueueInfo};
use async_trait::async_trait;

/// Trait defining the BullMQ client interface
///
/// This trait allows for different implementations:
/// - Native Redis client (for terminal/desktop)
/// - WebSocket client (for WASM)
#[async_trait]
pub trait BullMQClient: Send + Sync {
    /// Discover all BullMQ queues in Redis
    async fn discover_queues(&self) -> Result<Vec<String>>;

    /// Get detailed information about a queue
    async fn get_queue_info(&self, queue_name: &str) -> Result<QueueInfo>;

    /// Get job counts for all states in a queue
    async fn get_queue_counts(&self, queue_name: &str) -> Result<QueueCounts>;

    /// Get jobs in a specific state with pagination
    async fn get_jobs(
        &self,
        queue_name: &str,
        state: JobState,
        start: usize,
        count: usize,
    ) -> Result<Vec<Job>>;

    /// Get a single job by ID
    async fn get_job(&self, queue_name: &str, job_id: &str) -> Result<Option<Job>>;

    /// Retry a failed job
    async fn retry_job(&self, queue_name: &str, job_id: &str) -> Result<()>;

    /// Remove a job from the queue
    async fn remove_job(&self, queue_name: &str, job_id: &str) -> Result<()>;

    /// Pause a queue
    async fn pause_queue(&self, queue_name: &str) -> Result<()>;

    /// Resume a paused queue
    async fn resume_queue(&self, queue_name: &str) -> Result<()>;

    /// Check if the client is connected
    async fn is_connected(&self) -> bool;

    /// Get the connection URL (for display purposes)
    fn connection_url(&self) -> &str;
}

/// Extension trait for event subscription (not available in WASM)
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait BullMQEventClient: BullMQClient {
    /// Subscribe to queue events
    async fn subscribe_events(
        &self,
        queue_name: &str,
        last_id: Option<&str>,
    ) -> Result<Vec<QueueEvent>>;
}
