mod code;
mod commit;
mod ignore;
mod structural;
mod text;

pub use code::{apply_code_rules, CodeRule};
pub use ignore::collect_ignored_lines;
pub use structural::apply_structural_rules;
pub use text::apply_text_rules;
pub(crate) use text::is_word_boundary;

/// Apply user-defined rules from `cfg` to `content`, returning findings.
/// Searches case-insensitively (needle = pattern.to_lowercase()). Byte offsets
/// stored in `Finding.col` are always relative to the *original* line so that
/// `clean()` and JSON consumers can safely slice the original text.
pub fn apply_user_rules(content: &str, cfg: Option<&crate::config::Config>) -> Vec<Finding> {
    let Some(cfg) = cfg else { return vec![] };
    let mut findings = Vec::new();
    for rule in &cfg.rules {
        if !rule.enabled {
            continue;
        }
        let needle = rule.pattern.to_lowercase();
        let severity = match rule.severity.as_deref() {
            Some("critical") => Severity::Critical,
            Some("high") => Severity::High,
            Some("medium") => Severity::Medium,
            _ => Severity::Low,
        };
        for (line_idx, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Build char-index → byte-offset tables for both strings so we can
            // map a byte position found in `line_lower` back to the corresponding
            // byte position in `line`. `.to_lowercase()` can change byte lengths
            // for certain Unicode characters (e.g. 'İ' → 'i̇').
            let lower_char_bytes: Vec<usize> = {
                let mut v = Vec::new();
                let mut b = 0usize;
                for c in line_lower.chars() {
                    v.push(b);
                    b += c.len_utf8();
                }
                v.push(b);
                v
            };
            let orig_char_bytes: Vec<usize> = {
                let mut v = Vec::new();
                let mut b = 0usize;
                for c in line.chars() {
                    v.push(b);
                    b += c.len_utf8();
                }
                v.push(b);
                v
            };
            // Binary search: convert a byte offset in `line_lower` to a char index.
            let lower_byte_to_char = |byte: usize| -> Option<usize> {
                let i = lower_char_bytes.partition_point(|&b| b < byte);
                if lower_char_bytes.get(i) == Some(&byte) { Some(i) } else { None }
            };

            let mut start = 0;
            while let Some(pos) = line_lower[start..].find(&needle) {
                let col_lower = start + pos;
                let end_lower = col_lower + needle.len();
                if is_word_boundary(&line_lower, col_lower, end_lower) {
                    // Map offsets from `line_lower` back to `line`.
                    let (col, end) = match (
                        lower_byte_to_char(col_lower),
                        lower_byte_to_char(end_lower),
                    ) {
                        (Some(ci), Some(ei)) => (orig_char_bytes[ci], orig_char_bytes[ei]),
                        _ => {
                            start = end_lower;
                            continue;
                        }
                    };
                    let matched = line[col..end].to_string();
                    findings.push(Finding {
                        line: line_idx + 1,
                        col,
                        matched,
                        message: rule
                            .message
                            .clone()
                            .unwrap_or_else(|| format!("User rule: '{}'", rule.pattern)),
                        replacement: rule.replacement.clone(),
                        severity,
                    });
                }
                start = end_lower;
            }
        }
    }
    findings
}

/// Severity level of a finding.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    /// Numeric rank for filtering: higher = more severe.
    pub fn rank(&self) -> u8 {
        match self {
            Self::Critical => 3,
            Self::High => 2,
            Self::Medium => 1,
            Self::Low => 0,
        }
    }
}

/// A single match found in the input.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Finding {
    /// 1-based line number.
    pub line: usize,
    /// Column byte offset within the line (0-based).
    pub col: usize,
    /// Matched text.
    pub matched: String,
    /// Explanation / suggestion.
    pub message: String,
    /// Replacement text if auto-fixable, otherwise None.
    pub replacement: Option<String>,
    /// Severity classification.
    pub severity: Severity,
}

/// Produce a cleaned version of content by applying auto-fixable replacements.
pub fn clean(content: &str, findings: &[Finding]) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();

    let mut drop_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut fix_by_line: std::collections::HashMap<usize, Vec<&Finding>> =
        std::collections::HashMap::new();

    for f in findings {
        // f.line is 1-based; skip malformed findings with line == 0.
        let Some(idx) = f.line.checked_sub(1) else {
            continue;
        };
        if idx >= lines.len() {
            continue;
        }
        match f.replacement.as_deref() {
            Some("") => {
                drop_lines.insert(idx);
            }
            Some(_) => {
                fix_by_line.entry(idx).or_default().push(f);
            }
            None => {}
        }
    }

    for (idx, line_findings) in &fix_by_line {
        if drop_lines.contains(idx) {
            continue;
        }
        let mut line = lines[*idx].clone();
        let mut sorted = line_findings.clone();
        sorted.sort_by(|a, b| b.col.cmp(&a.col));

        for f in sorted {
            if let Some(ref replacement) = f.replacement {
                let end = f.col + f.matched.len();
                if end > line.len() || !line.is_char_boundary(f.col) || !line.is_char_boundary(end)
                {
                    eprintln!(
                        "unai: warning: skipping invalid offset at line {} col {} (line length {})",
                        f.line,
                        f.col,
                        line.len()
                    );
                    continue;
                }
                let original = &line[f.col..end];
                let fixed = apply_case(original, replacement);
                line = format!("{}{}{}", &line[..f.col], fixed, &line[end..]);
            }
        }
        lines[*idx] = line;
    }

    let joined = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            if drop_lines.contains(&idx) {
                None
            } else {
                Some(line.as_str())
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if content.ends_with('\n') {
        format!("{}\n", joined)
    } else {
        joined
    }
}

/// Preserve capitalization style of the original word when applying a replacement.
pub(crate) fn apply_case(original: &str, replacement: &str) -> String {
    if original.is_empty() || replacement.is_empty() {
        return replacement.to_string();
    }
    if let Some(first_char) = original.chars().next() {
        if first_char.is_uppercase() {
            let mut chars = replacement.chars();
            return match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            };
        }
    }
    replacement.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(line: usize, col: usize, matched: &str, replacement: Option<&str>) -> Finding {
        Finding {
            line,
            col,
            matched: matched.to_string(),
            message: "test".to_string(),
            replacement: replacement.map(str::to_string),
            severity: Severity::Low,
        }
    }

    // A finding with line == 0 is malformed (1-based) — clean() must skip it, not underflow.
    #[test]
    fn clean_skips_finding_with_line_zero() {
        let f = make_finding(0, 0, "x", Some("y"));
        let result = clean("hello\n", &[f]);
        assert_eq!(
            result, "hello\n",
            "line-zero finding must be skipped, not panic"
        );
    }

    // A finding with col beyond line length must be skipped gracefully, not panic.
    #[test]
    fn clean_skips_finding_with_out_of_bounds_col() {
        let f = make_finding(1, 100, "x", Some("y"));
        let result = clean("hello\n", &[f]);
        assert_eq!(
            result, "hello\n",
            "out-of-bounds col must be skipped, not panic"
        );
    }

    // Multiple non-overlapping matches on the same line must all be reported.
    #[test]
    fn apply_user_rules_finds_multiple_matches_same_line() {
        use crate::config::{Config, IgnoreConfig, UserRule};
        let cfg = Config {
            version: 1,
            rules: vec![UserRule {
                pattern: "ab".to_string(),
                replacement: None,
                severity: None,
                message: None,
                enabled: true,
            }],
            ignore: IgnoreConfig::default(),
        };
        let findings = apply_user_rules("ab ab ab", Some(&cfg));
        assert_eq!(
            findings.len(),
            3,
            "three non-overlapping 'ab' matches expected, got {}",
            findings.len()
        );
    }

    // The search cursor advances past each match (start = end), so a long line with many
    // matches must terminate in bounded time.
    #[test]
    fn apply_user_rules_terminates_on_repeated_pattern() {
        use crate::config::{Config, IgnoreConfig, UserRule};
        let cfg = Config {
            version: 1,
            rules: vec![UserRule {
                pattern: "x".to_string(),
                replacement: None,
                severity: None,
                message: None,
                enabled: true,
            }],
            ignore: IgnoreConfig::default(),
        };
        // Long line with many matches — must not hang.
        let line = "x ".repeat(1000);
        let findings = apply_user_rules(&line, Some(&cfg));
        assert_eq!(findings.len(), 1000);
    }
}
