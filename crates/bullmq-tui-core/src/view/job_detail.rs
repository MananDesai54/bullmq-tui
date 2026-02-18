//! Job detail view component

use crate::model::Model;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Render the job detail view
pub fn render_job_detail(model: &Model, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Job Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let job = match &model.job_detail.job {
        Some(j) => j,
        None => {
            let loading = Paragraph::new("Loading...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading, inner);
            return;
        }
    };

    // Build detail text
    let mut lines: Vec<Line> = vec![];

    // Header info
    lines.push(Line::from(vec![
        Span::styled("ID: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw(&job.id),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw(&job.name),
    ]));

    lines.push(Line::from(vec![
        Span::styled("State: ", Style::default().fg(Color::Cyan).bold()),
        Span::styled(
            job.state.display_name(),
            Style::default().fg(match job.state {
                bullmq_core::JobState::Active => Color::Green,
                bullmq_core::JobState::Waiting => Color::Yellow,
                bullmq_core::JobState::Completed => Color::Cyan,
                bullmq_core::JobState::Failed => Color::Red,
                bullmq_core::JobState::Delayed => Color::Magenta,
                _ => Color::White,
            }),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Queue: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw(&job.queue_name),
    ]));

    lines.push(Line::raw(""));

    // Timestamps
    lines.push(Line::from(Span::styled(
        "Timestamps",
        Style::default().fg(Color::Yellow).bold(),
    )));
    lines.push(Line::from("─".repeat(40)));

    if let Some(created) = job.created_at() {
        lines.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Gray)),
            Span::raw(created.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));
    }

    if let Some(started) = job.started_at() {
        lines.push(Line::from(vec![
            Span::styled("Started: ", Style::default().fg(Color::Gray)),
            Span::raw(started.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));
    }

    if let Some(finished) = job.finished_at() {
        lines.push(Line::from(vec![
            Span::styled("Finished: ", Style::default().fg(Color::Gray)),
            Span::raw(finished.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));
    }

    lines.push(Line::raw(""));

    // Progress
    lines.push(Line::from(Span::styled(
        "Progress",
        Style::default().fg(Color::Yellow).bold(),
    )));
    lines.push(Line::from("─".repeat(40)));

    if let Some(percent) = job.progress_percent() {
        let bar_width = 30;
        let filled = (percent as usize * bar_width) / 100;
        let empty = bar_width - filled;
        let bar = format!("[{}{}] {}%", "█".repeat(filled), "░".repeat(empty), percent);
        lines.push(Line::from(bar));
    } else {
        lines.push(Line::from("No progress data"));
    }

    lines.push(Line::from(vec![
        Span::styled("Attempts: ", Style::default().fg(Color::Gray)),
        Span::raw(format!("{}", job.attempts_made)),
    ]));

    lines.push(Line::raw(""));

    // Data
    lines.push(Line::from(Span::styled(
        "Data",
        Style::default().fg(Color::Yellow).bold(),
    )));
    lines.push(Line::from("─".repeat(40)));

    let data_str = serde_json::to_string_pretty(&job.data).unwrap_or_else(|_| "{}".to_string());
    for line in data_str.lines() {
        lines.push(Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        )));
    }

    lines.push(Line::raw(""));

    // Return value (if completed)
    if let Some(return_val) = &job.return_value {
        lines.push(Line::from(Span::styled(
            "Return Value",
            Style::default().fg(Color::Yellow).bold(),
        )));
        lines.push(Line::from("─".repeat(40)));

        let return_str =
            serde_json::to_string_pretty(return_val).unwrap_or_else(|_| "null".to_string());
        for line in return_str.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Green),
            )));
        }

        lines.push(Line::raw(""));
    }

    // Error (if failed)
    if let Some(error) = &job.failed_reason {
        lines.push(Line::from(Span::styled(
            "Error",
            Style::default().fg(Color::Red).bold(),
        )));
        lines.push(Line::from("─".repeat(40)));
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(Color::Red),
        )));

        if let Some(stack) = &job.stack_trace {
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "Stack Trace",
                Style::default().fg(Color::Red).bold(),
            )));
            for line in stack {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        lines.push(Line::raw(""));
    }

    // Options
    lines.push(Line::from(Span::styled(
        "Options",
        Style::default().fg(Color::Yellow).bold(),
    )));
    lines.push(Line::from("─".repeat(40)));

    let opts_str =
        serde_json::to_string_pretty(&job.opts).unwrap_or_else(|_| "{}".to_string());
    for line in opts_str.lines() {
        lines.push(Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Gray),
        )));
    }

    // Apply scroll offset
    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(model.job_detail.scroll)
        .collect();

    let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner);
}
