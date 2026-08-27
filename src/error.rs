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
    /// The contract would exceed the global escrow cap.
    EscrowCapReached = 15,
    /// The contract is paused and cannot accept new transfers.
    ContractPaused = 13,
    /// The supplied expiry is further out than the maximum allowed window.
    ExpiryTooFar = 14,
    /// The caller is not on the privileged callers allowlist.
    CallerNotAllowed = 16,
    /// `accept_admin` was called but no pending admin transfer has been initiated.
    NoPendingAdmin = 17,
    /// The supplied address resolves to the contract's own address where an
    /// external party address is required.
    InvalidAddress = 18,
    /// An account exceeded its allowed number of operations.
    AccountLimitReached = 19,
    /// A privileged call was attempted before the cooldown period elapsed.
    CooldownNotElapsed = 21,
    /// The contract's actual token balance is less than its internally
    /// tracked `TotalEscrowed` liability.
    SupplyInvariantViolation = 20,
    /// The number of operations in a batch_operations call exceeds
    /// MAX_BATCH_SIZE.
    BatchTooLarge = 22,
    /// The supplied idempotency key is zero and cannot identify a batch.
    InvalidBatchId = 24,
    /// An idempotency key was reused with a different batch payload.
    BatchIdConflict = 25,
}
