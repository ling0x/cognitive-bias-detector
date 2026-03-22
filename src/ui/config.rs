use crate::app::state::AppState;
use crate::config::Config;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use super::{ACCENT, BORDER, HIGHLIGHT, MUTED, SUCCESS};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let config_path = Config::config_path();

    let lines = vec![
        Line::from(Span::styled(
            "  ◈ Configuration",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Config file:  ", Style::default().fg(MUTED)),
            Span::styled(
                config_path.display().to_string(),
                Style::default().fg(HIGHLIGHT),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  AI Providers",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  openai    ", Style::default().fg(SUCCESS)),
            Span::raw("gpt-4o-mini (default), gpt-4o, gpt-4-turbo, o1-mini …"),
        ]),
        Line::from(vec![
            Span::styled("  anthropic ", Style::default().fg(SUCCESS)),
            Span::raw("claude-3-haiku-20240307, claude-3-5-sonnet-20241022 …"),
        ]),
        Line::from(vec![
            Span::styled("  gemini    ", Style::default().fg(SUCCESS)),
            Span::raw("gemini-1.5-flash (default), gemini-1.5-pro …"),
        ]),
        Line::from(vec![
            Span::styled("  ollama    ", Style::default().fg(SUCCESS)),
            Span::raw("llama3.2 (default) — runs locally, no API key needed"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Example configuration (paste into config file):",
            Style::default().fg(MUTED),
        )),
        Line::from(""),
    ];

    let mut text_lines = lines;
    for config_line in Config::example().lines() {
        text_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(config_line.to_string(), Style::default().fg(MUTED)),
        ]));
    }

    text_lines.push(Line::from(""));
    text_lines.push(Line::from(Span::styled(
        "  Keyboard shortcuts",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));
    text_lines.push(Line::from(""));

    let shortcuts = [
        ("F5 / Ctrl+Enter", "Run analysis"),
        ("F2", "Browse Cognitive Bias Codex"),
        ("F3", "Toggle AI analysis on/off"),
        ("F4", "Show this config screen"),
        ("↑ ↓ / j k", "Navigate results"),
        ("Enter", "Open detailed view"),
        ("← →", "Navigate between bias details"),
        ("e", "Run/re-run AI analysis from results"),
        ("c", "Copy results to clipboard"),
        ("q / Esc", "Go back / quit"),
        ("Ctrl+C", "Force quit"),
    ];

    for (key, action) in shortcuts {
        text_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<20}", key), Style::default().fg(HIGHLIGHT)),
            Span::raw(action),
        ]));
    }

    text_lines.push(Line::from(""));
    text_lines.push(Line::from(Span::styled(
        "  Press q or Esc to return",
        Style::default().fg(MUTED),
    )));

    let block = Block::default()
        .title(Span::styled(
            " ◈ Help & Configuration ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let para = Paragraph::new(Text::from(text_lines))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}
