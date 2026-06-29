use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::state::DashboardState;

use super::super::types::DIM;

pub(in crate::tui::ui) fn render_header(frame: &mut Frame, area: Rect, state: &DashboardState) {
    let title = format!("");
    let mut spans = vec![Span::styled(title, Style::default().fg(Color::Cyan).bold())];
    if let Some(user) = state.current_user() {
        spans.push(Span::styled(" Connected as ", Style::default().fg(Color::Blue)));
        spans.push(Span::styled(
            format!("{user} "),
            Style::default().fg(Color::Red).bold(),
        ));
        let (self_mute, self_deaf) = state.current_voice_self_status();
        if self_mute {
            spans.push(Span::styled("🔇 ", Style::default().fg(Color::Yellow)));
        }
        if self_deaf {
            spans.push(Span::styled("🎧 ", Style::default().fg(Color::Yellow)));
        }
    } else if let Some(error) = state.gateway_error() {
        spans.push(Span::styled(
            format!(" Connection issue: {} ", truncate_header_error(error)),
            Style::default().fg(Color::Red).bold(),
        ));
    } else {
        spans.push(Span::styled(
            " Loading... ",
            Style::default().fg(Color::Yellow).bold(),
        ));
    }
    if let Some(version) = state.update_available_version() {
        spans.push(Span::styled(
            format!(" New version available: v{version} "),
            Style::default().fg(Color::Yellow).bold(),
        ));
    }
    if let Some(label) = state.active_voice_connection_label() {
        spans.push(Span::styled(" Voice ", Style::default().fg(DIM)));
        spans.push(Span::styled(
            format!("{label} "),
            Style::default().fg(Color::Yellow).bold(),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Left),
        area,
    );
}

fn truncate_header_error(error: &str) -> String {
    const MAX_CHARS: usize = 96;
    let mut chars = error.chars();
    let truncated: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
