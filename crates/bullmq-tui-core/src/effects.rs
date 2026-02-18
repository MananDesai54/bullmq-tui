//! Side effects that need to be executed by the runtime

use bullmq_core::JobState;

/// Side effects that the update function can request
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect
    None,

    /// Multiple effects to execute
    Batch(Vec<Effect>),

    // === Data Loading ===
    /// Load/refresh queue list
    LoadQueues,

    /// Load queue info for a specific queue
    LoadQueueInfo(String),

    /// Load jobs for a queue and state
    LoadJobs {
        queue: String,
        state: JobState,
        offset: usize,
        count: usize,
    },

    /// Load a specific job's details
    LoadJobDetail { queue: String, job_id: String },

    /// Load queue counts
    LoadQueueCounts(String),

    // === Job Actions ===
    /// Retry a failed job
    RetryJob { queue: String, job_id: String },

    /// Remove a job
    RemoveJob { queue: String, job_id: String },

    // === Queue Actions ===
    /// Pause a queue
    PauseQueue(String),

    /// Resume a queue
    ResumeQueue(String),

    // === Events ===
    /// Subscribe to queue events
    SubscribeEvents {
        queue: String,
        last_id: Option<String>,
    },

    // === Connection ===
    /// Connect to Redis
    Connect(String),

    /// Check connection status
    CheckConnection,

    // === Misc ===
    /// Quit the application
    Quit,
}

impl Effect {
    /// Create a batch of effects
    pub fn batch(effects: Vec<Effect>) -> Self {
        // Filter out None effects
        let effects: Vec<Effect> = effects
            .into_iter()
            .filter(|e| !matches!(e, Effect::None))
            .collect();

        match effects.len() {
            0 => Effect::None,
            1 => effects.into_iter().next().unwrap(),
            _ => Effect::Batch(effects),
        }
    }

    /// Check if this is a None effect
    pub fn is_none(&self) -> bool {
        matches!(self, Effect::None)
    }
}

/// Result of an update: new model state and effects to execute
pub struct UpdateResult {
    /// Whether the model was modified
    pub changed: bool,
    /// Effects to execute
    pub effects: Vec<Effect>,
}

impl UpdateResult {
    /// No changes, no effects
    pub fn none() -> Self {
        Self {
            changed: false,
            effects: vec![],
        }
    }

    /// Model changed, no effects
    pub fn changed() -> Self {
        Self {
            changed: true,
            effects: vec![],
        }
    }

    /// Model changed with effect
    pub fn with_effect(effect: Effect) -> Self {
        Self {
            changed: true,
            effects: if effect.is_none() { vec![] } else { vec![effect] },
        }
    }

    /// Model changed with multiple effects
    pub fn with_effects(effects: Vec<Effect>) -> Self {
        Self {
            changed: true,
            effects: effects.into_iter().filter(|e| !e.is_none()).collect(),
        }
    }

    /// No model change but execute effect
    pub fn effect_only(effect: Effect) -> Self {
        Self {
            changed: false,
            effects: if effect.is_none() { vec![] } else { vec![effect] },
        }
    }
}
