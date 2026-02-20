/// End-to-end integration tests that invoke the compiled `unai` binary
/// via `std::process::Command`.
///
/// Run with: `cargo test --manifest-path cli/Cargo.toml --test integration`
use std::io::Write;
use std::process::{Command, Stdio};

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

    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();

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
