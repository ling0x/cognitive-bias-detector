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
        .constraints([Constraint::Min(10), Constraint::Length(8)])
        .split(area);

    render_input_box(f, chunks[0], state);
    render_tips(f, chunks[1], state);
}

fn render_input_box(f: &mut Frame, area: Rect, state: &AppState) {
    let is_analysing = state.mode == AppMode::Analysing;

    if is_analysing {
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame_idx =
            (chrono::Local::now().timestamp_millis() / 100) as usize % spinner_frames.len();
        let spinner = spinner_frames[frame_idx];
        let block = Block::default()
            .title(Span::styled(
                " ◈ Analysing… ",
                Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(HIGHLIGHT));
        let para = Paragraph::new(format!(
            "\n\n  {} Running bias detection engine{}",
            spinner,
            if state.ai_enabled { " + AI analysis…" } else { "…" }
        ))
        .block(block)
        .style(Style::default().fg(HIGHLIGHT));
        f.render_widget(para, area);
        return;
    }

    // ── Build display lines using char-indexed cursor ──────────────────────
    // cursor_pos is a CHAR index (not byte offset).
    let text = &state.input_text;
    let chars: Vec<char> = text.chars().collect();
    let total_chars = chars.len();
    let cursor = state.cursor_pos.min(total_chars);

    // Split into lines (Vec<Vec<char>>), tracking which line+col the cursor is on
    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut current_line: Vec<char> = Vec::new();
    let mut cursor_line = 0usize;
    let mut cursor_col = 0usize;
    let mut char_idx = 0usize;

    for &ch in &chars {
        if char_idx == cursor {
            cursor_line = lines.len();
            cursor_col = current_line.len();
        }
        if ch == '\n' {
            lines.push(std::mem::take(&mut current_line));
        } else {
            current_line.push(ch);
        }
        char_idx += 1;
    }
    // Handle cursor at the very end
    if char_idx == cursor {
        cursor_line = lines.len();
        cursor_col = current_line.len();
    }
    lines.push(current_line);

    // Build ratatui Lines
    let mut display_lines: Vec<Line> = Vec::new();
    for (li, line_chars) in lines.iter().enumerate() {
        if li == cursor_line {
            // Insert cursor highlight
            let before: String = line_chars[..cursor_col].iter().collect();
            let cursor_char: String = line_chars
                .get(cursor_col)
                .map(|c| c.to_string())
                .unwrap_or_else(|| " ".to_string());
            let after: String = line_chars
                .get(cursor_col + 1..)
                .unwrap_or(&[])
                .iter()
                .collect();

            let mut spans = vec![Span::raw(before)];
            spans.push(Span::styled(
                cursor_char,
                Style::default().bg(Color::White).fg(Color::Black),
            ));
            if !after.is_empty() {
                spans.push(Span::raw(after));
            }
            display_lines.push(Line::from(spans));
        } else {
            let s: String = line_chars.iter().collect();
            display_lines.push(Line::from(Span::raw(s)));
        }
    }

    if display_lines.is_empty() {
        display_lines.push(Line::from(vec![Span::styled(
            " ",
            Style::default().bg(Color::White).fg(Color::Black),
        )]));
    }

    let char_count = total_chars;
    let word_count = text.split_whitespace().count();
    let title = format!(
        " ◈ Enter Text to Analyse  [{} chars, {} words] ",
        char_count, word_count
    );

    let block = Block::default()
        .title(Span::styled(
            &title,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let para = Paragraph::new(Text::from(display_lines))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_tips(f: &mut Frame, area: Rect, state: &AppState) {
    let tips = vec![
        Line::from(vec![Span::styled(
            "  Tips  ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Paste (Ctrl+Shift+V / middle-click) or type any text to analyse"),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Press "),
            Span::styled(
                "F5",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or "),
            Span::styled(
                "Ctrl+Enter",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to analyse"),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Press "),
            Span::styled(
                "F3",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(if state.ai_enabled {
                " to disable AI analysis  [currently ON]"
            } else {
                " to enable AI analysis  [currently OFF]"
            }),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(" Press "),
            Span::styled(
                "F2",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to browse all 180+ cognitive biases in the Codex"),
        ]),
        Line::from(vec![
            Span::styled("  ›", Style::default().fg(HIGHLIGHT)),
            Span::raw(
                " Detects biases from all 10 categories of the Cognitive Bias Codex",
            ),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" ℹ Tips ", Style::default().fg(MUTED)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let para = Paragraph::new(Text::from(tips)).block(block);
    f.render_widget(para, area);
}
