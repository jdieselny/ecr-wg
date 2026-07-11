# Cleanroom JCS Edge Case Matrix: RFC 8785 for EP Verifier

**Source Material:**  
- `emilia-protocol/conformance/vectors/canonicalization.v1.json` (35 vectors)  
- `emilia-protocol/standards/posted/draft-schrock-ep-authorization-receipts-06.md` (JCS + profile sections)  
- RFC 8785 (JSON Canonicalization Scheme) — https://www.rfc-editor.org/rfc/rfc8785.txt  

**Target Audience:** Rust developer implementing `fn canonicalize(v: &serde_json::Value) -> Result<String, VerifyError>` (or bytes) in a strict cleanroom verifier.  

**Integrity Constraint:** This document derives exclusively from the IETF draft, the public conformance vectors, and RFC 8785. No reference implementation source was consulted.

**Purpose:** Provide an exhaustive matrix of edge cases so the Rust JCS layer produces *byte-identical* canonical forms (and thus matching SHA-256 digests) and correctly rejects all malformed cases. Every signature in EP is computed over JCS-canonical bytes (action, context, etc.). Divergence = signature failure across languages.

---

## 1. Overview of JCS in EP

EP mandates RFC 8785 JCS for:
- Computing `action_hash = SHA-256( JCS(Action Object) )`
- Computing `context_hash = SHA-256( JCS(Authorization Context) )`
- Any other signed or anchored structures.

The canonical form is then hashed (not the original JSON). The verifier must:
1. Parse input (with strict gates per EP profile).
2. Produce the exact JCS string (no whitespace, sorted keys, ECMAScript number/string rules).
3. SHA-256 the UTF-8 bytes of that string.
4. Compare to expected or use for signature verify.

EP adds a profile on top of RFC 8785 + I-JSON (RFC 7493):
- Only integers (no non-integer reals).
- Magnitude ≤ 2^53-1 (safe integer limit for IEEE 754 double).
- No duplicate member names (compared after escape decoding / unescaping).
- No unpaired UTF-16 surrogates.
- Container nesting depth ≤ 64 (pinned bound).
- Reject on violation before or at canonicalization + digest step.

The 35 vectors in `canonicalization.v1.json` are the machine-checked pins for cross-language agreement.

---

## 2. Key Sorting Semantics (RFC 8785 §3.2.3)

Critical for objects (including nested).

**Rules (must implement exactly):**
- Sort **after** recursing into child objects (properties sorted recursively).
- Arrays: preserve element order; only recurse into any object elements for sorting their props.
- Property names are sorted in their **raw (unescaped)** form.
- Names are treated as arrays of **UTF-16 code units** (unsigned 16-bit integers).
- Comparison: pure numeric `<` on the code unit values (no locale, no Unicode normalization, no codepoint collation).
- If one is a proper prefix of the other, the shorter precedes.
- Example sort order from RFC (mixed ASCII + astral + controls):
  - Keys after sort: "\r", "1", "\u0080", "\u00f6", "\u20ac", "\ud83d\ude00", "\ufb33" (corresponding labels shown in RFC).

**UTF-16 specifics for Rust:**
```rust
fn utf16_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let a16: Vec<u16> = a.encode_utf16().collect();
    let b16: Vec<u16> = b.encode_utf16().collect();
    a16.cmp(&b16)  // unsigned u16 comparison
}
```
- This differs from Rust `str::cmp` (Unicode scalar / codepoint) for astral plane characters (U+10000+).
- Surrogate pairs in JSON escapes (`\ud83d\ude00`) must be handled as two u16 units during parse + sort.

**Astral / non-BMP keys:** See vector `accept_astral_key_utf16_sort_order`.

**Implementation note for serde_json::Value:** Iterate `obj.iter()`, collect, sort with the UTF-16 comparator, then emit.

---

## 3. Number Serialization (RFC 8785 §3.2.2.3 + I-JSON + EP profile)

- Based on ECMAScript / IEEE 754 double-precision (64-bit).
- Use the exact algorithm from ECMA-262 7.1.12.1 (incl. Note 2 "round to even" etc.). References: V8 or Ryu implementations are compatible.
- **No NaN, no Infinity** — reject.
- **I-JSON + EP constraints (MUST enforce):**
  - Integers only for numbers in this profile (reject 1.5 etc.).
  - Magnitude ≤ 9007199254740991 (2^53-1). Larger integers or large exponents that escape safe range are rejected.
- **-0 handling:** Both `-0` and `-0.0` (and equivalents) serialize to `0` (positive zero) in canonical form. See vectors for `-0` / `-0.0`.
- **Aliases that must collapse to same canonical:**
  - `1`, `1.0`, `1e0`, `1E0` → all become `1`
  - `0`, `-0`, `0.0`, `-0.0` → `0`
- Output: no leading `+`, no unnecessary `.0`, lowercase `e` for exponents, minimal form per ECMAScript rules.
- In Rust: after parsing to `serde_json::Number`, inspect `as_i64()` / `as_u64()` / `as_f64()`. If the value is integral within safe range, emit as integer string. Otherwise apply profile reject.

See RFC Appendix B for many sample IEEE 754 → JSON serializations.

**EP reject vectors pin the boundary:**
- `9007199254740991` (max safe) = accept
- `9007199254740992` = reject
- `1e21` = reject (unsafe)
- `1.5` = reject (non-integer)

---

## 4. String Escaping (RFC 8785 §3.2.2.2)

- Enclose in `"`
- Escape ONLY:
  - `"` → `\"`
  - `\` → `\\`
  - U+0000–U+001F controls:
    - U+0008 → `\b`
    - U+0009 → `\t`
    - U+000A → `\n`
    - U+000C → `\f`
    - U+000D → `\r`
    - All other controls → `\u00xx` (lowercase hex, 4 digits)
- Everything else (including most Unicode) emitted literally (UTF-8 in final bytes).
- **MUST NOT** escape `/` as `\/` (unless the input had it that way? — but JCS produces the minimal).
- **Unpaired / lone surrogates:** MUST reject (U+D800–U+DBFF without low, or U+DC00–U+DFFF without high, or reversed). JSON escapes like `\ud800` alone are invalid for JCS.
- **Unicode normalization:** JCS does **NOT** normalize (NFC vs NFD produce different digests on purpose). See vectors for café examples.
- Escaped form vs literal must produce identical canonical when they decode to same code points (e.g. `\u00e9` == é).

**Format chars & specials tested:**
- `\u2028`, `\u2029` (line separators), `\u007f`, `\ufeff` (BOM), `\u200b` (zero-width), bidi overrides (`\u202e` RLO + `\u202c` PDF), etc. — emitted as `\uXXXX` or raw as appropriate, but consistently.

---

## 5. Structural Rules

- **No whitespace** anywhere between tokens (no spaces after `:`, `,`, around `{}` `[]`).
- **Recursive:** Objects inside arrays/objects get their keys sorted.
- **Duplicate member names:** MUST reject. Comparison is after unescaping the key strings (see `reject_duplicate_key_escape_alias`, `reject_duplicate_key_non_bmp_alias`).
- **Nesting depth:** ≤ 64 containers deep is accepted; 65+ rejected. (The vector uses a deep `"d"` chain to pin exactly.)
- **Arrays:** Element order is preserved exactly (never sorted).
- **Profile predicate (EP):** After parse, every scalar must be string/bool/null or integer (safe). Reals that are non-integral are rejected even if they would canonize.

The conformance vectors test the combination of:
- Strict parse gate (dups, surrogates, depth)
- I-JSON / EP profile
- Exact JCS serialization + SHA-256(digest of UTF-8 canonical bytes)

In the verifier crate, the harness runner often applies some gates; the JCS function itself must not produce a canonical form for bad input and must surface errors for dups/surrogates etc. if it sees them.

---

## 6. Vector-by-Vector Breakdown (canonicalization.v1.json)

All digests are SHA-256 of the UTF-8 bytes of the exact JCS string.

### Accept Vectors (valid=true) — must produce exact digest

1. **accept_nfc_composed**  
   Input: `{"note":"café"}` (precomposed é = U+00E9)  
   Digest: `a84c174531ab46d58aaeb9c85aed22981d418f25bead412cd282e97f427a0ba1`  
   Tests: NFC literal; no normalization applied by JCS.

2. **accept_nfd_decomposed**  
   Input: `{"note":"café"}` (e + combining U+0301)  
   Digest: `a959c3552a14d635acca3d4315e9097d11a2ff9d4ab2185dbfe4e66a90b06ed0`  
   Tests: NFD produces *different* digest from NFC (intentional).

3. **accept_escape_alias_of_nfc**  
   Input: `{"note":"caf\u00e9"}`  
   Digest: `a84c174531ab46d58aaeb9c85aed22981d418f25bead412cd282e97f427a0ba1` (same as #1)  
   Tests: `\u00e9` decodes to same codepoint; identical canonical.

4. **accept_angstrom_sign_not_normalized**  
   Input: `{"unit":"Å"}` (U+212B)  
   Digest: `f560f24d1654b3584612956a4d8a4c25961e3305f05ad8a2c6e6a5365b76e416`  
   Tests: Angstrom sign not auto-NFC'd to Å by JCS.

5. **accept_latin_a_ring_distinct**  
   Input: `{"unit":"Å"}` (U+00C5)  
   Digest: `08eb5b93c008738e0af881f0227f44b24ddb73bb7c3c12b74a4dcd9e0e8683bb`  
   Tests: Distinct from the angstrom vector.

6. **accept_bidi_rlo_override_raw**  
   Input: `{"payee":"acme\u202ereversed\u202c"}`  
   Digest: `e6438b4f8dc53adfcb492fd6a196c1a17fd25a019b777e76e4ddb35523388d46`  
   Tests: Bidi controls (RLO/PDF) preserved raw in string.

7. **accept_control_chars_escaped**  
   Input: `{"s":"\u0000\u0007\u001f"}`  
   Digest: `d923ab32e713e672a0eec4249db62facaddb155c1ba7bb215e8a3c36ac451c93`  
   Tests: Low controls use `\u00xx` form.

8. **accept_shorthand_escapes**  
   Input: `{"s":"\b\t\n\f\r"}`  
   Digest: `b087fcbc53f88174f9c3ccf7d6cb1c3906ab32d0198eb62c3e541c4aebdcdad0`

9. **accept_long_escape_alias_of_shorthand**  
   Input: `{"s":"\u0008\u0009\u000a\u000c\u000d"}`  
   Digest: same as #8  
   Tests: Shorthand vs long escape for the 5 special controls must match.

10. **accept_format_chars_raw**  
    Input: `{"s":"\u2028\u2029\u007f\ufeff\u200b"}`  
    Digest: `9a9d531e258f511a4c2b113d9da41b2d647b11a7599125ec51cf46026470b0ff`  
    Tests: Line sep, del, BOM, ZWSP etc.

11–19 are **rejects** (see below).

20. **accept_non_bmp_escaped_pair**  
    Input: `{"s":"\ud83d\ude00"}`  
    Digest: `f9949e1006d1ca22bc0b60ea94f09779f11a8f0a29bae250fc7ab313d879f5e7`

21. **accept_non_bmp_literal_alias**  
    Input: `{"s":"😀"}`  
    Digest: same  
    Tests: Escaped surrogate pair vs literal astral char → identical.

22. **accept_astral_key_utf16_sort_order**  
    Input: `{"｡":true,"😀":1}`  
    Digest: `bb5ef4528fd81606eabbfe0eb7c25784e924c22a03817fac46366381d32da411`  
    **Critical test for UTF-16 sort order** (not scalar codepoint order).

23. **accept_integer_one**  
    Input: `{"n":1}`  
    Digest: `2bfd14f43d17fc7cea24e0917a8879b4b2f880b8baeec1b9d90fbaad655e71bd`

24. **accept_number_alias_one_point_zero**  
    Input: `{"n":1.0}` → same digest  
25. **accept_number_alias_exponent**  
    Input: `{"n":1e0}` → same digest  
    Tests: All aliases produce identical `1` in canonical.

26. **accept_negative_zero_integer**  
    Input: `{"n":-0}`  
    Digest: `f3013f933b9fb80ab6d995e7ad9da36f683837ba1d81e950c943d40111eac2f0`

27. **accept_negative_zero_real_alias**  
    Input: `{"n":-0.0}` → same  
    Tests: -0 variants → `0`

28. **accept_max_safe_integer**  
    Input: `{"n":9007199254740991}`  
    Digest: `e1da48c6a6089f06ecb4e0a2259e658e3786b2420f52baccdf929ec6460d7b41`

29–31 are rejects.

32. **accept_key_order_whitespace_alias**  
    Input: `{ "b" : 2 ,\n  "a" : 1 }` (with ws + newlines)  
    Digest: `43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777`

33. **accept_key_order_canonical_twin**  
    Input: `{"a":1,"b":2}` → same digest  
    Tests: Parser must ignore original key order + whitespace; output is always sorted no-ws.

34. **accept_nested_depth_at_limit**  
    Input: 64-deep `{"d":{...{"d":1}...}}` (exactly at bound)  
    Digest: `3d521fae0e2ae82f37583c72212182e80a3feb5140dfa5bb61492804112644a7`

35. **reject_nested_depth_over_limit** is reject.

### Reject Vectors (valid=false) — must be rejected (parse / profile / canon gate)

11. **reject_duplicate_key_literal** — `{"a":1,"a":2}`  
12. **reject_duplicate_key_escape_alias** — `{"a":1,"\u0061":2}` (after unescape, same key)  
13. **reject_duplicate_key_nested**  
14. **reject_duplicate_key_non_bmp_alias** — astral key + its escaped surrogate alias  
15. **reject_lone_high_surrogate** — `"\ud800"`  
16. **reject_lone_low_surrogate** — `"\udc00"`  
17. **reject_reversed_surrogate_pair** — `"\udc00\ud800"`  
18. **reject_high_surrogate_then_bmp_escape** — `"\ud800\u0041"` (incomplete pair)  
19. **reject_lone_surrogate_in_member_name** — key `"\ud800"`

29. **reject_unsafe_integer_2_53** — 9007199254740992  
30. **reject_unsafe_large_exponent** — 1e21  
31. **reject_non_integer_real** — 1.5  
35. **reject_nested_depth_over_limit** — 65-deep chain

Also several other reject vectors listed in the suite (lone surrogates in names/strings, etc.).

---

## 7. Rust Implementation Recommendations

- Implement `canonicalize` by walking `&Value` and building the string per above rules (do not rely on `serde_json` default serialization).
- For numbers: prefer integer emission when `fract() == 0.0` within safe range.
- For keys: always use the UTF-16 vec cmp for sort (even if most keys are ASCII).
- Add strict checks inside or before canon:
  - Detect duplicate keys (you may need a custom parser or post-parse walker that tracks seen unescaped keys).
  - Detect lone surrogates in strings/keys (scan decoded strings or during escape).
  - Depth counter during recursion.
  - Integer-only + safe range for numbers.
- On any reject condition: return `Err(VerifyError::JcsError(...))` or specific variant; the vector harness treats non-success + `expect.valid == false` as pass for reject vectors.
- Final output for digest: `sha256( canonical_string.as_bytes() )` and hex-compare.
- Test the 35 vectors directly (see the harness in the ep-cleanroom-verifier crate).
- Cross-check against RFC Appendix B number samples if expanding beyond current vectors.

---

## 8. References & Next Steps

- RFC 8785 full text (especially 3.2.2–3.2.4 and Appendices A/B).
- EP draft sections on action_hash, context_hash, "canonicalize(...)", and the conformance profile paragraph (~line 1440+).
- The 35 vectors are the ground truth.
- Once JCS is solid, integrate into `verify_signed_envelope`, receipt verification, Merkle leaves, etc.
- Related handoffs: reject_autopsy, merkle_algorithm, overall spec extraction.

This matrix + the existing `cleanroom-verifier-spec-extraction.md` give a complete blueprint for the JCS layer of the Rust cleanroom verifier.

**Status for this handoff:** Complete when this document is committed to `planning/cleanroom-jcs-edge-cases.md` and the Rust JCS passes the full canonicalization suite without divergence.
