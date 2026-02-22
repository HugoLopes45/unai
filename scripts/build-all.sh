#!/usr/bin/env bash
# scripts/build-all.sh — build release binaries for all supported targets locally.
#
# Requirements (macOS ARM host):
#   brew install zig
#   cargo install cargo-zigbuild
#   rustup target add x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
#
# Windows (x86_64-pc-windows-msvc) can only be built natively on Windows — skipped here.
#
# Usage:
#   ./scripts/build-all.sh              # build all cross-compilable targets
#   ./scripts/build-all.sh --release    # same (default)
#   ./scripts/build-all.sh --check      # fmt + clippy only, no build
set -euo pipefail

MANIFEST="cli/Cargo.toml"
OUT_DIR="dist"
VERSION=$(grep '^version' "$MANIFEST" | head -1 | sed 's/.*"\(.*\)".*/\1/')
PROFILE="release"

TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
)

# ── helpers ──────────────────────────────────────────────────────────────────

die()  { echo "error: $*" >&2; exit 1; }
info() { echo "▶ $*"; }

require() {
  for cmd in "$@"; do
    command -v "$cmd" &>/dev/null || die "'$cmd' not found — run: brew install zig && cargo install cargo-zigbuild"
  done
}

# ── parse args ────────────────────────────────────────────────────────────────

CHECK_ONLY=false
for arg in "$@"; do
  [[ "$arg" == "--check" ]] && CHECK_ONLY=true
done

# ── lint / fmt ────────────────────────────────────────────────────────────────

info "fmt check"
cargo fmt --manifest-path "$MANIFEST" -- --check

info "clippy"
cargo clippy --manifest-path "$MANIFEST" -- -D warnings

info "tests"
cargo test --manifest-path "$MANIFEST"

[[ "$CHECK_ONLY" == "true" ]] && { echo "All checks passed."; exit 0; }

# ── build ─────────────────────────────────────────────────────────────────────

require zig cargo-zigbuild

mkdir -p "$OUT_DIR"

FAILED=()

for target in "${TARGETS[@]}"; do
  info "building $target"

  # Apple targets: use regular cargo (no zig needed — SDK available natively)
  if [[ "$target" == *"apple"* ]]; then
    if cargo build --"$PROFILE" --manifest-path "$MANIFEST" --target "$target" 2>&1; then
      bin="cli/target/$target/$PROFILE/unai"
      asset="${OUT_DIR}/unai-v${VERSION}-${target}.tar.gz"
      tar czf "$asset" -C "$(dirname "$bin")" "$(basename "$bin")"
      echo "  → $asset"
    else
      echo "  ✗ $target FAILED"
      FAILED+=("$target")
    fi
  else
    # Linux targets: use cargo-zigbuild for cross-compilation
    if cargo zigbuild --"$PROFILE" --manifest-path "$MANIFEST" --target "$target" 2>&1; then
      bin="cli/target/$target/$PROFILE/unai"
      asset="${OUT_DIR}/unai-v${VERSION}-${target}.tar.gz"
      tar czf "$asset" -C "$(dirname "$bin")" "$(basename "$bin")"
      echo "  → $asset"
    else
      echo "  ✗ $target FAILED"
      FAILED+=("$target")
    fi
  fi
done

echo ""
echo "── results ──────────────────────────────────────────────"
ls -lh "$OUT_DIR"/unai-v"${VERSION}"-*.tar.gz 2>/dev/null || true

if [[ ${#FAILED[@]} -gt 0 ]]; then
  echo ""
  echo "FAILED targets: ${FAILED[*]}"
  exit 1
else
  echo "All targets built successfully."
fi
