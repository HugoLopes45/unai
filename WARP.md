# unai — context for Warp AI

This repo is a tool that removes LLM-isms from text and code. It works as:
- a Claude Code skill (`/unai`)
- a Cursor rule (`.cursor/rules/unai.mdc`)
- an OpenCode prompt
- a Rust CLI (`echo "text" | unai`)

## Key files

| File | Purpose |
|------|---------|
| `prompts/claude-code.md` | Claude Code skill — copy to `~/.claude/skills/unai/SKILL.md` |
| `prompts/cursor.mdc` | Cursor rule — copy to `.cursor/rules/unai.mdc` |
| `prompts/opencode.md` | OpenCode prompt |
| `prompts/system-prompt.md` | Universal system prompt for any LLM |
| `rules/` | 9 rule files, 137 patterns with before/after examples |
| `cli/` | Rust CLI source |

## Install (quick)

```bash
make install-skill   # Claude Code
make install-cursor  # Cursor
cargo install --path cli/  # CLI binary
```

## What it detects

Text: banned words (leveraging, comprehensive, robust, seamlessly...), sycophantic
openers, chatbot closers, hedging, rule of three, em-dash overuse.

Code: tautological comments, section headers, bare TODOs, Manager/Handler/Helper
suffixes, type-in-name variables, LLM docstring openers, step-numbered comments.

## Rules format

Each file in `rules/` is markdown with this structure per pattern:
- Pattern name (sentence case heading)
- 1-2 sentences on why LLMs do it
- `LLM:` before example
- `Human:` after example
- `Rule:` one-line fix
