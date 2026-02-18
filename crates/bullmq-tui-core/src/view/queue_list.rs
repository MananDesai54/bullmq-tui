//! Queue list view component

use crate::model::Model;
use bullmq_core::QueueCounts;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

/// Render the queue list view
pub fn render_queue_list(model: &Model, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Queues ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if model.queue_list.queues.is_empty() {
        if model.queue_list.loading {
            let loading = ratatui::widgets::Paragraph::new("Loading queues...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading, inner);
        } else {
            let empty = ratatui::widgets::Paragraph::new("No queues found. Press 'r' to refresh.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(empty, inner);
        }
        return;
    }

    // Build table rows
    let rows: Vec<Row> = model
        .queue_list
        .queues
        .iter()
        .enumerate()
        .map(|(i, queue_name)| {
            let info = model.queue_list.queue_info.get(queue_name);

            let active = info
                .map(|i| QueueCounts::format_count(i.counts.active))
                .unwrap_or_else(|| "-".to_string());
            let waiting = info
                .map(|i| QueueCounts::format_count(i.counts.waiting))
                .unwrap_or_else(|| "-".to_string());
            let completed = info
                .map(|i| QueueCounts::format_count(i.counts.completed))
                .unwrap_or_else(|| "-".to_string());
            let failed = info
                .map(|i| QueueCounts::format_count(i.counts.failed))
                .unwrap_or_else(|| "-".to_string());
            let delayed = info
                .map(|i| QueueCounts::format_count(i.counts.delayed))
                .unwrap_or_else(|| "-".to_string());

            let paused = info
                .map(|i| if i.is_paused { "PAUSED" } else { "" })
                .unwrap_or("");

            let style = if i == model.queue_list.selected {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            let failed_style = if failed != "-" && failed != "0" {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(queue_name.as_str()),
                Cell::from(active).style(Style::default().fg(Color::Green)),
                Cell::from(waiting).style(Style::default().fg(Color::Yellow)),
                Cell::from(completed).style(Style::default().fg(Color::Cyan)),
                Cell::from(failed).style(failed_style),
                Cell::from(delayed).style(Style::default().fg(Color::Magenta)),
                Cell::from(paused).style(Style::default().fg(Color::Red)),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from("Queue"),
        Cell::from("Active"),
        Cell::from("Waiting"),
        Cell::from("Done"),
        Cell::from("Failed"),
        Cell::from("Delayed"),
        Cell::from("Status"),
    ])
    .style(Style::default().fg(Color::Cyan).bold())
    .bottom_margin(1);

    let widths = [
        Constraint::Min(20),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    state.select(Some(model.queue_list.selected));

    frame.render_stateful_widget(table, inner, &mut state);
}
