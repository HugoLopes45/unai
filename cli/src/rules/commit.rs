use super::{Finding, Severity};

pub(crate) fn check_commit_patterns(line: &str, lineno: usize, findings: &mut Vec<Finding>) {
    let lower = line.to_lowercase();

    // Vague commit verbs — Low
    let vague = [
        "update stuff",
        "fix things",
        "wip",
        "misc changes",
        "minor fixes",
    ];
    for phrase in &vague {
        if let Some(col) = lower.find(phrase) {
            findings.push(Finding {
                line: lineno,
                col,
                matched: phrase.to_string(),
                message: format!("Vague commit message: '{}'", phrase),
                replacement: None,
                severity: Severity::Low,
            });
        }
    }

    // Past tense in subject line — High
    // source: lopes2024 icse — human commits use imperative; LLM commits use past tense
    if lineno == 1 {
        let past_tense_verbs = [
            "added",
            "fixed",
            "updated",
            "changed",
            "removed",
            "modified",
            "implemented",
            "refactored",
            "created",
            "deleted",
            "moved",
            "improved",
            "enhanced",
            "cleaned",
            "bumped",
            "dropped",
            "replaced",
            "resolved",
            "addressed",
            "reverted",
        ];
        let first_word = lower.split_whitespace().next().unwrap_or("");
        // Strip conventional commit prefix if present (e.g. "feat: added" -> check "added")
        let effective_first = if first_word.ends_with(':') {
            lower.split_whitespace().nth(1).unwrap_or("")
        } else {
            first_word
        };
        if past_tense_verbs.contains(&effective_first) {
            let col = lower
                .find(effective_first)
                .unwrap_or_else(|| lower.find(':').map(|p| p + 2).unwrap_or(0));
            findings.push(Finding {
                line: lineno,
                col,
                matched: effective_first.to_string(),
                message: "Past tense in commit subject: use imperative mood ('add' not 'added')"
                    .to_string(),
                replacement: None,
                severity: Severity::High,
            });
        }
    }

    // Vague scope words in subject line — High
    // source: lopes2024 — human commits name one specific thing
    if lineno == 1 {
        let vague_scope = ["various", "several", "multiple", "many"];
        for word in &vague_scope {
            let mut start = 0;
            while let Some(pos) = lower[start..].find(word) {
                let abs = start + pos;
                let end = abs + word.len();
                let before_ok =
                    abs == 0 || !lower[..abs].chars().last().unwrap_or(' ').is_alphanumeric();
                let after_ok = end >= lower.len()
                    || !lower[end..].chars().next().unwrap_or(' ').is_alphanumeric();
                if before_ok && after_ok {
                    findings.push(Finding {
                        line: lineno,
                        col: abs,
                        matched: word.to_string(),
                        message: "Vague scope in commit subject: name the specific change"
                            .to_string(),
                        replacement: None,
                        severity: Severity::High,
                    });
                    break; // one finding per word
                }
                start = end;
            }
        }
    }

    // Title-case subject line — Medium
    if lineno == 1 {
        let words: Vec<&str> = line.split_whitespace().collect();
        // Skip conventional commit prefix (word ending in ':')
        let content_words: Vec<&str> = words
            .iter()
            .skip_while(|w| w.ends_with(':'))
            .copied()
            .collect();
        let capitalized_count = content_words
            .iter()
            .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            .count();
        if content_words.len() >= 3 && capitalized_count >= 3 {
            findings.push(Finding {
                line: lineno,
                col: 0,
                matched: line.to_string(),
                message: "Title-case commit subject: use sentence case".to_string(),
                replacement: None,
                severity: Severity::Medium,
            });
        }
    }

    // Multiline body on single-purpose fix — Low
    // source: arxiv2601.17406 — multiline commit ratio top fingerprint feature
    if lineno == 3 && !line.trim().is_empty() {
        findings.push(Finding {
            line: lineno,
            col: 0,
            matched: line.to_string(),
            message: "Commit body on single-purpose change may over-explain (arxiv:2601.17406)"
                .to_string(),
            replacement: None,
            severity: Severity::Low,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::Severity;
    use super::super::{apply_code_rules, CodeRule};

    #[test]
    fn commit_past_tense_fires() {
        let findings = apply_code_rules("Added authentication logic", &[CodeRule::Commits]);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("imperative mood")),
            "past tense should fire, got: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
        let f = findings
            .iter()
            .find(|f| f.message.contains("imperative"))
            .unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn commit_imperative_no_fire() {
        let findings = apply_code_rules("Add authentication logic", &[CodeRule::Commits]);
        assert!(
            !findings
                .iter()
                .any(|f| f.message.contains("imperative mood")),
            "imperative mood should not fire for 'Add'"
        );
    }

    #[test]
    fn commit_conventional_prefix_past_tense_fires() {
        let findings = apply_code_rules("feat: added authentication logic", &[CodeRule::Commits]);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("imperative mood")),
            "past tense should fire even with conventional commit prefix, got: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn commit_vague_scope_fires() {
        let findings = apply_code_rules("Updated several files for release", &[CodeRule::Commits]);
        assert!(
            findings.iter().any(|f| f.message.contains("Vague scope")),
            "vague scope should fire, got: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }
}
