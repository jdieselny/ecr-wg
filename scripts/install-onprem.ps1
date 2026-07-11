# SPDX-License-Identifier: Apache-2.0
# ECR-WG on-prem installer / smoke runner (Windows PowerShell)
#
# No cloud keys. No Vercel. Run from a local checkout.
#
#   powershell -ExecutionPolicy Bypass -File scripts/install-onprem.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/install-onprem.ps1 -SkipCleanroom
#   powershell -ExecutionPolicy Bypass -File scripts/install-onprem.ps1 -WithHostility

param(
    [switch]$SkipCleanroom,
    [switch]$WithHostility,
    [switch]$SkipPip
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

function Write-Banner {
    param([string]$Title)
    Write-Host ""
    Write-Host ("=" * 72) -ForegroundColor Cyan
    Write-Host ("  " + $Title) -ForegroundColor Cyan
    Write-Host ("=" * 72) -ForegroundColor Cyan
}

function Write-Ok {
    param([string]$Message)
    Write-Host ("  [OK]  " + $Message) -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host ("  [!!]  " + $Message) -ForegroundColor Yellow
}

function Write-Fail {
    param([string]$Message)
    Write-Host ("  [XX]  " + $Message) -ForegroundColor Red
}

$failures = 0
$results = New-Object System.Collections.Generic.List[string]

Write-Banner "ECR-WG on-prem installer - verifiable curtailment stack"
Write-Host ("  Root: " + $Root)
Write-Host "  Mode: local only (no hosted API keys required)"
Write-Host "  Repo: https://github.com/jdieselny/ecr-wg"

# --- prerequisites ----------------------------------------------------------
Write-Banner "1. Prerequisites"

try {
    $py = & python --version 2>&1 | Out-String
    $py = $py.Trim()
    Write-Ok ("Python: " + $py)
} catch {
    Write-Fail "Python not found on PATH"
    exit 1
}

$cargoOk = $false
try {
    $cv = & cargo --version 2>&1 | Out-String
    $cv = $cv.Trim()
    Write-Ok ("Cargo: " + $cv)
    $cargoOk = $true
} catch {
    Write-Warn "Cargo not found - cleanroom build will be skipped"
}

$nodeOk = $false
try {
    $nv = & node --version 2>&1 | Out-String
    $nv = $nv.Trim()
    Write-Ok ("Node: " + $nv)
    $nodeOk = $true
} catch {
    Write-Warn "Node not found - hostility lab needs Node if -WithHostility"
}

# --- Python deps ------------------------------------------------------------
Write-Banner "2. Python dependencies (flagship four-layer demo)"

$req = Join-Path $Root "examples\scitt_four_layer\requirements.txt"
if (-not $SkipPip) {
    try {
        & python -m pip install -q -r $req
        if ($LASTEXITCODE -ne 0) { throw "pip exit code $LASTEXITCODE" }
        Write-Ok "pip install -r examples/scitt_four_layer/requirements.txt"
        $results.Add("pip:OK") | Out-Null
    } catch {
        Write-Fail ("pip install failed: " + $_)
        $failures++
        $results.Add("pip:FAIL") | Out-Null
    }
} else {
    Write-Warn "Skipped pip (-SkipPip)"
    $results.Add("pip:SKIP") | Out-Null
}

# --- Flagship demo ----------------------------------------------------------
Write-Banner "3. Flagship: four-layer Proof-of-Curtailment (offline)"

try {
    & python examples\scitt_four_layer\demo.py
    if ($LASTEXITCODE -ne 0) { throw "demo exit $LASTEXITCODE" }
    Write-Ok "examples/scitt_four_layer/demo.py -> PASS"
    $results.Add("scitt_four_layer:PASS") | Out-Null
    $outDir = Join-Path $Root "examples\scitt_four_layer\out"
    if (Test-Path (Join-Path $outDir "bundle.json")) {
        Write-Ok "artifacts: packing_slip / bill_of_lading / cogobj / bundle under out/"
    }
} catch {
    Write-Fail ("four-layer demo failed: " + $_)
    $failures++
    $results.Add("scitt_four_layer:FAIL") | Out-Null
}

# --- L5 + L7 reference ------------------------------------------------------
Write-Banner "4. COSA L5 + EMILIA L7 composition (offline)"

try {
    & python examples\cosa\cosa_l5_l7.py --offline
    if ($LASTEXITCODE -ne 0) { throw "exit $LASTEXITCODE" }
    Write-Ok "examples/cosa/cosa_l5_l7.py --offline -> PASS"
    $results.Add("cosa_l5_l7:PASS") | Out-Null
} catch {
    Write-Fail ("L5+L7 demo failed: " + $_)
    $failures++
    $results.Add("cosa_l5_l7:FAIL") | Out-Null
}

# --- Cleanroom --------------------------------------------------------------
if (-not $SkipCleanroom) {
    Write-Banner "5. Independent Rust cleanroom (163 vector suite)"

    $vectorsCandidates = @()
    if ($env:EP_CONFORMANCE_VECTORS -and (Test-Path $env:EP_CONFORMANCE_VECTORS)) {
        $vectorsCandidates += $env:EP_CONFORMANCE_VECTORS
    }
    $sibling = Join-Path $Root "..\emilia-protocol\conformance\vectors"
    if (Test-Path $sibling) { $vectorsCandidates += (Resolve-Path $sibling).Path }
    $nested = Join-Path $Root "emilia-protocol\conformance\vectors"
    if (Test-Path $nested) { $vectorsCandidates += (Resolve-Path $nested).Path }

    if (-not $cargoOk) {
        Write-Warn "No cargo - skip cleanroom"
        $results.Add("cleanroom:SKIP_NO_CARGO") | Out-Null
    } elseif ($vectorsCandidates.Count -eq 0) {
        Write-Warn "Conformance vectors not found. Clone emilia-protocol beside ecr-wg or set EP_CONFORMANCE_VECTORS."
        Write-Warn "  Expected e.g. ..\emilia-protocol\conformance\vectors"
        $results.Add("cleanroom:SKIP_NO_VECTORS") | Out-Null
    } else {
        $env:EP_CONFORMANCE_VECTORS = $vectorsCandidates[0]
        Write-Ok ("vectors: " + $env:EP_CONFORMANCE_VECTORS)
        Push-Location (Join-Path $Root "rust\ep-cleanroom-verifier")
        try {
            & cargo build --release --bin conformance
            if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
            & python run_tests.py
            if ($LASTEXITCODE -ne 0) { throw "run_tests failed" }
            Write-Ok "cleanroom conformance -> PASS (expect 163/163)"
            $results.Add("cleanroom:PASS") | Out-Null
        } catch {
            Write-Fail ("cleanroom failed: " + $_)
            $failures++
            $results.Add("cleanroom:FAIL") | Out-Null
        } finally {
            Pop-Location
        }
    }
} else {
    Write-Warn "Cleanroom skipped (-SkipCleanroom)"
    $results.Add("cleanroom:SKIP") | Out-Null
}

# --- Hostility (optional) ---------------------------------------------------
if ($WithHostility) {
    Write-Banner "6. Hostility lab (optional)"
    $lab = Join-Path $Root "rust\ep-cleanroom-verifier\hostility-lab\hostility-rust-only.mjs"
    if (-not $nodeOk) {
        Write-Warn "Node required for hostility lab"
        $results.Add("hostility:SKIP_NO_NODE") | Out-Null
    } elseif (-not (Test-Path $lab)) {
        Write-Warn "hostility-lab not present (gitignored until hygiene handoff lands). See planning/composer_handoff_cleanroom_hygiene.md"
        $results.Add("hostility:SKIP_MISSING") | Out-Null
    } else {
        try {
            Push-Location (Join-Path $Root "rust\ep-cleanroom-verifier\hostility-lab")
            & node hostility-rust-only.mjs
            if ($LASTEXITCODE -ne 0) { throw "hostility exit $LASTEXITCODE" }
            Write-Ok "hostility-rust-only -> PASS"
            $results.Add("hostility:PASS") | Out-Null
        } catch {
            Write-Fail ("hostility failed: " + $_)
            $failures++
            $results.Add("hostility:FAIL") | Out-Null
        } finally {
            Pop-Location
        }
    }
}

# --- Summary ----------------------------------------------------------------
Write-Banner "RESULT"
foreach ($r in $results) {
    Write-Host ("  - " + $r)
}

Write-Host ""
Write-Host "  Flagship packet (after step 3):"
Write-Host "    examples/scitt_four_layer/out/packing_slip.json"
Write-Host "    examples/scitt_four_layer/out/bill_of_lading.json"
Write-Host "    examples/scitt_four_layer/out/cogobj.json"
Write-Host "    examples/scitt_four_layer/out/bundle.json"
Write-Host ""
Write-Host "  Docs:"
Write-Host "    https://github.com/jdieselny/ecr-wg"
Write-Host "    papers/05_ietf_cryptographic_grid_curtailment.md"
Write-Host "    planning/FLAGSHIP_VERIFIABLE_CURTAILMENT.md"
Write-Host ""

if ($failures -gt 0) {
    Write-Fail ("FAILED with $failures error(s) - do not claim green to a room until fixed.")
    exit 1
}

Write-Ok "ON-PREM SMOKE GREEN - stack is runnable without cloud API keys."
exit 0
