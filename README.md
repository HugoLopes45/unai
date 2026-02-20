# unai — a style linter for AI-characteristic writing patterns

You know the style. "Certainly! Here's a comprehensive overview..." Every AI
writes like this. Every PR comment, every generated docstring, every commit
message that says "refactor the codebase to enhance maintainability."

`unai` flags those patterns and fixes them. Claude Code skill, Cursor rule,
OpenCode prompt, Rust CLI. Works on prose, code comments, variable names,
commit messages, docstrings, tests, error handling — wherever LLM-characteristic
patterns appear.

## Research Basis

`unai`'s rules are grounded in peer-reviewed corpus studies, not intuition.

### Word frequency: what the data shows

**Kobak et al. (2025) — *Science Advances*** — 15 million PubMed abstracts (2010–2024)
- `delves` is 25× more frequent in post-ChatGPT biomedical text than expected
- `showcasing` (9.2×), `underscores` (9.1×) — these ratios are empirically measured, not guessed
- 280+ excess words identified; 66% are verbs, 18% adjectives
- Full word list: [github.com/berenslab/llm-excess-vocab](https://github.com/berenslab/llm-excess-vocab)
- Paper: [arXiv:2406.07016](https://arxiv.org/abs/2406.07016)

**Liang et al. (2024) — *Nature Human Behaviour*** — 950,000+ scientific papers
- `pivotal`, `intricate`, `showcasing`, `realm` approximately doubled post-2023
- Up to 22% of CS preprints show LLM modification signals
- [arXiv:2404.01268](https://arxiv.org/abs/2404.01268)

### Why these words appear: RLHF, not training data

**Juzek & Ward (COLING 2025)**
The excess vocabulary is not a training-data artifact. Human RLHF raters prefer "formal-sounding" outputs, so fine-tuned models overfit to this vocabulary. Any word can be tuned away; structural patterns driven by RLHF are more stable. [arXiv:2412.11385](https://arxiv.org/abs/2412.11385)

### Structural signals

**Rosenfeld & Lazebnik (2024)** — 6 LLMs vs 13,371 NYT articles
- LLMs produce less sentence length variation, more auxiliary verbs, fewer complex noun phrases
- Structural signals are more model-agnostic than word lists
- [PMC11422446](https://pmc.ncbi.nlm.nih.gov/articles/PMC11422446/)

### Code and commit patterns

**Bisztray et al. (AISec 2025)** — Comment density is the #1 discriminating feature for LLM code authorship attribution. Removing it drops multi-class accuracy 7.2pp. [arXiv:2506.17323](https://arxiv.org/abs/2506.17323)

**Lopes & Klotzman (ICSE 2024)** — LLM commit messages are 20× longer than human commits (median 381 vs 19 characters). LLM commits always include "why" explanations; human commits default to bare action keywords. [arXiv:2401.17622](https://arxiv.org/abs/2401.17622)

**arXiv:2601.17406 (2025)** — Across 33,580 pull requests, multiline commit ratio is the top fingerprint feature (44.7% discriminative weight). How agents communicate changes is more distinctive than what they change.

### On the limits of style linting

This tool enforces a writing style guide. It is not an AI content detector.

Per Sadasivan et al. (TMLR 2023): as LLMs improve at emulating human text, even theoretically optimal detectors approach random performance (AUROC → 0.5). Style linting sidesteps this bound: a rule that says "don't write like an LLM" works regardless of detection difficulty. [arXiv:2303.11156](https://arxiv.org/abs/2303.11156)

Word lists go stale as models retrain away from flagged vocabulary. `unai`'s word list is updated against new corpus data. See [CHANGELOG.md](CHANGELOG.md).

## What This Tool Is (And Isn't)

**IS:** A prescriptive style linter. It flags and auto-fixes patterns that corpus studies show are statistically overrepresented in LLM-generated writing. It helps you write in a voice that reads as distinctly human.

**IS NOT:** An AI content detector. It cannot determine whether a document was written by an LLM. No surface-level tool can do this reliably.

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

# diff mode — unified diff output
unai --diff < draft.md
```

Text rules are word-boundary aware and Unicode-safe: `"pivotale"` (French), `"pivotaler"` (German), `"这是pivotal决策"` (Chinese) all pass through unchanged — the match fires only when the banned word stands alone as a word in any script. Content inside fenced code blocks, inline backtick spans, and bare URL lines is never flagged.

Works with any programming language. Code rules match comment syntax (`#`, `//`, `--`, `/* */`), naming patterns, and commit messages regardless of what language the file is written in.

## Why this exists

LLMs generate the statistical median of "correct writing." That median is padded,
over-hedged, and full of words that sound confident while saying nothing.

The patterns are predictable because RLHF rewards formal-sounding text.
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

The bar: if a human developer would write it unprompted, it's not a style violation.

## License

MIT.
