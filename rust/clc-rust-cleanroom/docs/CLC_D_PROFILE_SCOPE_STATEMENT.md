# CLC-D profile layer — scope statement

**Written:** 2026-11-04
**Covers:** `--clc-d-profile`, commit `fd78ebc` onward
**Status:** local, unpushed, no claim made to the project

This note exists so that one specific thing cannot be misread: the single
`not applicable` case in the profile run is **our gap, not a defect in the
corpora**. It is stated here, in the runner's own output, and in the commit, so a
reader cannot encounter the number without the explanation.

---

## What was run

| Corpus | Cases | Passed | Failed | N/A |
|---|---:|---:|---:|---:|
| `containment-property-cases.json` (§13.4 property) | 784 | 784 | 0 | 0 |
| `reason-collapse-vectors.json` (§13.5) | 3 | 2 | 0 | 1 |
| **Total** | **787** | **786** | **0** | **1** |

Both vendored byte-exact from `varwof/capability@80e483f` and SHA-256 pinned in
`vectors/clc-d/PINS.sha256`. Base claim untouched: `--clc-d` still runs only the
64 + 15 corpora recorded in `reg-2026-003`.

## The 784, honestly qualified

The property cases carry no expected values; they carry a property. So the pass
count alone is not the evidence, and reporting it alone would be the vacuous-pass
trap in a new costume.

- **120 of 784** cases have `Contains(parent, child)` true, so the forward
  closure implication is reachable.
- **664 of 784** have it false. The implication is never reached; those cases
  evidence totality and antisymmetry only.
- **588** operation-level witnesses were actually exercised across the 120
  reachable cases, 26 at the deepest pair.

The runner reports the witness count (`witnesses exercised  588`) precisely so a
reader can see how much of the 784 was load-bearing.

## The one N/A — and whose gap it is

`rc-003` names the reason code `delegation_mode_not_narrower`, described in the
vector itself as a **binding-profile pre-check**. This engine:

- implements the `capabilities` projection;
- implements **no** binding-profile pre-check;
- therefore **never emits that code**.

Because it never emits the code, it cannot evidence whether that code may be
collapsed to `capability_not_authorized`. Reporting `PASS` would assert
something the engine never tested.

**So the direction of the gap is: our implementation, not her corpus.** The
vector is well-formed and correct. A profile that carries delegation mode would
be able to evidence it. We cannot, and saying so is the only honest outcome.

This is stated in three places so it cannot drift:

1. the per-case line, which reads "This is a gap in this engine, not in the
   vector…";
2. the suite table, which prints
   `gap   stated by this implementation, not a defect in the corpora`;
3. `SuiteReport::failed()`, which no longer counts an inapplicable case as a
   failure — otherwise the runner would report a false defect.

## What this means for a future CLC-D profile claim

A profile claim that included this layer would carry a **stated residual gap**:
delegation-mode collapse is not evidenced. `CONFORMANCE.md` §1 already requires
residual gaps to be listed, and `GOVERNANCE.md` rewards the same. A stated gap is
not a weakness in a conformance report; an unstated one is.

If the project ever wants that gap closed, the work is: implement the
binding-profile pre-check that produces `delegation_mode_not_narrower`, and
re-run. That is separate, larger, and needs the §13.4.5 profile semantics this
engine deliberately does not carry.

## Still not run, still not claimed

- containment crosswalk (47) and authorize crosswalk (1) — need carrier mapping;
- mode lattice (5) — needs profile semantics;
- CLC-E — separate profile.

## Negative control for this layer

After the corpus passed, the engine was temporarily perturbed to report a wider
child as contained. The corpus caught it: **218 failures**, exit 1, naming case
and class mismatch. Restored and verified byte-identical to `HEAD`.

Repeat note: a restored file can carry an older mtime, so `cargo` may not
rebuild and the next run silently tests the perturbed binary. Touch the file or
force a clean build before trusting the post-restore result.
