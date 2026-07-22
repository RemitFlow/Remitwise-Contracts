/// Maximum allowed token amount for a single escrowed transfer.
/// Guards against accidental or malicious outsized values while
/// staying well within the token's i128 range.
pub const MAX_AMOUNT: i128 = 1_000_000_000_000_000_000;

/// Maximum allowed distance, in seconds, between now and a transfer's expiry.
/// Caps how far in the future an escrow can be scheduled (roughly one year).
pub const MAX_EXPIRY_WINDOW: u64 = 31_536_000;

/// Maximum total amount (stroops) that may be held in escrow across
/// all pending transfers. Transfers that would push the aggregate past
/// this cap are rejected.
pub const MAX_TOTAL_ESCROWED: i128 = 100_000_000_000_000_000;

/// Maximum number of records returned by a single paged query.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Fee denominator used for basis-point calculations (100% = 10_000 bps).
pub const FEE_DENOMINATOR: i128 = 10_000;
