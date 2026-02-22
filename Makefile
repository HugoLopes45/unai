.PHONY: build install test lint fmt fmt-check release release-patch release-minor release-major tag build-all install-hooks install-skill install-cursor

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
