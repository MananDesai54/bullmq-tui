//! Event handling for terminal input

use bullmq_tui_core::{Message, Model, View};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

/// Convert a terminal event to a TUI message
pub fn handle_event(event: Event, model: &Model) -> Option<Message> {
    match event {
        Event::Key(key) => handle_key_event(key, model),
        Event::Resize(_, _) => None, // Ratatui handles resize
        _ => None,
    }
}

/// Convert a key event to a message based on current view
fn handle_key_event(key: KeyEvent, model: &Model) -> Option<Message> {
    // Global keybindings (work in any view)
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Message::Quit);
        }
        _ => {}
    }

    // View-specific keybindings
    match model.view {
        View::QueueList => handle_queue_list_keys(key),
        View::JobList => handle_job_list_keys(key),
        View::JobDetail => handle_job_detail_keys(key),
        View::Help => handle_help_keys(key),
        View::Search => handle_search_keys(key),
        View::Confirm => handle_confirm_keys(key),
    }
}

fn handle_queue_list_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
        KeyCode::Char('?') => Some(Message::ShowHelp),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextQueue),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevQueue),
        KeyCode::Enter => Some(Message::OpenQueue),
        KeyCode::Char('r') => Some(Message::RefreshQueues),
        KeyCode::Char('/') => Some(Message::ToggleSearch),
        _ => None,
    }
}

fn handle_job_list_keys(key: KeyEvent) -> Option<Message> {
    // Check for Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => return Some(Message::PageDown),
            KeyCode::Char('u') => return Some(Message::PageUp),
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::Back),
        KeyCode::Char('?') => Some(Message::ShowHelp),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextJob),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevJob),
        KeyCode::Tab => Some(Message::NextTab),
        KeyCode::BackTab => Some(Message::PrevTab),
        KeyCode::Enter => Some(Message::OpenJobDetail),
        KeyCode::Char('r') => Some(Message::RetryJob),
        KeyCode::Char('d') => Some(Message::RemoveJob),
        KeyCode::Char('p') => Some(Message::PauseQueue),
        KeyCode::Char('/') => Some(Message::ToggleSearch),
        KeyCode::Char('R') => Some(Message::RefreshJobs),
        KeyCode::PageDown => Some(Message::PageDown),
        KeyCode::PageUp => Some(Message::PageUp),
        // Number keys for quick tab switching
        KeyCode::Char('1') => Some(Message::SwitchTab(bullmq_core::JobState::Active)),
        KeyCode::Char('2') => Some(Message::SwitchTab(bullmq_core::JobState::Waiting)),
        KeyCode::Char('3') => Some(Message::SwitchTab(bullmq_core::JobState::Completed)),
        KeyCode::Char('4') => Some(Message::SwitchTab(bullmq_core::JobState::Failed)),
        KeyCode::Char('5') => Some(Message::SwitchTab(bullmq_core::JobState::Delayed)),
        KeyCode::Char('6') => Some(Message::SwitchTab(bullmq_core::JobState::Prioritized)),
        KeyCode::Char('7') => Some(Message::SwitchTab(bullmq_core::JobState::WaitingChildren)),
        _ => None,
    }
}

fn handle_job_detail_keys(key: KeyEvent) -> Option<Message> {
    // Check for Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => return Some(Message::ScrollDown),
            KeyCode::Char('u') => return Some(Message::ScrollUp),
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::Back),
        KeyCode::Char('?') => Some(Message::ShowHelp),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ScrollDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ScrollUp),
        KeyCode::Char('r') => Some(Message::RetryJob),
        KeyCode::Char('d') => Some(Message::RemoveJob),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        _ => None,
    }
}

fn handle_help_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => Some(Message::HideOverlay),
        _ => None,
    }
}

fn handle_search_keys(key: KeyEvent) -> Option<Message> {
    // Check for Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if key.code == KeyCode::Char('u') {
            return Some(Message::ClearSearch);
        }
    }

    match key.code {
        KeyCode::Esc => Some(Message::HideOverlay),
        KeyCode::Enter => Some(Message::ExecuteSearch),
        KeyCode::Backspace => Some(Message::SearchBackspace),
        KeyCode::Delete => Some(Message::SearchDelete),
        KeyCode::Left => Some(Message::SearchCursorLeft),
        KeyCode::Right => Some(Message::SearchCursorRight),
        KeyCode::Char(c) => Some(Message::SearchInput(c)),
        _ => None,
    }
}

fn handle_confirm_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(Message::Confirm),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Message::Cancel),
        _ => None,
    }
}
