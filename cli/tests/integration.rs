/// End-to-end integration tests that invoke the compiled `unai` binary
/// via `std::process::Command`.
///
/// Run with: `cargo test --manifest-path cli/Cargo.toml --test integration`
use std::io::Write;
use std::process::{Command, Stdio};

fn write_temp_config(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

/// Invoke the unai binary with the given arguments, feeding `stdin` to it.
/// Returns `(stdout, stderr, exit_code)`.
fn run_unai(args: &[&str], stdin: &str) -> (String, String, i32) {
    let binary = env!("CARGO_BIN_EXE_unai");

    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn unai binary");

    // Ignore BrokenPipe: the process may exit before consuming all stdin (e.g. config errors).
    if let Some(mut handle) = child.stdin.take() {
        let _ = handle.write_all(stdin.as_bytes());
    }

    let output = child.wait_with_output().expect("failed to wait on child");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}

/// Pipe text with "utilize" — it has an auto-fix replacement ("use"), so cleaned output omits it.
#[test]
fn pipe_text_replaces_utilize() {
    let input = "We should utilize this approach.\n";
    let (stdout, _stderr, code) = run_unai(&[], input);
    assert_eq!(code, 0, "exit code should be 0");
    assert!(
        !stdout.contains("utilize"),
        "'utilize' should be replaced in output, got: {:?}",
        stdout
    );
    assert!(
        stdout.contains("use"),
        "replacement 'use' should appear in output, got: {:?}",
        stdout
    );
}

/// --report mode shows CRITICAL in stderr output.
#[test]
fn report_mode_shows_severity() {
    let input = "Certainly! Let me delve into that.\n";
    let (_stdout, stderr, _code) = run_unai(&["--report"], input);
    assert!(
        stderr.contains("CRITICAL"),
        "--report should show CRITICAL severity, got: {:?}",
        stderr
    );
}

/// --diff mode produces unified diff output starting with "---".
#[test]
fn diff_mode_unified_format() {
    let input = "Certainly! I would be happy to utilize this.\n";
    let (stdout, _stderr, _code) = run_unai(&["--diff"], input);
    assert!(
        stdout.starts_with("---"),
        "--diff output should start with ---, got: {:?}",
        stdout
    );
}

/// --diff mode with only non-fixable findings emits a message on stderr.
#[test]
fn diff_mode_no_fixable_findings_message() {
    // "meticulous" is High but has no auto-fix replacement.
    let input = "The meticulous review was completed.\n";
    let (_stdout, stderr, _code) = run_unai(&["--diff"], input);
    assert!(
        stderr.contains("none auto-fixable"),
        "should report none auto-fixable when no replacements exist, got: {:?}",
        stderr
    );
}

/// --mode code applies code rules — naming suffix "Manager" is flagged.
#[test]
fn mode_code_applies_naming_rules() {
    let input = "let userManager = new UserManager();\n";
    let (_stdout, stderr, _code) = run_unai(&["--mode", "code", "--report"], input);
    assert!(
        stderr.to_lowercase().contains("manager") || stderr.to_lowercase().contains("anemic"),
        "--mode code should flag naming suffix, got: {:?}",
        stderr
    );
}

/// --min-severity high filters out Low-severity findings.
#[test]
fn min_severity_high_filters_low() {
    // "in order to" is Low severity; "Certainly!" is Critical.
    let input = "Certainly! In order to proceed.\n";
    let (_stdout, stderr, _code) = run_unai(&["--report", "--min-severity", "high"], input);
    assert!(
        stderr.contains("CRITICAL"),
        "should still show CRITICAL findings, got: {:?}",
        stderr
    );
    assert!(
        !stderr.contains("in order to"),
        "Low severity 'in order to' should be filtered, got: {:?}",
        stderr
    );
}

/// --fail exits with code 10 when findings exist.
#[test]
fn fail_flag_exits_10_with_findings() {
    let input = "Certainly! Let me delve into this.\n";
    let (_stdout, _stderr, code) = run_unai(&["--fail", "--report"], input);
    assert_eq!(
        code, 10,
        "--fail should exit 10 when findings exist, got: {}",
        code
    );
}

/// --fail exits 0 when no findings.
#[test]
fn fail_flag_exits_0_without_findings() {
    let input = "The cat sat on the mat.\n";
    let (_stdout, _stderr, code) = run_unai(&["--fail"], input);
    assert_eq!(
        code, 0,
        "--fail should exit 0 when no findings, got: {}",
        code
    );
}

/// --format json outputs valid JSON with expected fields.
#[test]
fn format_json_valid_output() {
    let input = "Certainly! We should utilize this.\n";
    let (stdout, _stderr, code) = run_unai(&["--format", "json"], input);
    assert_eq!(code, 0);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--format json should output valid JSON");
    assert!(
        parsed.get("findings").is_some(),
        "JSON must have 'findings' key"
    );
    assert!(
        parsed.get("summary").is_some(),
        "JSON must have 'summary' key"
    );
    assert!(
        parsed.get("version").is_some(),
        "JSON must have 'version' key"
    );
}

/// --format json summary counts are correct.
#[test]
fn format_json_summary_counts() {
    let input = "Certainly!\n";
    let (stdout, _stderr, _code) = run_unai(&["--format", "json"], input);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let total = parsed["summary"]["total"].as_u64().unwrap_or(0);
    assert!(total > 0, "summary.total should be > 0 for 'Certainly!'");
    let critical = parsed["summary"]["critical"].as_u64().unwrap_or(0);
    assert!(
        critical > 0,
        "summary.critical should be > 0 for 'Certainly!'"
    );
}

/// Inline ignore directive suppresses findings on ignored lines (T8 strengthened).
#[test]
fn ignore_directive_suppresses_findings() {
    // First verify the finding WOULD fire without directive
    let plain_input = "Certainly! Let me delve.\n";
    let (_stdout, stderr_plain, _code) = run_unai(&["--report"], plain_input);
    assert!(
        stderr_plain.contains("CRITICAL"),
        "baseline: CRITICAL should fire without ignore directive, got: {:?}",
        stderr_plain
    );

    // Now verify the directive suppresses it
    let input = "Good prose here.\n<!-- unai-ignore -->\nCertainly! Let me delve.\n<!-- /unai-ignore -->\nMore good prose.\n";
    let (_stdout, stderr, _code) = run_unai(&["--report"], input);
    assert!(
        !stderr.contains("CRITICAL"),
        "CRITICAL findings on ignored lines should be suppressed, got: {:?}",
        stderr
    );
    assert!(
        stderr.contains("finding"),
        "report header should still appear even with zero findings, got: {:?}",
        stderr
    );
}

/// --color never produces no ANSI escape sequences in report.
#[test]
fn color_never_no_ansi_in_report() {
    let input = "Certainly!\n";
    let (_stdout, stderr, _code) = run_unai(&["--report", "--color", "never"], input);
    assert!(
        !stderr.contains("\x1b["),
        "--color never should not emit ANSI escapes, got: {:?}",
        stderr
    );
}

/// A file named COMMIT_EDITMSG with past-tense subject fires the commit past-tense rule.
#[test]
fn commit_editmsg_fires_commit_rules() {
    use std::io::Write as _;
    let dir = std::env::temp_dir();
    let path = dir.join("COMMIT_EDITMSG");
    let mut f = std::fs::File::create(&path).expect("create temp commit file");
    writeln!(f, "Added new feature").expect("write commit msg");
    drop(f);

    let binary = env!("CARGO_BIN_EXE_unai");
    let output = Command::new(binary)
        .args(["--report", path.to_str().unwrap()])
        .output()
        .expect("failed to run unai on COMMIT_EDITMSG");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    std::fs::remove_file(&path).ok();

    assert!(
        stderr.contains("imperative") || stderr.contains("Past tense"),
        "COMMIT_EDITMSG should fire commit past-tense rule, got: {:?}",
        stderr
    );
}

// ===== T1: enabled = false rule is skipped =====
#[test]
fn user_rule_disabled_is_skipped() {
    let toml = r#"version = 1
[[rules]]
pattern = "synergize"
severity = "critical"
enabled = false
"#;
    let cfg = write_temp_config(toml);
    let (_stdout, stderr, code) = run_unai(
        &["--report", "--config", cfg.path().to_str().unwrap()],
        "We should synergize our efforts.\n",
    );
    assert_eq!(code, 0);
    assert!(
        !stderr.to_lowercase().contains("synergize"),
        "disabled rule should produce no finding, got: {:?}",
        stderr
    );
}

// ===== T2: ignore.words end-to-end =====
#[test]
fn ignore_words_suppresses_findings() {
    let toml = r#"version = 1
[ignore]
words = ["certainly!"]
"#;
    let cfg = write_temp_config(toml);
    let (_stdout, stderr, _code) = run_unai(
        &["--report", "--config", cfg.path().to_str().unwrap()],
        "Certainly!\n",
    );
    assert!(
        !stderr.contains("CRITICAL"),
        "ignored word should suppress CRITICAL finding, got: {:?}",
        stderr
    );
}

// ===== T3: --fail + --min-severity high exits 0 for low-only findings =====
#[test]
fn fail_with_min_severity_high_exits_0_for_low_only() {
    // "moreover" and "furthermore" are Low severity
    let input = "Moreover, furthermore.\n";
    let (_stdout, _stderr, code) = run_unai(&["--fail", "--min-severity", "high"], input);
    assert_eq!(
        code, 0,
        "--fail --min-severity high should exit 0 when only Low findings exist, got: {}",
        code
    );
}

// ===== T4: Config error exits code 2 =====
#[test]
fn invalid_config_exits_2() {
    let toml = "version = 99\n";
    let cfg = write_temp_config(toml);
    let (_stdout, _stderr, code) =
        run_unai(&["--config", cfg.path().to_str().unwrap()], "some input\n");
    assert_eq!(
        code, 2,
        "invalid config should exit with code 2, got: {}",
        code
    );
}

// ===== T5: Non-commit file with --mode code does NOT fire commit rules =====
#[test]
fn code_mode_non_commit_file_no_commit_rules() {
    // "Added feature description" would trigger imperative-mood rule in commit mode
    let input = "Added feature description\n";
    let (_stdout, stderr, _code) = run_unai(&["--mode", "code", "--report"], input);
    assert!(
        !stderr.contains("imperative mood"),
        "code mode on non-commit file should not fire commit rules, got: {:?}",
        stderr
    );
}

// ===== T6: --color always emits ANSI escapes =====
#[test]
fn color_always_emits_ansi_in_report() {
    let input = "Certainly!\n";
    let (_stdout, stderr, _code) = run_unai(&["--report", "--color", "always"], input);
    assert!(
        stderr.contains("\x1b["),
        "--color always should emit ANSI escapes, got: {:?}",
        stderr
    );
}

// ===== T7: --format json + --fail exits 10 with valid JSON =====
#[test]
fn format_json_fail_exits_10_with_findings() {
    let input = "Certainly!\n";
    let (stdout, _stderr, code) = run_unai(&["--format", "json", "--fail"], input);
    assert_eq!(
        code, 10,
        "--format json --fail should exit 10 when findings exist, got: {}",
        code
    );
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("--format json should output valid JSON even with --fail");
    assert!(
        parsed.get("findings").is_some(),
        "JSON must have 'findings'"
    );
}
