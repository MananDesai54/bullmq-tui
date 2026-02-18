//! Redis-based BullMQ client implementation

#[cfg(not(target_arch = "wasm32"))]
use crate::client::{BullMQClient, BullMQEventClient};
#[cfg(not(target_arch = "wasm32"))]
use crate::error::Result;
#[cfg(not(target_arch = "wasm32"))]
use crate::events::EventOps;
#[cfg(not(target_arch = "wasm32"))]
use crate::job::JobOps;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::{Job, JobState, QueueCounts, QueueEvent, QueueInfo};
#[cfg(not(target_arch = "wasm32"))]
use crate::queue::QueueOps;
#[cfg(not(target_arch = "wasm32"))]
use async_trait::async_trait;
#[cfg(not(target_arch = "wasm32"))]
use deadpool_redis::{Config, Pool, Runtime};
#[cfg(not(target_arch = "wasm32"))]
use tracing::info;

/// Configuration for the Redis client
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,

    /// Maximum number of connections in the pool
    pub pool_size: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl RedisConfig {
    /// Create a new config with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set the pool size
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }
}

/// Redis-based BullMQ client
#[cfg(not(target_arch = "wasm32"))]
pub struct RedisClient {
    pool: Pool,
    config: RedisConfig,
    queue_ops: QueueOps,
    job_ops: JobOps,
    event_ops: EventOps,
}

#[cfg(not(target_arch = "wasm32"))]
impl RedisClient {
    /// Create a new Redis client with the given configuration
    pub async fn new(config: RedisConfig) -> Result<Self> {
        let cfg = Config::from_url(&config.url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

        // Test connection
        let mut conn = pool.get().await?;
        let _: String = redis::cmd("PING").query_async(&mut *conn).await?;

        info!("Connected to Redis at {}", config.url);

        let queue_ops = QueueOps::new(pool.clone());
        let job_ops = JobOps::new(pool.clone());
        let event_ops = EventOps::new(pool.clone());

        Ok(Self {
            pool,
            config,
            queue_ops,
            job_ops,
            event_ops,
        })
    }

    /// Create a client with default configuration (localhost:6379)
    pub async fn connect_default() -> Result<Self> {
        Self::new(RedisConfig::default()).await
    }

    /// Create a client from a Redis URL
    pub async fn connect(url: impl Into<String>) -> Result<Self> {
        Self::new(RedisConfig::new(url)).await
    }

    /// Get access to the underlying connection pool
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get access to queue operations
    pub fn queue_ops(&self) -> &QueueOps {
        &self.queue_ops
    }

    /// Get access to job operations
    pub fn job_ops(&self) -> &JobOps {
        &self.job_ops
    }

    /// Get access to event operations
    pub fn event_ops(&self) -> &EventOps {
        &self.event_ops
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl BullMQClient for RedisClient {
    async fn discover_queues(&self) -> Result<Vec<String>> {
        self.queue_ops.discover_queues().await
    }

    async fn get_queue_info(&self, queue_name: &str) -> Result<QueueInfo> {
        self.queue_ops.get_queue_info(queue_name).await
    }

    async fn get_queue_counts(&self, queue_name: &str) -> Result<QueueCounts> {
        self.queue_ops.get_queue_counts(queue_name).await
    }

    async fn get_jobs(
        &self,
        queue_name: &str,
        state: JobState,
        start: usize,
        count: usize,
    ) -> Result<Vec<Job>> {
        self.job_ops.get_jobs(queue_name, state, start, count).await
    }

    async fn get_job(&self, queue_name: &str, job_id: &str) -> Result<Option<Job>> {
        self.job_ops.get_job(queue_name, job_id).await
    }

    async fn retry_job(&self, queue_name: &str, job_id: &str) -> Result<()> {
        self.job_ops.retry_job(queue_name, job_id).await
    }

    async fn remove_job(&self, queue_name: &str, job_id: &str) -> Result<()> {
        self.job_ops.remove_job(queue_name, job_id).await
    }

    async fn pause_queue(&self, queue_name: &str) -> Result<()> {
        self.queue_ops.pause_queue(queue_name).await
    }

    async fn resume_queue(&self, queue_name: &str) -> Result<()> {
        self.queue_ops.resume_queue(queue_name).await
    }

    async fn is_connected(&self) -> bool {
        if let Ok(mut conn) = self.pool.get().await {
            redis::cmd("PING")
                .query_async::<String>(&mut *conn)
                .await
                .is_ok()
        } else {
            false
        }
    }

    fn connection_url(&self) -> &str {
        &self.config.url
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl BullMQEventClient for RedisClient {
    async fn subscribe_events(
        &self,
        queue_name: &str,
        last_id: Option<&str>,
    ) -> Result<Vec<QueueEvent>> {
        self.event_ops
            .read_events(queue_name, last_id, Some(1000), 100)
            .await
    }
}
