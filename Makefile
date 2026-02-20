.PHONY: build install test install-skill install-cursor

build:
	cargo build --release --manifest-path cli/Cargo.toml

install:
	cargo install --path cli/

test:
	cargo test --manifest-path cli/Cargo.toml

install-skill:
	mkdir -p ~/.claude/skills/unai
	cp prompts/claude-code.md ~/.claude/skills/unai/SKILL.md
	@echo "Skill installed. Use /unai in Claude Code."

install-cursor:
	mkdir -p .cursor/rules
	cp prompts/cursor.mdc .cursor/rules/unai.mdc
	@echo "Cursor rule installed at .cursor/rules/unai.mdc"
