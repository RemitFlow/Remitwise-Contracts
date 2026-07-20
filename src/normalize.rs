/// Saturating increment for u64 counters.
/// Returns value + 1, clamping at u64::MAX to prevent overflow.
pub fn saturating_increment_u64(value: u64) -> u64 {
    value.saturating_add(1)
}

/// Saturating addition with an explicit cap.
/// Returns (value + delta).min(cap), preventing overflow.
pub fn saturating_add_with_cap(value: u64, delta: u64, cap: u64) -> u64 {
    value.saturating_add(delta).min(cap)
}

/// Clamp an i128 amount to the valid range (0..=MAX_AMOUNT).
pub fn clamp_amount(amount: i128, max_amount: i128) -> i128 {
    amount.max(0).min(max_amount)
}

/// Validate that a Stellar address string is non-empty and starts with 'G'.
/// Returns the trimmed address or None.
pub fn normalize_stellar_address(addr: &str) -> Option<&str> {
    let trimmed = addr.trim();
    if trimmed.is_empty() || !trimmed.starts_with('G') {
        return None;
    }
    Some(trimmed)
}

/// Validate expiry: must be strictly in the future and within MAX_EXPIRY_WINDOW.
pub fn validate_expiry(now: u64, expiry: u64, max_window: u64) -> Result<u64, &'static str> {
    if expiry <= now {
        return Err("Expiry must be in the future");
    }
    if expiry - now > max_window {
        return Err("Expiry too far in the future");
    }
    Ok(expiry)
}
