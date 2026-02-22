use std::collections::HashSet;

/// Returns 1-based line numbers that should be skipped due to ignore directives.
///
/// Supported directives:
/// - `<!-- unai-ignore -->` ... `<!-- /unai-ignore -->` (HTML block)
/// - `// unai-ignore-start` / `// unai-ignore-end` (code block, also `#` prefix)
/// - `// unai-ignore-next-line` / `# unai-ignore-next-line` (next line only)
pub fn collect_ignored_lines(content: &str) -> HashSet<usize> {
    let mut ignored = HashSet::new();
    let mut in_html_block = false;
    let mut in_code_block = false;
    let mut skip_next = false;

    for (idx, line) in content.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim();

        if skip_next {
            ignored.insert(lineno);
            skip_next = false;
            continue;
        }

        // HTML block open
        if trimmed == "<!-- unai-ignore -->" {
            in_html_block = true;
            continue;
        }

        // HTML block close
        if trimmed == "<!-- /unai-ignore -->" {
            in_html_block = false;
            continue;
        }

        // Code block start (// or #)
        if trimmed == "// unai-ignore-start" || trimmed == "# unai-ignore-start" {
            in_code_block = true;
            continue;
        }

        // Code block end (// or #)
        if trimmed == "// unai-ignore-end" || trimmed == "# unai-ignore-end" {
            in_code_block = false;
            continue;
        }

        // Next-line directive (// or #)
        if trimmed == "// unai-ignore-next-line" || trimmed == "# unai-ignore-next-line" {
            skip_next = true;
            continue;
        }

        if in_html_block || in_code_block {
            ignored.insert(lineno);
        }
    }

    ignored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_block_ignore() {
        let content =
            "line 1\n<!-- unai-ignore -->\nline 3\nline 4\n<!-- /unai-ignore -->\nline 6\n";
        let ignored = collect_ignored_lines(content);
        assert!(!ignored.contains(&1));
        assert!(ignored.contains(&3));
        assert!(ignored.contains(&4));
        assert!(!ignored.contains(&6));
    }

    #[test]
    fn next_line_ignore() {
        let content = "line 1\n# unai-ignore-next-line\nline 3\nline 4\n";
        let ignored = collect_ignored_lines(content);
        assert!(!ignored.contains(&1));
        assert!(!ignored.contains(&2));
        assert!(ignored.contains(&3));
        assert!(!ignored.contains(&4));
    }

    #[test]
    fn start_end_block() {
        let content = "line 1\n// unai-ignore-start\nline 3\nline 4\n// unai-ignore-end\nline 6\n";
        let ignored = collect_ignored_lines(content);
        assert!(!ignored.contains(&1));
        assert!(ignored.contains(&3));
        assert!(ignored.contains(&4));
        assert!(!ignored.contains(&6));
    }

    #[test]
    fn hash_start_end_block() {
        let content = "line 1\n# unai-ignore-start\nline 3\nline 4\n# unai-ignore-end\nline 6\n";
        let ignored = collect_ignored_lines(content);
        assert!(!ignored.contains(&1));
        assert!(ignored.contains(&3));
        assert!(ignored.contains(&4));
        assert!(!ignored.contains(&6));
    }

    #[test]
    fn slash_next_line_ignore() {
        let content = "line 1\n// unai-ignore-next-line\nline 3\nline 4\n";
        let ignored = collect_ignored_lines(content);
        assert!(!ignored.contains(&1));
        assert!(!ignored.contains(&2));
        assert!(ignored.contains(&3));
        assert!(!ignored.contains(&4));
    }

    #[test]
    fn empty_content() {
        assert!(collect_ignored_lines("").is_empty());
    }

    #[test]
    fn directive_lines_not_ignored() {
        let content = "<!-- unai-ignore -->\nline 2\n<!-- /unai-ignore -->\n";
        let ignored = collect_ignored_lines(content);
        assert!(!ignored.contains(&1));
        assert!(ignored.contains(&2));
        assert!(!ignored.contains(&3));
    }

    #[test]
    fn next_line_at_end_of_file() {
        let content = "line 1\n# unai-ignore-next-line\n";
        let ignored = collect_ignored_lines(content);
        assert!(ignored.is_empty() || !ignored.contains(&1));
    }
}
