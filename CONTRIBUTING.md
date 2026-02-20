# Contributing

## Adding a rule

1. Find the right category file in `rules/` (text, comments, naming, commits, etc.).
2. Add a TOML entry:

```toml
[[rules]]
id = "short-unique-id"
pattern = "regex or literal to match"
message = "One sentence: what this is and why it's bad."
fix = "optional replacement string"
```

3. If no category fits, create `rules/<category>.toml` and add it to `Makefile` and `cli/src/categories.rs`.

## Testing a rule

```bash
echo "your test input" | cargo run --manifest-path cli/Cargo.toml
cargo test
```

All rule files have snapshot tests in `cli/tests/`. Add a case for your rule there.

## Submitting a PR

Open an issue first if the rule is debatable. For clear-cut LLM-isms, a PR alone is fine.
Keep the diff small — one rule per PR is easiest to review.
