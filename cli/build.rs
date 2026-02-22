fn main() {
    // Register .githooks automatically on every build so contributors
    // don't need to run `git config core.hooksPath .githooks` manually.
    let _ = std::process::Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .status();
}
