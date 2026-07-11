#!/usr/bin/env python3
"""E2E harness: run the cleanroom Rust conformance binary over the public EP vector pack.

Vector discovery order:
  1. EP_CONFORMANCE_VECTORS (env)
  2. Sibling checkout: <repo>/../emilia-protocol/conformance/vectors
  3. Nested: <crate>/../../emilia-protocol/conformance/vectors
  4. Legacy: <crate>/../conformance/vectors
"""

from __future__ import annotations

import json
import os
import platform
import subprocess
import sys


def resolve_vectors_dir(script_dir: str) -> str:
    candidates: list[str] = []
    env = os.environ.get("EP_CONFORMANCE_VECTORS")
    if env:
        candidates.append(os.path.abspath(env))

    # ecr-wg/rust/ep-cleanroom-verifier -> ecr-wg/../emilia-protocol/...
    repo_root = os.path.abspath(os.path.join(script_dir, "..", ".."))
    candidates.append(
        os.path.abspath(os.path.join(repo_root, "..", "emilia-protocol", "conformance", "vectors"))
    )
    candidates.append(
        os.path.abspath(os.path.join(repo_root, "emilia-protocol", "conformance", "vectors"))
    )
    # legacy layout (crate sibling)
    candidates.append(
        os.path.abspath(os.path.join(script_dir, "..", "conformance", "vectors"))
    )
    candidates.append(
        os.path.abspath(os.path.join(script_dir, "..", "..", "conformance", "vectors"))
    )

    for path in candidates:
        if path and os.path.isdir(path):
            return path

    print("ERROR: conformance vectors directory not found.", file=sys.stderr)
    print("Set EP_CONFORMANCE_VECTORS or clone emilia-protocol beside ecr-wg:", file=sys.stderr)
    print("  export EP_CONFORMANCE_VECTORS=/path/to/emilia-protocol/conformance/vectors", file=sys.stderr)
    print("  # or", file=sys.stderr)
    print("  git clone https://github.com/emiliaprotocol/emilia-protocol ../emilia-protocol", file=sys.stderr)
    print("Tried:", file=sys.stderr)
    for path in candidates:
        print(f"  - {path}", file=sys.stderr)
    sys.exit(2)


def main() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    manifest_path = os.path.join(script_dir, "Cargo.toml")
    vectors_dir = resolve_vectors_dir(script_dir)

    # 16 suite files — current public pack is 163 vectors (2026-07-11)
    suite_files = [
        "receipts.v1.json",
        "signoffs.v1.json",
        "quorum.v1.json",
        "revocation.exec.v1.json",
        "time-attestation.v1.json",
        "trust-receipt.exec.v1.json",
        "trust-receipt.timestamp-forms.v1.json",
        "provenance.exec.v1.json",
        "evidence-record.v1.json",
        "canonicalization.v1.json",
        "boundary.v1.json",
        "currency.v1.json",
        "initiator-attestation.v1.json",
        "consumption-proof.v1.json",
        "witness.v1.json",
        "timestamp-proof.v1.json",
    ]

    binary_name = "conformance.exe" if platform.system() == "Windows" else "conformance"
    debug_bin = os.path.join(script_dir, "target", "debug", binary_name)
    release_bin = os.path.join(script_dir, "target", "release", binary_name)

    binary_path = None
    if os.path.exists(release_bin):
        binary_path = release_bin
    elif os.path.exists(debug_bin):
        binary_path = debug_bin

    if not binary_path:
        print("Rust CLI verifier binary not found in target/debug or target/release.")
        print(f"Attempting to compile via cargo build --manifest-path {manifest_path} --bin conformance...")
        try:
            result = subprocess.run(
                ["cargo", "build", "--manifest-path", manifest_path, "--bin", "conformance", "--release"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            if result.returncode != 0:
                print("Error: Compilation failed via cargo build.", file=sys.stderr)
                print(result.stderr, file=sys.stderr)
                sys.exit(1)

            if os.path.exists(release_bin):
                binary_path = release_bin
            elif os.path.exists(debug_bin):
                binary_path = debug_bin
            else:
                print(
                    "Error: cargo build succeeded, but binary was not found.",
                    file=sys.stderr,
                )
                sys.exit(1)

            print(f"Compilation successful! Compiled binary found at: {binary_path}\n")
        except FileNotFoundError:
            print("Error: 'cargo' executable not found. Please install Rust and Cargo.", file=sys.stderr)
            sys.exit(1)
        except Exception as e:
            print(f"Error during compilation attempt: {e}", file=sys.stderr)
            sys.exit(1)

    print(f"Using binary at: {binary_path}")
    print(f"Scanning for vectors under: {vectors_dir}\n")

    total_vectors = 0
    passed_vectors = 0
    mismatches = []
    missing_suites = []

    for filename in suite_files:
        filepath = os.path.join(vectors_dir, filename)
        if not os.path.exists(filepath):
            print(f"Suite file not found: {filename} (skipped)", file=sys.stderr)
            missing_suites.append(filename)
            continue

        try:
            with open(filepath, "r", encoding="utf-8") as f:
                suite_data = json.load(f)
        except Exception as e:
            print(f"Error reading vector file {filename}: {e}", file=sys.stderr)
            continue

        vectors = suite_data.get("vectors", [])
        suite_name = suite_data.get("suite", filename)
        print(f"Suite: {suite_name} ({len(vectors)} vectors)")

        try:
            res = subprocess.run(
                [binary_path, filepath],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )
        except Exception as e:
            print(f"  Error invoking binary on {filename}: {e}", file=sys.stderr)
            mismatches.append(f"{filename}: Failed to invoke binary ({e})")
            continue

        if res.returncode != 0:
            print(f"  Warning: Rust binary exited with code {res.returncode}")
            if res.stderr:
                print(f"  Stderr: {res.stderr.strip()}")

        try:
            results = json.loads(res.stdout)
        except Exception as e:
            print(f"  Error: Failed to parse stdout as JSON: {e}", file=sys.stderr)
            if res.stdout:
                print(f"  Raw output: {res.stdout.strip()}", file=sys.stderr)
            mismatches.append(f"{filename}: Malformed output (JSON parse error)")
            continue

        got_map = {}
        if isinstance(results, list):
            for item in results:
                if isinstance(item, dict) and "id" in item:
                    got_map[item["id"]] = item.get("valid")

        def pad(s, width):
            return str(s).ljust(width)

        print(f"  {pad('Vector ID', 40)} | {pad('Expect', 8)} | {pad('Got', 8)} | Status")
        print("  " + "-" * 75)

        for v in vectors:
            v_id = v.get("id")
            if not v_id:
                continue
            expected = v.get("expect", {}).get("valid", False)
            got = got_map.get(v_id)
            total_vectors += 1

            expect_str = "valid" if expected else "reject"
            got_str = "valid" if got is True else "reject" if got is False else "None"

            if got is None:
                status = "FAIL (missing ID)"
                mismatches.append(f"{filename}::{v_id} (Expected {expect_str}, got missing)")
            elif got == expected:
                status = "PASS"
                passed_vectors += 1
            else:
                status = "FAIL"
                mismatches.append(f"{filename}::{v_id} (Expected {expect_str}, got {got_str})")

            print(f"  {pad(v_id, 40)} | {pad(expect_str, 8)} | {pad(got_str, 8)} | {status}")
        print()

    print(f"E2E Conformance Summary: {passed_vectors}/{total_vectors} vectors passed.")
    if missing_suites:
        print(f"⚠️  Missing suite files: {', '.join(missing_suites)}")
    if mismatches:
        print(f"❌ Verification failed with {len(mismatches)} mismatch(es):")
        for m in mismatches:
            print(f"  - {m}")
        sys.exit(1)

    if total_vectors == 0:
        print("❌ No vectors executed.", file=sys.stderr)
        sys.exit(1)

    print("✅ All conformance vectors verified successfully!")
    sys.exit(0)


if __name__ == "__main__":
    main()
