//! BullMQ Redis client library
//!
//! This crate provides a Rust interface to BullMQ queues stored in Redis.
//! It supports queue discovery, job management, and real-time event subscription.
//!
//! # Features
//!
//! - Queue discovery using SCAN (production-safe, never uses KEYS)
//! - Job listing with pagination for all states
//! - Job operations: retry, remove
//! - Queue operations: pause, resume
//! - Real-time event streaming via Redis Streams
//!
//! # Example
//!
//! ```no_run
//! use bullmq_core::{RedisClient, BullMQClient, JobState};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Connect to Redis
//!     let client = RedisClient::connect("redis://localhost:6379").await?;
//!
//!     // Discover queues
//!     let queues = client.discover_queues().await?;
//!     println!("Found queues: {:?}", queues);
//!
//!     // Get jobs from a queue
//!     if let Some(queue) = queues.first() {
//!         let jobs = client.get_jobs(queue, JobState::Waiting, 0, 10).await?;
//!         for job in jobs {
//!             println!("Job {}: {}", job.id, job.name);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod models;

#[cfg(not(target_arch = "wasm32"))]
pub mod events;
#[cfg(not(target_arch = "wasm32"))]
pub mod job;
#[cfg(not(target_arch = "wasm32"))]
pub mod queue;
#[cfg(not(target_arch = "wasm32"))]
pub mod redis_client;

// Re-exports
pub use client::BullMQClient;
pub use error::{BullMQError, Result};
pub use models::*;

#[cfg(not(target_arch = "wasm32"))]
pub use client::BullMQEventClient;
#[cfg(not(target_arch = "wasm32"))]
pub use events::EventOps;
#[cfg(not(target_arch = "wasm32"))]
pub use job::JobOps;
#[cfg(not(target_arch = "wasm32"))]
pub use queue::QueueOps;
#[cfg(not(target_arch = "wasm32"))]
pub use redis_client::{RedisClient, RedisConfig};
