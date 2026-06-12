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
}
