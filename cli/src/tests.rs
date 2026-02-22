use super::*;

// RED → GREEN: pipeline() isolated from rendering — verifies findings are returned
// without any output side effects.
#[test]
fn pipeline_returns_findings_without_rendering() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("input.txt");
    std::fs::write(&input_path, "We should utilize this.\n").unwrap();

    let args = Args {
        file: Some(input_path.to_str().unwrap().to_string()),
        mode: ModeArg::Text,
        rules: vec![],
        dry_run: false,
        diff: false,
        annotate: false,
        report: false,
        min_severity: MinSeverityArg::Low,
        format: FormatArg::Text,
        output: None,
        config: None,
        fail: false,
        color: ColorArg::Never,
    };

    let result = pipeline(&args).unwrap();
    assert_eq!(result.mode, Mode::Text);
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.matched.to_lowercase().contains("utilize")),
        "pipeline should surface the 'utilize' finding"
    );
    assert!(
        result.content.contains("utilize"),
        "content is original unmodified input"
    );
    assert!(result.filename.is_some());
}

#[test]
fn pipeline_no_findings_on_clean_input() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("clean.txt");
    std::fs::write(&input_path, "The sky is blue.\n").unwrap();

    let args = Args {
        file: Some(input_path.to_str().unwrap().to_string()),
        mode: ModeArg::Text,
        rules: vec![],
        dry_run: false,
        diff: false,
        annotate: false,
        report: false,
        min_severity: MinSeverityArg::Low,
        format: FormatArg::Text,
        output: None,
        config: None,
        fail: false,
        color: ColorArg::Never,
    };

    let result = pipeline(&args).unwrap();
    assert!(
        result.findings.is_empty(),
        "clean prose should produce no findings"
    );
}

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

fn make_finding(severity: Severity) -> Finding {
    Finding {
        line: 1,
        col: 0,
        matched: "x".to_string(),
        message: "test".to_string(),
        replacement: None,
        severity,
    }
}

#[test]
fn count_by_severity_counts_correctly() {
    let findings = vec![
        make_finding(Severity::Critical),
        make_finding(Severity::Critical),
        make_finding(Severity::High),
    ];
    assert_eq!(count_by_severity(&findings, Severity::Critical), 2);
    assert_eq!(count_by_severity(&findings, Severity::High), 1);
    assert_eq!(count_by_severity(&findings, Severity::Low), 0);
}

// --- Formatter dispatch (OCP) ---

fn make_pipeline_result(content: &str, findings: Vec<Finding>, mode: Mode) -> PipelineResult {
    PipelineResult {
        findings,
        mode,
        content: content.to_string(),
        filename: None,
    }
}

fn default_args(format: FormatArg) -> Args {
    Args {
        file: None,
        mode: ModeArg::Text,
        rules: vec![],
        dry_run: false,
        diff: false,
        annotate: false,
        report: false,
        min_severity: MinSeverityArg::Low,
        format,
        output: None,
        config: None,
        fail: false,
        color: ColorArg::Never,
    }
}

#[test]
fn formatter_json_produces_valid_json() {
    let formatter = Formatter::Json;
    let result = make_pipeline_result("utilize this", vec![make_finding(Severity::High)], Mode::Text);
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.json");
    let args_with_output = Args {
        output: Some(out_path.to_str().unwrap().to_string()),
        ..default_args(FormatArg::Json)
    };

    let had_findings = formatter
        .render(result, &args_with_output)
        .expect("Formatter::Json render should succeed");

    let written = std::fs::read_to_string(&out_path).expect("output file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&written).expect("output must be valid JSON");

    assert!(had_findings);
    assert!(parsed.get("findings").is_some(), "JSON must have 'findings' key");
    assert!(parsed.get("summary").is_some(), "JSON must have 'summary' key");
    assert!(parsed.get("version").is_some(), "JSON must have 'version' key");
}

#[test]
fn formatter_text_passthrough_on_clean_input() {
    let formatter = Formatter::Text;
    let result = make_pipeline_result("hello world\n", vec![], Mode::Text);
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.txt");
    let args_with_output = Args {
        output: Some(out_path.to_str().unwrap().to_string()),
        ..default_args(FormatArg::Text)
    };

    formatter
        .render(result, &args_with_output)
        .expect("Formatter::Text render should succeed");

    let written = std::fs::read_to_string(&out_path).expect("output file should exist");
    assert_eq!(written, "hello world\n", "text formatter should pass through clean content unchanged");
}

#[test]
fn formatter_from_args_selects_json() {
    let args = default_args(FormatArg::Json);
    assert!(
        matches!(Formatter::from_args(&args), Formatter::Json),
        "FormatArg::Json must map to Formatter::Json"
    );
}

#[test]
fn formatter_from_args_selects_text() {
    let args = default_args(FormatArg::Text);
    assert!(
        matches!(Formatter::from_args(&args), Formatter::Text),
        "FormatArg::Text must map to Formatter::Text"
    );
}
