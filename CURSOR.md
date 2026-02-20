# unai — context for Cursor AI

This repo provides a Cursor rule that prevents LLM-isms in generated code and text.

## Install

```bash
# In your project directory:
cp /path/to/unai/prompts/cursor.mdc .cursor/rules/unai.mdc

# Or from the repo:
make install-cursor
```

## What the rule does

When active, the Cursor rule instructs the AI to avoid generating:
- AI vocabulary in prose (leveraging, comprehensive, robust, seamlessly...)
- Tautological code comments that restate the next line
- Manager/Handler/Helper suffixes on class names
- Type-encoded variable names (userDataObject, configDict)
- Bare TODO comments without context
- LLM docstring openers ("This function serves as...")
- Step-numbered comments inside functions
- Past-tense commit messages

The rule file is at `prompts/cursor.mdc`. It uses `alwaysApply: false` so it
only activates when you explicitly include it.

## Full rules reference

See [`rules/`](rules/) for 137 patterns with before/after examples.
