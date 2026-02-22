use super::{Finding, Severity};

/// Apply structural rules that catch paragraph-level patterns.
/// These operate on whole-document structure, not individual lines.
// source: rosenfeld2024 — structural signals more stable than lexical patterns
pub fn apply_structural_rules(content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut line_offset = 1usize;

    // source: rosenfeld2024 — structural signals more stable than lexical
    let connectors: &[&str] = &[
        "moreover",
        "furthermore",
        "additionally",
        "consequently",
        "subsequently",
        "nevertheless",
        "nonetheless",
        "in addition",
        "as a result",
        "on the other hand",
        "with that said",
        "that being said",
        "to summarize",
        "in summary",
        "in conclusion",
    ];

    for para in &paragraphs {
        let para_lower = para.to_lowercase();

        let count: usize = connectors
            .iter()
            .map(|&c| {
                let mut n = 0;
                let mut start = 0;
                while let Some(pos) = para_lower[start..].find(c) {
                    n += 1;
                    start += pos + c.len();
                }
                n
            })
            .sum();

        if count >= 3 {
            findings.push(Finding {
                line: line_offset,
                col: 0,
                matched: format!("{} discourse connectors", count),
                message: format!(
                    "High connector density ({}): reads as machine-generated transitions (Rosenfeld 2024)",
                    count
                ),
                replacement: None,
                severity: Severity::High,
            });
        }

        // Sentence length uniformity check
        // source: rosenfeld2024 sentence-length-clustering
        let sentence_endings = [". ", "! ", "? ", ".\n", "!\n", "?\n"];
        let mut sentences: Vec<&str> = Vec::new();
        let mut remaining = para.trim();
        while !remaining.is_empty() {
            let cut = sentence_endings
                .iter()
                .filter_map(|ending| remaining.find(ending).map(|pos| pos + ending.len()))
                .min()
                .unwrap_or(remaining.len());
            let (sentence, rest) = remaining.split_at(cut);
            sentences.push(sentence);
            remaining = rest.trim_start();
        }

        if sentences.len() >= 4 {
            let word_counts: Vec<f64> = sentences
                .iter()
                .map(|s| s.split_whitespace().count() as f64)
                .collect();
            let mean = word_counts.iter().sum::<f64>() / word_counts.len() as f64;
            let variance = word_counts.iter().map(|&x| (x - mean).powi(2)).sum::<f64>()
                / word_counts.len() as f64;
            let stddev = variance.sqrt();

            if stddev < 3.0 && mean > 5.0 {
                findings.push(Finding {
                    line: line_offset,
                    col: 0,
                    matched: format!("stddev={:.1}", stddev),
                    message: "Uniform sentence length — LLMs cluster in 10-30 token range (Rosenfeld 2024)".to_string(),
                    replacement: None,
                    severity: Severity::Medium,
                });
            }
        }

        // split("\n\n") consumes both newlines — the separator is one blank line,
        // so the next paragraph starts 1 line after the last line of this one.
        line_offset += para.lines().count() + 1;
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_density_fires() {
        let para = "Moreover, this is important. Furthermore, we note that. Additionally, as a result, the data shows. Consequently, we conclude.";
        let findings = apply_structural_rules(para);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("connector density")),
            "high connector density should fire, got: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn connector_density_exactly_three_fires() {
        let para =
            "Moreover, this is the case. Furthermore, it matters. Additionally, we note this.";
        let findings = apply_structural_rules(para);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("connector density")),
            "exactly 3 connectors should fire at the >= 3 threshold"
        );
    }

    #[test]
    fn connector_density_low_count_no_fire() {
        let para = "Moreover, this is important. Furthermore, this helps.";
        let findings = apply_structural_rules(para);
        assert!(
            !findings
                .iter()
                .any(|f| f.message.contains("connector density")),
            "2 connectors should not fire"
        );
    }

    #[test]
    fn structural_rules_empty_input() {
        let findings = apply_structural_rules("");
        assert!(findings.is_empty());
    }
}
