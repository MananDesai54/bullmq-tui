//! Queue discovery and operations

#[cfg(not(target_arch = "wasm32"))]
use crate::error::Result;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::{QueueCounts, QueueInfo, QueueMeta};
#[cfg(not(target_arch = "wasm32"))]
use deadpool_redis::Pool;
#[cfg(not(target_arch = "wasm32"))]
use redis::AsyncCommands;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, trace};

/// Queue operations helper
#[cfg(not(target_arch = "wasm32"))]
pub struct QueueOps {
    pool: Pool,
}

#[cfg(not(target_arch = "wasm32"))]
impl QueueOps {
    /// Create a new QueueOps instance
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Discover all BullMQ queues using SCAN (never use KEYS in production)
    ///
    /// BullMQ queues are identified by the presence of `bull:{queue}:meta` keys
    pub async fn discover_queues(&self) -> Result<Vec<String>> {
        let mut conn = self.pool.get().await?;
        let mut queues = HashSet::new();
        let mut cursor: u64 = 0;

        debug!("Starting queue discovery with SCAN");

        loop {
            // SCAN for meta keys: bull:*:meta
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg("bull:*:meta")
                .arg("COUNT")
                .arg(100)
                .query_async(&mut *conn)
                .await?;

            trace!("SCAN cursor {} -> {}, found {} keys", cursor, new_cursor, keys.len());

            for key in keys {
                // Extract queue name from "bull:{queue}:meta"
                if let Some(queue_name) = key
                    .strip_prefix("bull:")
                    .and_then(|s| s.strip_suffix(":meta"))
                {
                    queues.insert(queue_name.to_string());
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        // Also scan for queue:id keys as a fallback (some queues might not have meta)
        cursor = 0;
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg("bull:*:id")
                .arg("COUNT")
                .arg(100)
                .query_async(&mut *conn)
                .await?;

            for key in keys {
                if let Some(queue_name) = key
                    .strip_prefix("bull:")
                    .and_then(|s| s.strip_suffix(":id"))
                {
                    queues.insert(queue_name.to_string());
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        let mut result: Vec<String> = queues.into_iter().collect();
        result.sort();

        debug!("Discovered {} queues", result.len());
        Ok(result)
    }

    /// Get queue metadata
    pub async fn get_queue_meta(&self, queue_name: &str) -> Result<QueueMeta> {
        let mut conn = self.pool.get().await?;
        let key = format!("bull:{}:meta", queue_name);

        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Ok(QueueMeta::default()),
        }
    }

    /// Get detailed queue information including counts
    pub async fn get_queue_info(&self, queue_name: &str) -> Result<QueueInfo> {
        let meta = self.get_queue_meta(queue_name).await?;
        let counts = self.get_queue_counts(queue_name).await?;

        // Check if queue is paused
        let mut conn = self.pool.get().await?;
        let paused_key = format!("bull:{}:paused", queue_name);
        let is_paused: bool = conn.exists(&paused_key).await?;

        Ok(QueueInfo {
            name: queue_name.to_string(),
            prefix: format!("bull:{}", queue_name),
            is_paused,
            counts,
            meta,
        })
    }

    /// Get job counts for all states in a queue
    pub async fn get_queue_counts(&self, queue_name: &str) -> Result<QueueCounts> {
        let mut conn = self.pool.get().await?;
        let prefix = format!("bull:{}", queue_name);

        // Use a pipeline for efficiency
        let mut pipe = redis::pipe();

        // List lengths (wait, active)
        pipe.llen(format!("{}:wait", prefix));
        pipe.llen(format!("{}:active", prefix));

        // Sorted set counts (completed, failed, delayed, prioritized, waiting-children)
        pipe.zcard(format!("{}:completed", prefix));
        pipe.zcard(format!("{}:failed", prefix));
        pipe.zcard(format!("{}:delayed", prefix));
        pipe.zcard(format!("{}:prioritized", prefix));
        pipe.zcard(format!("{}:waiting-children", prefix));

        // Also check paused list
        pipe.llen(format!("{}:paused", prefix));

        let results: Vec<u64> = pipe.query_async(&mut *conn).await?;

        Ok(QueueCounts {
            waiting: results.get(0).copied().unwrap_or(0),
            active: results.get(1).copied().unwrap_or(0),
            completed: results.get(2).copied().unwrap_or(0),
            failed: results.get(3).copied().unwrap_or(0),
            delayed: results.get(4).copied().unwrap_or(0),
            prioritized: results.get(5).copied().unwrap_or(0),
            waiting_children: results.get(6).copied().unwrap_or(0),
            paused: results.get(7).copied().unwrap_or(0),
        })
    }

    /// Pause a queue
    pub async fn pause_queue(&self, queue_name: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let paused_key = format!("bull:{}:paused", queue_name);

        // Set paused marker
        let _: () = conn.set(&paused_key, "1").await?;

        debug!("Paused queue: {}", queue_name);
        Ok(())
    }

    /// Resume a paused queue
    pub async fn resume_queue(&self, queue_name: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let paused_key = format!("bull:{}:paused", queue_name);

        // Remove paused marker
        let _: () = conn.del(&paused_key).await?;

        debug!("Resumed queue: {}", queue_name);
        Ok(())
    }

    /// Check if a queue exists
    pub async fn queue_exists(&self, queue_name: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;

        // Check for meta key or id key
        let meta_key = format!("bull:{}:meta", queue_name);
        let id_key = format!("bull:{}:id", queue_name);

        let (meta_exists, id_exists): (bool, bool) =
            redis::pipe()
                .exists(&meta_key)
                .exists(&id_key)
                .query_async(&mut *conn)
                .await?;

        Ok(meta_exists || id_exists)
    }
}
