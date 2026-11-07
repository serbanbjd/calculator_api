# ADR-0003: Decimal number representation

Status: Proposed · Date: 2026-11-07
Drivers: NFR-3 (exact decimals, Q-1), FR-1 – FR-10, FR-13

## Context

Serban decided (Q-1) that `0.1 + 0.2` must be exactly `0.3`. Binary floating
point (`f64`) cannot do that, so we need a decimal type. It must also support
square root and fractional powers (FR-5, FR-6), detect overflow (FR-10), and
read JSON numbers without first turning them into `f64` (otherwise `0.1` is
already wrong before we compute).

## Options considered

### Option A: `f64` plus rounding for display

- Pros: stdlib, fast, has `sqrt` and `powf`.
- Cons: `0.1` is not representable; rounding the output hides the error in
  simple cases but not in general. Fails Q-1. Rejected.

### Option B: `rust_decimal` (96-bit integer mantissa, scale 0–28)

- Pros: exact decimal arithmetic, `checked_*` operations return `None` on
  overflow, `maths` feature adds `sqrt` and `powd`, parses from strings
  exactly, widely used (financial code).
- Cons: fixed limits: magnitude up to ~7.9 × 10^28 and at most 28 digits
  after the decimal point. Fractional powers go through `exp(ln(x))` and are
  approximate.

### Option C: `bigdecimal` (arbitrary precision)

- Pros: no fixed limits.
- Cons: no fractional power; we would write `exp`/`ln` ourselves. Unbounded
  precision also means unbounded CPU and memory per request, which NFR-2 and
  NFR-4 would then need extra limits for.

## Spike

Before deciding, a throwaway program ran the PRD acceptance examples through
`rust_decimal` 1.43 (`maths` feature):

| Input | Result | Matches PRD? |
|-------|--------|--------------|
| `0.1 + 0.2` | `0.3` | Yes |
| `1 ÷ 3` | `0.3333333333333333333333333333` | Yes |
| `1 ÷ 300` | `0.0033333333333333333333333333` (26 significant digits) | **No**: limit is 28 decimal places, not 28 significant digits |
| `√2` | `1.4142135623730950488016887242` | Yes (≥ 20 digits correct) |
| `4 ^ 0.5` | `2.0000000000000000000000000003` | **No**: PRD says `2` |
| `-8 ^ 0.5` | `-2.8284271247461900976033774488` | **No**: must be `UNDEFINED_RESULT`; library does not check the domain |
| `2 ^ -1` | `0.50` | Needs trailing zeros removed |
| `10 ^ 30`, `1e20 × 1e10` | `None` | Yes, overflow detected |
| `-10 mod 3`, `5.5 mod 2` | `-1`, `1.5` | Yes, sign follows dividend |
| `-3 × 0` | `0` | Yes |
| `1e-40`, `1e400` as input | parse error | Yes, can be mapped to `INVALID_INPUT` |
| `serde_json` with `arbitrary_precision`, `{"a":0.1}` | raw text `0.1` kept | Yes, no `f64` on the way in |

## Decision

Option B, `rust_decimal` with the `maths` feature, plus `serde_json` with
`arbitrary_precision` so JSON numbers reach us as their original text.

The domain layer applies these rules around the library:

1. **Parse** operands from their text with `Decimal::from_str_exact` (or
   `from_scientific` for exponent notation). Parse failure, more than 28
   decimal places, or magnitude ≥ 10^28 → `INVALID_INPUT`.
2. **Compute** with `checked_*` operations only; `None` → `RESULT_OUT_OF_RANGE`.
   Results with magnitude ≥ 10^28 are also `RESULT_OUT_OF_RANGE` (the library's
   limit is ~7.9 × 10^28, the PRD's is 10^28).
3. **Check domains ourselves before computing**: divisor zero, `√` of a
   negative, `0 ^ negative`, negative base with a non-integer exponent.
4. **Round approximate results** (`√` and non-integer powers) to 20
   significant digits, half-to-even. That gives `4 ^ 0.5 = 2` and only digits
   we know are correct.
5. **Normalise** every result: strip trailing zeros (`0.50` → `0.5`) and turn
   `-0` into `0` (FR-13).

This needs a PRD amendment (docs change before code): NFR-3 said "28
significant digits", the library and rule 4 give "28 decimal places" and
"20 significant digits for `√` and fractional powers". The amendment is in the
same pull request for Serban to accept or reject.

## Consequences

- Positive: `0.1 + 0.2 = 0.3` end to end, including JSON input.
- Positive: overflow is a `None`, not a silent infinity; easy to map to FR-10.
- Negative: very small results lose significant digits (`1 ÷ 300` has 26) and
  anything below 10^-28 rounds to `0`. Acceptable for a calculator; documented
  in NFR-3.
- Negative: irrational results carry 20 digits, not 28.
- Negative: domain checks for power are our code, so they need their own unit
  and property tests (ADR-0005).
- Watch: `arbitrary_precision` changes how `serde_json::Value` stores numbers
  crate-wide; keep number parsing in one place.
