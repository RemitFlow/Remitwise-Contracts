//! Reusable arithmetic helpers for contract amounts, counters, and fees.
//!
//! Operations that affect contract validity return `Option` so callers must
//! handle overflow explicitly. Aggregate helpers saturate deliberately, which
//! keeps read-only tallies bounded when an exact result is no longer
//! representable.

/// Number of basis points in 100%.
pub const BASIS_POINTS_DENOMINATOR: u32 = 10_000;

/// Add two token amounts, returning `None` when the result is out of range.
pub const fn checked_add_amount(lhs: i128, rhs: i128) -> Option<i128> {
    lhs.checked_add(rhs)
}

/// Subtract one token amount from another, returning `None` on overflow.
pub const fn checked_sub_amount(lhs: i128, rhs: i128) -> Option<i128> {
    lhs.checked_sub(rhs)
}

/// Increment a counter, returning `None` when it has reached `u64::MAX`.
pub const fn checked_increment(value: u64) -> Option<u64> {
    value.checked_add(1)
}

/// Add two token amounts and clamp the result to the `i128` range.
pub const fn saturating_add_amount(lhs: i128, rhs: i128) -> i128 {
    lhs.saturating_add(rhs)
}

/// Add to an aggregate counter and clamp the result to `cap`.
pub const fn saturating_add_with_cap(value: u64, delta: u64, cap: u64) -> u64 {
    let sum = value.saturating_add(delta);
    if sum > cap {
        cap
    } else {
        sum
    }
}

/// Calculate a non-negative fee expressed in basis points.
///
/// Returns `None` when `amount` is negative or `basis_points` is greater than
/// 100%. The calculation splits the amount into quotient and remainder before
/// multiplying, avoiding intermediate overflow for every valid `i128` amount.
/// Fractional units are rounded down.
pub const fn calculate_fee(amount: i128, basis_points: u32) -> Option<i128> {
    if amount < 0 || basis_points > BASIS_POINTS_DENOMINATOR {
        return None;
    }

    let denominator = BASIS_POINTS_DENOMINATOR as i128;
    let rate = basis_points as i128;
    let quotient = amount / denominator;
    let remainder = amount % denominator;

    match quotient.checked_mul(rate) {
        Some(base) => match remainder.checked_mul(rate) {
            Some(remainder_product) => base.checked_add(remainder_product / denominator),
            None => None,
        },
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_amount_arithmetic_handles_boundaries() {
        assert_eq!(checked_add_amount(40, 2), Some(42));
        assert_eq!(checked_add_amount(i128::MAX, 1), None);
        assert_eq!(checked_sub_amount(40, 2), Some(38));
        assert_eq!(checked_sub_amount(i128::MIN, 1), None);
    }

    #[test]
    fn checked_increment_rejects_overflow() {
        assert_eq!(checked_increment(7), Some(8));
        assert_eq!(checked_increment(u64::MAX), None);
    }

    #[test]
    fn saturating_helpers_clamp_at_their_limits() {
        assert_eq!(saturating_add_amount(i128::MAX, 1), i128::MAX);
        assert_eq!(saturating_add_amount(i128::MIN, -1), i128::MIN);
        assert_eq!(saturating_add_with_cap(5, 10, 12), 12);
        assert_eq!(saturating_add_with_cap(5, 2, 12), 7);
        assert_eq!(saturating_add_with_cap(u64::MAX, 1, u64::MAX), u64::MAX);
    }

    #[test]
    fn calculate_fee_supports_common_rates_and_rounds_down() {
        assert_eq!(calculate_fee(10_000, 100), Some(100));
        assert_eq!(calculate_fee(999, 250), Some(24));
        assert_eq!(calculate_fee(42, 0), Some(0));
        assert_eq!(calculate_fee(42, BASIS_POINTS_DENOMINATOR), Some(42));
    }

    #[test]
    fn calculate_fee_handles_maximum_amount_without_overflow() {
        assert_eq!(
            calculate_fee(i128::MAX, BASIS_POINTS_DENOMINATOR),
            Some(i128::MAX)
        );
        assert_eq!(calculate_fee(i128::MAX, 1), Some(i128::MAX / 10_000));
    }

    #[test]
    fn calculate_fee_rejects_invalid_inputs() {
        assert_eq!(calculate_fee(-1, 100), None);
        assert_eq!(calculate_fee(100, BASIS_POINTS_DENOMINATOR + 1), None);
    }

    #[test]
    fn minimum_value_checked_add_sub_handles_boundaries() {
        // Zero — identity and subtraction from identity
        assert_eq!(checked_add_amount(0, 0), Some(0));
        assert_eq!(checked_add_amount(0, 42), Some(42));
        assert_eq!(checked_sub_amount(0, 0), Some(0));
        assert_eq!(checked_sub_amount(42, 0), Some(42));

        // Smallest representable values
        assert_eq!(checked_add_amount(i128::MIN, 0), Some(i128::MIN));
        assert_eq!(checked_add_amount(i128::MIN, 1), Some(i128::MIN + 1));
        assert_eq!(checked_sub_amount(i128::MIN, 0), Some(i128::MIN));

        // Just past the minimum overflows
        assert_eq!(checked_add_amount(i128::MIN, -1), None);
        assert_eq!(checked_sub_amount(i128::MIN, 1), None);
        assert_eq!(checked_sub_amount(i128::MAX, -1), None);
    }

    #[test]
    fn minimum_value_checked_increment_handles_boundaries() {
        // Zero — smallest possible value
        assert_eq!(checked_increment(0), Some(1));

        // Just below overflow
        assert_eq!(checked_increment(u64::MAX - 1), Some(u64::MAX));
        assert_eq!(checked_increment(u64::MAX), None);
    }

    #[test]
    fn minimum_value_saturating_add_handles_boundaries() {
        // Zero — identity
        assert_eq!(saturating_add_amount(0, 0), 0);
        assert_eq!(saturating_add_amount(0, 42), 42);
        assert_eq!(saturating_add_amount(42, 0), 42);

        // Negative minimum clamps inward
        assert_eq!(saturating_add_amount(i128::MIN, 0), i128::MIN);
        assert_eq!(saturating_add_amount(i128::MIN, 1), i128::MIN + 1);
        assert_eq!(saturating_add_amount(i128::MIN, -1), i128::MIN);

        // Smallest positive deltas
        assert_eq!(saturating_add_amount(1, 1), 2);
        assert_eq!(saturating_add_amount(1, -1), 0);
        assert_eq!(saturating_add_amount(-1, 1), 0);
    }

    #[test]
    fn minimum_value_saturating_add_with_cap_handles_boundaries() {
        // All zeros
        assert_eq!(saturating_add_with_cap(0, 0, 0), 0);
        assert_eq!(saturating_add_with_cap(0, 0, 100), 0);

        // Zero delta, non-zero value and cap
        assert_eq!(saturating_add_with_cap(5, 0, 100), 5);

        // Cap of zero — sum always clamped
        assert_eq!(saturating_add_with_cap(0, 0, 0), 0);
        assert_eq!(saturating_add_with_cap(5, 3, 0), 0);

        // Smallest non-zero deltas
        assert_eq!(saturating_add_with_cap(0, 1, 1), 1);
        assert_eq!(saturating_add_with_cap(0, 1, 100), 1);

        // u64::MIN through u64::MAX range
        assert_eq!(saturating_add_with_cap(u64::MIN, u64::MIN, u64::MAX), 0);
    }

    #[test]
    fn minimum_value_calculate_fee_handles_boundaries() {
        // Zero amount — always zero fee
        assert_eq!(calculate_fee(0, 0), Some(0));
        assert_eq!(calculate_fee(0, 100), Some(0));
        assert_eq!(calculate_fee(0, BASIS_POINTS_DENOMINATOR), Some(0));

        // Smallest positive amount — rounding down to zero for small rates
        assert_eq!(calculate_fee(1, 1), Some(0)); // 1 * 0.01% rounds down
        assert_eq!(calculate_fee(1, 100), Some(0)); // 1 * 1% rounds down
        assert_eq!(calculate_fee(1, 5000), Some(0)); // 1 * 50% rounds down
        assert_eq!(calculate_fee(1, BASIS_POINTS_DENOMINATOR), Some(1)); // 1 * 100% = 1

        // Smallest rate (1 bp) on small amounts
        assert_eq!(calculate_fee(100, 1), Some(0)); // 1bp of 100
        assert_eq!(calculate_fee(10_000, 1), Some(1)); // 1bp of 10,000

        // Amount of exactly 1 with varying rates
        assert_eq!(calculate_fee(1, BASIS_POINTS_DENOMINATOR), Some(1));

        // Minimum valid basis points
        assert_eq!(calculate_fee(10_000, 1), Some(1));
        assert_eq!(calculate_fee(10_000, 0), Some(0));
    }
}

/// Checked conversion from u32 to i128.
///
/// Returns `None` if the value exceeds i128::MAX.
pub fn u32_to_i128(value: u32) -> Option<i128> {
    if value as u128 > i128::MAX as u128 {
        None
    } else {
        Some(value as i128)
    }
}