#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE_DIR="$ROOT_DIR/target/coverage/profraw"
REPORT_DIR="$ROOT_DIR/target/coverage/report"
LCOV_FILE="$ROOT_DIR/target/coverage/lcov.info"

if ! command -v grcov >/dev/null 2>&1; then
  echo "grcov is not installed. Install with: cargo install grcov"
  exit 1
fi

rm -rf "$ROOT_DIR/target/coverage"
mkdir -p "$PROFILE_DIR" "$REPORT_DIR"

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="$PROFILE_DIR/nexis-%p-%m.profraw"

cd "$ROOT_DIR"

cargo test --workspace

grcov "$ROOT_DIR" \
  --binary-path "$ROOT_DIR/target/debug/deps" \
  --source-dir "$ROOT_DIR" \
  --output-type html \
  --output-path "$REPORT_DIR" \
  --branch \
  --ignore-not-existing \
  --ignore "/*" \
  --ignore "*/tests/*" \
  --ignore "*/target/*"

grcov "$ROOT_DIR" \
  --binary-path "$ROOT_DIR/target/debug/deps" \
  --source-dir "$ROOT_DIR" \
  --output-type lcov \
  --output-path "$LCOV_FILE" \
  --branch \
  --ignore-not-existing \
  --ignore "/*" \
  --ignore "*/tests/*" \
  --ignore "*/target/*"

echo "Coverage report generated:"
echo "  HTML: $REPORT_DIR/index.html"
echo "  LCOV: $LCOV_FILE"
