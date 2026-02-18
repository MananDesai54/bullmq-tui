//! View components for rendering the TUI

mod help;
mod job_detail;
mod job_table;
mod queue_list;
mod status_tabs;
mod widgets;

pub use help::render_help;
pub use job_detail::render_job_detail;
pub use job_table::render_job_table;
pub use queue_list::render_queue_list;
pub use status_tabs::render_status_tabs;
pub use widgets::*;

use crate::model::{Model, View};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Main render function that dispatches to the appropriate view
pub fn render(model: &Model, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer/status
        ])
        .split(area);

    // Render header
    render_header(model, frame, chunks[0]);

    // Render main content based on current view
    match model.view {
        View::QueueList => render_queue_list(model, frame, chunks[1]),
        View::JobList => render_job_list_view(model, frame, chunks[1]),
        View::JobDetail => render_job_detail(model, frame, chunks[1]),
        View::Help => {
            // Render underlying view first
            if let Some(prev) = model.prev_view {
                match prev {
                    View::QueueList => render_queue_list(model, frame, chunks[1]),
                    View::JobList => render_job_list_view(model, frame, chunks[1]),
                    View::JobDetail => render_job_detail(model, frame, chunks[1]),
                    _ => {}
                }
            }
            // Render help overlay
            render_help_overlay(model, frame, area);
        }
        View::Search => {
            // Render underlying view first
            if let Some(prev) = model.prev_view {
                match prev {
                    View::QueueList => render_queue_list(model, frame, chunks[1]),
                    View::JobList => render_job_list_view(model, frame, chunks[1]),
                    View::JobDetail => render_job_detail(model, frame, chunks[1]),
                    _ => {}
                }
            }
            // Render search overlay
            render_search_overlay(model, frame, area);
        }
        View::Confirm => {
            // Render underlying view first
            if let Some(prev) = model.prev_view {
                match prev {
                    View::QueueList => render_queue_list(model, frame, chunks[1]),
                    View::JobList => render_job_list_view(model, frame, chunks[1]),
                    View::JobDetail => render_job_detail(model, frame, chunks[1]),
                    _ => {}
                }
            }
            // Render confirm overlay
            render_confirm_overlay(model, frame, area);
        }
    }

    // Render footer/status bar
    render_footer(model, frame, chunks[2]);
}

/// Render the header bar
fn render_header(model: &Model, frame: &mut Frame, area: Rect) {
    let connection_status = format!(
        "{} {}",
        model.connection.symbol(),
        model.connection.display()
    );

    let title = match model.view {
        View::QueueList => "BullMQ Monitor".to_string(),
        View::JobList => format!("Queue: {}", model.job_list.queue_name),
        View::JobDetail => {
            if let Some(job) = &model.job_detail.job {
                format!("Job: {}", job.id)
            } else {
                "Job Detail".to_string()
            }
        }
        View::Help => "Help".to_string(),
        View::Search => "Search".to_string(),
        View::Confirm => "Confirm".to_string(),
    };

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(area);

    let title_widget = Paragraph::new(title)
        .style(Style::default().bold())
        .alignment(Alignment::Left);

    let status_style = match &model.connection {
        crate::model::ConnectionStatus::Connected => Style::default().fg(Color::Green),
        crate::model::ConnectionStatus::Connecting => Style::default().fg(Color::Yellow),
        crate::model::ConnectionStatus::Disconnected => Style::default().fg(Color::Red),
        crate::model::ConnectionStatus::Error(_) => Style::default().fg(Color::Red),
    };

    let status_widget = Paragraph::new(connection_status)
        .style(status_style)
        .alignment(Alignment::Right);

    frame.render_widget(title_widget, header_layout[0]);
    frame.render_widget(status_widget, header_layout[1]);
}

/// Render the footer/status bar
fn render_footer(model: &Model, frame: &mut Frame, area: Rect) {
    let footer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(40)])
        .split(area);

    // Status message (left)
    let status_style = if model.status.is_error {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    let status = Paragraph::new(model.status.text.as_str())
        .style(status_style)
        .alignment(Alignment::Left);

    // Key hints (right)
    let hints = match model.view {
        View::QueueList => "[j/k]Move [Enter]Open [r]Refresh [?]Help [q]Quit",
        View::JobList => "[Tab]Tabs [j/k]Move [Enter]Detail [r]Retry [d]Del [?]Help [q]Back",
        View::JobDetail => "[j/k]Scroll [r]Retry [d]Del [?]Help [q]Back",
        View::Help => "[q/Esc]Close",
        View::Search => "[Enter]Search [Esc]Cancel",
        View::Confirm => "[y]Confirm [n]Cancel",
    };

    let hints_widget = Paragraph::new(hints)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);

    frame.render_widget(status, footer_layout[0]);
    frame.render_widget(hints_widget, footer_layout[1]);
}

/// Render the job list view (tabs + table)
fn render_job_list_view(model: &Model, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs
            Constraint::Min(0),    // Job table
        ])
        .split(area);

    // Render status tabs
    render_status_tabs(model, frame, chunks[0]);

    // Render job table
    render_job_table(model, frame, chunks[1]);
}

/// Render help as an overlay
fn render_help_overlay(model: &Model, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 80, area);
    frame.render_widget(Clear, popup_area);
    render_help(model, frame, popup_area);
}

/// Render search as an overlay
fn render_search_overlay(model: &Model, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 15, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let input = Paragraph::new(model.search.input.as_str())
        .style(Style::default().fg(Color::White));
    frame.render_widget(input, inner);

    // Show cursor
    frame.set_cursor_position(Position::new(
        inner.x + model.search.cursor as u16,
        inner.y,
    ));
}

/// Render confirmation dialog as an overlay
fn render_confirm_overlay(model: &Model, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 20, area);
    frame.render_widget(Clear, popup_area);

    let message = model
        .confirm_action
        .as_ref()
        .map(|a| a.message())
        .unwrap_or_else(|| "Confirm action?".to_string());

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let message_widget = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let buttons = Paragraph::new("[y] Yes   [n] No")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(message_widget, layout[0]);
    frame.render_widget(buttons, layout[1]);
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
