mod config;
mod detector;
mod diff;
mod error;
mod rules;

use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;
use std::process;

use anstyle::{AnsiColor, Style};
use clap::{Parser, ValueEnum};

use detector::{detect_mode, is_commit_msg_file, Mode};
use error::{exit_code, Result, UnaiError};
use rules::{
    apply_code_rules, apply_structural_rules, apply_text_rules, apply_user_rules, clean,
    collect_ignored_lines, CodeRule, Finding, Severity,
};

/// Maximum bytes accepted from stdin. Inputs larger than this are rejected.
const MAX_STDIN_BYTES: usize = 64 * 1024 * 1024; // 64 MiB

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

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    format: FormatArg,

    /// Write output to a file instead of stdout.
    #[arg(long, value_name = "FILE")]
    output: Option<String>,

    /// Path to config file. Defaults to ./unai.toml if present.
    #[arg(long, value_name = "FILE")]
    config: Option<String>,

    /// Exit with code 10 if any findings exist at or above --min-severity.
    #[arg(long)]
    fail: bool,

    /// Colorize output. Auto-detects TTY when set to 'auto'.
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorArg,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum ModeArg {
    Auto,
    Text,
    Code,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum FormatArg {
    Text,
    Json,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum ColorArg {
    Auto,
    Always,
    Never,
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

#[derive(serde::Serialize)]
struct JsonReport {
    version: &'static str,
    mode: String,
    file: Option<String>,
    findings: Vec<JsonFinding>,
    summary: JsonSummary,
}

#[derive(serde::Serialize)]
struct JsonFinding {
    line: usize,
    column: usize,
    end_column: usize,
    matched: String,
    message: String,
    severity: Severity,
    replacement: Option<String>,
    source: String,
}

#[derive(serde::Serialize)]
struct JsonSummary {
    total: usize,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
}

fn build_json_report(findings: &[Finding], mode: &Mode, filename: Option<&str>) -> JsonReport {
    let json_findings: Vec<JsonFinding> = findings
        .iter()
        .map(|f| JsonFinding {
            line: f.line,
            column: f.col,
            end_column: f.col + f.matched.len(),
            matched: f.matched.clone(),
            message: f.message.clone(),
            severity: f.severity,
            replacement: f.replacement.clone(),
            source: mode_label(mode).to_string(),
        })
        .collect();

    let summary = JsonSummary {
        total: findings.len(),
        critical: findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count(),
        high: findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count(),
        medium: findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .count(),
        low: findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .count(),
    };

    JsonReport {
        version: env!("CARGO_PKG_VERSION"),
        mode: mode_label(mode).to_string(),
        file: filename.map(|s| s.to_string()),
        findings: json_findings,
        summary,
    }
}

fn mode_label(mode: &Mode) -> &'static str {
    match mode {
        Mode::Text => "text",
        Mode::Code => "code",
        Mode::CommitMsg => "commit",
    }
}

fn write_output(content: &str, output_path: Option<&str>) -> Result<()> {
    match output_path {
        Some(path) => {
            // Refuse to write through symlinks to prevent clobbering unintended targets.
            if let Ok(meta) = std::fs::symlink_metadata(path) {
                if meta.file_type().is_symlink() {
                    return Err(UnaiError::FileWrite {
                        path: path.into(),
                        source: std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "output path is a symlink; refusing to follow",
                        ),
                    });
                }
            }
            fs::write(path, content).map_err(|source| UnaiError::FileWrite {
                path: path.into(),
                source,
            })?;
            Ok(())
        }
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}

fn main() {
    let args = Args::parse();
    let fail = args.fail;

    match run(args) {
        Ok(had_findings) => {
            if fail && had_findings {
                process::exit(exit_code::FINDINGS);
            }
        }
        Err(e) => {
            eprintln!("unai: {e}");
            let code = match &e {
                UnaiError::ConfigParse { .. }
                | UnaiError::ConfigInvalid(_)
                | UnaiError::InvalidRule(_) => exit_code::CONFIG_ERROR,
                _ => exit_code::IO_ERROR,
            };
            process::exit(code);
        }
    }
}

fn run(args: Args) -> Result<bool> {
    let cfg = match &args.config {
        Some(path) => Some(config::Config::load(std::path::Path::new(path))?),
        None => config::Config::load_from_cwd()?,
    };

    let (content, filename) = read_input(&args.file)?;

    let mode = resolve_mode(&args.mode, filename.as_deref(), &content);
    let code_rules = parse_code_rules(&args.rules)?;

    let mut all_findings = gather_findings(&content, &mode, &code_rules, filename.as_deref());
    all_findings.extend(apply_user_rules(&content, cfg.as_ref()));

    let ignored_words: std::collections::HashSet<String> = cfg
        .as_ref()
        .map(|c| c.ignore.words.iter().map(|w| w.to_lowercase()).collect())
        .unwrap_or_default();

    let ignored_lines = collect_ignored_lines(&content);
    let min_rank = args.min_severity.as_severity().rank();
    let findings: Vec<Finding> = all_findings
        .into_iter()
        .filter(|f| !ignored_words.contains(&f.matched.to_lowercase()))
        .filter(|f| !ignored_lines.contains(&f.line))
        .filter(|f| f.severity.rank() >= min_rank)
        .collect();

    let use_color = match args.color {
        ColorArg::Always => true,
        ColorArg::Never => false,
        ColorArg::Auto => std::io::stderr().is_terminal(),
    };

    let had_findings = !findings.is_empty();

    if args.format == FormatArg::Json {
        let report = build_json_report(&findings, &mode, filename.as_deref());
        let json = serde_json::to_string_pretty(&report).map_err(|e| UnaiError::FileWrite {
            path: args.output.as_deref().unwrap_or("<stdout>").into(),
            source: std::io::Error::other(e.to_string()),
        })?;
        write_output(&json, args.output.as_deref())?;
        return Ok(had_findings);
    }

    if !had_findings && !args.report {
        // Nothing to do — emit input unchanged.
        write_output(&content, args.output.as_deref())?;
        return Ok(false);
    }

    if args.report {
        print_report(&findings, &mode, use_color);
    }

    if args.diff {
        let cleaned = clean(&content, &findings);
        let diff_output = diff::unified_diff(&content, &cleaned, "original", "cleaned");
        if diff_output.is_empty() {
            let fixable = findings.iter().filter(|f| f.replacement.is_some()).count();
            if !had_findings {
                eprintln!("unai: no findings");
            } else if fixable == 0 {
                eprintln!(
                    "unai: {} finding(s), none auto-fixable (run --report to see them)",
                    findings.len()
                );
            } else {
                eprintln!("unai: no changes");
            }
        } else {
            write_output(&diff_output, args.output.as_deref())?;
            return Ok(had_findings);
        }
        return Ok(had_findings);
    }

    if args.dry_run {
        print_dry_run(&content, &findings);
        return Ok(had_findings);
    }

    if args.annotate {
        print_annotated(&content, &findings);
        return Ok(had_findings);
    }

    // Default: emit cleaned output for piping
    let cleaned = clean(&content, &findings);
    write_output(&cleaned, args.output.as_deref())?;
    Ok(had_findings)
}

fn read_input(file_arg: &Option<String>) -> Result<(String, Option<String>)> {
    match file_arg {
        Some(path) => {
            let meta = fs::metadata(path).map_err(|source| UnaiError::FileRead {
                path: path.into(),
                source,
            })?;
            if meta.len() > MAX_STDIN_BYTES as u64 {
                return Err(UnaiError::FileTooLarge { path: path.into() });
            }
            let content = fs::read_to_string(path).map_err(|source| UnaiError::FileRead {
                path: path.into(),
                source,
            })?;
            let filename = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();
            Ok((content, Some(filename)))
        }
        None => {
            let mut buf = Vec::new();
            io::stdin()
                .take(MAX_STDIN_BYTES as u64 + 1)
                .read_to_end(&mut buf)
                .map_err(|source| UnaiError::StdinRead { source })?;
            if buf.len() > MAX_STDIN_BYTES {
                return Err(UnaiError::StdinTooLarge);
            }
            let content = String::from_utf8(buf).map_err(|_| UnaiError::StdinRead {
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "stdin is not valid UTF-8",
                ),
            })?;
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

fn parse_code_rules(raw: &[String]) -> Result<Vec<CodeRule>> {
    raw.iter()
        .map(|s| s.parse::<CodeRule>().map_err(UnaiError::InvalidRule))
        .collect()
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
            let is_commit_file = filename.map(is_commit_msg_file).unwrap_or(false);
            // When no explicit rules are given ("all"), exclude commit-message rules for
            // non-commit files — they produce false positives on line 1 of arbitrary code.
            let effective_rules: &[CodeRule] = if code_rules.is_empty() && !is_commit_file {
                &[
                    CodeRule::Comments,
                    CodeRule::Naming,
                    CodeRule::Docstrings,
                    CodeRule::Tests,
                    CodeRule::Errors,
                    CodeRule::Api,
                ]
            } else {
                code_rules
            };
            let mut findings = apply_code_rules(content, effective_rules);
            // Ensure commit rules fire for commit message files when the caller restricted
            // rules and did not explicitly include commits.
            if is_commit_file && !code_rules.is_empty() && !code_rules.contains(&CodeRule::Commits)
            {
                findings.extend(apply_code_rules(content, &[CodeRule::Commits]));
            }
            findings
        }
    }
}

fn print_dry_run(content: &str, findings: &[Finding]) {
    let (fixable, unfixable): (Vec<&Finding>, Vec<&Finding>) =
        findings.iter().partition(|f| f.replacement.is_some());

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
    print!("{}", content);
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

const RESET: &str = "\x1b[0m";

fn severity_style(sev: Severity, color: bool) -> String {
    if !color {
        return String::new();
    }
    match sev {
        Severity::Critical => Style::new()
            .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Red)))
            .bold()
            .render()
            .to_string(),
        Severity::High => Style::new()
            .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Yellow)))
            .bold()
            .render()
            .to_string(),
        Severity::Medium => Style::new()
            .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Yellow)))
            .render()
            .to_string(),
        Severity::Low => String::new(),
    }
}

fn print_report(findings: &[Finding], mode: &Mode, color: bool) {
    eprintln!(
        "Mode: {}  |  {} finding(s)",
        mode_label(mode),
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

        let style_start = severity_style(*sev, color);
        let reset = if color { RESET } else { "" };
        eprintln!("\n{}{} ({}){}", style_start, label, group.len(), reset);
        for f in group {
            eprintln!("  line {}: {} '{}'", f.line, f.message, f.matched);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn write_output_refuses_symlink() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        std::fs::write(&target, "original").unwrap();
        let link = dir.path().join("link.txt");
        symlink(&target, &link).unwrap();
        let result = write_output("content", Some(link.to_str().unwrap()));
        assert!(result.is_err(), "should refuse to write through symlink");
        // Original target must be untouched
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "original");
    }

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
