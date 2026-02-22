use super::commit::check_commit_patterns;
use super::{Finding, Severity};

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

impl std::str::FromStr for CodeRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "comments" => Ok(Self::Comments),
            "naming" => Ok(Self::Naming),
            "commits" => Ok(Self::Commits),
            "docstrings" => Ok(Self::Docstrings),
            "tests" => Ok(Self::Tests),
            "errors" => Ok(Self::Errors),
            "api" => Ok(Self::Api),
            _ => Err(format!(
                "unknown rule '{}'. Valid: comments, naming, commits, docstrings, tests, errors, api",
                s
            )),
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

        if all || enabled.contains(&CodeRule::Comments) {
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

        if all || enabled.contains(&CodeRule::Docstrings) {
            let docstring_phrases = [
                "this function serves as",
                "this class represents",
                "this method handles",
                "this module provides",
            ];
            for phrase in &docstring_phrases {
                if let Some(col) = line_lower.find(phrase) {
                    findings.push(Finding {
                        line: lineno,
                        col,
                        matched: phrase.to_string(),
                        message: format!("LLM docstring boilerplate: '{phrase}'"),
                        replacement: None,
                        severity: Severity::High,
                    });
                }
            }
        }

        if all || enabled.contains(&CodeRule::Naming) {
            check_naming(line, lineno, &mut findings);
        }

        if all || enabled.contains(&CodeRule::Commits) {
            check_commit_patterns(trimmed, lineno, &mut findings);
        }
    }

    findings
}

fn is_section_header(line: &str) -> bool {
    if !line.starts_with('#') && !line.starts_with("//") && !line.starts_with("--") {
        return false;
    }

    let after_marker = line
        .trim_start_matches('#')
        .trim_start_matches('/')
        .trim_start();

    if after_marker
        .chars()
        .all(|c| c == '-' || c == '=' || c == ' ')
        && after_marker.len() >= 3
    {
        return true;
    }

    if after_marker.starts_with("---") || after_marker.starts_with("===") {
        return true;
    }
    if after_marker.ends_with("---") || after_marker.ends_with("===") {
        return true;
    }

    let words: Vec<&str> = after_marker.split_whitespace().collect();
    if !words.is_empty()
        && words.iter().all(|w| {
            let trimmed = w.trim_end_matches(':');
            trimmed.len() > 1 && trimmed.chars().all(|c| c.is_uppercase() || c == '_')
        })
    {
        return true;
    }

    false
}

fn is_bare_todo(line: &str) -> bool {
    let todo_prefixes = ["# todo:", "// todo:", "-- todo:", "/* todo:"];
    let lower = line.to_lowercase();
    for prefix in &todo_prefixes {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let rest = rest.trim();
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
    let suffixes = ["Manager", "Handler", "Helper", "Util", "Utility", "Service"];
    for suffix in &suffixes {
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

    let redundant = [
        ("userDataObject", "user"),
        ("configurationSettings", "config"),
        ("errorMessageString", "message"),
        ("listOfUsers", "users"),
    ];
    let line_lower = line.to_lowercase();
    for (bad, suggestion) in &redundant {
        if let Some(col) = line_lower.find(&bad.to_lowercase()) {
            findings.push(Finding {
                line: lineno,
                col,
                matched: bad.to_string(),
                message: format!("Type-in-name anti-pattern: use '{}' instead", suggestion),
                replacement: None,
                severity: Severity::Medium,
            });
        }
    }
}

fn find_suffix_token(line: &str, suffix: &str) -> Option<usize> {
    let mut search_start = 0;
    while search_start < line.len() {
        let slice = &line[search_start..];
        let pos = slice.find(suffix)?;
        let abs_pos = search_start + pos;
        let end = abs_pos + suffix.len();

        let after_ok = line[end..]
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric() && c != '_')
            .unwrap_or(true);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_header_detected() {
        let findings = apply_code_rules("# --- Setup ---\nfn main() {}", &[CodeRule::Comments]);
        assert!(findings
            .iter()
            .any(|f| f.message.contains("Section header")));
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
    fn severity_critical_for_bare_todo() {
        let findings = apply_code_rules("# TODO: fix this", &[CodeRule::Comments]);
        let f = findings
            .iter()
            .find(|f| f.message.contains("Bare TODO"))
            .unwrap();
        assert_eq!(f.severity, Severity::Critical);
    }

    #[test]
    fn severity_high_for_section_header() {
        let findings = apply_code_rules("# --- Setup ---\nfn main() {}", &[CodeRule::Comments]);
        let f = findings
            .iter()
            .find(|f| f.message.contains("Section header"))
            .unwrap();
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
        let f = findings
            .iter()
            .find(|f| f.message.contains("Type-in-name"))
            .unwrap();
        assert_eq!(f.severity, Severity::Medium);
    }
}
