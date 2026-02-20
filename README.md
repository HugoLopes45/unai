# unai — strip AI writing patterns from text and code

You know it when you read it.

> "Certainly! Here's a comprehensive overview of the key considerations..."
> "It is worth noting that this pivotal approach leverages robust methodologies."
> "Added authentication feature to enhance security of the application."

Every AI assistant writes like this. The patterns are baked in by RLHF — human raters
reward formal-sounding text, so models overfit to words like *delve*, *meticulous*,
*leveraging*, and openers like *Certainly!*

`unai` is a CLI linter that flags and auto-fixes those patterns. Pipe text through it,
check files, or drop it in a git hook. Works on prose, code comments, commit messages,
docstrings, variable names — anything LLMs touch.

```bash
$ echo "Certainly! Let me delve into this comprehensive topic." | unai
Let me explore this thorough topic.
```

---

## Install

### From crates.io (recommended)

```bash
cargo install unai
```

Requires Rust 1.75+. The binary ends up at `~/.cargo/bin/unai`.

No Rust installed? One line gets it:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then `cargo install unai`.

### From source

```bash
git clone https://github.com/HugoLopes45/unai
cd unai
make install
```

### Claude Code skill

```bash
mkdir -p ~/.claude/skills/unai
curl -sL https://raw.githubusercontent.com/HugoLopes45/unai/main/prompts/claude-code.md \
  > ~/.claude/skills/unai/SKILL.md
```

Type `/unai` in any Claude Code session. It applies the rules to the current file or selection.

Or from the repo: `make install-skill`

### Cursor

```bash
make install-cursor   # copies prompts/cursor.mdc → .cursor/rules/unai.mdc
```

Cursor picks up the rules automatically on every generation in that project.

### Any LLM (ChatGPT, Gemini, API)

Copy `prompts/system-prompt.md` into your system prompt. Done.

---

## How it works

`unai` reads from a file or stdin, applies pattern rules, and writes the cleaned text to stdout.
Diagnostics go to stderr so they don't pollute pipes.

Three rule sets run depending on the input:

- **Text mode** — prose patterns: sycophantic openers, LLM filler words, chatbot closers, structural tells
- **Code mode** — code patterns: comment boilerplate, anemic naming, docstring bloat, test anti-patterns
- **Commit mode** — triggered automatically on `COMMIT_EDITMSG` / `MERGE_MSG`: past-tense subjects, vague messages, over-long bodies

Mode is detected automatically from the filename and content. You can override it with `--mode text` or `--mode code`.

---

## Usage

### The basic case — clean and move on

```bash
# pipe text through unai, get clean text back
echo "Certainly! We should utilize this approach." | unai
# → "We should use this approach."

# clean a file
unai draft.md > draft-clean.md

# overwrite in place
unai draft.md | sponge draft.md
```

Only patterns with an auto-fix get replaced. Everything else passes through unchanged.
Use `--report` to see what was flagged but couldn't be auto-fixed.

### `--report` — what's wrong and why

Nothing gets changed. `unai` just tells you what it found and cites the source.

```bash
unai --report draft.md
unai --report < draft.md

# cut the noise — only High and Critical
unai --report --min-severity high draft.md
```

```
Mode: text  |  11 finding(s)

CRITICAL (2)
  line 1: Sycophantic opener: 'Certainly!' (RLHF-induced, Juzek 2025)
  line 1: LLM tell: 'delve' (25× excess frequency, Kobak 2025)

HIGH (3)
  line 2: LLM filler: 'leveraging' (Kobak 2025)
  line 2: LLM tell: 'pivotal' (Kobak 2025, Liang 2024)
  line 3: LLM cliché: 'stands as a testament' (Neri 2024)

MEDIUM (3)
  line 1: LLM filler: 'comprehensive' (Kobak 2025)
  line 2: LLM filler: 'robust' (Kobak 2025)
  line 3: LLM filler: 'innovative' (Kobak 2025)

LOW (3)
  line 1: LLM hedge: 'it is worth noting' (Kobak 2025)
  line 2: LLM connector: 'furthermore' (Rosenfeld 2024)
  line 3: LLM connector: 'in conclusion' (Rosenfeld 2024)
```

Every finding links back to the corpus study that measured the excess frequency. Not vibes — data.

### `--diff` — preview changes before applying them

```bash
unai --diff draft.md
unai --diff < draft.md
```

```diff
--- original
+++ cleaned
@@ -1,3 +1,3 @@
-It is worth noting that this comprehensive approach is pivotal to our success.
-Furthermore, leveraging these robust methodologies facilitates seamless collaboration.
+This thorough approach is key to our success.
+Using these robust methodologies facilitates collaboration.
 In conclusion, this innovative solution stands as a testament to meticulous engineering.
```

Patterns without an auto-fix (like `meticulous`, `innovative`) won't show in the diff —
they only appear in `--report`. Run both to see the full picture.

### `--dry-run` — list every change before committing

```bash
unai --dry-run draft.md
```

```
--- Auto-fixable (4) ---
  line  1: "utilize" → "use"  — LLM filler: 'utilize' (Kobak 2025)
  line  2: "leveraging" → "using"  — LLM filler: 'leveraging' (Kobak 2025)
  line  3: "facilitate" → "help"  — LLM filler: 'facilitate' (Kobak 2025)
  line  4: "in order to" → "to"  — Filler: 'in order to'

--- Flagged (no auto-fix) (2) ---
  line  5: "meticulous"  — LLM tell: 'meticulous' (Kobak 2025, Neri 2024)
  line  6: "innovative"  — LLM filler: 'innovative' (Kobak 2025, lower ratio)
```

Original text is printed unchanged after the list — safe to pipe elsewhere.

### `--annotate` — mark findings inline

```bash
unai --annotate draft.md
```

Prints each line of the original, with finding markers on stderr at the exact column.
Good for quickly spotting where the problems are in a longer document.

### Code mode

```bash
# explicit code mode
unai --mode code --report service.py

# only flag naming and comments (skip the rest)
unai --mode code --rules naming,comments --report service.ts

# docstrings only
unai --mode code --rules docstrings --report api.go
```

Available `--rules` values: `comments`, `naming`, `commits`, `docstrings`, `tests`, `errors`, `api`

**What code mode catches:**

```python
# Before
def getUserDataObject(userDataObject):
    # This function serves as the core handler for processing user data
    # Initialize the result variable to store the output
    result = None
    return result
```

```
HIGH   line 2: LLM docstring boilerplate: 'this function serves as'
MEDIUM line 1: Type-in-name anti-pattern: use 'user' instead of 'userDataObject'
LOW    line 3: Over-explaining comment: states what the code already shows
```

### Commit message mode

`unai` fires commit-specific rules automatically when the filename is `COMMIT_EDITMSG` or `MERGE_MSG`.

```bash
# check a commit message file directly
unai --report COMMIT_EDITMSG

# or pipe one through
echo "Added new authentication feature" | unai --mode code --rules commits --report
# HIGH: Past tense in commit subject — use imperative mood ('add' not 'added')
```

---

## Severity levels

| Level | Meaning | Example |
|-------|---------|---------|
| `critical` | Statistically extreme (r > 10×) or RLHF-specific pattern | `delve`, `Certainly!`, `feel free to` |
| `high` | Strong corpus signal (r > 3×) or clear LLM boilerplate | `leveraging`, `pivotal`, `meticulous` |
| `medium` | Elevated in LLM text, also common in marketing copy | `comprehensive`, `robust`, `innovative` |
| `low` | Filler and hedging language that reads as padded | `in order to`, `moreover`, `furthermore` |

Filter with `--min-severity`:

```bash
unai --report --min-severity high draft.md   # High + Critical only
unai --report --min-severity critical post.md  # Critical only
```

---

## Git hooks

Drop unai into your commit flow and catch LLM-isms before they land.

**Non-blocking (report only):**

```bash
# .git/hooks/commit-msg
#!/bin/sh
unai --mode code --rules commits --report --min-severity high "$1"
```

**Blocking (reject if findings exist):**

```bash
# .git/hooks/commit-msg
#!/bin/sh
findings=$(unai --mode code --rules commits --report --min-severity high "$1" 2>&1)
if echo "$findings" | grep -q "finding(s)"; then
  echo "$findings" >&2
  echo "" >&2
  echo "Fix the commit message and try again." >&2
  exit 1
fi
```

```bash
chmod +x .git/hooks/commit-msg
```

---

## What it catches

### Text patterns

| Pattern | Severity | Source |
|---------|----------|--------|
| `Certainly!`, `Of course!`, `Absolutely!`, `Great question!` | Critical | Juzek 2025 (RLHF) |
| `I'd be happy to`, `Happy to help`, `feel free to` | Critical | Juzek 2025 (RLHF) |
| `delve`, `delves` | Critical | Kobak 2025 (r=25×) |
| `leveraging`, `utilize`, `facilitate`, `commence` | High | Kobak 2025 |
| `pivotal`, `meticulous`, `intricate`, `realm` | High | Kobak 2025, Liang 2024 |
| `stands as a testament`, `tapestry` | High | Neri 2024 |
| `comprehensive`, `robust`, `seamlessly`, `innovative` | Medium | Kobak 2025 |
| `in order to`, `moreover`, `furthermore`, `in conclusion` | Low | Rosenfeld 2024 |

### Code patterns

| Pattern | Severity | Rule |
|---------|----------|------|
| `# This function serves as...`, `# This class represents...` | High | `docstrings` |
| `# Initialize the X variable to Y` | Low | `comments` |
| `UserDataManager`, `ErrorHandler`, `ProcessingHelper` | High | `naming` |
| `userDataObject`, `configurationSettings` | Medium | `naming` |
| `Added X` in commit subject | High | `commits` |
| `assert result is not None` with no message | Medium | `tests` |
| bare `except: pass` | High | `errors` |

137 patterns across 9 rule files. [Browse them in `rules/`](rules/).

### What unai doesn't touch

- Content inside fenced code blocks in Markdown (` ``` ` fences)
- Inline backtick spans (`` `code` ``)
- Bare URL lines
- Non-English text: `"pivotale"` (French), `"pivotaler"` (German), `"这是pivotal决策"` (Chinese) — word-boundary matching is Unicode-safe, so foreign words that contain an English banned word as a substring pass through unchanged

---

## Why this exists

LLMs generate the statistical median of "correct writing." That median is padded,
over-hedged, and full of words that sound confident while saying nothing specific.

The patterns are predictable because RLHF rewards formal-sounding text.
*Leveraging* because business writing uses it. *Furthermore* because academic papers use it.
*Certainly!* because it sounds agreeable to a human rater grading a response.

None of it is wrong, exactly. It just doesn't sound like a person wrote it.
A person would say *using* instead of *leveraging*. A person's code comment
would say `# don't touch this — it works and I don't know why` instead of
`# This function serves as the core processing handler for user data operations`.

`unai` draws a line. Write like yourself, not like a chatbot trying to sound helpful.

---

## Research basis

Rules are grounded in peer-reviewed corpus studies, not intuition.

**Kobak et al. (2025), *Science Advances*** — 15 million PubMed abstracts (2010–2024).
`delves` appeared 25× more often post-ChatGPT than baseline. 280+ excess words identified,
66% verbs, 18% adjectives. This is the core word list.
[arXiv:2406.07016](https://arxiv.org/abs/2406.07016)

**Liang et al. (2024), *Nature Human Behaviour*** — 950,000+ scientific papers.
`pivotal`, `intricate`, `showcasing`, `realm` approximately doubled post-2023.
[arXiv:2404.01268](https://arxiv.org/abs/2404.01268)

**Juzek & Ward (COLING 2025)** — Explains *why* the vocabulary diverges: RLHF, not training data.
Human raters prefer formal-sounding outputs. Models overfit to that preference.
[arXiv:2412.11385](https://arxiv.org/abs/2412.11385)

**Rosenfeld & Lazebnik (2024)** — 6 LLMs vs 13,371 NYT articles. LLMs produce less sentence
length variation, more auxiliary verbs, fewer complex noun phrases.
[PMC11422446](https://pmc.ncbi.nlm.nih.gov/articles/PMC11422446/)

**Bisztray et al. (AISec 2025)** — Comment density is the #1 feature for LLM code authorship
attribution. Removing it drops accuracy 7.2pp.
[arXiv:2506.17323](https://arxiv.org/abs/2506.17323)

**Lopes & Klotzman (ICSE 2024)** — LLM commit messages are 20× longer than human commits
(median 381 vs 19 chars). LLM commits always include "why" explanations.
[arXiv:2401.17622](https://arxiv.org/abs/2401.17622)

This tool is a **style linter**, not an AI detector. It cannot tell you whether a document
was written by an LLM — no surface-level tool can do that reliably. What it can do is
help you write in a voice that reads as distinctly human.

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
