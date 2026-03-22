use crate::app::state::AppState;
use crate::biases::codex::BIAS_CODEX;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use super::{ACCENT, BORDER, CATEGORY_COLORS, MUTED, HIGHLIGHT};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    render_search_bar(f, chunks[0], state);
    render_codex_list(f, chunks[1], state);
}

fn render_search_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let query = if state.codex_search_active {
        format!("{}_", state.codex_search)
    } else {
        format!(
            "{}  [Press / to search, ↑↓ to navigate, q/Esc to close]",
            state.codex_search
        )
    };

    let block = Block::default()
        .title(Span::styled(" / Search Codex ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.codex_search_active { HIGHLIGHT } else { BORDER }));

    let para = Paragraph::new(format!("  {}", query)).block(block);
    f.render_widget(para, area);
}

fn render_codex_list(f: &mut Frame, area: Rect, state: &AppState) {
    let search = state.codex_search.to_lowercase();

    let filtered: Vec<_> = BIAS_CODEX
        .iter()
        .filter(|b| {
            if search.is_empty() {
                true
            } else {
                b.name.to_lowercase().contains(&search)
                    || b.category.display_name().to_lowercase().contains(&search)
                    || b.description.to_lowercase().contains(&search)
                    || b.alt_names.iter().any(|n| n.to_lowercase().contains(&search))
            }
        })
        .collect();

    // Group by category
    let mut categories: Vec<String> = Vec::new();
    for bias in &filtered {
        let cat = bias.category.display_name().to_string();
        if !categories.contains(&cat) {
            categories.push(cat);
        }
    }

    let mut items: Vec<ListItem> = Vec::new();
    for cat in &categories {
        // Category header
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  ▾ {}", cat), Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)),
        ])));

        for bias in filtered.iter().filter(|b| b.category.display_name() == cat.as_str()) {
            let color = CATEGORY_COLORS[bias.category.color_index() as usize];
            items.push(ListItem::new(Line::from(vec![
                Span::raw("      "),
                Span::styled(bias.name, Style::default().fg(color)),
                Span::styled(
                    format!("  — {}", truncate(bias.description, 60)),
                    Style::default().fg(MUTED),
                ),
            ])));
        }
        items.push(ListItem::new(Line::from("")));
    }

    let total = filtered.len();
    let title = if search.is_empty() {
        format!(" ◈ Cognitive Bias Codex  ({} biases) ", total)
    } else {
        format!(" ◈ Codex  ({} matching \"{}\") ", total, state.codex_search)
    };

    let block = Block::default()
        .title(Span::styled(&title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    // Apply scroll
    let visible: Vec<ListItem> = items.into_iter().skip(state.codex_scroll).collect();

    let list = List::new(visible).block(block);
    f.render_widget(list, area);
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let s: String = chars[..max].iter().collect();
        format!("{}…", s.trim_end())
    }
}
