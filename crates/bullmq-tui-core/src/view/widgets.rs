//! Reusable widget components

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

/// Render a progress bar
pub fn progress_bar(percent: u8, area: Rect, frame: &mut Frame) {
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(if percent >= 100 {
                    Color::Green
                } else if percent >= 50 {
                    Color::Yellow
                } else {
                    Color::Cyan
                })
                .bg(Color::DarkGray),
        )
        .percent(percent as u16)
        .label(format!("{}%", percent));

    frame.render_widget(gauge, area);
}

/// Create a spinner character based on tick count
pub fn spinner(tick: u64) -> char {
    const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    SPINNER_CHARS[(tick as usize) % SPINNER_CHARS.len()]
}

/// Create a loading indicator string
pub fn loading_indicator(tick: u64) -> String {
    format!("{} Loading...", spinner(tick))
}

/// Truncate a string to fit in a given width
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        ".".repeat(max_width)
    } else {
        format!("{}...", &s[..max_width - 3])
    }
}

/// Format a duration in a human-readable way
pub fn format_duration(seconds: i64) -> String {
    if seconds < 0 {
        return "unknown".to_string();
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format bytes in a human-readable way
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
