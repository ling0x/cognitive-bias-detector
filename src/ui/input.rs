use crate::app::state::{AppMode, AppState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use super::{ACCENT, BORDER, HIGHLIGHT, MUTED};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(8),
        ])
        .split(area);

    render_input_box(f, chunks[0], state);
    render_tips(f, chunks[1], state);
}

fn render_input_box(f: &mut Frame, area: Rect, state: &AppState) {
    let is_analysing = state.mode == AppMode::Analysing;

    let title = if is_analysing {
        " ◈ Analysing… "
    } else {
        " ◈ Enter Text to Analyse "
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_analysing { HIGHLIGHT } else { BORDER }));

    if is_analysing {
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame_idx = (chrono::Local::now().timestamp_millis() / 100) as usize % spinner_frames.len();
        let spinner = spinner_frames[frame_idx];
        let para = Paragraph::new(Text::from(format!(
            "\n\n  {} Running bias detection engine{}",
            spinner,
            if state.ai_enabled { " + AI analysis…" } else { "…" }
        )))
        .block(block)
        .style(Style::default().fg(HIGHLIGHT));
        f.render_widget(para, area);
        return;
    }

    // Build text with cursor
    let text = &state.input_text;
    let cursor = state.cursor_pos.min(text.len());
    let before = &text[..cursor];
    let after = if cursor < text.len() { &text[cursor..] } else { "" };

    // Split into lines for display
    let mut display_lines: Vec<Line> = Vec::new();
    let before_lines: Vec<&str> = before.split('\n').collect();
    let after_lines: Vec<&str> = after.split('\n').collect();

    for (i, line) in before_lines.iter().enumerate() {
        if i == before_lines.len() - 1 {
            // Last before-line: add cursor + first after-line
            let cursor_char = if after_lines[0].is_empty() { " " } else { &after_lines[0][..after_lines[0].char_indices().next().map(|(_, c)| c.len_utf8()).unwrap_or(1)] };
            let after_rest = if after_lines[0].len() > cursor_char.len() { &after_lines[0][cursor_char.len()..] } else { "" };

            let mut spans = vec![
                Span::raw(line.to_string()),
                Span::styled(cursor_char.to_string(), Style::default().bg(Color::White).fg(Color::Black)),
            ];
            if !after_rest.is_empty() {
                spans.push(Span::raw(after_rest.to_string()));
            }
            // Append remaining after-lines
            if after_lines.len() > 1 {
                display_lines.push(Line::from(spans));
                for after_line in &after_lines[1..] {
                    display_lines.push(Line::from(Span::raw(after_line.to_string())));
                }
            } else {
                display_lines.push(Line::from(spans));
            }
        } else {
            display_lines.push(Line::from(Span::raw(line.to_string())));
        }
    }

    if display_lines.is_empty() {
        display_lines.push(Line::from(vec![
            Span::styled("_", Style::default().bg(Color::White).fg(Color::Black)),
        ]));
    }

    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();
    let title_with_count = format!(
        " ◈ Enter Text to Analyse  [{} chars, {} words] ",
        char_count, word_count
    );

    let block = Block::default()
        .title(Span::styled(&title_with_count, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let para = Paragraph::new(Text::from(display_lines))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_tips(f: &mut Frame, area: Rect, state: &AppState) {
    let tips = vec![
        Line::from(vec![
            Span::styled("  Tips  ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Paste or type any text — news article, argument, plan, speech, email"),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Press "),
            Span::styled("F5", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("Ctrl+Enter", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" to analyse"),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Press "),
            Span::styled("F3", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(if state.ai_enabled { " to disable AI analysis  [currently ON]" } else { " to enable AI analysis  [currently OFF]" }),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Press "),
            Span::styled("F2", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" to browse all 180+ cognitive biases in the Codex"),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Detects biases from all categories of the Cognitive Bias Codex"),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" ℹ Tips ", Style::default().fg(MUTED)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let para = Paragraph::new(Text::from(tips)).block(block);
    f.render_widget(para, area);
}
