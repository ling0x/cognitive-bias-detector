use crate::ai::AiDetectionResult;
use crate::biases::DetectionResult;

pub fn print_results(
    text: &str,
    results: &[DetectionResult],
    ai_result: Option<&AiDetectionResult>,
) {
    let separator = "─".repeat(60);
    println!("{}", separator);
    println!("  Cognitive Bias Detector");
    println!("{}", separator);

    let preview: String = text.chars().take(80).collect();
    let ellipsis = if text.chars().count() > 80 { "…" } else { "" };
    println!("  Text: \"{}{}\"\n", preview, ellipsis);

    println!("  Rule-Based Detections: {}", results.len());
    println!("{}", separator);

    if results.is_empty() {
        println!("  ✓ No biases detected by rule engine.");
    } else {
        for (i, r) in results.iter().enumerate() {
            println!(
                "  {}. {} [{} | {:.0}% confidence | {}]",
                i + 1,
                r.bias_name,
                r.category,
                r.confidence * 100.0,
                r.severity
            );
            println!("     {}", r.description);
            if !r.evidence.is_empty() {
                println!("     Evidence:");
                for ev in r.evidence.iter().take(3) {
                    println!("       › \"{}\"", ev.matched_phrase);
                }
            }
            println!();
        }
    }

    if let Some(ai) = ai_result {
        println!("{}", separator);
        println!("  AI Analysis ({} / {})", ai.provider, ai.model);
        println!("{}", separator);
        println!("  {}\n", ai.summary);

        for (i, b) in ai.detected_biases.iter().enumerate() {
            println!("  {}. {} [{}]", i + 1, b.name, b.confidence);
            println!("     {}", b.reasoning);
            if !b.relevant_excerpt.is_empty() {
                println!("     Excerpt: \"{}\"", b.relevant_excerpt);
            }
            println!();
        }
    }

    println!("{}", separator);
}
