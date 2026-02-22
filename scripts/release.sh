#!/usr/bin/env bash
# scripts/release.sh — automate version bump, changelog entry, git tag, and push.
#
# Usage:
#   ./scripts/release.sh patch        # 0.3.1 → 0.3.2
#   ./scripts/release.sh minor        # 0.3.1 → 0.4.0
#   ./scripts/release.sh major        # 0.3.1 → 1.0.0
#   ./scripts/release.sh 0.5.0        # explicit version
#   ./scripts/release.sh patch --dry-run   # preview without writing anything
set -euo pipefail

CARGO_TOML="cli/Cargo.toml"
CHANGELOG="CHANGELOG.md"
BINARY_NAME="unai"

# ── helpers ──────────────────────────────────────────────────────────────────

die() { echo "error: $*" >&2; exit 1; }

require() {
  for cmd in "$@"; do
    command -v "$cmd" &>/dev/null || die "'$cmd' not found — please install it"
  done
}

semver_bump() {
  local current="$1" bump="$2"
  IFS='.' read -r major minor patch <<< "$current"
  case "$bump" in
    major) echo "$((major + 1)).0.0" ;;
    minor) echo "${major}.$((minor + 1)).0" ;;
    patch) echo "${major}.${minor}.$((patch + 1))" ;;
    *)     echo "$bump" ;;   # treat as explicit version
  esac
}

# ── parse args ────────────────────────────────────────────────────────────────

BUMP="${1:-}"
DRY_RUN=false

for arg in "$@"; do
  [[ "$arg" == "--dry-run" ]] && DRY_RUN=true
done

[[ -z "$BUMP" || "$BUMP" == "--dry-run" ]] && die "Usage: $0 <patch|minor|major|x.y.z> [--dry-run]"
[[ "$BUMP" == "--dry-run" ]] && die "Usage: $0 <patch|minor|major|x.y.z> [--dry-run]"

require git cargo sed

# ── current version ───────────────────────────────────────────────────────────

CURRENT=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/.*"\(.*\)".*/\1/')
[[ -z "$CURRENT" ]] && die "Could not read version from $CARGO_TOML"

NEW=$(semver_bump "$CURRENT" "$BUMP")

# Validate semver shape
[[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Invalid version: $NEW"

TODAY=$(date +%Y-%m-%d)
TAG="v${NEW}"

echo "  current : v${CURRENT}"
echo "  new     : ${TAG}"
echo "  date    : ${TODAY}"
echo "  dry-run : ${DRY_RUN}"
echo ""

# ── guard ────────────────────────────────────────────────────────────────────

git diff --quiet || die "Working tree has unstaged changes — commit or stash first"
git diff --cached --quiet || die "Staged changes exist — commit first"

git fetch --tags --quiet
git rev-parse "$TAG" &>/dev/null && die "Tag $TAG already exists"

# ── diff since last tag ───────────────────────────────────────────────────────

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -n "$LAST_TAG" ]]; then
  echo "Commits since ${LAST_TAG}:"
  git log "${LAST_TAG}..HEAD" --oneline
else
  echo "No previous tag found — showing last 10 commits:"
  git log --oneline -10
fi
echo ""

# ── apply changes ─────────────────────────────────────────────────────────────

if [[ "$DRY_RUN" == "true" ]]; then
  echo "[dry-run] Would bump Cargo.toml: $CURRENT → $NEW"
  echo "[dry-run] Would prepend CHANGELOG entry for $TAG"
  echo "[dry-run] Would commit: chore: release $TAG"
  echo "[dry-run] Would tag: $TAG"
  echo "[dry-run] Would push branch + tag"
  exit 0
fi

# 1. Bump version in Cargo.toml (first occurrence only)
sed -i.bak "0,/^version = \"${CURRENT}\"/s//version = \"${NEW}\"/" "$CARGO_TOML"
rm -f "${CARGO_TOML}.bak"

# 2. Update Cargo.lock (cargo fetch is enough — no build needed)
cargo generate-lockfile --manifest-path "$CARGO_TOML" 2>/dev/null || true

# 3. Prepend CHANGELOG entry
CHANGELOG_ENTRY="## ${TAG} — ${TODAY}

### Added

### Fixed

### Changed

"
# Insert after the first line (the "# Changelog" header + blank line)
awk -v entry="$CHANGELOG_ENTRY" '
  NR == 3 { print entry }
  { print }
' "$CHANGELOG" > "${CHANGELOG}.tmp" && mv "${CHANGELOG}.tmp" "$CHANGELOG"

echo "CHANGELOG updated — please fill in the release notes, then press Enter to continue (Ctrl-C to abort)."
read -r

# 4. Commit
git add "$CARGO_TOML" "cli/Cargo.lock" "$CHANGELOG"
git commit -m "chore: release ${TAG}"

# 5. Tag
git tag "$TAG"

echo ""
echo "Ready to push. This will trigger the GitHub release workflow."
read -rp "Push branch + tag to origin? [y/N] " confirm
if [[ "${confirm,,}" == "y" ]]; then
  git push origin HEAD
  git push origin "$TAG"
  echo "Pushed. Watch the release at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
else
  echo "Not pushed. Run when ready:"
  echo "  git push origin HEAD && git push origin ${TAG}"
fi
