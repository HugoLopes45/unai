/// Line-level unified diff between `original` and `modified` content.
///
/// Produces standard unified diff format (3 lines of context).
/// Returns empty string if there are no differences.
pub fn unified_diff(original: &str, modified: &str, orig_name: &str, mod_name: &str) -> String {
    let orig_lines: Vec<&str> = original.lines().collect();
    let mod_lines: Vec<&str> = modified.lines().collect();

    if orig_lines == mod_lines {
        return String::new();
    }

    let lcs = compute_lcs(&orig_lines, &mod_lines);
    let hunks = build_hunks(&orig_lines, &mod_lines, &lcs, 3);

    if hunks.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!("--- {}\n", orig_name));
    out.push_str(&format!("+++ {}\n", mod_name));

    for hunk in hunks {
        out.push_str(&hunk);
    }

    out
}

/// Compute the longest common subsequence of lines.
/// Returns a 2D table where lcs[i][j] = length of LCS for orig[..i] and mod[..j].
fn compute_lcs(orig: &[&str], modified: &[&str]) -> Vec<Vec<usize>> {
    let m = orig.len();
    let n = modified.len();
    let mut table = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if orig[i - 1] == modified[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }

    table
}

/// Diff operation for a single line.
#[derive(Debug, PartialEq)]
enum DiffOp {
    Equal,
    Delete,
    Insert,
}

/// Walk the LCS table to produce a sequence of diff operations.
fn diff_ops(orig: &[&str], modified: &[&str], lcs: &[Vec<usize>]) -> Vec<(DiffOp, usize, usize)> {
    let mut ops = Vec::new();
    let mut i = orig.len();
    let mut j = modified.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && orig[i - 1] == modified[j - 1] {
            ops.push((DiffOp::Equal, i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            ops.push((DiffOp::Insert, i, j - 1));
            j -= 1;
        } else {
            ops.push((DiffOp::Delete, i - 1, j));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

/// Build unified diff hunk strings with `context` lines of context.
fn build_hunks(orig: &[&str], modified: &[&str], lcs: &[Vec<usize>], context: usize) -> Vec<String> {
    let ops = diff_ops(orig, modified, lcs);

    // Find ranges of non-Equal ops, expanded by context lines
    let mut change_ranges: Vec<(usize, usize)> = Vec::new(); // (start_op_idx, end_op_idx)

    let mut i = 0;
    while i < ops.len() {
        if ops[i].0 != DiffOp::Equal {
            let start = i;
            while i < ops.len() && ops[i].0 != DiffOp::Equal {
                i += 1;
            }
            change_ranges.push((start, i));
        } else {
            i += 1;
        }
    }

    let mut hunks = Vec::new();

    let mut range_idx = 0;
    while range_idx < change_ranges.len() {
        let (start, end) = change_ranges[range_idx];

        // Expand with context
        let ctx_start = start.saturating_sub(context);
        let mut ctx_end = (end + context).min(ops.len());

        // Merge with next range if it overlaps
        while range_idx + 1 < change_ranges.len() {
            let (next_start, next_end) = change_ranges[range_idx + 1];
            if next_start <= ctx_end + context {
                ctx_end = (next_end + context).min(ops.len());
                range_idx += 1;
            } else {
                break;
            }
        }

        // Build hunk
        let hunk_ops = &ops[ctx_start..ctx_end];

        // Compute orig and mod line ranges for the @@ header
        let orig_start_line = hunk_ops.iter()
            .filter(|(op, _, _)| *op == DiffOp::Equal || *op == DiffOp::Delete)
            .map(|(_, oi, _)| *oi + 1)
            .next()
            .unwrap_or(1);
        let orig_count = hunk_ops.iter()
            .filter(|(op, _, _)| *op == DiffOp::Equal || *op == DiffOp::Delete)
            .count();
        let mod_start_line = hunk_ops.iter()
            .filter(|(op, _, _)| *op == DiffOp::Equal || *op == DiffOp::Insert)
            .map(|(_, _, mi)| *mi + 1)
            .next()
            .unwrap_or(1);
        let mod_count = hunk_ops.iter()
            .filter(|(op, _, _)| *op == DiffOp::Equal || *op == DiffOp::Insert)
            .count();

        let mut hunk = format!(
            "@@ -{},{} +{},{} @@\n",
            orig_start_line, orig_count, mod_start_line, mod_count
        );

        for (op, oi, mi) in hunk_ops {
            match op {
                DiffOp::Equal => hunk.push_str(&format!(" {}\n", orig[*oi])),
                DiffOp::Delete => hunk.push_str(&format!("-{}\n", orig[*oi])),
                DiffOp::Insert => hunk.push_str(&format!("+{}\n", modified[*mi])),
            }
        }

        hunks.push(hunk);
        range_idx += 1;
    }

    hunks
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
        assert!(diff.contains("-We should utilize"), "should show removed line");
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
        assert!(diff.contains(" line 1") || diff.contains(" line 3"), "should show context lines");
    }

    #[test]
    fn diff_empty_inputs() {
        assert_eq!(unified_diff("", "", "a", "b"), "");
    }
}
