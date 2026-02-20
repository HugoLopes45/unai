# Changelog

All changes to this project are documented here.

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
