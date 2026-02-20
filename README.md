# unai — strip LLM-isms from text and code

You know the style. "Certainly! Here's a comprehensive overview..." Every AI
writes like this. Every PR comment, every generated docstring, every commit
message that says "refactor the codebase to enhance maintainability."

`unai` finds those patterns and kills them. Claude Code skill, Cursor rule,
OpenCode prompt, Rust CLI. Works on prose, code comments, variable names,
commit messages, docstrings, tests, error handling — wherever LLMs leave
fingerprints.

## What it catches

| Text | Code |
|------|------|
| "Certainly!", "Of course!", "Great question!" | `# Initialize the counter variable to zero` |
| "it's worth noting", "it is important to note" | `class UserDataManager` / `processUserDataObject` |
| "robust", "seamless", "leveraging", "comprehensive" | `# TODO: add error handling` |
| em-dash overuse — like this — everywhere — | `assert result is not None` |
| "In conclusion", "Moreover", "Furthermore" | `def handleUserAuthenticationRequest()` |
| "I hope this helps! Let me know if..." | `# Step 1: validate. Step 2: process.` |
| rule of three (innovation, iteration, and insights) | past-tense commit messages |
| "Not only X, but also Y" | `result = ...` used for every return value |

137 patterns across 9 rule files. [Browse them in `rules/`](rules/).

## Install

### Claude Code

```bash
mkdir -p ~/.claude/skills/unai
curl -sL https://raw.githubusercontent.com/HugoLopes45/unai/main/prompts/claude-code.md \
  > ~/.claude/skills/unai/SKILL.md
```

Then `/unai` in any session.

Or if you cloned the repo:

```bash
make install-skill
```

### Cursor

```bash
make install-cursor
```

Copies `prompts/cursor.mdc` to `.cursor/rules/unai.mdc` in the current project.

### OpenCode

Add `prompts/opencode.md` as a rule in your OpenCode config.

### Any LLM (ChatGPT, Gemini, API)

Copy `prompts/system-prompt.md` into the system prompt field. That's it.

### CLI (Rust)

```bash
cargo install --path cli/
```

Requires Rust 1.75+. Then:

```bash
# pipe anything through it
echo "Certainly! Here's a thorough breakdown..." | unai

# check a file
unai response.md

# report mode — see what's wrong without changing it
unai --report < draft.md

# filter by severity (critical, high, medium, low)
unai --report --min-severity high < draft.md

# code mode — flag naming, comments, docstrings
cat service.py | unai --mode code --report

# annotate mode — marks each finding inline
unai --annotate < file.ts
```

Text rules are word-boundary aware and Unicode-safe: `"pivotale"` (French), `"pivotaler"` (German), `"这是pivotal决策"` (Chinese) all pass through unchanged — the match fires only when the banned word stands alone as a word in any script. Content inside fenced code blocks, inline backtick spans, and bare URL lines is never flagged.

Works with any programming language. Code rules match comment syntax (`#`, `//`, `--`, `/* */`), naming patterns, and commit messages regardless of what language the file is written in.

## Why this exists

LLMs generate the statistical median of "correct writing." That median is padded,
over-hedged, and full of words that sound confident while saying nothing.

The patterns are predictable because they come from the same training data.
"Leveraging" because business writing uses it. "Furthermore" because academic
papers use it. The rule of three because rhetoric textbooks say three things sound
complete.

None of it is wrong, exactly. It just doesn't sound like a person wrote it.
A person would say "using" instead of "leveraging." A person's code comment
would say "don't touch this — it works and I have no idea why" instead of
"# This function serves as the core processing handler for user data operations."

## Rules

Nine files, 137 patterns. Each has a before/after and a one-line fix rule.

- [`rules/text.md`](rules/text.md) — 24 prose patterns (Wikipedia Signs of AI Writing)
- [`rules/code-comments.md`](rules/code-comments.md) — 18 comment patterns
- [`rules/naming.md`](rules/naming.md) — 14 naming patterns
- [`rules/commits.md`](rules/commits.md) — 10 commit message patterns
- [`rules/docstrings.md`](rules/docstrings.md) — 8 docstring patterns
- [`rules/tests.md`](rules/tests.md) — 15 test patterns
- [`rules/error-handling.md`](rules/error-handling.md) — 15 error handling patterns
- [`rules/api-design.md`](rules/api-design.md) — 17 API design patterns
- [`rules/llm-tells.md`](rules/llm-tells.md) — 16 fingerprints unique to LLMs

## Contributing

Open an issue with the [new rule template](.github/ISSUE_TEMPLATE/new-rule.md)
before writing code. Rules that already exist or that are too subjective get closed.

The bar: if a human developer would write it unprompted, it's not an LLM-ism.

## License

MIT.
