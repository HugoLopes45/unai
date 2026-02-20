# unai — a style linter for AI-characteristic writing patterns

You know the style. "Certainly! Here's a comprehensive overview..." Every AI
writes like this. Every PR comment, every generated docstring, every commit
message that says "refactor the codebase to enhance maintainability."

`unai` flags those patterns and fixes them. Claude Code skill, Cursor rule,
OpenCode prompt, Rust CLI. Works on prose, code comments, variable names,
commit messages, docstrings, tests, error handling — wherever LLM-characteristic
patterns appear.

## Install

### CLI (fastest path)

```bash
cargo install unai
```

Requires Rust 1.75+. Installs the `unai` binary to `~/.cargo/bin/`.

Don't have Rust? Install it in one line: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### From source

```bash
git clone https://github.com/HugoLopes45/unai
cd unai
make install   # cargo install --path cli/
```

### Claude Code skill

```bash
mkdir -p ~/.claude/skills/unai
curl -sL https://raw.githubusercontent.com/HugoLopes45/unai/main/prompts/claude-code.md \
  > ~/.claude/skills/unai/SKILL.md
```

Then use `/unai` in any Claude Code session to apply unai rules to the current file or selection.

Or if you cloned the repo: `make install-skill`

### Cursor

```bash
# from the repo root
make install-cursor
```

Copies `prompts/cursor.mdc` to `.cursor/rules/unai.mdc` in the current project.
Cursor will apply the rules automatically on every generation.

### OpenCode

Add `prompts/opencode.md` as a rule in your OpenCode config.

### Any LLM (ChatGPT, Gemini, API)

Copy `prompts/system-prompt.md` into the system prompt field. That's it.

---

## Usage

`unai` reads from a file or stdin and writes cleaned output to stdout.
Diagnostics (findings, reports, diffs) go to stderr so they don't pollute pipes.

### Basic pipe — clean and move on

```bash
# replace LLM-isms in-place (auto-fixable patterns only)
echo "Certainly! We should utilize this approach." | unai
# → "We should use this approach."

# clean a file, write result to a new file
unai draft.md > draft-clean.md

# clean in-place (shell trick)
unai draft.md | sponge draft.md
```

### Report mode — see what's wrong without changing anything

```bash
unai --report draft.md
unai --report < draft.md

# filter to High and Critical only (less noise)
unai --report --min-severity high draft.md
```

Output goes to stderr, grouped by severity:
```
Mode: text  |  5 finding(s)

CRITICAL (2)
  line 1: Sycophantic opener: 'Certainly!' (RLHF-induced, Juzek 2025) 'Certainly!'
  line 3: LLM tell: 'delve' (25× excess frequency, Kobak 2025) 'delve'

HIGH (2)
  line 5: LLM filler: 'leveraging' (Kobak 2025) 'leveraging'
  line 7: LLM filler: 'utilize' (Kobak 2025) 'utilize'

LOW (1)
  line 9: Filler: 'in order to' 'in order to'
```

### Diff mode — preview changes as a unified diff

```bash
unai --diff draft.md
unai --diff < draft.md
```

```diff
--- original
+++ cleaned
@@ -1,3 +1,3 @@
-Certainly! We should utilize this approach.
+We should use this approach.
```

If a finding has no auto-fix (e.g. `meticulous` has no direct replacement), it won't appear in
the diff but will appear in `--report`. Run both to see the full picture.

### Annotate mode — inline markers at the point of each finding

```bash
unai --annotate draft.md
```

Prints the original text to stdout with finding annotations on stderr, positioned
at the exact column where each pattern appears.

### Dry-run mode — list changes without applying them

```bash
unai --dry-run draft.md
```

Shows every auto-fixable substitution (`"utilize" → "use"`) and every non-fixable
flag, then emits the original text unchanged. Good for reviewing before committing
to a change.

### Mode selection

`unai` detects the mode automatically from the filename and content:

| Mode | Triggers on | Rules applied |
|------|-------------|---------------|
| `text` | `.md`, `.txt`, plain prose | Word-frequency rules, sycophantic openers, chatbot closers, structural patterns |
| `code` | `.rs`, `.py`, `.ts`, `.go`, `.js`, etc. | Comment rules, naming rules, docstring rules, test rules, error-handling rules, API rules |
| `commit` | `COMMIT_EDITMSG`, `MERGE_MSG` | Text rules + commit-specific rules (past tense, vague subjects, over-long bodies) |

Override detection with `--mode text` or `--mode code`:

```bash
# force text mode on a file with no extension
unai --mode text < notes

# force code mode on a .md file that's actually a code dump
unai --mode code service_dump.md --report
```

### Code rules — apply only a subset

In code mode, all rule categories run by default. Use `--rules` to narrow:

```bash
# only naming and comment rules
unai --mode code --rules naming,comments --report service.py

# only commit message rules
unai --mode code --rules commits --report COMMIT_EDITMSG
```

Available categories: `comments`, `naming`, `commits`, `docstrings`, `tests`, `errors`, `api`

### Severity filter

```bash
# only show Critical and High findings
unai --report --min-severity high draft.md

# only Critical
unai --report --min-severity critical draft.md
```

Severity levels: `critical` > `high` > `medium` > `low`

---

## Git hooks

Run unai automatically on every commit message:

```bash
# .git/hooks/commit-msg
#!/bin/sh
unai --mode code --rules commits --report --min-severity high "$1"
```

```bash
chmod +x .git/hooks/commit-msg
```

This flags past-tense subjects, vague messages ("wip", "fix stuff"), and over-long bodies
before the commit lands. The hook exits 0 (non-blocking) — it reports but doesn't block.
Change `--report` to a failing exit code if you want it to block:

```bash
#!/bin/sh
output=$(unai --mode code --rules commits --report --min-severity high "$1" 2>&1)
if [ -n "$output" ]; then
  echo "$output" >&2
  exit 1
fi
```

---

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

Text rules are word-boundary aware and Unicode-safe: `"pivotale"` (French), `"pivotaler"` (German),
`"这是pivotal决策"` (Chinese) all pass through unchanged — the match fires only when the banned word
stands alone as a word in any script. Content inside fenced code blocks, inline backtick spans,
and bare URL lines is never flagged.

---

## Research Basis

`unai`'s rules are grounded in peer-reviewed corpus studies, not intuition.

**Kobak et al. (2025) — *Science Advances*** — 15 million PubMed abstracts (2010–2024)
- `delves` is 25× more frequent in post-ChatGPT biomedical text than expected
- `showcasing` (9.2×), `underscores` (9.1×) — empirically measured, not guessed
- Full word list: [github.com/berenslab/llm-excess-vocab](https://github.com/berenslab/llm-excess-vocab)
- Paper: [arXiv:2406.07016](https://arxiv.org/abs/2406.07016)

**Liang et al. (2024) — *Nature Human Behaviour*** — 950,000+ scientific papers
- `pivotal`, `intricate`, `showcasing`, `realm` approximately doubled post-2023
- [arXiv:2404.01268](https://arxiv.org/abs/2404.01268)

**Juzek & Ward (COLING 2025)** — The excess vocabulary is an RLHF artifact, not a training-data artifact.
Human raters prefer formal-sounding outputs, so fine-tuned models overfit to this vocabulary.
[arXiv:2412.11385](https://arxiv.org/abs/2412.11385)

**Bisztray et al. (AISec 2025)** — Comment density is the #1 discriminating feature for LLM code authorship attribution. [arXiv:2506.17323](https://arxiv.org/abs/2506.17323)

**Lopes & Klotzman (ICSE 2024)** — LLM commit messages are 20× longer than human commits.
[arXiv:2401.17622](https://arxiv.org/abs/2401.17622)

This tool enforces a writing style guide. It is **not** an AI content detector — it cannot
determine whether a document was written by an LLM. No surface-level tool can do this reliably.

---

## Rules

Nine files, 137 patterns. Each has a before/after and a one-line fix.

- [`rules/text.md`](rules/text.md) — 24 prose patterns
- [`rules/code-comments.md`](rules/code-comments.md) — 18 comment patterns
- [`rules/naming.md`](rules/naming.md) — 14 naming patterns
- [`rules/commits.md`](rules/commits.md) — 10 commit message patterns
- [`rules/docstrings.md`](rules/docstrings.md) — 8 docstring patterns
- [`rules/tests.md`](rules/tests.md) — 15 test patterns
- [`rules/error-handling.md`](rules/error-handling.md) — 15 error handling patterns
- [`rules/api-design.md`](rules/api-design.md) — 17 API design patterns
- [`rules/llm-tells.md`](rules/llm-tells.md) — 16 fingerprints unique to LLMs

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Open an issue with the
[new rule template](.github/ISSUE_TEMPLATE/new-rule.md) before writing code.

The bar: if a human developer would write it unprompted, it's not a style violation.

## License

MIT.
