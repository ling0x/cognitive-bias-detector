use crate::app::state::AppState;
use crate::biases::codex::BIAS_CODEX;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use super::{ACCENT, BORDER, CATEGORY_COLORS, MUTED, HIGHLIGHT, SUCCESS, ERROR};
use super::widgets::{confidence_bar, severity_badge, wrap_text};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    render_results_list(f, chunks[0], state);
    render_result_preview(f, chunks[1], state);
}

fn render_results_list(f: &mut Frame, area: Rect, state: &AppState) {
    let rule_count = state.rule_results.len();
    let ai_count = state
        .ai_result
        .as_ref()
        .map(|r| r.detected_biases.len())
        .unwrap_or(0);
    let total = rule_count + ai_count;

    let mut items: Vec<ListItem> = Vec::new();

    // Section header for rule-based
    items.push(ListItem::new(Line::from(Span::styled(
        format!("  ── Rule-based detections ({}) ──", rule_count),
        Style::default().fg(MUTED),
    ))));

    if rule_count == 0 {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No biases detected by rule engine",
            Style::default().fg(MUTED),
        ))));
    } else {
        for (i, result) in state.rule_results.iter().enumerate() {
            let is_selected = i + 1 == state.selected_result_idx + 1
                && state.selected_result_idx < rule_count;

            let bias_entry = BIAS_CODEX.iter().find(|b| b.name == result.bias_name);
            let color_idx = bias_entry
                .map(|b| b.category.color_index() as usize)
                .unwrap_or(0);
            let color = CATEGORY_COLORS[color_idx];

            let prefix = if is_selected { " ▶ " } else { "   " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(HIGHLIGHT)),
                Span::styled(result.bias_name.clone(), Style::default().fg(color).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })),
                Span::raw("  "),
                Span::styled(
                    format!("{:.0}%", result.confidence * 100.0),
                    Style::default().fg(if result.confidence >= 0.7 { ERROR } else if result.confidence >= 0.45 { HIGHLIGHT } else { SUCCESS }),
                ),
            ]);

            let style = if is_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            items.push(ListItem::new(line).style(style));
        }
    }

    // AI section
    if let Some(ref ai) = state.ai_result {
        items.push(ListItem::new(Line::from(Span::raw(""))));
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  ── AI detections ({} / {}) ──", ai.detected_biases.len(), ai.provider),
            Style::default().fg(MUTED),
        ))));

        for (i, bias) in ai.detected_biases.iter().enumerate() {
            let real_idx = rule_count + i;
            let is_selected = real_idx == state.selected_result_idx;
            let prefix = if is_selected { " ▶ " } else { "   " };

            let conf_color = match bias.confidence.as_str() {
                "High" => ERROR,
                "Medium" => HIGHLIGHT,
                _ => SUCCESS,
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(ACCENT)),
                Span::styled(bias.name.clone(), Style::default().fg(ACCENT).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })),
                Span::raw("  "),
                Span::styled(
                    format!("[{}]", bias.confidence),
                    Style::default().fg(conf_color),
                ),
            ]);

            let style = if is_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            items.push(ListItem::new(line).style(style));
        }
    }

    let title = format!(
        " ◈ Detected Biases ({}) ",
        total
    );
    let block = Block::default()
        .title(Span::styled(&title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_result_idx.min(items.len().saturating_sub(1))));

    f.render_stateful_widget(List::new(items).block(block), area, &mut list_state);
}

fn render_result_preview(f: &mut Frame, area: Rect, state: &AppState) {
    let inner_width = area.width.saturating_sub(4) as usize;

    let mut lines: Vec<Line> = Vec::new();

    // Summary header
    let rule_count = state.rule_results.len();
    let ai_count = state
        .ai_result
        .as_ref()
        .map(|r| r.detected_biases.len())
        .unwrap_or(0);

    if rule_count == 0 && ai_count == 0 {
        lines.push(Line::from(Span::styled(
            "  ✓ No significant cognitive biases detected.",
            Style::default().fg(SUCCESS),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  The text does not appear to contain obvious cognitive bias indicators.",
            Style::default().fg(MUTED),
        )));
        lines.push(Line::from(Span::styled(
            "  This does not mean the text is perfectly objective — AI analysis",
            Style::default().fg(MUTED),
        )));
        lines.push(Line::from(Span::styled(
            "  may detect subtler patterns. Press 'e' to run AI analysis.",
            Style::default().fg(MUTED),
        )));
    } else {
        // Category breakdown
        let mut cat_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for r in &state.rule_results {
            *cat_counts.entry(r.category.clone()).or_insert(0) += 1;
        }

        lines.push(Line::from(vec![
            Span::styled("  Summary  ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} bias(es) detected", rule_count + ai_count),
                Style::default().fg(HIGHLIGHT),
            ),
        ]));
        lines.push(Line::from(""));

        // Show the selected result
        let idx = state.selected_result_idx;

        if idx < rule_count {
            let result = &state.rule_results[idx];
            let bias_entry = BIAS_CODEX.iter().find(|b| b.name == result.bias_name);
            let color_idx = bias_entry.map(|b| b.category.color_index() as usize).unwrap_or(0);
            let color = CATEGORY_COLORS[color_idx];

            lines.push(Line::from(vec![
                Span::styled("  ◈ ", Style::default().fg(color)),
                Span::styled(result.bias_name.clone(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                severity_badge(&result.severity),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::styled("  Category: ", Style::default().fg(MUTED)),
                Span::styled(result.category.clone(), Style::default().fg(color)),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("  Confidence", Style::default().fg(MUTED))));
            lines.push(confidence_bar(result.confidence, 28, color));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("  What is it?", Style::default().fg(ACCENT))));
            for desc_line in wrap_text(&result.description, inner_width.saturating_sub(4)) {
                lines.push(Line::from(format!("  {}", desc_line)));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("  Evidence found:", Style::default().fg(ACCENT))));
            for ev in result.evidence.iter().take(4) {
                lines.push(Line::from(vec![
                    Span::styled("  › ", Style::default().fg(HIGHLIGHT)),
                    Span::styled(
                        format!("\"{}\"", ev.matched_phrase),
                        Style::default().fg(if ev.is_phrase { HIGHLIGHT } else { MUTED }),
                    ),
                    Span::styled(
                        if ev.is_phrase { "  (phrase match)" } else { "  (keyword match)" },
                        Style::default().fg(MUTED),
                    ),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("  Press Enter for full details", Style::default().fg(MUTED))));
        } else {
            let ai_idx = idx - rule_count;
            if let Some(ai) = &state.ai_result {
                if let Some(bias) = ai.detected_biases.get(ai_idx) {
                    lines.push(Line::from(vec![
                        Span::styled("  ◈ AI: ", Style::default().fg(ACCENT)),
                        Span::styled(bias.name.clone(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                    ]));
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("  Confidence: ", Style::default().fg(MUTED)),
                        Span::styled(bias.confidence.clone(), Style::default().fg(HIGHLIGHT)),
                    ]));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("  AI Reasoning:", Style::default().fg(ACCENT))));
                    for reasoning_line in wrap_text(&bias.reasoning, inner_width.saturating_sub(4)) {
                        lines.push(Line::from(format!("  {}", reasoning_line)));
                    }
                    lines.push(Line::from(""));
                    if !bias.relevant_excerpt.is_empty() {
                        lines.push(Line::from(Span::styled("  Relevant excerpt:", Style::default().fg(MUTED))));
                        lines.push(Line::from(vec![
                            Span::styled("  \"", Style::default().fg(HIGHLIGHT)),
                            Span::styled(bias.relevant_excerpt.clone(), Style::default().fg(HIGHLIGHT)),
                            Span::styled("\"", Style::default().fg(HIGHLIGHT)),
                        ]));
                    }
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("  Press Enter for full details", Style::default().fg(MUTED))));
                }
            }
        }

        // Category bar at bottom
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Categories:", Style::default().fg(MUTED))));
        let mut cat_line = vec![Span::raw("  ")];
        let mut sorted_cats: Vec<_> = cat_counts.iter().collect();
        sorted_cats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in sorted_cats.iter().take(6) {
            cat_line.push(Span::styled(
                format!(" {} ({}) ", cat, count),
                Style::default().fg(MUTED),
            ));
        }
        lines.push(Line::from(cat_line));
    }

    // AI summary
    if let Some(ai) = &state.ai_result {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  AI Summary ({})", ai.provider),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        for sum_line in wrap_text(&ai.summary, inner_width.saturating_sub(4)) {
            lines.push(Line::from(format!("  {}", sum_line)));
        }
    }

    // Timestamp
    if let Some(ts) = &state.last_analysed {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  Analysed at {}", ts.format("%H:%M:%S")),
            Style::default().fg(MUTED),
        )));
    }

    let block = Block::default()
        .title(Span::styled(" Preview ", Style::default().fg(ACCENT)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let visible_lines = lines
        .into_iter()
        .skip(state.scroll_offset)
        .collect::<Vec<_>>();

    let para = Paragraph::new(Text::from(visible_lines))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}
