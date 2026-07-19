use soroban_sdk::contracterror;

/// Errors that the RemitFlow contract can return to callers.
#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Error {
    /// The contract has already been initialized with an admin.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// No transfer exists for the supplied id.
    TransferNotFound = 3,
    /// The supplied amount was not strictly positive.
    InvalidAmount = 4,
    /// The supplied expiry is not in the future.
    InvalidExpiry = 5,
    /// The transfer counter would overflow its u64 range.
    CounterOverflow = 6,
    /// The caller is not authorized to act on this transfer.
    Unauthorized = 7,
    /// The transfer is not in the pending state.
    NotPending = 8,
    /// The transfer has passed its expiry timestamp.
    Expired = 9,
    /// The transfer has not yet reached its expiry timestamp.
    NotExpired = 10,
    /// The sender and recipient must be different addresses.
    SameParty = 11,
    /// The supplied amount exceeds the maximum allowed per transfer.
    AmountTooLarge = 12,
    /// The contract is paused and cannot accept new transfers.
    ContractPaused = 13,
    /// The supplied expiry is further out than the maximum allowed window.
    ExpiryTooFar = 14,
    /// The caller is not on the privileged callers allowlist.
    CallerNotAllowed = 15,
}
