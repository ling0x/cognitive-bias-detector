mod input;
mod results;
mod detail;
mod codex;
mod config;
mod widgets;
pub mod plain;

use crate::app::state::{AppMode, AppState};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub const ACCENT: Color = Color::Cyan;
pub const HIGHLIGHT: Color = Color::Yellow;
pub const SUCCESS: Color = Color::Green;
pub const ERROR: Color = Color::Red;
pub const MUTED: Color = Color::DarkGray;
pub const HEADER_BG: Color = Color::Rgb(20, 30, 48);
pub const BORDER: Color = Color::Rgb(60, 80, 120);

/// Category colours (10 distinct colours)
pub const CATEGORY_COLORS: [Color; 10] = [
    Color::Rgb(230, 100, 100), // Memory
    Color::Rgb(230, 160, 80),  // MeaningMaking
    Color::Rgb(220, 220, 80),  // ActionBias
    Color::Rgb(100, 200, 100), // RecencyAndSaliency
    Color::Rgb(80, 180, 220),  // BeliefAndConfirmation
    Color::Rgb(120, 100, 220), // SocialAndGroup
    Color::Rgb(200, 100, 200), // ProbabilityAndStats
    Color::Rgb(255, 140, 0),   // SelfPerception
    Color::Rgb(0, 200, 180),   // CausalAttribution
    Color::Rgb(180, 100, 140), // DecisionMaking
];

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    // Outer layout: header + content + footer
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, outer[0], state);

    match &state.mode {
        AppMode::Input | AppMode::Analysing => input::render(f, outer[1], state),
        AppMode::Results => results::render(f, outer[1], state),
        AppMode::BiasDetail => detail::render(f, outer[1], state),
        AppMode::CodexBrowser => codex::render(f, outer[1], state),
        AppMode::Config => config::render(f, outer[1], state),
    }

    render_statusbar(f, outer[2], state);
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let ai_indicator = if state.ai_enabled {
        Span::styled(" [AI ON] ", Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [AI OFF] ", Style::default().fg(MUTED))
    };

    let title_line = Line::from(vec![
        Span::styled(
            "  ◈ Cognitive Bias Detector  ",
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        ai_indicator,
        Span::styled(
            " — Cognitive Bias Codex v1.0  ",
            Style::default().fg(MUTED),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(HEADER_BG));

    let para = Paragraph::new(title_line)
        .block(block);
    f.render_widget(para, area);
}

fn render_statusbar(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let msg = if let Some(ref err) = state.error_message {
        Span::styled(format!(" ✗ {}", err), Style::default().fg(ERROR))
    } else if let Some(ref status) = state.status_message {
        Span::styled(format!(" ✓ {}", status), Style::default().fg(SUCCESS))
    } else {
        let hint = match state.mode {
            AppMode::Input => " F5/Ctrl+Enter: Analyse  |  F2: Bias Codex  |  F3: Toggle AI  |  F4: Config  |  Esc: Quit",
            AppMode::Results => " ↑↓: Navigate  |  Enter: Details  |  e: AI  |  c: Copy  |  q: Back",
            AppMode::BiasDetail => " ←→: Prev/Next  |  ↑↓: Scroll  |  q/Esc: Back",
            AppMode::CodexBrowser => " ↑↓: Scroll  |  /: Search  |  q/Esc: Back",
            AppMode::Config => " q/Esc: Back",
            AppMode::Analysing => " Analysing...",
        };
        Span::styled(hint, Style::default().fg(MUTED))
    };

    let para = Paragraph::new(Line::from(msg));
    f.render_widget(para, area);
}
