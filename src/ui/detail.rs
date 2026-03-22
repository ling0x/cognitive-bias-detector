use crate::app::state::AppState;
use crate::biases::codex::BIAS_CODEX;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use super::{ACCENT, BORDER, CATEGORY_COLORS, MUTED, HIGHLIGHT, SUCCESS};
use super::widgets::{confidence_bar, severity_badge, wrap_text};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let inner_width = area.width.saturating_sub(6) as usize;
    let idx = state.selected_result_idx;
    let rule_count = state.rule_results.len();

    let mut lines: Vec<Line> = Vec::new();

    if idx < rule_count {
        let result = &state.rule_results[idx];
        let bias_entry = BIAS_CODEX.iter().find(|b| b.name == result.bias_name);
        let color_idx = bias_entry.map(|b| b.category.color_index() as usize).unwrap_or(0);
        let color = CATEGORY_COLORS[color_idx];

        // Title
        lines.push(Line::from(vec![
            Span::styled("  ◈ ", Style::default().fg(color)),
            Span::styled(
                result.bias_name.clone(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            severity_badge(&result.severity),
            Span::raw("  "),
            Span::styled(
                format!("[ Rule-based detection — {} ]", idx + 1),
                Style::default().fg(MUTED),
            ),
        ]));
        lines.push(Line::from(""));

        // Category
        lines.push(Line::from(vec![
            Span::styled("  Category       ", Style::default().fg(MUTED)),
            Span::styled(result.category.clone(), Style::default().fg(color)),
        ]));

        // Confidence bar
        lines.push(Line::from(vec![
            Span::styled("  Confidence     ", Style::default().fg(MUTED)),
        ]));
        let mut bar = confidence_bar(result.confidence, 30, color);
        let mut bar_spans = vec![Span::raw("  ")];
        bar_spans.append(&mut bar.spans);
        lines.push(Line::from(bar_spans));
        lines.push(Line::from(""));

        // Description
        lines.push(Line::from(Span::styled(
            "  What is this bias?",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        for l in wrap_text(&result.description, inner_width) {
            lines.push(Line::from(format!("  {}", l)));
        }
        lines.push(Line::from(""));

        // Example
        lines.push(Line::from(Span::styled(
            "  Real-world example",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        for l in wrap_text(&result.example, inner_width) {
            lines.push(Line::from(vec![
                Span::styled("  › ", Style::default().fg(HIGHLIGHT)),
                Span::raw(l),
            ]));
        }
        lines.push(Line::from(""));

        // Evidence
        lines.push(Line::from(Span::styled(
            "  Evidence in your text",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        if result.evidence.is_empty() {
            lines.push(Line::from(Span::styled("  No specific phrases captured.", Style::default().fg(MUTED))));
        } else {
            for ev in &result.evidence {
                lines.push(Line::from(vec![
                    Span::styled("  › ", Style::default().fg(HIGHLIGHT)),
                    Span::styled(
                        format!("\"{}\"", ev.matched_phrase),
                        Style::default().fg(if ev.is_phrase { HIGHLIGHT } else { Color::White }),
                    ),
                    Span::styled(
                        format!("  {} match  score: {:.2}", if ev.is_phrase { "phrase" } else { "keyword" }, ev.score),
                        Style::default().fg(MUTED),
                    ),
                ]));
            }
        }
        lines.push(Line::from(""));

        // Mitigation tips
        lines.push(Line::from(Span::styled(
            "  How to mitigate",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        let mitigations = get_mitigation_tips(&result.bias_name);
        for tip in mitigations {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(SUCCESS)),
                Span::raw(tip),
            ]));
        }
        lines.push(Line::from(""));

        // Navigation hint
        lines.push(Line::from(Span::styled(
            "  ← → to navigate biases   ↑ ↓ to scroll   q / Esc to go back",
            Style::default().fg(MUTED),
        )));
    } else {
        // AI bias detail
        let ai_idx = idx - rule_count;
        if let Some(ai) = &state.ai_result {
            if let Some(bias) = ai.detected_biases.get(ai_idx) {
                lines.push(Line::from(vec![
                    Span::styled("  ◈ AI: ", Style::default().fg(ACCENT)),
                    Span::styled(
                        bias.name.clone(),
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("[ AI: {} / {} ]", ai.provider, ai.model),
                        Style::default().fg(MUTED),
                    ),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![
                    Span::styled("  Confidence  ", Style::default().fg(MUTED)),
                    Span::styled(bias.confidence.clone(), Style::default().fg(HIGHLIGHT)),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(Span::styled(
                    "  AI Reasoning",
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )));
                for l in wrap_text(&bias.reasoning, inner_width) {
                    lines.push(Line::from(format!("  {}", l)));
                }
                lines.push(Line::from(""));

                if !bias.relevant_excerpt.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  Relevant excerpt from your text",
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    )));
                    for l in wrap_text(&bias.relevant_excerpt, inner_width) {
                        lines.push(Line::from(vec![
                            Span::styled("  \"", Style::default().fg(HIGHLIGHT)),
                            Span::raw(l),
                            Span::styled("\"", Style::default().fg(HIGHLIGHT)),
                        ]));
                    }
                    lines.push(Line::from(""));
                }

                // Check if we have rule-based info about this bias too
                if let Some(codex_entry) = BIAS_CODEX.iter().find(|b| b.name == bias.name) {
                    let color = CATEGORY_COLORS[codex_entry.category.color_index() as usize];
                    lines.push(Line::from(Span::styled(
                        "  Codex definition",
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    )));
                    for l in wrap_text(codex_entry.description, inner_width) {
                        lines.push(Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled(l, Style::default().fg(color)),
                        ]));
                    }
                    lines.push(Line::from(""));
                }

                lines.push(Line::from(Span::styled(
                    "  ← → to navigate   ↑ ↓ to scroll   q / Esc to go back",
                    Style::default().fg(MUTED),
                )));
            }
        }
    }

    let visible: Vec<Line> = lines.into_iter().skip(state.scroll_offset).collect();

    let block = Block::default()
        .title(Span::styled(
            " ◈ Bias Detail  ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let para = Paragraph::new(Text::from(visible))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn get_mitigation_tips(bias_name: &str) -> Vec<&'static str> {
    match bias_name {
        "Confirmation Bias" => vec![
            "Actively seek out sources and evidence that contradict your current view.",
            "Ask yourself: what would change my mind on this?",
            "Engage with people who hold different perspectives.",
            "Keep a record of times your predictions were wrong.",
        ],
        "Sunk Cost Fallacy" => vec![
            "Focus only on future costs and benefits — the past is gone.",
            "Ask: if I were starting fresh today, would I make this choice?",
            "Separate emotional attachment from rational evaluation.",
        ],
        "Availability Heuristic" => vec![
            "Look up base rates and statistics rather than relying on examples that come to mind.",
            "Ask: am I recalling this because it is common, or because it was memorable?",
        ],
        "Anchoring Bias" => vec![
            "Generate your own estimate before looking at any provided numbers.",
            "Consider multiple reference points from different sources.",
        ],
        "Dunning-Kruger Effect" => vec![
            "Seek feedback from genuine domain experts.",
            "Study the topic deeply before forming strong opinions.",
            "Cultivate intellectual humility — acknowledge what you do not know.",
        ],
        "Bandwagon Effect" => vec![
            "Form your own view before checking what others think.",
            "Ask: what is the evidence, not what do most people believe?",
        ],
        "Survivorship Bias" => vec![
            "Actively look for examples of failure, not just success.",
            "Ask: what data is missing from what I am seeing?",
        ],
        "Hindsight Bias" => vec![
            "Keep written records of predictions before outcomes are known.",
            "Remember that outcomes are only obvious in hindsight.",
        ],
        "Planning Fallacy" => vec![
            "Use historical data from similar projects to estimate timelines.",
            "Add a margin — multiply estimates by 1.5x or more.",
            "Use reference class forecasting: how long did similar tasks take?",
        ],
        _ => vec![
            "Slow down and consider alternative viewpoints before concluding.",
            "Seek disconfirming evidence as actively as you seek confirming evidence.",
            "Discuss your reasoning with someone who will challenge it.",
        ],
    }
}
