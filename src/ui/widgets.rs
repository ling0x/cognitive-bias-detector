use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Render a confidence bar as a string of block characters
pub fn confidence_bar(confidence: f32, width: usize, filled_color: Color) -> Line<'static> {
    let filled = ((confidence * width as f32) as usize).min(width);
    let empty = width - filled;
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);
    let pct = format!(" {:.0}%", confidence * 100.0);

    Line::from(vec![
        Span::styled(filled_str, Style::default().fg(filled_color)),
        Span::styled(empty_str, Style::default().fg(Color::DarkGray)),
        Span::styled(pct, Style::default().fg(Color::White)),
    ])
}

/// Severity badge
pub fn severity_badge(severity: &str) -> Span<'static> {
    let (label, color) = match severity {
        "High" => (" HIGH ", Color::Red),
        "Medium" => (" MED  ", Color::Yellow),
        _ => (" LOW  ", Color::Green),
    };
    Span::styled(label.to_string(), Style::default().fg(Color::Black).bg(color))
}

/// Confidence label badge
pub fn confidence_badge(label: &str) -> Span<'static> {
    let color = match label {
        "High" => Color::Red,
        "Medium" => Color::Yellow,
        _ => Color::Green,
    };
    Span::styled(format!("[{}]", label), Style::default().fg(color))
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut current = String::new();
        for word in words {
            if current.is_empty() {
                current = word.to_string();
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current.clone());
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}
