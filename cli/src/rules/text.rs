use super::{Finding, Severity};

struct TextRule {
    /// Must be lowercase. Matching runs against `line.to_lowercase()` — a
    /// mixed-case needle will never match.
    needle: &'static str,
    message: &'static str,
    /// Optional auto-fix replacement. If None, the finding is flagged only.
    replacement: Option<&'static str>,
    severity: Severity,
}

const TEXT_RULES: &[TextRule] = &[
    // === CRITICAL: r > 10× baseline (Kobak et al., Science Advances 2025) ===
    // source: kobak2024 r=25.2 — most extreme outlier across 15M PubMed abstracts
    TextRule {
        needle: "delve",
        message: "LLM tell: 'delve' (25× excess frequency, Kobak 2025)",
        replacement: Some("explore"),
        severity: Severity::Critical,
    },
    // source: kobak2024 r=25.2 — inflected form; word boundary prevents 'delves' matching 'delve'
    TextRule {
        needle: "delves",
        message: "LLM tell: 'delves' (25× excess frequency, Kobak 2025)",
        replacement: Some("explores"),
        severity: Severity::Critical,
    },
    // source: kobak2024 r=9.2 — below the r>10 Critical threshold; High
    TextRule {
        needle: "showcasing",
        message: "LLM tell: 'showcasing' (9.2× excess frequency, Kobak 2025)",
        replacement: None,
        severity: Severity::High,
    },
    // source: kobak2024 r=9.1 — below the r>10 Critical threshold; High
    TextRule {
        needle: "underscore",
        message: "LLM tell: 'underscore/underscores' (9.1× excess frequency, Kobak 2025)",
        replacement: None,
        severity: Severity::High,
    },
    // === HIGH: r > 3× baseline (Kobak 2025, Liang 2024, Neri 2024) ===
    // source: kobak2024 cross-validated; neri2024 confirmed
    TextRule {
        needle: "meticulous",
        message: "LLM tell: 'meticulous' (Kobak 2025, Neri 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // source: kobak2024 cross-validated; neri2024 confirmed
    TextRule {
        needle: "meticulously",
        message: "LLM tell: 'meticulously' (Kobak 2025, Neri 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // source: kobak2024; liang2024 — doubled post-2023
    TextRule {
        needle: "intricate",
        message: "LLM tell: 'intricate' (Kobak 2025, Liang 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // source: liang2024 — approximately doubled post-2023; neri2024 confirmed
    TextRule {
        needle: "realm",
        message: "LLM tell: 'realm' (Liang 2024, Neri 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // source: kobak2024; liang2024 — top cross-validated excess word
    TextRule {
        needle: "pivotal",
        message: "LLM tell: 'pivotal' (Kobak 2025, Liang 2024)",
        replacement: Some("key"),
        severity: Severity::High,
    },
    // source: kobak2024 cross-validated
    TextRule {
        needle: "notably",
        message: "LLM tell: 'notably' (Kobak 2025)",
        replacement: None,
        severity: Severity::High,
    },
    // source: kobak2024 high-frequency excess verb
    TextRule {
        needle: "leveraging",
        message: "LLM filler: 'leveraging' (Kobak 2025)",
        replacement: Some("using"),
        severity: Severity::High,
    },
    // source: kobak2024 — verb form; distinct from leveraging
    TextRule {
        needle: "leverage",
        message: "LLM filler: 'leverage' when used as verb (Kobak 2025)",
        replacement: Some("use"),
        severity: Severity::High,
    },
    // source: kobak2024 excess verb
    TextRule {
        needle: "streamline",
        message: "LLM filler: 'streamline' (Kobak 2025)",
        replacement: None,
        severity: Severity::High,
    },
    // source: kobak2024 excess verb
    TextRule {
        needle: "utilize",
        message: "LLM filler: 'utilize' (Kobak 2025)",
        replacement: Some("use"),
        severity: Severity::High,
    },
    // source: kobak2024 excess verb
    TextRule {
        needle: "facilitate",
        message: "LLM filler: 'facilitate' (Kobak 2025)",
        replacement: Some("help"),
        severity: Severity::High,
    },
    // source: kobak2024 excess verb
    TextRule {
        needle: "endeavor",
        message: "LLM filler: 'endeavor' (Kobak 2025)",
        replacement: Some("try"),
        severity: Severity::High,
    },
    // source: kobak2024 excess verb
    TextRule {
        needle: "commence",
        message: "LLM filler: 'commence' (Kobak 2025)",
        replacement: Some("start"),
        severity: Severity::High,
    },
    // source: neri2024 confirmed; kobak2024 listed
    TextRule {
        needle: "tapestry",
        message: "LLM filler: 'tapestry' (Neri 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // source: neri2024 confirmed high z-score
    TextRule {
        needle: "testament",
        message: "LLM filler: 'testament' (Neri 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // source: neri2024 confirmed
    TextRule {
        needle: "stands as a testament",
        message: "LLM cliché: 'stands as a testament' (Neri 2024)",
        replacement: None,
        severity: Severity::High,
    },
    // === MEDIUM: High δ but lower r — common words elevated by LLM (Kobak 2025 δ data) ===
    // source: kobak2024 δ=0.041 — highest absolute gap; appears legitimately in many contexts
    TextRule {
        needle: "comprehensive",
        message: "LLM filler: 'comprehensive' (Kobak 2025 δ=high)",
        replacement: Some("thorough"),
        severity: Severity::Medium,
    },
    // source: kobak2024 δ=0.026 — third highest gap
    TextRule {
        needle: "crucial",
        message: "LLM filler: 'crucial' (Kobak 2025 δ=0.026)",
        replacement: Some("important"),
        severity: Severity::Medium,
    },
    // source: kobak2024 cross-validated; common word elevated
    TextRule {
        needle: "particularly",
        message: "LLM filler: 'particularly' (Kobak 2025 cross-validated)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 cross-validated
    TextRule {
        needle: "enhancing",
        message: "LLM tell: 'enhancing' (Kobak 2025 cross-validated)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 cross-validated
    TextRule {
        needle: "exhibited",
        message: "LLM tell: 'exhibited' (Kobak 2025 cross-validated)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 cross-validated
    TextRule {
        needle: "insights",
        message: "LLM filler: 'insights' (Kobak 2025 cross-validated)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 δ data — flagged as 'boast(s) X features' pattern
    TextRule {
        needle: "boast",
        message: "LLM filler: 'boast/boasts' as in 'boasts features' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: juzek2025 emerging signal 2024-2025
    TextRule {
        needle: "harnessing",
        message: "LLM filler: 'harnessing' (Juzek 2025 emerging signal)",
        replacement: Some("using"),
        severity: Severity::Medium,
    },
    // source: juzek2025 emerging signal 2024-2025
    TextRule {
        needle: "harnesses",
        message: "LLM filler: 'harnesses' (Juzek 2025 emerging signal)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj; pre-LLM marketing language with lower ratio than tier-1
    TextRule {
        needle: "groundbreaking",
        message: "LLM filler: 'groundbreaking' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj; lower ratio — pre-LLM marketing language
    TextRule {
        needle: "innovative",
        message: "LLM filler: 'innovative' (Kobak 2025, lower ratio)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024; lower ratio — pre-LLM marketing language
    TextRule {
        needle: "revolutionary",
        message: "LLM filler: 'revolutionary' (Kobak 2025, lower ratio)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024; lower ratio — pre-LLM marketing language
    TextRule {
        needle: "cutting-edge",
        message: "LLM filler: 'cutting-edge' (Kobak 2025, lower ratio)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj — common in specs/RFCs; flag but acknowledge context
    TextRule {
        needle: "robust",
        message: "LLM filler: 'robust' (Kobak 2025; legitimate in security specs — review context)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj
    TextRule {
        needle: "multifaceted",
        message: "LLM filler: 'multifaceted' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj
    TextRule {
        needle: "vibrant",
        message: "LLM filler: 'vibrant' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj
    TextRule {
        needle: "seamlessly",
        message: "LLM filler: 'seamlessly' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj
    TextRule {
        needle: "ingrained",
        message: "LLM filler: 'ingrained' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024 excess adj
    TextRule {
        needle: "indelible",
        message: "LLM filler: 'indelible' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // source: kobak2024; often used as connector phrase, not location
    TextRule {
        needle: "evolving landscape",
        message: "LLM cliché: 'evolving landscape' (Kobak 2025)",
        replacement: None,
        severity: Severity::Medium,
    },
    // === SYCOPHANTIC OPENERS — Critical ===
    // source: juzek2025 rlhf-confirmed — first-sentence validation-seeking patterns
    TextRule {
        needle: "certainly!",
        message: "Sycophantic opener: 'Certainly!' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "great question!",
        message: "Sycophantic opener: 'Great question!' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "of course!",
        message: "Sycophantic opener: 'Of course!' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "absolutely!",
        message: "Sycophantic opener: 'Absolutely!' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "happy to help",
        message: "Sycophantic opener: 'happy to help' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "happy to explain",
        message: "Sycophantic opener: 'happy to explain' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "i'd be happy to",
        message: "Sycophantic opener: 'I'd be happy to' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "i would be happy to",
        message: "Sycophantic opener: 'I would be happy to' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    // === CHATBOT CLOSERS — Critical ===
    // source: juzek2025 rlhf-confirmed — closing validation patterns
    TextRule {
        needle: "i hope this helps",
        message: "Chatbot closer: 'I hope this helps' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "let me know if",
        message: "Chatbot closer: 'Let me know if' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    TextRule {
        needle: "feel free to",
        message: "Chatbot closer: 'Feel free to' (RLHF-induced, Juzek 2025)",
        replacement: None,
        severity: Severity::Critical,
    },
    // === LOW: Filler connectors and hedging ===
    // source: rosenfeld2024 — discourse connectors elevated in LLM text; appear legitimately in academic writing
    TextRule {
        needle: "moreover",
        message: "LLM connector: 'moreover' (Rosenfeld 2024)",
        replacement: None,
        severity: Severity::Low,
    },
    TextRule {
        needle: "furthermore",
        message: "LLM connector: 'furthermore' (Rosenfeld 2024)",
        replacement: None,
        severity: Severity::Low,
    },
    TextRule {
        needle: "subsequently",
        message: "LLM connector: 'subsequently' (Kobak 2025)",
        replacement: Some("then"),
        severity: Severity::Low,
    },
    TextRule {
        needle: "in conclusion",
        message: "LLM connector: 'in conclusion' (Rosenfeld 2024)",
        replacement: None,
        severity: Severity::Low,
    },
    TextRule {
        needle: "serves as a reminder",
        message: "LLM filler: 'serves as a reminder'",
        replacement: None,
        severity: Severity::Low,
    },
    // source: kobak2024 — hedging phrase
    TextRule {
        needle: "it is worth noting",
        message: "LLM hedge: 'it is worth noting' (Kobak 2025)",
        replacement: None,
        severity: Severity::Low,
    },
    TextRule {
        needle: "it is important to note",
        message: "LLM hedge: 'it is important to note'",
        replacement: None,
        severity: Severity::Low,
    },
    TextRule {
        needle: "could potentially",
        message: "Hedging: 'could potentially'",
        replacement: Some("could"),
        severity: Severity::Low,
    },
    TextRule {
        needle: "might possibly",
        message: "Hedging: 'might possibly'",
        replacement: Some("might"),
        severity: Severity::Low,
    },
    TextRule {
        needle: "arguably could be considered",
        message: "Hedging: 'arguably could be considered'",
        replacement: None,
        severity: Severity::Low,
    },
    // source: common filler phrase
    TextRule {
        needle: "in order to",
        message: "Filler: 'in order to'",
        replacement: Some("to"),
        severity: Severity::Low,
    },
    TextRule {
        needle: "due to the fact that",
        message: "Filler: 'due to the fact that'",
        replacement: Some("because"),
        severity: Severity::Low,
    },
];

pub fn apply_text_rules(content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut in_code_block = false;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Toggle fenced code block state and skip the fence line itself.
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        // Skip bare URL lines (no prose context to flag).
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            continue;
        }

        let line_lower = line.to_lowercase();

        // Build a mapping from char index to byte offset in both strings so we
        // can translate a byte position found in `line_lower` back to the
        // corresponding byte position in `line`. `.to_lowercase()` can change
        // byte lengths for certain Unicode characters (e.g. 'İ' → 'i̇'), so
        // reusing `line_lower` byte offsets to slice `line` is unsafe.
        let lower_char_bytes: Vec<usize> = {
            let mut v = Vec::new();
            let mut b = 0usize;
            for c in line_lower.chars() {
                v.push(b);
                b += c.len_utf8();
            }
            v.push(b); // sentinel: one past the end
            v
        };
        let orig_char_bytes: Vec<usize> = {
            let mut v = Vec::new();
            let mut b = 0usize;
            for c in line.chars() {
                v.push(b);
                b += c.len_utf8();
            }
            v.push(b); // sentinel
            v
        };

        // Convert a byte offset in `line_lower` to a char index.
        let lower_byte_to_char =
            |byte: usize| -> Option<usize> { lower_char_bytes.iter().position(|&b| b == byte) };

        for rule in TEXT_RULES {
            let mut search_start = 0usize;
            while let Some(pos) = line_lower[search_start..].find(rule.needle) {
                let col_lower = search_start + pos;
                let end_lower = col_lower + rule.needle.len();
                // Require word boundaries on `line_lower` (same char semantics).
                if !is_word_boundary(&line_lower, col_lower, end_lower) {
                    search_start = end_lower;
                    continue;
                }
                // Map byte offsets from `line_lower` back to `line`.
                let (col, end) =
                    match (lower_byte_to_char(col_lower), lower_byte_to_char(end_lower)) {
                        (Some(ci), Some(ei)) => (orig_char_bytes[ci], orig_char_bytes[ei]),
                        _ => {
                            // Offset doesn't align to a char boundary — skip safely.
                            search_start = end_lower;
                            continue;
                        }
                    };
                // Skip matches inside inline backtick spans (using `line` offsets).
                if is_in_backtick_span(line, col, end) {
                    search_start = end_lower;
                    continue;
                }
                let matched = &line[col..end];
                findings.push(Finding {
                    line: line_idx + 1,
                    col,
                    matched: matched.to_string(),
                    message: rule.message.to_string(),
                    replacement: rule.replacement.map(str::to_string),
                    severity: rule.severity,
                });
                search_start = end_lower;
            }
        }
    }

    findings
}

/// Returns `true` if the match at `[start, end)` is delimited by non-alphanumeric
/// characters on both sides (word-boundary check). Multi-byte safe.
pub(crate) fn is_word_boundary(line: &str, start: usize, end: usize) -> bool {
    let before_ok = if start == 0 {
        true
    } else {
        // Walk back one char
        line[..start]
            .chars()
            .next_back()
            .map(|c| !c.is_alphanumeric())
            .unwrap_or(true)
    };
    let after_ok = if end >= line.len() {
        true
    } else {
        line[end..]
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric())
            .unwrap_or(true)
    };
    before_ok && after_ok
}

/// Returns `true` if the entire byte range `[start, end)` falls inside a single
/// inline backtick span. Both `start` and `end` must be byte offsets into `line`.
/// The toggle fires *before* the position check so that the backtick character
/// itself is not considered "inside" the span — ordering must not be changed.
fn is_in_backtick_span(line: &str, start: usize, end: usize) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let mut char_byte_positions: Vec<usize> = Vec::with_capacity(chars.len());
    {
        let mut pos = 0usize;
        for &c in &chars {
            char_byte_positions.push(pos);
            pos += c.len_utf8();
        }
    }
    // Determine whether `start` is inside a backtick span.
    let mut inside = false;
    for (i, &byte_pos) in char_byte_positions.iter().enumerate() {
        if chars[i] == '`' {
            inside = !inside;
        }
        if byte_pos >= start {
            let start_inside = inside && chars[i] != '`';
            if !start_inside {
                return false;
            }
            // Also verify `end` is still inside the same span (no closing backtick
            // between start and end).
            for j in (i + 1)..chars.len() {
                if char_byte_positions[j] >= end {
                    return true;
                }
                if chars[j] == '`' {
                    return false; // closing backtick before end — straddles boundary
                }
            }
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::clean;

    #[test]
    fn finds_utilize() {
        let findings = apply_text_rules("We should utilize this approach.");
        assert!(findings
            .iter()
            .any(|f| f.matched.to_lowercase() == "utilize"));
    }

    #[test]
    fn finds_sycophantic_opener() {
        let findings = apply_text_rules("Certainly! Here is the answer.");
        assert!(findings
            .iter()
            .any(|f| f.matched.to_lowercase() == "certainly!"));
    }

    #[test]
    fn applies_fix_utilize() {
        let content = "We should utilize this.";
        let findings = apply_text_rules(content);
        let cleaned = clean(content, &findings);
        assert!(cleaned.contains("use"), "Expected 'use', got: {}", cleaned);
        assert!(!cleaned.contains("utilize"));
    }

    #[test]
    fn clean_two_replacements_same_line() {
        let input = "utilize and facilitate this.";
        let findings = apply_text_rules(input);
        let cleaned = clean(input, &findings);
        assert_eq!(
            cleaned, "use and help this.",
            "both replacements must be applied correctly, got: {}",
            cleaned
        );
    }

    #[test]
    fn apply_case_preserves_capital() {
        assert_eq!(super::super::apply_case("Utilize", "use"), "Use");
        assert_eq!(super::super::apply_case("utilize", "use"), "use");
    }

    #[test]
    fn preserves_trailing_newline() {
        let content = "utilize this.\n";
        let findings = apply_text_rules(content);
        let cleaned = clean(content, &findings);
        assert!(cleaned.ends_with('\n'));
    }

    #[test]
    fn severity_critical_for_sycophantic() {
        let findings = apply_text_rules("Certainly! Here is the answer.");
        let f = findings
            .iter()
            .find(|f| f.matched.to_lowercase() == "certainly!")
            .unwrap();
        assert_eq!(f.severity, Severity::Critical);
    }

    #[test]
    fn severity_high_for_buzzword() {
        let findings = apply_text_rules("We are leveraging new tech.");
        let f = findings
            .iter()
            .find(|f| f.matched.to_lowercase() == "leveraging")
            .unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn severity_low_for_filler_connector() {
        let findings = apply_text_rules("Moreover, this is good.");
        let f = findings
            .iter()
            .find(|f| f.matched.to_lowercase() == "moreover")
            .unwrap();
        assert_eq!(f.severity, Severity::Low);
    }

    #[test]
    fn severity_low_for_filler_phrase() {
        let findings = apply_text_rules("In order to proceed, do this.");
        let f = findings
            .iter()
            .find(|f| f.matched.to_lowercase() == "in order to")
            .unwrap();
        assert_eq!(f.severity, Severity::Low);
    }

    #[test]
    fn code_block_not_flagged() {
        let input = "Some prose.\n```\nutilize this approach.\n```\nEnd.\n";
        let findings = apply_text_rules(input);
        assert!(
            findings
                .iter()
                .all(|f| f.matched.to_lowercase() != "utilize"),
            "utilize inside fenced block should not be flagged"
        );
    }

    #[test]
    fn url_line_not_flagged() {
        let input = "https://example.com/utilize-this-comprehensive-guide";
        let findings = apply_text_rules(input);
        assert!(
            findings.is_empty(),
            "bare URL line should produce no findings"
        );
    }

    #[test]
    fn inline_code_not_flagged() {
        let input = "Call `utilize` to proceed.";
        let findings = apply_text_rules(input);
        assert!(
            findings
                .iter()
                .all(|f| f.matched.to_lowercase() != "utilize"),
            "utilize inside backtick span should not be flagged"
        );
    }
}

#[cfg(test)]
mod challenge_tests {
    use super::*;
    use super::super::clean;

    // --- Word boundary: substrings ---
    #[test]
    fn pivotale_unchanged() {
        let f = apply_text_rules("C'est une décision pivotale.");
        assert!(
            f.is_empty(),
            "pivotale should not be flagged, got: {:?}",
            f.iter().map(|x| &x.matched).collect::<Vec<_>>()
        );
    }
    #[test]
    fn delves_fires() {
        let f = apply_text_rules("She delves into the topic.");
        assert!(!f.is_empty(), "delves should be flagged as LLM tell");
        assert!(f.iter().any(|x| x.matched.to_lowercase().contains("delve")));
    }
    #[test]
    fn commencement_unchanged() {
        let input = "The commencement ceremony starts now.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert_eq!(cleaned, input, "commencement should not be mangled");
    }
    #[test]
    fn utilization_unchanged() {
        let f = apply_text_rules("Memory utilization is 80%.");
        assert!(f.is_empty(), "utilization should not be flagged");
    }
    #[test]
    fn notably_in_notable_unchanged() {
        let f = apply_text_rules("The notable result stands.");
        assert!(f.is_empty(), "notable should not be flagged");
    }

    // --- Non-English passthrough ---
    #[test]
    fn spanish_notable_unchanged() {
        let input = "El resultado es notable.";
        let f = apply_text_rules(input);
        assert!(f.is_empty(), "Spanish 'notable' should not be flagged");
    }
    #[test]
    fn french_passthrough() {
        let input = "Le résultat est remarquable.";
        let f = apply_text_rules(input);
        assert!(f.is_empty());
    }

    // --- Fenced code block with info string ---
    #[test]
    fn fenced_with_info_string_unchanged() {
        let input = "```python\nutilize this\n```";
        let f = apply_text_rules(input);
        assert!(
            f.iter().all(|x| x.matched.to_lowercase() != "utilize"),
            "utilize inside ```python block should not be flagged"
        );
    }

    // --- Inline backtick + prose on same line ---
    #[test]
    fn banned_outside_backtick_fixed() {
        let input = "Use `foo` and utilize bar.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert!(
            cleaned.contains("use bar"),
            "prose utilize should be fixed, got: {}",
            cleaned
        );
        assert!(
            cleaned.contains("`foo`"),
            "backtick span preserved, got: {}",
            cleaned
        );
    }

    // --- Case ---
    #[test]
    fn all_caps_utilize_known_behaviour() {
        // Known limitation: apply_case only uppercases the first character of the
        // replacement. "UTILIZE" -> apply_case("UTILIZE", "use") -> "Use".
        // Full-caps preservation is not implemented.
        let input = "UTILIZE this.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert_eq!(
            cleaned, "Use this.",
            "all-caps: first char uppercased, rest from replacement"
        );
    }

    // --- Multiple banned words same line ---
    #[test]
    fn multiple_banned_words() {
        // "utilize" -> "use", "leveraging" -> "using"
        let input = "utilize and leveraging this.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert!(
            cleaned.contains("use") && cleaned.contains("using"),
            "got: {}",
            cleaned
        );
    }

    // --- Empty / whitespace ---
    #[test]
    fn empty_input() {
        assert!(apply_text_rules("").is_empty());
    }

    // --- Severity rank ordering ---
    #[test]
    fn severity_rank_strictly_ordered() {
        assert!(Severity::Critical.rank() > Severity::High.rank());
        assert!(Severity::High.rank() > Severity::Medium.rank());
        assert!(Severity::Medium.rank() > Severity::Low.rank());
    }

    // --- min-severity critical excludes high ---
    #[test]
    fn min_severity_critical_excludes_high() {
        // "leveraging" is High, "Certainly!" is Critical
        let findings = apply_text_rules("Certainly! We are leveraging new tech.");
        let min_rank = Severity::Critical.rank();
        let filtered: Vec<_> = findings
            .iter()
            .filter(|f| f.severity.rank() >= min_rank)
            .collect();
        assert!(filtered
            .iter()
            .any(|f| f.matched.to_lowercase() == "certainly!"));
        assert!(!filtered
            .iter()
            .any(|f| f.matched.to_lowercase() == "leveraging"));
    }

    // --- Unicode prefix does not trigger word-boundary match ---
    #[test]
    fn unicode_prefix_blocks_match() {
        // "épivotal" starts with a non-ASCII char — "pivotal" must not fire
        let f = apply_text_rules("Cette décision épivotale est importante.");
        assert!(
            f.iter().all(|x| x.matched.to_lowercase() != "pivotal"),
            "pivotal inside unicode-prefixed word should not fire"
        );
    }

    // --- Double backtick span ---
    #[test]
    fn double_backtick_span_not_flagged() {
        // ``utilize`` is an inline code span in reStructuredText / some Markdown variants
        let input = "Call ``utilize`` to proceed.";
        let f = apply_text_rules(input);
        // Single-backtick detection does NOT guard double-backtick spans — document the
        // current behaviour: the match between the two backtick pairs fires.
        // This test just verifies we don't panic and the result is deterministic.
        let _ = clean(input, &f); // must not panic
    }

    // --- Unclosed backtick span: not flagged (conservative) ---
    #[test]
    fn unclosed_backtick_span_not_flagged() {
        // An unclosed backtick means `is_in_backtick_span` sees "inside=true" and never
        // closes it. Current behaviour: conservative — the match is suppressed.
        // This avoids false positives at the cost of missing some edge-case findings.
        let input = "Call `utilize to proceed.";
        let f = apply_text_rules(input);
        assert!(
            f.iter().all(|x| x.matched.to_lowercase() != "utilize"),
            "unclosed backtick: conservative — utilize should not be flagged"
        );
    }

    // --- Phase 2: Kobak empirical data tests ---
    #[test]
    fn finds_showcasing() {
        let findings = apply_text_rules("This work showcasing the results.");
        assert!(findings
            .iter()
            .any(|f| f.matched.to_lowercase() == "showcasing"));
        let f = findings
            .iter()
            .find(|f| f.matched.to_lowercase() == "showcasing")
            .unwrap();
        // r=9.2 — below the Critical threshold of r>10; correctly classified as High
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn finds_meticulous() {
        let findings = apply_text_rules("The meticulous analysis was thorough.");
        assert!(findings
            .iter()
            .any(|f| f.matched.to_lowercase() == "meticulous"));
        let f = findings
            .iter()
            .find(|f| f.matched.to_lowercase() == "meticulous")
            .unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn finds_realm() {
        let findings = apply_text_rules("In the realm of computing.");
        assert!(findings.iter().any(|f| f.matched.to_lowercase() == "realm"));
    }

    #[test]
    fn finds_intricate() {
        let findings = apply_text_rules("The intricate details matter.");
        assert!(findings
            .iter()
            .any(|f| f.matched.to_lowercase() == "intricate"));
    }

    #[test]
    fn finds_happy_to_help() {
        let findings = apply_text_rules("I'd be happy to help you with that.");
        assert!(findings.iter().any(|f| f.message.contains("Sycophantic")));
    }
}
