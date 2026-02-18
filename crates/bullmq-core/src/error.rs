//! Error types for bullmq-core

use thiserror::Error;

/// Result type for bullmq-core operations
pub type Result<T> = std::result::Result<T, BullMQError>;

/// Errors that can occur when interacting with BullMQ
#[derive(Error, Debug)]
pub enum BullMQError {
    /// Redis connection error
    #[error("Redis connection error: {0}")]
    Connection(String),

    /// Redis command execution error
    #[error("Redis command error: {0}")]
    Command(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Queue not found
    #[error("Queue not found: {0}")]
    QueueNotFound(String),

    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(String),

    /// Invalid job state
    #[error("Invalid job state: {0}")]
    InvalidState(String),

    /// Pool error
    #[error("Connection pool error: {0}")]
    Pool(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<redis::RedisError> for BullMQError {
    fn from(err: redis::RedisError) -> Self {
        BullMQError::Command(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<deadpool_redis::PoolError> for BullMQError {
    fn from(err: deadpool_redis::PoolError) -> Self {
        BullMQError::Pool(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<deadpool_redis::CreatePoolError> for BullMQError {
    fn from(err: deadpool_redis::CreatePoolError) -> Self {
        BullMQError::Pool(err.to_string())
    }
}
