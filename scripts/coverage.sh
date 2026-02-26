#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE_DIR="$ROOT_DIR/target/coverage/profraw"
REPORT_DIR="$ROOT_DIR/target/coverage/report"
LCOV_FILE="$ROOT_DIR/target/coverage/lcov.info"
THRESHOLD="${COVERAGE_THRESHOLD:-80}"

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

THRESHOLD="$THRESHOLD" LCOV_FILE="$LCOV_FILE" python - <<'PY'
import os

threshold = float(os.environ.get("THRESHOLD", "80"))
total = 0
covered = 0
with open(os.environ["LCOV_FILE"], "r", encoding="utf-8") as f:
    for line in f:
        if line.startswith("DA:"):
            total += 1
            _, data = line.strip().split(":", 1)
            _, hits = data.split(",", 1)
            if int(hits) > 0:
                covered += 1

if total == 0:
    raise SystemExit("No coverage lines found in lcov report")

ratio = covered / total * 100
print(f"Line coverage: {ratio:.2f}% ({covered}/{total})")
if ratio < threshold:
    raise SystemExit(f"Coverage threshold not met: {ratio:.2f}% < {threshold:.2f}%")
PY
