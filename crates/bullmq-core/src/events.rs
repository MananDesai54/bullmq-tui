//! Redis Stream event subscription

#[cfg(not(target_arch = "wasm32"))]
use crate::error::Result;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::QueueEvent;
#[cfg(not(target_arch = "wasm32"))]
use deadpool_redis::Pool;
#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, trace};

/// Event subscription and streaming operations
#[cfg(not(target_arch = "wasm32"))]
pub struct EventOps {
    pool: Pool,
}

#[cfg(not(target_arch = "wasm32"))]
impl EventOps {
    /// Create a new EventOps instance
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Read events from a queue's event stream
    ///
    /// Uses XREAD with BLOCK for efficient waiting.
    /// Returns events starting after `last_id`, or from latest if `None`.
    pub async fn read_events(
        &self,
        queue_name: &str,
        last_id: Option<&str>,
        block_ms: Option<u64>,
        count: usize,
    ) -> Result<Vec<QueueEvent>> {
        let mut conn = self.pool.get().await?;
        let stream_key = format!("bull:{}:events", queue_name);

        // Start ID: either after last received, or "$" for latest
        let start_id = last_id.unwrap_or("$");

        let mut cmd = redis::cmd("XREAD");

        // Add BLOCK if specified (for waiting)
        if let Some(ms) = block_ms {
            cmd.arg("BLOCK").arg(ms);
        }

        cmd.arg("COUNT")
            .arg(count)
            .arg("STREAMS")
            .arg(&stream_key)
            .arg(start_id);

        // Execute command - Redis returns: Vec<(stream_name, Vec<(entry_id, Vec<(field, value)>)>)>
        let result: Option<Vec<(String, Vec<(String, Vec<(String, String)>)>)>> =
            cmd.query_async(&mut *conn).await?;

        let events = match result {
            Some(streams) => {
                let mut events = Vec::new();

                for (_stream_name, entries) in streams {
                    for (entry_id, fields) in entries {
                        // fields is already Vec<(String, String)>
                        if let Some(event) =
                            QueueEvent::from_stream_entry(entry_id, queue_name.to_string(), &fields)
                        {
                            trace!("Parsed event: {:?}", event.event_type);
                            events.push(event);
                        }
                    }
                }

                events
            }
            None => Vec::new(),
        };

        if !events.is_empty() {
            debug!("Read {} events from {}", events.len(), queue_name);
        }

        Ok(events)
    }

    /// Read events from multiple queues at once
    pub async fn read_events_multi(
        &self,
        queues: &[(String, String)], // (queue_name, last_id)
        block_ms: Option<u64>,
        count: usize,
    ) -> Result<Vec<QueueEvent>> {
        if queues.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.pool.get().await?;

        let mut cmd = redis::cmd("XREAD");

        if let Some(ms) = block_ms {
            cmd.arg("BLOCK").arg(ms);
        }

        cmd.arg("COUNT").arg(count).arg("STREAMS");

        // Add all stream keys
        for (queue_name, _) in queues {
            cmd.arg(format!("bull:{}:events", queue_name));
        }

        // Add all IDs
        for (_, last_id) in queues {
            cmd.arg(last_id);
        }

        let result: Option<Vec<(String, Vec<(String, Vec<(String, String)>)>)>> =
            cmd.query_async(&mut *conn).await?;

        let mut all_events = Vec::new();

        if let Some(streams) = result {
            for (stream_name, entries) in streams {
                // Extract queue name from stream key
                let queue_name = stream_name
                    .strip_prefix("bull:")
                    .and_then(|s| s.strip_suffix(":events"))
                    .unwrap_or(&stream_name)
                    .to_string();

                for (entry_id, fields) in entries {
                    // fields is already Vec<(String, String)>
                    if let Some(event) =
                        QueueEvent::from_stream_entry(entry_id, queue_name.clone(), &fields)
                    {
                        all_events.push(event);
                    }
                }
            }
        }

        Ok(all_events)
    }

    /// Get the latest event ID from a stream (for initialization)
    pub async fn get_latest_event_id(&self, queue_name: &str) -> Result<Option<String>> {
        let mut conn = self.pool.get().await?;
        let stream_key = format!("bull:{}:events", queue_name);

        // XREVRANGE to get the latest entry
        let result: Vec<(String, Vec<String>)> = redis::cmd("XREVRANGE")
            .arg(&stream_key)
            .arg("+")
            .arg("-")
            .arg("COUNT")
            .arg(1)
            .query_async(&mut *conn)
            .await?;

        Ok(result.first().map(|(id, _)| id.clone()))
    }

    /// Trim old events from the stream (housekeeping)
    pub async fn trim_events(&self, queue_name: &str, max_len: usize) -> Result<u64> {
        let mut conn = self.pool.get().await?;
        let stream_key = format!("bull:{}:events", queue_name);

        let trimmed: u64 = redis::cmd("XTRIM")
            .arg(&stream_key)
            .arg("MAXLEN")
            .arg("~") // Approximate trimming for efficiency
            .arg(max_len)
            .query_async(&mut *conn)
            .await?;

        if trimmed > 0 {
            debug!("Trimmed {} events from {}", trimmed, queue_name);
        }

        Ok(trimmed)
    }
}
