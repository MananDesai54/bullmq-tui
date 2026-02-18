//! Status tabs component for job list view

use crate::model::Model;
use bullmq_core::{JobState, QueueCounts};
use ratatui::prelude::*;
use ratatui::widgets::Tabs;

/// Render the status tabs
pub fn render_status_tabs(model: &Model, frame: &mut Frame, area: Rect) {
    let tabs: Vec<Line> = JobState::all_tabs()
        .iter()
        .map(|&state| {
            let count = model.job_list.counts.get(state);
            let count_str = QueueCounts::format_count(count);
            let label = format!("[{}:{}]", state.display_name(), count_str);

            let style = if state == model.job_list.current_tab {
                match state {
                    JobState::Active => Style::default().fg(Color::Black).bg(Color::Green).bold(),
                    JobState::Waiting => Style::default().fg(Color::Black).bg(Color::Yellow).bold(),
                    JobState::Completed => Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    JobState::Failed => Style::default().fg(Color::White).bg(Color::Red).bold(),
                    JobState::Delayed => Style::default().fg(Color::Black).bg(Color::Magenta).bold(),
                    JobState::Prioritized => Style::default().fg(Color::Black).bg(Color::Blue).bold(),
                    JobState::WaitingChildren => {
                        Style::default().fg(Color::Black).bg(Color::LightBlue).bold()
                    }
                    _ => Style::default().fg(Color::Black).bg(Color::White).bold(),
                }
            } else {
                let fg = match state {
                    JobState::Active => Color::Green,
                    JobState::Waiting => Color::Yellow,
                    JobState::Completed => Color::Cyan,
                    JobState::Failed => Color::Red,
                    JobState::Delayed => Color::Magenta,
                    JobState::Prioritized => Color::Blue,
                    JobState::WaitingChildren => Color::LightBlue,
                    _ => Color::White,
                };
                Style::default().fg(fg)
            };

            Line::from(Span::styled(label, style))
        })
        .collect();

    let tabs_widget = Tabs::new(tabs)
        .divider(Span::raw(" "))
        .padding(" ", " ");

    frame.render_widget(tabs_widget, area);
}
