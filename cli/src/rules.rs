/// Severity level of a finding.
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone)]
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

// ---------------------------------------------------------------------------
// Text rules
// ---------------------------------------------------------------------------

struct TextRule {
    /// Lowercase needle for case-insensitive matching.
    needle: &'static str,
    message: &'static str,
    /// Optional auto-fix replacement. If None, the finding is flagged only.
    replacement: Option<&'static str>,
    severity: Severity,
}

const TEXT_RULES: &[TextRule] = &[
    // Banned LLM buzzwords — High
    TextRule { needle: "tapestry", message: "LLM filler: 'tapestry'", replacement: None, severity: Severity::High },
    TextRule { needle: "testament", message: "LLM filler: 'testament'", replacement: None, severity: Severity::High },
    TextRule { needle: "stands as a testament", message: "LLM filler: 'stands as a testament'", replacement: None, severity: Severity::High },
    TextRule { needle: "delve", message: "LLM filler: 'delve'", replacement: Some("explore"), severity: Severity::High },
    TextRule { needle: "pivotal", message: "LLM filler: 'pivotal'", replacement: Some("key"), severity: Severity::High },
    TextRule { needle: "comprehensive", message: "LLM filler: 'comprehensive'", replacement: Some("thorough"), severity: Severity::High },
    TextRule { needle: "multifaceted", message: "LLM filler: 'multifaceted'", replacement: None, severity: Severity::High },
    TextRule { needle: "evolving landscape", message: "LLM cliché: 'evolving landscape'", replacement: None, severity: Severity::High },
    TextRule { needle: "vibrant", message: "LLM filler: 'vibrant'", replacement: None, severity: Severity::High },
    TextRule { needle: "crucial", message: "LLM filler: 'crucial'", replacement: Some("important"), severity: Severity::High },
    TextRule { needle: "ingrained", message: "LLM filler: 'ingrained'", replacement: None, severity: Severity::High },
    TextRule { needle: "indelible", message: "LLM filler: 'indelible'", replacement: None, severity: Severity::High },
    TextRule { needle: "leveraging", message: "LLM filler: 'leveraging'", replacement: Some("using"), severity: Severity::High },
    TextRule { needle: "seamlessly", message: "LLM filler: 'seamlessly'", replacement: None, severity: Severity::High },
    TextRule { needle: "robust", message: "LLM filler: 'robust'", replacement: None, severity: Severity::High },
    TextRule { needle: "cutting-edge", message: "LLM filler: 'cutting-edge'", replacement: None, severity: Severity::High },
    TextRule { needle: "revolutionary", message: "LLM filler: 'revolutionary'", replacement: None, severity: Severity::High },
    TextRule { needle: "innovative", message: "LLM filler: 'innovative'", replacement: None, severity: Severity::High },
    TextRule { needle: "groundbreaking", message: "LLM filler: 'groundbreaking'", replacement: None, severity: Severity::High },
    TextRule { needle: "streamline", message: "LLM filler: 'streamline'", replacement: None, severity: Severity::High },
    TextRule { needle: "utilize", message: "LLM filler: 'utilize'", replacement: Some("use"), severity: Severity::High },
    TextRule { needle: "facilitate", message: "LLM filler: 'facilitate'", replacement: Some("help"), severity: Severity::High },
    TextRule { needle: "endeavor", message: "LLM filler: 'endeavor'", replacement: Some("try"), severity: Severity::High },
    TextRule { needle: "commence", message: "LLM filler: 'commence'", replacement: Some("start"), severity: Severity::High },
    TextRule { needle: "subsequently", message: "LLM filler: 'subsequently'", replacement: Some("then"), severity: Severity::High },
    TextRule { needle: "notably", message: "LLM filler: 'notably'", replacement: None, severity: Severity::High },
    // Filler connectors and hedging — Medium
    TextRule { needle: "moreover", message: "LLM filler: 'moreover'", replacement: None, severity: Severity::Medium },
    TextRule { needle: "furthermore", message: "LLM filler: 'furthermore'", replacement: None, severity: Severity::Medium },
    TextRule { needle: "in conclusion", message: "LLM filler: 'in conclusion'", replacement: None, severity: Severity::Medium },
    TextRule { needle: "serves as a reminder", message: "LLM filler: 'serves as a reminder'", replacement: None, severity: Severity::Medium },
    TextRule { needle: "it is worth noting", message: "LLM hedge: 'it is worth noting'", replacement: None, severity: Severity::Medium },
    TextRule { needle: "it is important to note", message: "LLM hedge: 'it is important to note'", replacement: None, severity: Severity::Medium },
    TextRule { needle: "could potentially", message: "Hedging: 'could potentially'", replacement: Some("could"), severity: Severity::Medium },
    TextRule { needle: "might possibly", message: "Hedging: 'might possibly'", replacement: Some("might"), severity: Severity::Medium },
    TextRule { needle: "arguably could be considered", message: "Hedging: 'arguably could be considered'", replacement: None, severity: Severity::Medium },
    // Sycophantic openers — Critical
    TextRule { needle: "certainly!", message: "Sycophantic opener: 'Certainly!'", replacement: None, severity: Severity::Critical },
    TextRule { needle: "great question!", message: "Sycophantic opener: 'Great question!'", replacement: None, severity: Severity::Critical },
    TextRule { needle: "of course!", message: "Sycophantic opener: 'Of course!'", replacement: None, severity: Severity::Critical },
    TextRule { needle: "absolutely!", message: "Sycophantic opener: 'Absolutely!'", replacement: None, severity: Severity::Critical },
    // Chatbot closers — Critical
    TextRule { needle: "i hope this helps", message: "Chatbot closer: 'I hope this helps'", replacement: None, severity: Severity::Critical },
    TextRule { needle: "let me know if", message: "Chatbot closer: 'Let me know if'", replacement: None, severity: Severity::Critical },
    TextRule { needle: "feel free to", message: "Chatbot closer: 'Feel free to'", replacement: None, severity: Severity::Critical },
    // Filler phrases — Low
    TextRule { needle: "in order to", message: "Filler: 'in order to'", replacement: Some("to"), severity: Severity::Low },
    TextRule { needle: "due to the fact that", message: "Filler: 'due to the fact that'", replacement: Some("because"), severity: Severity::Low },
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
        let lower_byte_to_char = |byte: usize| -> Option<usize> {
            lower_char_bytes.iter().position(|&b| b == byte)
        };

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
                let (col, end) = match (
                    lower_byte_to_char(col_lower),
                    lower_byte_to_char(end_lower),
                ) {
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
fn is_word_boundary(line: &str, start: usize, end: usize) -> bool {
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

// ---------------------------------------------------------------------------
// Code rules
// ---------------------------------------------------------------------------

/// Which code rule categories to apply.
#[derive(Debug, Clone, PartialEq)]
pub enum CodeRule {
    Comments,
    Naming,
    Commits,
    Docstrings,
    Tests,
    Errors,
    Api,
}

impl CodeRule {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "comments" => Some(Self::Comments),
            "naming" => Some(Self::Naming),
            "commits" => Some(Self::Commits),
            "docstrings" => Some(Self::Docstrings),
            "tests" => Some(Self::Tests),
            "errors" => Some(Self::Errors),
            "api" => Some(Self::Api),
            _ => None,
        }
    }
}

pub fn apply_code_rules(content: &str, enabled: &[CodeRule]) -> Vec<Finding> {
    let all = enabled.is_empty();
    let mut findings = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_lower = trimmed.to_lowercase();
        let lineno = idx + 1;

        // Comments rules
        if all || enabled.contains(&CodeRule::Comments) {
            // Section header comments — High
            if is_section_header(trimmed) {
                findings.push(Finding {
                    line: lineno,
                    col: 0,
                    matched: trimmed.to_string(),
                    message: "Section header comment: dividers add noise without value".to_string(),
                    replacement: None,
                    severity: Severity::High,
                });
            }

            // Tautological: comment immediately followed by code that says the same thing — Medium
            if let Some(next_line) = lines.get(idx + 1) {
                if is_tautological_comment(trimmed, next_line.trim()) {
                    findings.push(Finding {
                        line: lineno,
                        col: 0,
                        matched: trimmed.to_string(),
                        message: "Tautological comment: restates the next line of code".to_string(),
                        replacement: Some(String::new()),
                        severity: Severity::Medium,
                    });
                }
            }

            // Bare TODO without context — Critical
            if is_bare_todo(trimmed) {
                findings.push(Finding {
                    line: lineno,
                    col: 0,
                    matched: trimmed.to_string(),
                    message: "Bare TODO without context or ticket reference".to_string(),
                    replacement: None,
                    severity: Severity::Critical,
                });
            }
        }

        // Docstrings / prose-in-code rules — High
        if all || enabled.contains(&CodeRule::Docstrings) {
            let docstring_phrases = [
                "this function serves as",
                "this class represents",
                "this method handles",
                "this module provides",
            ];
            for phrase in &docstring_phrases {
                if line_lower.contains(phrase) {
                    findings.push(Finding {
                        line: lineno,
                        col: line_lower.find(phrase).expect("contains() guarantees find() succeeds"),
                        matched: phrase.to_string(),
                        message: format!("LLM docstring boilerplate: '{phrase}'"),
                        replacement: None,
                        severity: Severity::High,
                    });
                }
            }
        }

        // Naming rules
        if all || enabled.contains(&CodeRule::Naming) {
            check_naming(line, lineno, &mut findings);
        }

        // Commit message patterns (applied when rule is active, e.g. COMMIT_EDITMSG)
        if all || enabled.contains(&CodeRule::Commits) {
            check_commit_patterns(trimmed, lineno, &mut findings);
        }
    }

    findings
}

fn is_section_header(line: &str) -> bool {
    // Matches patterns like:
    //   # ----, # ====, # === Foo ===, # --- Setup ---
    //   # SECTION:, # SETUP, // --- helpers ---
    if !line.starts_with('#') && !line.starts_with("//") && !line.starts_with("--") {
        return false;
    }

    // Strip leading comment marker and whitespace only (not dashes)
    let after_marker = line
        .trim_start_matches('#')
        .trim_start_matches('/')
        .trim_start();

    // Pure divider lines: all dashes or all equals
    if after_marker.chars().all(|c| c == '-' || c == '=' || c == ' ') && after_marker.len() >= 3 {
        return true;
    }

    // Lines that start and/or end with --- or === (e.g. # --- Foo ---, # === Bar)
    if after_marker.starts_with("---") || after_marker.starts_with("===") {
        return true;
    }
    if after_marker.ends_with("---") || after_marker.ends_with("===") {
        return true;
    }

    // All-caps label comments: # SECTION, # SETUP:, # HELPERS
    let words: Vec<&str> = after_marker.split_whitespace().collect();
    if !words.is_empty()
        && words.iter().all(|w| {
            let trimmed = w.trim_end_matches(':');
            trimmed.len() > 1
                && trimmed.chars().all(|c| c.is_uppercase() || c == '_')
        })
    {
        return true;
    }

    false
}

fn is_tautological_comment(comment_line: &str, next_line: &str) -> bool {
    if !comment_line.starts_with('#') && !comment_line.starts_with("//") {
        return false;
    }

    let comment_text = comment_line
        .trim_start_matches(['#', '/', ' '])
        .to_lowercase();

    if comment_text.len() < 5 {
        return false;
    }

    // Extract the key identifier from the next code line and check if comment
    // is essentially rephrasing it. This is a heuristic.
    let code_lower = next_line.to_lowercase();

    // Check if the comment words substantially overlap with code tokens
    let comment_words: Vec<&str> = comment_text
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();

    if comment_words.is_empty() {
        return false;
    }

    let overlap = comment_words
        .iter()
        .filter(|&&w| code_lower.contains(w))
        .count();

    // Tautological if more than half the meaningful words appear in the code
    overlap * 2 > comment_words.len()
}

fn is_bare_todo(line: &str) -> bool {
    let todo_prefixes = ["# todo:", "// todo:", "-- todo:", "/* todo:"];
    let lower = line.to_lowercase();
    for prefix in &todo_prefixes {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let rest = rest.trim();
            // Bare if the message is generic / empty
            let bare_messages = [
                "",
                "add error handling",
                "fix this",
                "handle this",
                "implement",
                "add tests",
                "clean up",
                "refactor",
            ];
            if bare_messages.contains(&rest) {
                return true;
            }
        }
    }
    false
}

fn check_naming(line: &str, lineno: usize, findings: &mut Vec<Finding>) {
    // Variable / class names ending in Manager, Handler, Helper, Util, Utility, Service — High
    let suffixes = ["Manager", "Handler", "Helper", "Util", "Utility", "Service"];
    for suffix in &suffixes {
        // Match word boundaries: the suffix must end the token
        if let Some(pos) = find_suffix_token(line, suffix) {
            findings.push(Finding {
                line: lineno,
                col: pos,
                matched: suffix.to_string(),
                message: format!(
                    "Anemic type suffix '{}': name the responsibility, not the role",
                    suffix
                ),
                replacement: None,
                severity: Severity::High,
            });
        }
    }

    // Redundant type-in-name patterns — Medium
    let redundant = [
        ("userDataObject", "user"),
        ("configurationSettings", "config"),
        ("errorMessageString", "message"),
        ("listOfUsers", "users"),
    ];
    let line_lower = line.to_lowercase();
    for (bad, suggestion) in &redundant {
        if line_lower.contains(&bad.to_lowercase()) {
            findings.push(Finding {
                line: lineno,
                col: line_lower.find(&bad.to_lowercase()).expect("contains() guarantees find() succeeds"),
                matched: bad.to_string(),
                message: format!("Type-in-name anti-pattern: use '{}' instead", suggestion),
                replacement: None, // naming changes require manual review — no auto-fix
                severity: Severity::Medium,
            });
        }
    }
}

/// Find a word-boundary occurrence of `suffix` at the end of an identifier.
fn find_suffix_token(line: &str, suffix: &str) -> Option<usize> {
    let mut search_start = 0;
    while search_start < line.len() {
        let slice = &line[search_start..];
        let pos = slice.find(suffix)?;
        let abs_pos = search_start + pos;
        let end = abs_pos + suffix.len();

        // Check that what follows is a word boundary (non-alphanumeric / underscore)
        let after_ok = line[end..]
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric() && c != '_')
            .unwrap_or(true);

        // Check that the character before the suffix exists (avoid matching the suffix standalone)
        let before_ok = abs_pos > 0
            && line[..abs_pos]
                .chars()
                .last()
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false);

        if after_ok && before_ok {
            return Some(abs_pos);
        }
        search_start = abs_pos + suffix.len();
    }
    None
}

fn check_commit_patterns(line: &str, lineno: usize, findings: &mut Vec<Finding>) {
    let lower = line.to_lowercase();

    // Vague commit verbs — Low
    let vague = ["update stuff", "fix things", "wip", "misc changes", "minor fixes"];
    for phrase in &vague {
        if lower.contains(phrase) {
            findings.push(Finding {
                line: lineno,
                col: lower.find(phrase).expect("contains() guarantees find() succeeds"),
                matched: phrase.to_string(),
                message: format!("Vague commit message: '{}'", phrase),
                replacement: None,
                severity: Severity::Low,
            });
        }
    }
}

/// Produce a cleaned version of content, removing empty-replacement lines and
/// substituting fixable patterns. This is the main entry point for --fix output.
pub fn clean(content: &str, findings: &[Finding]) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();

    // Which lines should be dropped (replacement == "")
    let mut drop_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // Group fixable (non-drop) findings by line index
    let mut fix_by_line: std::collections::HashMap<usize, Vec<&Finding>> =
        std::collections::HashMap::new();

    for f in findings {
        match f.replacement.as_deref() {
            Some("") => {
                drop_lines.insert(f.line - 1); // 0-based
            }
            Some(_) => {
                fix_by_line.entry(f.line - 1).or_default().push(f);
            }
            None => {}
        }
    }

    // Apply inline replacements
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
                if end <= line.len() {
                    let original = &line[f.col..end];
                    let fixed = apply_case(original, replacement);
                    line = format!("{}{}{}", &line[..f.col], fixed, &line[end..]);
                }
            }
        }
        lines[*idx] = line;
    }

    // Rebuild, skipping dropped lines
    let mut result_lines: Vec<&str> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if !drop_lines.contains(&idx) {
            result_lines.push(line.as_str());
        }
    }

    let joined = result_lines.join("\n");

    // Preserve trailing newline if original had one
    if content.ends_with('\n') {
        format!("{}\n", joined)
    } else {
        joined
    }
}

/// Preserve capitalization style of the original word when applying a replacement.
fn apply_case(original: &str, replacement: &str) -> String {
    if original.is_empty() || replacement.is_empty() {
        return replacement.to_string();
    }
    let first_char = original.chars().next().unwrap();
    if first_char.is_uppercase() {
        let mut chars = replacement.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        replacement.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_utilize() {
        let findings = apply_text_rules("We should utilize this approach.");
        assert!(findings.iter().any(|f| f.matched.to_lowercase() == "utilize"));
    }

    #[test]
    fn finds_sycophantic_opener() {
        let findings = apply_text_rules("Certainly! Here is the answer.");
        assert!(findings.iter().any(|f| f.matched.to_lowercase() == "certainly!"));
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
    fn section_header_detected() {
        let findings = apply_code_rules("# --- Setup ---\nfn main() {}", &[CodeRule::Comments]);
        assert!(findings.iter().any(|f| f.message.contains("Section header")));
    }

    #[test]
    fn bare_todo_detected() {
        let findings = apply_code_rules("# TODO: add error handling", &[CodeRule::Comments]);
        assert!(findings.iter().any(|f| f.message.contains("Bare TODO")));
    }

    #[test]
    fn naming_suffix_detected() {
        let findings = apply_code_rules("let userManager = ...", &[CodeRule::Naming]);
        assert!(findings.iter().any(|f| f.matched == "Manager"));
    }

    #[test]
    fn apply_case_preserves_capital() {
        assert_eq!(apply_case("Utilize", "use"), "Use");
        assert_eq!(apply_case("utilize", "use"), "use");
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
        let f = findings.iter().find(|f| f.matched.to_lowercase() == "certainly!").unwrap();
        assert_eq!(f.severity, Severity::Critical);
    }

    #[test]
    fn severity_high_for_buzzword() {
        let findings = apply_text_rules("We are leveraging new tech.");
        let f = findings.iter().find(|f| f.matched.to_lowercase() == "leveraging").unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn severity_medium_for_filler_connector() {
        let findings = apply_text_rules("Moreover, this is good.");
        let f = findings.iter().find(|f| f.matched.to_lowercase() == "moreover").unwrap();
        assert_eq!(f.severity, Severity::Medium);
    }

    #[test]
    fn severity_low_for_filler_phrase() {
        let findings = apply_text_rules("In order to proceed, do this.");
        let f = findings.iter().find(|f| f.matched.to_lowercase() == "in order to").unwrap();
        assert_eq!(f.severity, Severity::Low);
    }

    #[test]
    fn severity_critical_for_bare_todo() {
        let findings = apply_code_rules("# TODO: fix this", &[CodeRule::Comments]);
        let f = findings.iter().find(|f| f.message.contains("Bare TODO")).unwrap();
        assert_eq!(f.severity, Severity::Critical);
    }

    #[test]
    fn severity_high_for_section_header() {
        let findings = apply_code_rules("# --- Setup ---\nfn main() {}", &[CodeRule::Comments]);
        let f = findings.iter().find(|f| f.message.contains("Section header")).unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn severity_high_for_anemic_suffix() {
        let findings = apply_code_rules("let userManager = ...", &[CodeRule::Naming]);
        let f = findings.iter().find(|f| f.matched == "Manager").unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn severity_medium_for_type_in_name() {
        let findings = apply_code_rules("let userDataObject = ...", &[CodeRule::Naming]);
        let f = findings.iter().find(|f| f.message.contains("Type-in-name")).unwrap();
        assert_eq!(f.severity, Severity::Medium);
    }

    #[test]
    fn code_block_not_flagged() {
        let input = "Some prose.\n```\nutilize this approach.\n```\nEnd.\n";
        let findings = apply_text_rules(input);
        assert!(
            findings.iter().all(|f| f.matched.to_lowercase() != "utilize"),
            "utilize inside fenced block should not be flagged"
        );
    }

    #[test]
    fn url_line_not_flagged() {
        let input = "https://example.com/utilize-this-comprehensive-guide";
        let findings = apply_text_rules(input);
        assert!(findings.is_empty(), "bare URL line should produce no findings");
    }

    #[test]
    fn inline_code_not_flagged() {
        let input = "Call `utilize` to proceed.";
        let findings = apply_text_rules(input);
        assert!(
            findings.iter().all(|f| f.matched.to_lowercase() != "utilize"),
            "utilize inside backtick span should not be flagged"
        );
    }
}

#[cfg(test)]
mod challenge_tests {
    use super::*;

    // --- Word boundary: substrings ---
    #[test] fn pivotale_unchanged() {
        let f = apply_text_rules("C'est une décision pivotale.");
        assert!(f.is_empty(), "pivotale should not be flagged, got: {:?}", f.iter().map(|x|&x.matched).collect::<Vec<_>>());
    }
    #[test] fn delves_unchanged() {
        let f = apply_text_rules("She delves into the topic.");
        assert!(f.is_empty(), "delves should not be flagged, got: {:?}", f.iter().map(|x|&x.matched).collect::<Vec<_>>());
    }
    #[test] fn commencement_unchanged() {
        let input = "The commencement ceremony starts now.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert_eq!(cleaned, input, "commencement should not be mangled");
    }
    #[test] fn utilization_unchanged() {
        let f = apply_text_rules("Memory utilization is 80%.");
        assert!(f.is_empty(), "utilization should not be flagged");
    }
    #[test] fn notably_in_notable_unchanged() {
        let f = apply_text_rules("The notable result stands.");
        assert!(f.is_empty(), "notable should not be flagged");
    }

    // --- Non-English passthrough ---
    #[test] fn spanish_notable_unchanged() {
        let input = "El resultado es notable.";
        let f = apply_text_rules(input);
        assert!(f.is_empty(), "Spanish 'notable' should not be flagged");
    }
    #[test] fn french_passthrough() {
        let input = "Le résultat est remarquable.";
        let f = apply_text_rules(input);
        assert!(f.is_empty());
    }

    // --- Fenced code block with info string ---
    #[test] fn fenced_with_info_string_unchanged() {
        let input = "```python\nutilize this\n```";
        let f = apply_text_rules(input);
        assert!(f.iter().all(|x| x.matched.to_lowercase() != "utilize"),
            "utilize inside ```python block should not be flagged");
    }

    // --- Inline backtick + prose on same line ---
    #[test] fn banned_outside_backtick_fixed() {
        let input = "Use `foo` and utilize bar.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert!(cleaned.contains("use bar"), "prose utilize should be fixed, got: {}", cleaned);
        assert!(cleaned.contains("`foo`"), "backtick span preserved, got: {}", cleaned);
    }

    // --- Case ---
    #[test] fn all_caps_utilize_known_behaviour() {
        // Known limitation: apply_case only uppercases the first character of the
        // replacement. "UTILIZE" -> apply_case("UTILIZE", "use") -> "Use".
        // Full-caps preservation is not implemented.
        let input = "UTILIZE this.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert_eq!(cleaned, "Use this.", "all-caps: first char uppercased, rest from replacement");
    }

    // --- Multiple banned words same line ---
    #[test] fn multiple_banned_words() {
        // "utilize" -> "use", "leveraging" -> "using"
        let input = "utilize and leveraging this.";
        let f = apply_text_rules(input);
        let cleaned = clean(input, &f);
        assert!(cleaned.contains("use") && cleaned.contains("using"), "got: {}", cleaned);
    }

    // --- Empty / whitespace ---
    #[test] fn empty_input() {
        assert!(apply_text_rules("").is_empty());
    }
}
