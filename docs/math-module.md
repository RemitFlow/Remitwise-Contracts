# Reusable Math Module

Contract arithmetic lives in `src/math.rs`. Entrypoints and storage helpers
should use this module instead of implementing arithmetic locally so overflow
and rounding behavior remain consistent and auditable.

## Checked operations

- `checked_add_amount` and `checked_sub_amount` return `None` when an `i128`
  token amount would overflow.
- `checked_increment` returns `None` when a `u64` counter cannot be incremented.
- Callers must translate `None` into the contract error appropriate to their
  operation. State-changing code must not silently clamp invalid inputs.

## Saturating aggregates

`saturating_add_amount` and `saturating_add_with_cap` are intended for
read-only totals and counters where a bounded result is preferable to a panic.
They must not be used to decide whether a state-changing financial operation is
valid.

## Fee calculations

`calculate_fee(amount, basis_points)` calculates a fee where 10,000 basis
points equals 100%. It accepts non-negative amounts and rates from 0 through
10,000, rounds fractional token units down, and returns `None` for invalid
inputs.

The implementation avoids multiplying the full amount by the rate. It splits
the amount into quotient and remainder first, so even `i128::MAX` at 100% can
be calculated without intermediate overflow.

Examples:

| Amount | Basis points | Result |
| ---: | ---: | ---: |
| 10,000 | 100 (1%) | 100 |
| 999 | 250 (2.5%) | 24 |
| 42 | 10,000 (100%) | 42 |

Unit tests cover normal values, rounding, invalid inputs, and numeric
boundaries. Run them with `cargo test`.
