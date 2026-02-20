# unai

Strip LLM-isms from text and code.

## Why

LLMs write in a recognizable style. Overconfident. Padded. Full of words that sound
professional but say nothing. "Leveraging", "robust", "seamlessly", "comprehensive" —
no human writes like this unprompted.

`unai` catches those patterns and flags or replaces them. Use it as a CLI filter,
an editor rule, or a system prompt that trains LLMs to stop doing it in the first
place.

## What it catches

| Text patterns | Code patterns |
|---|---|
| Filler openers ("Certainly!", "Of course!") | Comments with "Note:" / "Important:" |
| Confidence hedges ("It's worth noting") | Docstrings padded with "This function..." |
| Vague intensifiers ("robust", "seamless") | `handleError` / `processData` naming |
| Em-dash overuse | Smart/curly quotes in strings |
| Bullet-point everything | Over-documented obvious code |
| Redundant sign-offs ("I hope this helps") | Test names like `testItWorksCorrectly` |
| Passive voice stacking | Magic `// TODO: implement` stubs |
| Fake lists ("First... Second... Finally...") | Commit messages: "Add initial implementation" |

Full list: [rules/](rules/)

## Install

### Claude Code skill

```bash
make install-skill
```

Then use `/unai` in any Claude Code session.

### Cursor rule

```bash
make install-cursor
```

Or copy `prompts/cursor.mdc` to `.cursor/rules/unai.mdc` in your project.

### OpenCode

Add `prompts/opencode.md` as a system prompt in your OpenCode config.

### Any LLM (system prompt)

Copy the contents of `prompts/system.md` into your LLM's system prompt field.
Works with ChatGPT, Gemini, any OpenAI-compatible API.

### CLI

```bash
cargo install --path cli/
```

Requires Rust 1.75+.

## Usage

```bash
# Pipe text through unai
echo "Certainly! Here's a comprehensive solution..." | unai

# Filter a file
unai < response.md

# Fix mode — apply replacements instead of flagging
unai --fix < response.md

# Check a specific category only
unai --only text < response.md
unai --only comments < file.rs

# Output as JSON (for editor integrations)
unai --json < response.md
```

## Rules reference

Rules live in [`rules/`](rules/). Each file is a plain TOML list of patterns.
Categories:

- `rules/text.toml` — prose LLM-isms
- `rules/comments.toml` — code comment patterns
- `rules/naming.toml` — identifier naming patterns
- `rules/commits.toml` — commit message patterns
- `rules/docstrings.toml` — docstring padding
- `rules/tests.toml` — test naming and structure
- `rules/errors.toml` — error message patterns
- `rules/api.toml` — API response language

## Contributing

Open an issue using the [new rule template](.github/ISSUE_TEMPLATE/new-rule.md),
or add a TOML entry to the relevant file in `rules/` and open a PR.

## License

MIT. See [LICENSE](LICENSE).
