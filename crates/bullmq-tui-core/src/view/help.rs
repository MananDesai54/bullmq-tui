//! Help view component

use crate::model::Model;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Render the help screen
pub fn render_help(_model: &Model, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(Span::styled(
            "BullMQ Monitor - Keyboard Shortcuts",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from("─".repeat(40)),
        Line::from(vec![
            Span::styled("q / Esc    ", Style::default().fg(Color::Green)),
            Span::raw("Go back / Quit"),
        ]),
        Line::from(vec![
            Span::styled("?          ", Style::default().fg(Color::Green)),
            Span::raw("Show this help"),
        ]),
        Line::from(vec![
            Span::styled("j / ↓      ", Style::default().fg(Color::Green)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("k / ↑      ", Style::default().fg(Color::Green)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("Enter      ", Style::default().fg(Color::Green)),
            Span::raw("Open selected item"),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Queue List",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from("─".repeat(40)),
        Line::from(vec![
            Span::styled("r          ", Style::default().fg(Color::Green)),
            Span::raw("Refresh queue list"),
        ]),
        Line::from(vec![
            Span::styled("Enter      ", Style::default().fg(Color::Green)),
            Span::raw("Open queue"),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Job List",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from("─".repeat(40)),
        Line::from(vec![
            Span::styled("Tab        ", Style::default().fg(Color::Green)),
            Span::raw("Next status tab"),
        ]),
        Line::from(vec![
            Span::styled("Shift+Tab  ", Style::default().fg(Color::Green)),
            Span::raw("Previous status tab"),
        ]),
        Line::from(vec![
            Span::styled("r          ", Style::default().fg(Color::Green)),
            Span::raw("Retry failed job"),
        ]),
        Line::from(vec![
            Span::styled("d          ", Style::default().fg(Color::Green)),
            Span::raw("Delete job"),
        ]),
        Line::from(vec![
            Span::styled("p          ", Style::default().fg(Color::Green)),
            Span::raw("Pause/Resume queue"),
        ]),
        Line::from(vec![
            Span::styled("/          ", Style::default().fg(Color::Green)),
            Span::raw("Search jobs"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+D     ", Style::default().fg(Color::Green)),
            Span::raw("Page down"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+U     ", Style::default().fg(Color::Green)),
            Span::raw("Page up"),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Job Detail",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from("─".repeat(40)),
        Line::from(vec![
            Span::styled("j / ↓      ", Style::default().fg(Color::Green)),
            Span::raw("Scroll down"),
        ]),
        Line::from(vec![
            Span::styled("k / ↑      ", Style::default().fg(Color::Green)),
            Span::raw("Scroll up"),
        ]),
        Line::from(vec![
            Span::styled("r          ", Style::default().fg(Color::Green)),
            Span::raw("Retry job (if failed)"),
        ]),
        Line::from(vec![
            Span::styled("d          ", Style::default().fg(Color::Green)),
            Span::raw("Delete job"),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Confirmation Dialog",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from("─".repeat(40)),
        Line::from(vec![
            Span::styled("y          ", Style::default().fg(Color::Green)),
            Span::raw("Confirm action"),
        ]),
        Line::from(vec![
            Span::styled("n / Esc    ", Style::default().fg(Color::Green)),
            Span::raw("Cancel action"),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Press q or Esc to close this help",
            Style::default().fg(Color::Gray).italic(),
        )),
    ];

    let paragraph = Paragraph::new(help_text).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner);
}
