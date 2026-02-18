//! Job table component

use crate::model::Model;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

/// Render the job table
pub fn render_job_table(model: &Model, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(format!(
            " Jobs ({}/{}) ",
            model.job_list.jobs.len(),
            model.job_list.total
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if model.job_list.jobs.is_empty() {
        if model.job_list.loading {
            let loading = ratatui::widgets::Paragraph::new("Loading jobs...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading, inner);
        } else {
            let empty = ratatui::widgets::Paragraph::new("No jobs in this state")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(empty, inner);
        }
        return;
    }

    // Build table rows
    let rows: Vec<Row> = model
        .job_list
        .jobs
        .iter()
        .enumerate()
        .map(|(i, job)| {
            let id_display = if job.id.len() > 12 {
                format!("{}...", &job.id[..12])
            } else {
                job.id.clone()
            };

            let name_display = if job.name.len() > 20 {
                format!("{}...", &job.name[..20])
            } else {
                job.name.clone()
            };

            let progress = job
                .progress_percent()
                .map(|p| format!("{}%", p))
                .unwrap_or_else(|| "-".to_string());

            let attempts = if job.attempts_made > 0 {
                format!("{}", job.attempts_made)
            } else {
                "-".to_string()
            };

            let age = job.age_display();

            let error = if job.failed_reason.is_some() {
                "!"
            } else {
                ""
            };

            let style = if i == model.job_list.selected {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            let progress_style = match job.progress_percent() {
                Some(p) if p >= 100 => Style::default().fg(Color::Green),
                Some(p) if p >= 50 => Style::default().fg(Color::Yellow),
                Some(_) => Style::default().fg(Color::Cyan),
                None => Style::default().fg(Color::Gray),
            };

            Row::new(vec![
                Cell::from(if i == model.job_list.selected {
                    ">"
                } else {
                    " "
                }),
                Cell::from(id_display),
                Cell::from(name_display),
                Cell::from(progress).style(progress_style),
                Cell::from(attempts),
                Cell::from(age).style(Style::default().fg(Color::Gray)),
                Cell::from(error).style(Style::default().fg(Color::Red)),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(" "),
        Cell::from("ID"),
        Cell::from("Name"),
        Cell::from("Progress"),
        Cell::from("Tries"),
        Cell::from("Age"),
        Cell::from(""),
    ])
    .style(Style::default().fg(Color::Cyan).bold())
    .bottom_margin(1);

    let widths = [
        Constraint::Length(1),
        Constraint::Length(15),
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    state.select(Some(model.job_list.selected));

    frame.render_stateful_widget(table, inner, &mut state);
}
