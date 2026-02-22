use similar::TextDiff;

/// Line-level unified diff between `original` and `modified` content.
///
/// Produces standard unified diff format (3 lines of context).
/// Returns empty string if there are no differences.
pub fn unified_diff(original: &str, modified: &str, orig_name: &str, mod_name: &str) -> String {
    let diff = TextDiff::from_lines(original, modified);
    let udiff = diff
        .unified_diff()
        .context_radius(3)
        .header(orig_name, mod_name)
        .to_string();

    if udiff.is_empty() {
        return String::new();
    }

    udiff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_no_change() {
        let content = "line one\nline two\nline three\n";
        assert_eq!(unified_diff(content, content, "original", "cleaned"), "");
    }

    #[test]
    fn diff_simple_replacement() {
        let orig = "We should utilize this approach.\n";
        let modified = "We should use this approach.\n";
        let diff = unified_diff(orig, modified, "original", "cleaned");
        assert!(!diff.is_empty(), "diff should not be empty");
        assert!(
            diff.contains("-We should utilize"),
            "should show removed line"
        );
        assert!(diff.contains("+We should use"), "should show added line");
        assert!(diff.contains("--- original"), "should have orig header");
        assert!(diff.contains("+++ cleaned"), "should have mod header");
    }

    #[test]
    fn diff_preserves_context() {
        let orig = "line 1\nline 2\nline 3\nWe should utilize this.\nline 5\nline 6\nline 7\n";
        let modified = "line 1\nline 2\nline 3\nWe should use this.\nline 5\nline 6\nline 7\n";
        let diff = unified_diff(orig, modified, "original", "cleaned");
        // Should show context lines around the change
        assert!(
            diff.contains(" line 1") || diff.contains(" line 3"),
            "should show context lines"
        );
    }

    #[test]
    fn diff_empty_inputs() {
        assert_eq!(unified_diff("", "", "a", "b"), "");
    }
}
