mod detector;
mod rules;

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use clap::{Parser, ValueEnum};

use detector::{detect_mode, is_commit_msg_file, Mode};
use rules::{apply_code_rules, apply_text_rules, clean, CodeRule, Finding};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "unai",
    version,
    about = "Remove LLM-isms from text and code",
    long_about = None
)]
struct Args {
    /// Input file. Reads from stdin if omitted.
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Processing mode. Defaults to automatic detection.
    #[arg(long, value_enum, default_value = "auto")]
    mode: ModeArg,

    /// Code rules to apply (comma-separated). Applies all when omitted.
    /// Values: comments, naming, commits, docstrings, tests, errors, api
    #[arg(long, value_delimiter = ',')]
    rules: Vec<String>,

    /// Show what would change without modifying output.
    #[arg(long)]
    dry_run: bool,

    /// Show inline annotations of what was changed.
    #[arg(long)]
    annotate: bool,

    /// Print a summary of patterns found.
    #[arg(long)]
    report: bool,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum ModeArg {
    Auto,
    Text,
    Code,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("unai: {e}");
            process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let (content, filename) = read_input(&args.file)?;

    let mode = resolve_mode(&args.mode, filename.as_deref(), &content);
    let code_rules = parse_code_rules(&args.rules)?;

    let findings = gather_findings(&content, &mode, &code_rules, filename.as_deref());

    if findings.is_empty() && !args.report {
        // Nothing to do — emit input unchanged.
        print_output(&content);
        return Ok(());
    }

    if args.report {
        print_report(&findings, &mode);
    }

    if args.dry_run {
        print_dry_run(&content, &findings);
        return Ok(());
    }

    if args.annotate {
        print_annotated(&content, &findings);
        return Ok(());
    }

    // Default: emit cleaned output for piping
    let cleaned = clean(&content, &findings);
    print_output(&cleaned);

    Ok(())
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

fn read_input(file_arg: &Option<String>) -> Result<(String, Option<String>), String> {
    match file_arg {
        Some(path) => {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("cannot read '{}': {}", path, e))?;
            let filename = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();
            Ok((content, Some(filename)))
        }
        None => {
            let mut content = String::new();
            io::stdin()
                .read_to_string(&mut content)
                .map_err(|e| format!("cannot read stdin: {}", e))?;
            Ok((content, None))
        }
    }
}

// ---------------------------------------------------------------------------
// Mode resolution
// ---------------------------------------------------------------------------

fn resolve_mode(mode_arg: &ModeArg, filename: Option<&str>, content: &str) -> Mode {
    match mode_arg {
        ModeArg::Text => Mode::Text,
        ModeArg::Code => Mode::Code,
        ModeArg::Auto => detect_mode(filename, content),
    }
}

// ---------------------------------------------------------------------------
// Rule parsing
// ---------------------------------------------------------------------------

fn parse_code_rules(raw: &[String]) -> Result<Vec<CodeRule>, String> {
    raw.iter()
        .map(|s| {
            CodeRule::from_str(s.as_str())
                .ok_or_else(|| format!("unknown rule '{}'. Valid: comments, naming, commits, docstrings, tests, errors, api", s))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Finding collection
// ---------------------------------------------------------------------------

fn gather_findings(
    content: &str,
    mode: &Mode,
    code_rules: &[CodeRule],
    filename: Option<&str>,
) -> Vec<Finding> {
    match mode {
        Mode::Text => apply_text_rules(content),
        Mode::Code => {
            let mut findings = apply_code_rules(content, code_rules);
            // Also apply commit-message rules when the file is COMMIT_EDITMSG
            if filename.map(is_commit_msg_file).unwrap_or(false) {
                findings.extend(apply_code_rules(content, &[CodeRule::Commits]));
            }
            findings
        }
    }
}

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------

fn print_output(text: &str) {
    // Avoid double newline: `print!` is sufficient since clean() preserves
    // the trailing newline from the original content.
    print!("{}", text);
}

fn print_dry_run(content: &str, findings: &[Finding]) {
    let fixable: Vec<&Finding> = findings.iter().filter(|f| f.replacement.is_some()).collect();
    let unfixable: Vec<&Finding> = findings.iter().filter(|f| f.replacement.is_none()).collect();

    if !fixable.is_empty() {
        eprintln!("--- Auto-fixable ({}) ---", fixable.len());
        for f in &fixable {
            let repl = f.replacement.as_deref().unwrap_or("");
            if repl.is_empty() {
                eprintln!("  line {:>4}: [remove] {:?}  — {}", f.line, f.matched, f.message);
            } else {
                eprintln!(
                    "  line {:>4}: {:?} → {:?}  — {}",
                    f.line, f.matched, repl, f.message
                );
            }
        }
    }

    if !unfixable.is_empty() {
        eprintln!("--- Flagged (no auto-fix) ({}) ---", unfixable.len());
        for f in &unfixable {
            eprintln!("  line {:>4}: {:?}  — {}", f.line, f.matched, f.message);
        }
    }

    // Emit original content unchanged when dry-running (suitable for piping inspection)
    print_output(content);
}

fn print_annotated(content: &str, findings: &[Finding]) {
    // Group findings by line number for inline display
    let mut by_line: std::collections::HashMap<usize, Vec<&Finding>> =
        std::collections::HashMap::new();

    for f in findings {
        by_line.entry(f.line).or_default().push(f);
    }

    for (idx, line) in content.lines().enumerate() {
        let lineno = idx + 1;
        println!("{}", line);
        if let Some(line_findings) = by_line.get(&lineno) {
            for f in line_findings {
                let arrow = " ".repeat(f.col) + "^";
                let fix_hint = match f.replacement.as_deref() {
                    Some("") => " (remove line)".to_string(),
                    Some(r) => format!(" → \"{}\"", r),
                    None => String::new(),
                };
                eprintln!("  {}{}", arrow, fix_hint);
                eprintln!("  {}", f.message);
            }
        }
    }
}

fn print_report(findings: &[Finding], mode: &Mode) {
    eprintln!(
        "Mode: {}  |  {} finding(s)",
        match mode {
            Mode::Text => "text",
            Mode::Code => "code",
        },
        findings.len()
    );

    let mut counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();

    for f in findings {
        *counts.entry(f.message.as_str()).or_insert(0) += 1;
    }

    let mut sorted: Vec<(&&str, &usize)> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    for (msg, count) in sorted {
        eprintln!("  {:>3}x  {}", count, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mode_explicit_text() {
        assert_eq!(resolve_mode(&ModeArg::Text, None, "fn main() {}"), Mode::Text);
    }

    #[test]
    fn resolve_mode_explicit_code() {
        assert_eq!(resolve_mode(&ModeArg::Code, None, "hello world"), Mode::Code);
    }

    #[test]
    fn resolve_mode_auto_code_by_filename() {
        assert_eq!(
            resolve_mode(&ModeArg::Auto, Some("main.rs"), "hello"),
            Mode::Code
        );
    }

    #[test]
    fn resolve_mode_auto_text_by_content() {
        assert_eq!(
            resolve_mode(&ModeArg::Auto, None, "just prose here, nothing to see"),
            Mode::Text
        );
    }

    #[test]
    fn parse_valid_rules() {
        let rules = parse_code_rules(&["comments".to_string(), "naming".to_string()]).unwrap();
        assert!(rules.contains(&CodeRule::Comments));
        assert!(rules.contains(&CodeRule::Naming));
    }

    #[test]
    fn parse_invalid_rule_errors() {
        let result = parse_code_rules(&["bogus".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn end_to_end_text_clean() {
        let input = "We should utilize this to facilitate growth.\n";
        let findings = apply_text_rules(input);
        let cleaned = clean(input, &findings);
        assert!(!cleaned.contains("utilize"), "utilize should be replaced");
        assert!(!cleaned.contains("facilitate"), "facilitate should be replaced");
        assert!(cleaned.ends_with('\n'));
    }
}
