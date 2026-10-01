#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
cargo build --locked -p nexis-gateway
pnpm --filter @wisdoverse/nexus-sdk build
"${NEXIS_SMOKE_PYTHON:-python3}" tests/smoke/m1_gateway.py
