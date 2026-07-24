use soroban_sdk::{contracttype, Address};

/// Parameters for creating a transfer as part of a batch.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateTransferOperation {
    pub from: Address,
    pub recipient: Address,
    pub amount: i128,
    pub expiry: u64,
}

/// Parameters for claiming a transfer as part of a batch.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimTransferOperation {
    pub id: u64,
    pub recipient: Address,
}

/// Parameters for cancelling a transfer as part of a batch.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancelTransferOperation {
    pub id: u64,
    pub from: Address,
}

/// A state-changing operation accepted by the batch entrypoint.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchOperation {
    Create(CreateTransferOperation),
    Claim(ClaimTransferOperation),
    Cancel(CancelTransferOperation),
}

/// Result produced for each successfully executed batch operation.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchOperationResult {
    Created(u64),
    Claimed,
    Cancelled,
}

/// Lifecycle status of a remittance transfer held in escrow.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Funds are locked in escrow awaiting the recipient's claim.
    Pending = 0,
    /// The recipient has successfully claimed the funds.
    Claimed = 1,
    /// The sender cancelled the transfer (or it expired) and reclaimed funds.
    Cancelled = 2,
}

/// A single remittance transfer record stored in escrow.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    /// Unique sequential identifier for this transfer.
    pub id: u64,
    /// Address that funded and owns the transfer.
    pub from: Address,
    /// Address entitled to claim the funds.
    pub recipient: Address,
    /// Amount of the token held in escrow.
    pub amount: i128,
    /// Ledger timestamp after which the transfer can be cancelled.
    pub expiry: u64,
    /// Current lifecycle status of the transfer.
    pub status: Status,
}

/// Configured resource and operation limits for the contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredLimits {
    /// Largest token amount accepted for a single escrowed transfer.
    pub max_amount: i128,
    /// Maximum allowed distance, in seconds, between now and a transfer's expiry.
    pub max_expiry_window: u64,
    /// Global cap on the total escrowed amount.
    pub max_total_escrowed: i128,
    /// Maximum number of records returned by a paginated transfer query.
    pub max_page_size: u32,
}
