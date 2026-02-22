/// Mode of content being processed.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Text,
    Code,
    CommitMsg,
}

const CODE_EXTENSIONS: &[&str] = &[
    "py", "ts", "tsx", "js", "jsx", "rs", "go", "java", "kt", "swift", "c", "cpp", "h", "hpp",
    "cs", "rb", "php", "sh", "bash", "zsh", "fish", "lua", "r", "scala", "hs", "ml", "ex", "exs",
    "clj", "cljs", "dart", "nim", "zig",
];

const CODE_CONTENT_SIGNALS: &[&str] = &[
    "def ",
    "fn ",
    "func ",
    "function ",
    "class ",
    "import ",
    "from ",
    "use ",
    "mod ",
    "const ",
    "let ",
    "var ",
    "type ",
    "interface ",
    "struct ",
    "enum ",
    "impl ",
    "pub fn",
    "async fn",
    "pub struct",
    "pub enum",
    "pub trait",
    "#include",
    "package ",
    "namespace ",
];

pub fn detect_mode(filename: Option<&str>, content: &str) -> Mode {
    if let Some(name) = filename {
        if is_commit_msg_file(name) {
            return Mode::CommitMsg;
        }
        if let Some(ext) = extension_of(name) {
            if CODE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                return Mode::Code;
            }
        }
    }

    detect_from_content(content)
}

pub fn is_commit_msg_file(filename: &str) -> bool {
    let base = std::path::Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename);
    base == "COMMIT_EDITMSG" || base == "MERGE_MSG" || base == "SQUASH_MSG"
}

fn extension_of(filename: &str) -> Option<&str> {
    std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
}

fn detect_from_content(content: &str) -> Mode {
    // Sample the first 50 non-empty lines for efficiency on large files.
    let sample: String = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(50)
        .collect::<Vec<_>>()
        .join("\n");

    let signal_count = CODE_CONTENT_SIGNALS
        .iter()
        .filter(|&&sig| sample.contains(sig))
        .count();

    if signal_count >= 2 {
        Mode::Code
    } else {
        Mode::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_code_by_extension() {
        assert_eq!(detect_mode(Some("main.rs"), "hello world"), Mode::Code);
        assert_eq!(detect_mode(Some("script.py"), "hello world"), Mode::Code);
        assert_eq!(detect_mode(Some("app.ts"), "hello world"), Mode::Code);
    }

    #[test]
    fn detects_text_by_extension_absence() {
        assert_eq!(detect_mode(Some("README.md"), "hello world"), Mode::Text);
        assert_eq!(detect_mode(Some("notes.txt"), "hello world"), Mode::Text);
    }

    #[test]
    fn detects_code_by_content_signals() {
        let python = "def foo():\n    import os\n    return True";
        assert_eq!(detect_mode(None, python), Mode::Code);
    }

    #[test]
    fn detects_text_without_signals() {
        let prose = "This is a blog post about dogs. Dogs are great.";
        assert_eq!(detect_mode(None, prose), Mode::Text);
    }

    #[test]
    fn commit_msg_file_is_commit_msg() {
        assert_eq!(
            detect_mode(Some("COMMIT_EDITMSG"), "feat: add thing"),
            Mode::CommitMsg
        );
        assert_eq!(
            detect_mode(Some("MERGE_MSG"), "Merge branch foo"),
            Mode::CommitMsg
        );
    }

    #[test]
    fn never_infer_commit_mode_from_content() {
        // This was the bug: short first line triggered commit mode
        assert_eq!(detect_mode(None, "Of course!"), Mode::Text);
        assert_eq!(detect_mode(None, "Of course! Let me help you."), Mode::Text);
        assert_eq!(detect_mode(None, "wip"), Mode::Text);
    }

    #[test]
    fn commit_mode_only_by_filename() {
        assert_eq!(
            detect_mode(Some("COMMIT_EDITMSG"), "anything"),
            Mode::CommitMsg
        );
        assert_eq!(detect_mode(Some("MERGE_MSG"), "anything"), Mode::CommitMsg);
        assert_eq!(detect_mode(Some("SQUASH_MSG"), "anything"), Mode::CommitMsg);
    }
}
