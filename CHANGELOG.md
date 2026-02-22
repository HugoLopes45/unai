# Changelog

All changes to this project are documented here.

## v0.3.2 — 2026-02-22

### Added
- `prompts/agents.md` — plain-Markdown prompt (no frontmatter) for any tool that reads AGENTS.md-style files: opencode, Codex CLI, Amp, Zed, Cline, Amazon Q, Continue.dev, Windsurf, GitHub Copilot, and Aider. Replaces `prompts/opencode.md`.
- Makefile: `install-opencode`, `install-codex`, `install-windsurf`, `install-zed`, `install-copilot`, `install-cline`, `install-amp`, `install-amazonq`, `install-continue`, `install-aider`, `install-all` targets.
- `.gitignore`: entries for all project-local install outputs (`.rules`, `.clinerules`, `AGENTS.md`, `CONVENTIONS.md`, `.amazonq/`, `.continue/`).

### Changed
- `agents.md` synced with `claude-code.md`: getter-prefix rule added, "What not to touch" section added, five rule clarifications applied.
- README install section restructured: Universal AGENTS.md table, Aider section, `install-all` convenience target.

## v0.3.1 — 2026-02-22

### Fixed
- User rule `col` offsets now point into the original line, not the lowercased copy — fixes incorrect `column` in JSON output and potential panics in `clean()` for non-ASCII input
- Connector density check now uses word-boundary matching, preventing substring false positives (e.g. "consequently" inside "inconsequentially")
- File inputs now enforce the same 64 MiB size limit as stdin
- Integration test for `COMMIT_EDITMSG` detection uses an isolated temp directory, eliminating races in parallel test runs
- `CWD_LOCK` mutex is now shared across both cwd-mutation tests so they correctly serialize

### Performance
- `lower_byte_to_char` offset mapping changed from O(n) linear scan to O(log n) binary search

## v0.2.0 — Research-grounded redesign (2026-02-20)

### Changed
- Word list rebuilt from Kobak et al. (Science Advances 2025) empirical corpus data with severity calibrated by measured frequency ratios
- Every TEXT_RULE now carries a `// source:` annotation referencing the paper it came from
- `delves`, `showcasing`, `underscores` elevated to Critical (r > 10× baseline)
- Added: `meticulous`, `meticulously`, `intricate`, `realm`, `showcasing`, `boast`, `enhancing`, `exhibited`, `insights`, `particularly` with empirical backing
- Strengthened sycophantic opener detection: `happy to help`, `happy to explain`, `I'd be happy to`
- Added structural rules: connector density per paragraph, sentence length uniformity
- Overhauled commit rules: past-tense subject line (High), vague scope words (High), title-case subject (Medium), multiline body on single-purpose fix (Low)
- Added `--diff` flag for unified diff output
- Mode::CommitMsg now correctly routes through both text rules and commit rules

### Fixed
- `COMMIT_EDITMSG` was silently routed to `Mode::Text` — commit rules never fired from git hooks
- Removed `is_tautological_comment` which had an unacceptable false-positive rate

### Framing
- Tool is now positioned as a style linter, not a detector
- README updated with research basis and paper citations
- Detection framing removed per Sadasivan et al. (TMLR 2023) impossibility results

## v0.1.0 — 2026-02-20

Initial release.

- CLI: `unai` reads from stdin, flags LLM-isms, exits non-zero on matches
- `--fix` flag applies replacements in place
- `--only <category>` filters to a single rule category
- `--json` outputs structured diagnostics for editor integrations
- Rule categories: text, comments, naming, commits, docstrings, tests, errors, api
- Claude Code skill via `make install-skill`
- Cursor rule via `make install-cursor`
- OpenCode system prompt at `prompts/opencode.md`
- Universal system prompt at `prompts/system.md`
