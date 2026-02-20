# Contributing

## Adding a rule

Rules live in `rules/*.md` — plain Markdown files, one per category (text, comments, naming, commits, etc.).

The implementation reads these files and maps them into Rust structs in `cli/src/rules.rs`.
To add a rule:

1. Open the relevant `rules/<category>.md` (e.g. `rules/text.md`).
2. Add an entry following the existing pattern — each rule has an `id`, `pattern`, `message`, optional `fix`, and `severity`.
3. In `cli/src/rules.rs`, add the rule to the appropriate constant (`TEXT_RULES`, `COMMIT_RULES`, `CODE_RULES`, etc.) in the same format as existing entries.

If no category fits, create `rules/<category>.md` and add corresponding entries to `cli/src/rules.rs`.

## Testing a rule

```bash
echo "your test input" | cargo run --manifest-path cli/Cargo.toml
make test
```

Unit tests live in `cli/src/rules.rs` under `#[cfg(test)]`.
Integration tests (binary-level) live in `cli/tests/integration.rs`.

Add a test case for your new rule in the appropriate module.

## Running the full suite

```bash
make test       # unit + integration tests
make lint       # clippy -D warnings
make fmt-check  # formatting check
```

## Submitting a PR

- Open an issue first if the rule is debatable.
- For clear-cut LLM-isms, a PR alone is fine.
- One rule per PR is easiest to review.
- Make sure `make test && make lint && make fmt-check` all pass before opening.
