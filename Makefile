.PHONY: build install test lint fmt fmt-check release release-patch release-minor release-major tag build-all setup install-hooks install-skill install-cursor install-opencode install-codex install-windsurf install-zed install-copilot install-cline install-amp install-amazonq install-continue install-aider install-all

setup: install-hooks

build:
	cargo build --manifest-path cli/Cargo.toml

release:
	cargo build --release --manifest-path cli/Cargo.toml

install:
	cargo install --path cli/

test:
	cargo test --manifest-path cli/Cargo.toml

lint:
	cargo clippy --manifest-path cli/Cargo.toml -- -D warnings

fmt:
	cargo fmt --manifest-path cli/Cargo.toml

fmt-check:
	cargo fmt --manifest-path cli/Cargo.toml -- --check

build-all:
	./scripts/build-all.sh

install-hooks:
	git config core.hooksPath .githooks
	@echo "Pre-push hook installed."

release-patch:
	./scripts/release.sh patch

release-minor:
	./scripts/release.sh minor

release-major:
	./scripts/release.sh major

tag:
	git tag -a v$(shell grep '^version' cli/Cargo.toml | head -1 | cut -d'"' -f2) \
	  -m "Release v$(shell grep '^version' cli/Cargo.toml | head -1 | cut -d'"' -f2)"

install-skill:
	mkdir -p ~/.claude/skills/unai
	cp prompts/claude-code.md ~/.claude/skills/unai/SKILL.md
	@echo "Skill installed. Use /unai in Claude Code."

install-cursor:
	mkdir -p .cursor/rules
	cp prompts/cursor.mdc .cursor/rules/unai.mdc
	@echo "Cursor rule installed at .cursor/rules/unai.mdc"

install-opencode:
	mkdir -p ~/.config/opencode
	cp prompts/agents.md ~/.config/opencode/AGENTS.md
	@echo "opencode: installed at ~/.config/opencode/AGENTS.md"

install-codex:
	mkdir -p ~/.codex
	cp prompts/agents.md ~/.codex/AGENTS.md
	@echo "Codex CLI: installed at ~/.codex/AGENTS.md"

install-windsurf:
	mkdir -p .windsurf/rules
	cp prompts/agents.md .windsurf/rules/unai.md
	@echo "Windsurf: installed at .windsurf/rules/unai.md"

install-zed:
	cp prompts/agents.md .rules
	@echo "Zed: installed at .rules"

install-copilot:
	mkdir -p .github
	cp prompts/agents.md .github/copilot-instructions.md
	@echo "GitHub Copilot: installed at .github/copilot-instructions.md"

install-cline:
	cp prompts/agents.md .clinerules
	@echo "Cline: installed at .clinerules"

install-amp:
	cp prompts/agents.md AGENTS.md
	@echo "Amp (project-local): installed at AGENTS.md"

install-amazonq:
	mkdir -p .amazonq/rules
	cp prompts/agents.md .amazonq/rules/unai.md
	@echo "Amazon Q: installed at .amazonq/rules/unai.md"

install-continue:
	mkdir -p .continue/rules
	cp prompts/agents.md .continue/rules/unai.md
	@echo "Continue.dev: installed at .continue/rules/unai.md"

install-aider:
	cp prompts/agents.md CONVENTIONS.md
	@echo "Aider: installed at CONVENTIONS.md (add 'read: CONVENTIONS.md' to .aider.conf.yml)"

install-all: install-skill install-cursor install-opencode install-codex install-windsurf install-zed install-copilot install-cline install-amp install-amazonq install-continue install-aider
	@echo "All integrations installed."
