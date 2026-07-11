#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# ECR-WG on-prem installer / smoke runner (POSIX)
# No cloud keys. Run from a local checkout of https://github.com/jdieselny/ecr-wg
#
#   ./scripts/install-onprem.sh
#   ./scripts/install-onprem.sh --skip-cleanroom
#   ./scripts/install-onprem.sh --with-hostility

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SKIP_CLEANROOM=0
WITH_HOSTILITY=0
SKIP_PIP=0
for arg in "$@"; do
  case "$arg" in
    --skip-cleanroom) SKIP_CLEANROOM=1 ;;
    --with-hostility) WITH_HOSTILITY=1 ;;
    --skip-pip) SKIP_PIP=1 ;;
  esac
done

ok()   { printf '  [OK]  %s\n' "$*"; }
warn() { printf '  [!!]  %s\n' "$*"; }
fail() { printf '  [XX]  %s\n' "$*"; }
banner() {
  echo
  echo "========================================================================"
  echo "  $*"
  echo "========================================================================"
}

failures=0
results=()

banner "ECR-WG on-prem installer — verifiable curtailment stack"
echo "  Root: $ROOT"
echo "  Repo: https://github.com/jdieselny/ecr-wg"

banner "1. Prerequisites"
if ! command -v python3 >/dev/null 2>&1 && ! command -v python >/dev/null 2>&1; then
  fail "Python not found"; exit 1
fi
PY=python3; command -v python3 >/dev/null 2>&1 || PY=python
ok "Python: $($PY --version 2>&1)"

CARGO_OK=0
if command -v cargo >/dev/null 2>&1; then
  ok "Cargo: $(cargo --version 2>&1)"; CARGO_OK=1
else
  warn "Cargo not found — cleanroom build will be skipped"
fi

NODE_OK=0
if command -v node >/dev/null 2>&1; then
  ok "Node: $(node --version 2>&1)"; NODE_OK=1
else
  warn "Node not found — hostility lab needs Node if --with-hostility"
fi

banner "2. Python dependencies"
if [[ "$SKIP_PIP" -eq 0 ]]; then
  $PY -m pip install -q -r examples/scitt_four_layer/requirements.txt
  ok "pip install -r examples/scitt_four_layer/requirements.txt"
  results+=("pip:OK")
else
  warn "Skipped pip"; results+=("pip:SKIP")
fi

banner "3. Flagship: four-layer Proof-of-Curtailment (offline)"
if $PY examples/scitt_four_layer/demo.py; then
  ok "scitt_four_layer → PASS"; results+=("scitt_four_layer:PASS")
else
  fail "scitt_four_layer failed"; failures=$((failures+1)); results+=("scitt_four_layer:FAIL")
fi

banner "4. COSA L5 + EMILIA L7 composition (offline)"
if $PY examples/cosa/cosa_l5_l7.py --offline; then
  ok "cosa_l5_l7 → PASS"; results+=("cosa_l5_l7:PASS")
else
  fail "cosa_l5_l7 failed"; failures=$((failures+1)); results+=("cosa_l5_l7:FAIL")
fi

if [[ "$SKIP_CLEANROOM" -eq 0 ]]; then
  banner "5. Independent Rust cleanroom"
  VECTORS="${EP_CONFORMANCE_VECTORS:-}"
  if [[ -z "$VECTORS" || ! -d "$VECTORS" ]]; then
    for c in \
      "$ROOT/../emilia-protocol/conformance/vectors" \
      "$ROOT/emilia-protocol/conformance/vectors"; do
      [[ -d "$c" ]] && VECTORS="$c" && break
    done
  fi
  if [[ "$CARGO_OK" -eq 0 ]]; then
    warn "No cargo — skip"; results+=("cleanroom:SKIP_NO_CARGO")
  elif [[ -z "$VECTORS" || ! -d "$VECTORS" ]]; then
    warn "Vectors not found; set EP_CONFORMANCE_VECTORS or clone emilia-protocol beside ecr-wg"
    results+=("cleanroom:SKIP_NO_VECTORS")
  else
    export EP_CONFORMANCE_VECTORS="$VECTORS"
    ok "vectors: $VECTORS"
    (
      cd rust/ep-cleanroom-verifier
      cargo build --release --bin conformance
      $PY run_tests.py
    ) && { ok "cleanroom → PASS"; results+=("cleanroom:PASS"); } \
      || { fail "cleanroom failed"; failures=$((failures+1)); results+=("cleanroom:FAIL"); }
  fi
else
  warn "Cleanroom skipped"; results+=("cleanroom:SKIP")
fi

if [[ "$WITH_HOSTILITY" -eq 1 ]]; then
  banner "6. Hostility lab"
  LAB=rust/ep-cleanroom-verifier/hostility-lab/hostility-rust-only.mjs
  if [[ "$NODE_OK" -eq 0 ]]; then
    warn "Node required"; results+=("hostility:SKIP_NO_NODE")
  elif [[ ! -f "$LAB" ]]; then
    warn "hostility-lab missing (see planning/composer_handoff_cleanroom_hygiene.md)"
    results+=("hostility:SKIP_MISSING")
  else
    (cd rust/ep-cleanroom-verifier/hostility-lab && node hostility-rust-only.mjs) \
      && { ok "hostility → PASS"; results+=("hostility:PASS"); } \
      || { fail "hostility failed"; failures=$((failures+1)); results+=("hostility:FAIL"); }
  fi
fi

banner "RESULT"
for r in "${results[@]}"; do echo "  - $r"; done
echo
echo "  Flagship packet: examples/scitt_four_layer/out/{packing_slip,bill_of_lading,cogobj,bundle}.json"
echo "  Repo: https://github.com/jdieselny/ecr-wg"
echo

if [[ "$failures" -gt 0 ]]; then
  fail "FAILED with $failures error(s)"
  exit 1
fi
ok "ON-PREM SMOKE GREEN — stack is runnable without cloud API keys."
exit 0
