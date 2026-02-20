mod detector;
mod diff;
mod rules;

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use clap::{Parser, ValueEnum};

use detector::{detect_mode, is_commit_msg_file, Mode};
use rules::{
    apply_code_rules, apply_structural_rules, apply_text_rules, clean, CodeRule, Finding, Severity,
};

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

    /// Show a unified diff of changes instead of applying them.
    #[arg(long)]
    diff: bool,

    /// Show inline annotations of what was changed.
    #[arg(long)]
    annotate: bool,

    /// Print a summary of patterns found, grouped by severity.
    #[arg(long)]
    report: bool,

    /// Only show findings at or above this severity level.
    #[arg(long, value_enum, default_value = "low")]
    min_severity: MinSeverityArg,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum ModeArg {
    Auto,
    Text,
    Code,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum MinSeverityArg {
    Critical,
    High,
    Medium,
    Low,
}

impl MinSeverityArg {
    fn as_severity(&self) -> Severity {
        match self {
            Self::Critical => Severity::Critical,
            Self::High => Severity::High,
            Self::Medium => Severity::Medium,
            Self::Low => Severity::Low,
        }
    }
}

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

    let all_findings = gather_findings(&content, &mode, &code_rules, filename.as_deref());

    let min_rank = args.min_severity.as_severity().rank();
    let findings: Vec<Finding> = all_findings
        .into_iter()
        .filter(|f| f.severity.rank() >= min_rank)
        .collect();

    if findings.is_empty() && !args.report {
        // Nothing to do — emit input unchanged.
        print_output(&content);
        return Ok(());
    }

    if args.report {
        print_report(&findings, &mode);
    }

    if args.diff {
        let cleaned = clean(&content, &findings);
        let diff_output = diff::unified_diff(&content, &cleaned, "original", "cleaned");
        if diff_output.is_empty() {
            let fixable = findings.iter().filter(|f| f.replacement.is_some()).count();
            if findings.is_empty() {
                eprintln!("unai: no findings");
            } else if fixable == 0 {
                // Findings exist but none have auto-fixes; diff would always be empty.
                eprintln!(
                    "unai: {} finding(s), none auto-fixable (run --report to see them)",
                    findings.len()
                );
            } else {
                eprintln!("unai: no changes");
            }
        } else {
            print!("{}", diff_output);
        }
        return Ok(());
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

fn read_input(file_arg: &Option<String>) -> Result<(String, Option<String>), String> {
    match file_arg {
        Some(path) => {
            let content =
                fs::read_to_string(path).map_err(|e| format!("cannot read '{}': {}", path, e))?;
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

fn resolve_mode(mode_arg: &ModeArg, filename: Option<&str>, content: &str) -> Mode {
    match mode_arg {
        ModeArg::Text => Mode::Text,
        ModeArg::Code => Mode::Code,
        ModeArg::Auto => detect_mode(filename, content),
    }
}

fn parse_code_rules(raw: &[String]) -> Result<Vec<CodeRule>, String> {
    raw.iter().map(|s| s.parse::<CodeRule>()).collect()
}

fn gather_findings(
    content: &str,
    mode: &Mode,
    code_rules: &[CodeRule],
    filename: Option<&str>,
) -> Vec<Finding> {
    match mode {
        Mode::Text => {
            let mut findings = apply_text_rules(content);
            findings.extend(apply_structural_rules(content));
            findings
        }
        Mode::CommitMsg => {
            let mut findings = apply_text_rules(content);
            findings.extend(apply_code_rules(content, &[CodeRule::Commits]));
            findings.extend(apply_structural_rules(content));
            findings
        }
        Mode::Code => {
            let mut findings = apply_code_rules(content, code_rules);
            // Ensure commit rules always fire for commit message files, even when
            // the caller restricted active rules — but avoid duplicating if already included.
            if filename.map(is_commit_msg_file).unwrap_or(false)
                && !code_rules.is_empty()
                && !code_rules.contains(&CodeRule::Commits)
            {
                findings.extend(apply_code_rules(content, &[CodeRule::Commits]));
            }
            findings
        }
    }
}

fn print_output(text: &str) {
    // Avoid double newline: `print!` is sufficient since clean() preserves
    // the trailing newline from the original content.
    print!("{}", text);
}

fn print_dry_run(content: &str, findings: &[Finding]) {
    let fixable: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.replacement.is_some())
        .collect();
    let unfixable: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.replacement.is_none())
        .collect();

    if !fixable.is_empty() {
        eprintln!("--- Auto-fixable ({}) ---", fixable.len());
        for f in &fixable {
            let repl = f.replacement.as_deref().unwrap_or("");
            if repl.is_empty() {
                eprintln!(
                    "  line {:>4}: [remove] {:?}  — {}",
                    f.line, f.matched, f.message
                );
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
            Mode::CommitMsg => "commit",
        },
        findings.len()
    );

    // Group findings by severity in descending order
    let severity_levels: &[(&str, Severity)] = &[
        ("CRITICAL", Severity::Critical),
        ("HIGH", Severity::High),
        ("MEDIUM", Severity::Medium),
        ("LOW", Severity::Low),
    ];

    for (label, sev) in severity_levels {
        let group: Vec<&Finding> = findings.iter().filter(|f| f.severity == *sev).collect();

        if group.is_empty() {
            continue;
        }

        eprintln!("\n{} ({})", label, group.len());
        for f in group {
            eprintln!("  line {}: {} '{}'", f.line, f.message, f.matched);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mode_explicit_text() {
        assert_eq!(
            resolve_mode(&ModeArg::Text, None, "fn main() {}"),
            Mode::Text
        );
    }

    #[test]
    fn resolve_mode_explicit_code() {
        assert_eq!(
            resolve_mode(&ModeArg::Code, None, "hello world"),
            Mode::Code
        );
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
        assert!(
            !cleaned.contains("facilitate"),
            "facilitate should be replaced"
        );
        assert!(cleaned.ends_with('\n'));
    }

    #[test]
    fn gather_findings_commit_msg_fires_commit_rules() {
        let findings = gather_findings("wip", &Mode::CommitMsg, &[], None);
        assert!(
            findings.iter().any(|f| f.message.contains("Vague commit")),
            "commit rules should fire for CommitMsg mode"
        );
    }

    #[test]
    fn gather_findings_commit_msg_fires_both_text_and_commit_rules() {
        // Both a text tell ("utilize") and a commit tell (past tense "Added") must fire
        let findings =
            gather_findings("Added utilize to the codebase", &Mode::CommitMsg, &[], None);
        assert!(
            findings
                .iter()
                .any(|f| f.matched.to_lowercase().contains("utilize")),
            "text rules should fire in CommitMsg mode"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("imperative mood")),
            "commit past-tense rule should fire in CommitMsg mode"
        );
    }

    #[test]
    fn min_severity_filters_low() {
        let findings = apply_text_rules("Certainly! In order to proceed.");
        let min_rank = Severity::High.rank();
        let filtered: Vec<_> = findings
            .into_iter()
            .filter(|f| f.severity.rank() >= min_rank)
            .collect();
        // "Certainly!" is Critical (rank 3 >= 2), "in order to" is Low (rank 0 < 2)
        assert!(filtered
            .iter()
            .any(|f| f.matched.to_lowercase() == "certainly!"));
        assert!(!filtered
            .iter()
            .any(|f| f.matched.to_lowercase() == "in order to"));
    }

    #[test]
    fn min_severity_arg_converts_correctly() {
        assert_eq!(MinSeverityArg::Critical.as_severity().rank(), 3);
        assert_eq!(MinSeverityArg::High.as_severity().rank(), 2);
        assert_eq!(MinSeverityArg::Medium.as_severity().rank(), 1);
        assert_eq!(MinSeverityArg::Low.as_severity().rank(), 0);
    }
}
