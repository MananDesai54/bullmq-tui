//! Application state (Model) for the TUI

use bullmq_core::{Job, JobState, QueueCounts, QueueInfo};
use std::collections::HashMap;

/// Current view/screen in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Queue list view (main menu)
    #[default]
    QueueList,
    /// Job list view for a specific queue
    JobList,
    /// Job detail view
    JobDetail,
    /// Help screen
    Help,
    /// Search/filter overlay
    Search,
    /// Confirmation dialog
    Confirm,
}

/// Connection status
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    /// Not connected
    #[default]
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected successfully
    Connected,
    /// Connection error
    Error(String),
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    pub fn display(&self) -> &str {
        match self {
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Error(_) => "Error",
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            ConnectionStatus::Disconnected => "○",
            ConnectionStatus::Connecting => "◐",
            ConnectionStatus::Connected => "●",
            ConnectionStatus::Error(_) => "✗",
        }
    }
}

/// Confirmation action type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Confirm retry of a job
    RetryJob { queue: String, job_id: String },
    /// Confirm removal of a job
    RemoveJob { queue: String, job_id: String },
    /// Confirm pause of a queue
    PauseQueue { queue: String },
    /// Confirm resume of a queue
    ResumeQueue { queue: String },
}

impl ConfirmAction {
    pub fn message(&self) -> String {
        match self {
            ConfirmAction::RetryJob { job_id, .. } => {
                format!("Retry job {}?", job_id)
            }
            ConfirmAction::RemoveJob { job_id, .. } => {
                format!("Remove job {}? This cannot be undone.", job_id)
            }
            ConfirmAction::PauseQueue { queue } => {
                format!("Pause queue {}?", queue)
            }
            ConfirmAction::ResumeQueue { queue } => {
                format!("Resume queue {}?", queue)
            }
        }
    }
}

/// State for the queue list view
#[derive(Debug, Clone, Default)]
pub struct QueueListState {
    /// List of discovered queue names
    pub queues: Vec<String>,
    /// Index of selected queue
    pub selected: usize,
    /// Queue information (cached)
    pub queue_info: HashMap<String, QueueInfo>,
    /// Whether we're loading queues
    pub loading: bool,
}

impl QueueListState {
    pub fn selected_queue(&self) -> Option<&str> {
        self.queues.get(self.selected).map(|s| s.as_str())
    }

    pub fn select_next(&mut self) {
        if !self.queues.is_empty() {
            self.selected = (self.selected + 1) % self.queues.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.queues.is_empty() {
            self.selected = self.selected.checked_sub(1).unwrap_or(self.queues.len() - 1);
        }
    }
}

/// State for the job list view
#[derive(Debug, Clone, Default)]
pub struct JobListState {
    /// Current queue name
    pub queue_name: String,
    /// Current job state tab
    pub current_tab: JobState,
    /// Jobs for the current state
    pub jobs: Vec<Job>,
    /// Index of selected job
    pub selected: usize,
    /// Scroll offset for pagination
    pub offset: usize,
    /// Total jobs in current state
    pub total: u64,
    /// Page size for fetching
    pub page_size: usize,
    /// Whether we're loading jobs
    pub loading: bool,
    /// Queue counts (for tabs)
    pub counts: QueueCounts,
}

impl JobListState {
    pub fn new(queue_name: String) -> Self {
        Self {
            queue_name,
            current_tab: JobState::Active,
            page_size: 50,
            ..Default::default()
        }
    }

    pub fn selected_job(&self) -> Option<&Job> {
        self.jobs.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.jobs.is_empty() {
            self.selected = (self.selected + 1).min(self.jobs.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn next_tab(&mut self) {
        let tabs = JobState::all_tabs();
        if let Some(idx) = tabs.iter().position(|&s| s == self.current_tab) {
            self.current_tab = tabs[(idx + 1) % tabs.len()];
            self.selected = 0;
            self.offset = 0;
        }
    }

    pub fn prev_tab(&mut self) {
        let tabs = JobState::all_tabs();
        if let Some(idx) = tabs.iter().position(|&s| s == self.current_tab) {
            self.current_tab = tabs[idx.checked_sub(1).unwrap_or(tabs.len() - 1)];
            self.selected = 0;
            self.offset = 0;
        }
    }

    pub fn page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.page_size);
        self.selected = 0;
    }

    pub fn page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.page_size);
        self.selected = 0;
    }
}

/// State for the job detail view
#[derive(Debug, Clone, Default)]
pub struct JobDetailState {
    /// The job being viewed
    pub job: Option<Job>,
    /// Scroll position for long content
    pub scroll: usize,
    /// Whether we're loading job details
    pub loading: bool,
}

impl JobDetailState {
    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

/// Search/filter state
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Search input text
    pub input: String,
    /// Cursor position in input
    pub cursor: usize,
    /// Whether search is active
    pub active: bool,
}

impl SearchState {
    pub fn insert(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.input.len());
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }
}

/// Status message for the status bar
#[derive(Debug, Clone, Default)]
pub struct StatusMessage {
    /// Message text
    pub text: String,
    /// Whether this is an error message
    pub is_error: bool,
    /// Timestamp when message was set (for auto-clear)
    pub timestamp: Option<i64>,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
            timestamp: Some(chrono::Utc::now().timestamp()),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
            timestamp: Some(chrono::Utc::now().timestamp()),
        }
    }

    pub fn clear() -> Self {
        Self::default()
    }
}

/// Main application state (Model)
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Current view
    pub view: View,
    /// Previous view (for back navigation)
    pub prev_view: Option<View>,
    /// Connection status
    pub connection: ConnectionStatus,
    /// Redis URL
    pub redis_url: String,
    /// Queue list state
    pub queue_list: QueueListState,
    /// Job list state
    pub job_list: JobListState,
    /// Job detail state
    pub job_detail: JobDetailState,
    /// Search state
    pub search: SearchState,
    /// Pending confirmation action
    pub confirm_action: Option<ConfirmAction>,
    /// Status bar message
    pub status: StatusMessage,
    /// Whether the application should quit
    pub should_quit: bool,
    /// Last event ID per queue (for event subscription)
    pub last_event_ids: HashMap<String, String>,
}

impl Model {
    /// Create a new model with the given Redis URL
    pub fn new(redis_url: impl Into<String>) -> Self {
        Self {
            redis_url: redis_url.into(),
            ..Default::default()
        }
    }

    /// Navigate to a new view
    pub fn navigate(&mut self, view: View) {
        self.prev_view = Some(self.view);
        self.view = view;
    }

    /// Go back to previous view
    pub fn go_back(&mut self) {
        if let Some(prev) = self.prev_view.take() {
            self.view = prev;
        }
    }

    /// Set status message
    pub fn set_status(&mut self, msg: StatusMessage) {
        self.status = msg;
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status = StatusMessage::clear();
    }
}
