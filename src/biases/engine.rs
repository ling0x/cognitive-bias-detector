use super::codex::{Bias, BIAS_CODEX};
use super::patterns::PATTERNS;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// A single sentence or clause within the input text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    pub index: usize,
    pub text: String,
    /// Character start offset in the original text
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedEvidence {
    pub matched_phrase: String,
    pub segment_index: usize,
    pub is_phrase: bool,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub bias_name: String,
    pub category: String,
    pub confidence: f32,
    pub severity: String,
    pub description: String,
    pub example: String,
    pub evidence: Vec<MatchedEvidence>,
}

impl DetectionResult {
    pub fn confidence_label(&self) -> &'static str {
        if self.confidence >= 0.75 {
            "High"
        } else if self.confidence >= 0.45 {
            "Medium"
        } else {
            "Low"
        }
    }
}

/// Tokenise input into sentences using simple heuristics
fn split_into_segments(text: &str) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut idx = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    while start < len {
        // Skip leading whitespace
        while start < len && (bytes[start] == b' ' || bytes[start] == b'\n' || bytes[start] == b'\r') {
            start += 1;
        }
        if start >= len {
            break;
        }
        let seg_start = start;
        let mut pos = start;
        while pos < len {
            let ch = bytes[pos];
            if ch == b'.' || ch == b'!' || ch == b'?' || ch == b'\n' {
                pos += 1;
                // skip any following whitespace
                while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\r') {
                    pos += 1;
                }
                break;
            }
            pos += 1;
        }
        let raw = &text[seg_start..pos.min(len)];
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let grapheme_start = text[..seg_start].graphemes(true).count();
            let grapheme_end = grapheme_start + trimmed.graphemes(true).count();
            segments.push(TextSegment {
                index: idx,
                text: trimmed.to_string(),
                char_start: grapheme_start,
                char_end: grapheme_end,
            });
            idx += 1;
        }
        start = pos;
    }
    segments
}

fn normalise(s: &str) -> String {
    s.to_lowercase()
}

fn word_boundary_match(haystack: &str, needle: &str) -> bool {
    let needle_lower = normalise(needle);
    let hay_lower = normalise(haystack);

    if let Some(pos) = hay_lower.find(&needle_lower) {
        let before = pos.checked_sub(1).map(|i| hay_lower.as_bytes()[i]).unwrap_or(b' ');
        let after_idx = pos + needle_lower.len();
        let after = hay_lower.as_bytes().get(after_idx).copied().unwrap_or(b' ');
        let is_word_start = !before.is_ascii_alphanumeric();
        let is_word_end = !after.is_ascii_alphanumeric();
        is_word_start && is_word_end
    } else {
        false
    }
}

/// Core rule-based analysis function
pub fn analyse(text: &str) -> Vec<DetectionResult> {
    if text.trim().is_empty() {
        return vec![];
    }

    let segments = split_into_segments(text);
    let full_lower = normalise(text);

    // Map bias_name -> (score, evidence list)
    let mut scores: HashMap<&'static str, (f32, Vec<MatchedEvidence>)> = HashMap::new();

    // Word count for normalisation
    let word_count = text.split_whitespace().count().max(1) as f32;

    for pattern in PATTERNS.iter() {
        let entry = scores.entry(pattern.bias_name).or_insert((0.0, Vec::new()));

        // Check keywords in full text
        for kw in pattern.keywords.iter() {
            let kw_lower = normalise(kw);
            if full_lower.contains(&kw_lower) {
                // Find which segment contains it
                let seg_idx = segments
                    .iter()
                    .find(|s| normalise(&s.text).contains(&kw_lower))
                    .map(|s| s.index)
                    .unwrap_or(0);

                let score = 1.0 / word_count.sqrt();
                entry.0 += score;
                entry.1.push(MatchedEvidence {
                    matched_phrase: kw.to_string(),
                    segment_index: seg_idx,
                    is_phrase: false,
                    score,
                });
            }
        }

        // Check phrases (higher weight)
        for phrase in pattern.phrases.iter() {
            if word_boundary_match(text, phrase) {
                let seg_idx = segments
                    .iter()
                    .find(|s| word_boundary_match(&s.text, phrase))
                    .map(|s| s.index)
                    .unwrap_or(0);

                let score = pattern.phrase_weight as f32 / word_count.sqrt();
                entry.0 += score;
                entry.1.push(MatchedEvidence {
                    matched_phrase: phrase.to_string(),
                    segment_index: seg_idx,
                    is_phrase: true,
                    score,
                });
            }
        }
    }

    // Convert to DetectionResult, filter low-confidence
    let mut results: Vec<DetectionResult> = scores
        .into_iter()
        .filter(|(_, (score, _))| *score > 0.0)
        .filter_map(|(name, (raw_score, evidence))| {
            let bias = BIAS_CODEX.iter().find(|b| b.name == name)?;

            // Normalise confidence to 0..1 with a soft cap
            let confidence = (raw_score / (raw_score + 1.5)).clamp(0.0, 0.99);

            // Only return detections above a threshold
            if confidence < 0.10 {
                return None;
            }

            Some(DetectionResult {
                bias_name: bias.name.to_string(),
                category: bias.category.display_name().to_string(),
                confidence,
                severity: bias.severity.label().to_string(),
                description: bias.description.to_string(),
                example: bias.example.to_string(),
                evidence,
            })
        })
        .collect();

    // Sort by confidence descending
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    // Deduplicate overlapping evidence
    for result in results.iter_mut() {
        result.evidence.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        result.evidence.dedup_by(|a, b| a.matched_phrase == b.matched_phrase);
        result.evidence.truncate(5);
    }

    results
}

/// Helper: get a Bias struct for a given name
pub fn get_bias(name: &str) -> Option<&'static Bias> {
    BIAS_CODEX.iter().find(|b| b.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sunk_cost_detection() {
        let text = "I've already put in 3 years, I can't quit now, it would be a waste of all that effort.";
        let results = analyse(text);
        assert!(!results.is_empty(), "Should detect at least one bias");
        let names: Vec<_> = results.iter().map(|r| r.bias_name.as_str()).collect();
        assert!(names.contains(&"Sunk Cost Fallacy"), "Should detect Sunk Cost Fallacy, got: {names:?}");
    }

    #[test]
    fn test_confirmation_bias_detection() {
        let text = "This clearly proves my point. I always knew I was right about this. Confirms what I already believed.";
        let results = analyse(text);
        let names: Vec<_> = results.iter().map(|r| r.bias_name.as_str()).collect();
        assert!(names.contains(&"Confirmation Bias"), "Should detect Confirmation Bias, got: {names:?}");
    }

    #[test]
    fn test_empty_text() {
        let results = analyse("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_neutral_text() {
        let text = "The weather today is partly cloudy with a chance of rain in the afternoon.";
        let results = analyse(text);
        // Should be empty or very low confidence
        assert!(results.iter().all(|r| r.confidence < 0.3));
    }
}
