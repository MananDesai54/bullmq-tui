//! Messages (actions) for the TUI

use bullmq_core::{Job, JobState, QueueCounts, QueueEvent, QueueInfo};

/// All possible messages/actions in the application
#[derive(Debug, Clone)]
pub enum Message {
    // === Navigation ===
    /// Quit the application
    Quit,
    /// Go back to previous view
    Back,
    /// Show help screen
    ShowHelp,
    /// Hide help/overlay
    HideOverlay,

    // === Queue List ===
    /// Refresh queue list
    RefreshQueues,
    /// Queue list loaded
    QueuesLoaded(Vec<String>),
    /// Queue info loaded
    QueueInfoLoaded(String, QueueInfo),
    /// Select next queue
    SelectNextQueue,
    /// Select previous queue
    SelectPrevQueue,
    /// Open selected queue
    OpenQueue,

    // === Job List ===
    /// Refresh job list
    RefreshJobs,
    /// Jobs loaded for current state
    JobsLoaded {
        jobs: Vec<Job>,
        total: u64,
    },
    /// Queue counts updated
    CountsUpdated(QueueCounts),
    /// Select next job
    SelectNextJob,
    /// Select previous job
    SelectPrevJob,
    /// Next status tab
    NextTab,
    /// Previous status tab
    PrevTab,
    /// Switch to specific tab
    SwitchTab(JobState),
    /// Open selected job detail
    OpenJobDetail,
    /// Page down in job list
    PageDown,
    /// Page up in job list
    PageUp,

    // === Job Detail ===
    /// Job detail loaded
    JobDetailLoaded(Job),
    /// Scroll down in detail view
    ScrollDown,
    /// Scroll up in detail view
    ScrollUp,

    // === Job Actions ===
    /// Retry failed job
    RetryJob,
    /// Remove job
    RemoveJob,
    /// Job retried successfully
    JobRetried(String),
    /// Job removed successfully
    JobRemoved(String),

    // === Queue Actions ===
    /// Pause queue
    PauseQueue,
    /// Resume queue
    ResumeQueue,
    /// Queue paused
    QueuePaused(String),
    /// Queue resumed
    QueueResumed(String),

    // === Search ===
    /// Toggle search mode
    ToggleSearch,
    /// Search input changed
    SearchInput(char),
    /// Search backspace
    SearchBackspace,
    /// Search delete
    SearchDelete,
    /// Move cursor left in search
    SearchCursorLeft,
    /// Move cursor right in search
    SearchCursorRight,
    /// Clear search
    ClearSearch,
    /// Execute search
    ExecuteSearch,

    // === Confirmation ===
    /// Show confirmation dialog
    ShowConfirm,
    /// Confirm action
    Confirm,
    /// Cancel action
    Cancel,

    // === Connection ===
    /// Connection established
    Connected,
    /// Connection lost
    Disconnected,
    /// Connection error
    ConnectionError(String),

    // === Events ===
    /// Real-time event received
    EventReceived(QueueEvent),
    /// Multiple events received
    EventsReceived(Vec<QueueEvent>),

    // === Errors ===
    /// Error occurred
    Error(String),

    // === Status ===
    /// Set status message
    SetStatus(String),
    /// Clear status message
    ClearStatus,

    // === Loading ===
    /// Set loading state
    SetLoading(bool),

    // === Tick ===
    /// Periodic tick for animations/updates
    Tick,
}
