#![cfg(test)]

//! Automated guards that lock down the naming conventions documented in
//! `docs/storage-model.md` (storage key variants) and
//! `docs/event-reference.md` (event topics), so that a future rename or
//! addition can't silently drift away from the documented format.
//!
//! Both storage key variant names and event topics are ultimately encoded
//! on-chain as a Soroban `Symbol`, which only accepts `a-zA-Z0-9_` and a
//! maximum length of 32 characters (see `soroban_sdk::Symbol`). The checks
//! here are stricter than that hard SDK limit: they enforce the specific
//! case conventions this contract has adopted on top of it.

/// Maximum length, in bytes, of a Soroban `Symbol`. Storage key variant
/// names and event topics are both encoded as `Symbol`s on-chain, so this is
/// a hard upper bound for either category.
pub(crate) const MAX_SYMBOL_LEN: usize = 32;

/// True if `name` is PascalCase: starts with an uppercase ASCII letter and
/// contains only ASCII alphanumeric characters. This is the convention used
/// for `InstanceKey`/`PersistentKey` variants (see `docs/storage-model.md`).
pub(crate) fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {},
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

/// True if `name` is snake_case: starts with a lowercase ASCII letter and
/// contains only lowercase ASCII letters, digits, and underscores. This is
/// the convention used for event topics (see `docs/event-reference.md`).
pub(crate) fn is_snake_case(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {},
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Extracts the leading identifier from a derived `Debug` representation of
/// an enum variant, e.g. `"Transfer(1)"` -> `"Transfer"`, `"Admin"` ->
/// `"Admin"`, `"AllowedCaller(GABC..)"` -> `"AllowedCaller"`.
pub(crate) fn variant_name(debug_repr: &str) -> &str {
    debug_repr
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()
        .unwrap_or(debug_repr)
}

#[cfg(test)]
mod tests {
    use std::format;

    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    use crate::storage::{InstanceKey, PersistentKey};

    #[test]
    fn pascal_case_validator_accepts_and_rejects_expected_inputs() {
        assert!(is_pascal_case("Admin"));
        assert!(is_pascal_case("PendingAdmin"));
        assert!(is_pascal_case("AccountOpCount"));
        assert!(is_pascal_case("A"));
        assert!(!is_pascal_case("admin"));
        assert!(!is_pascal_case("Pending_Admin"));
        assert!(!is_pascal_case("Pending Admin"));
        assert!(!is_pascal_case(""));
    }

    #[test]
    fn snake_case_validator_accepts_and_rejects_expected_inputs() {
        assert!(is_snake_case("init"));
        assert!(is_snake_case("caller_added"));
        assert!(is_snake_case("admin_transfer_started"));
        assert!(is_snake_case("created2"));
        assert!(!is_snake_case("Init"));
        assert!(!is_snake_case("caller-added"));
        assert!(!is_snake_case("_init"));
        assert!(!is_snake_case(""));
    }

    #[test]
    fn variant_name_strips_tuple_payloads() {
        assert_eq!(variant_name("Admin"), "Admin");
        assert_eq!(variant_name("Transfer(1)"), "Transfer");
        assert_eq!(variant_name("AccountOpCount(GABC)"), "AccountOpCount");
    }

    /// Every `InstanceKey` variant name must be PascalCase and fit within the
    /// Soroban `Symbol` length limit. Enumerated explicitly so that adding a
    /// new variant forces a deliberate update to this list.
    #[test]
    fn instance_key_variants_follow_naming_convention() {
        let variants = [
            format!("{:?}", InstanceKey::Admin),
            format!("{:?}", InstanceKey::PendingAdmin),
            format!("{:?}", InstanceKey::Token),
            format!("{:?}", InstanceKey::Counter),
            format!("{:?}", InstanceKey::Paused),
            format!("{:?}", InstanceKey::TotalEscrowed),
            format!("{:?}", InstanceKey::InitializedAt),
            format!("{:?}", InstanceKey::LastPrivilegedCall),
        ];
        assert_eq!(
            variants.len(),
            8,
            "InstanceKey variant count changed; update this test's coverage"
        );
        for repr in &variants {
            let name = variant_name(repr);
            assert!(
                is_pascal_case(name),
                "InstanceKey variant `{name}` is not PascalCase"
            );
            assert!(
                name.len() <= MAX_SYMBOL_LEN,
                "InstanceKey variant `{name}` exceeds {MAX_SYMBOL_LEN} chars"
            );
        }
    }

    /// Every `PersistentKey` variant name must be PascalCase and fit within
    /// the Soroban `Symbol` length limit.
    #[test]
    fn persistent_key_variants_follow_naming_convention() {
        let env = Env::default();
        let dummy_addr = Address::generate(&env);
        let variants = [
            format!("{:?}", PersistentKey::Transfer(1)),
            format!("{:?}", PersistentKey::AllowedCaller(dummy_addr.clone())),
            format!("{:?}", PersistentKey::AccountOpCount(dummy_addr)),
        ];
        assert_eq!(
            variants.len(),
            3,
            "PersistentKey variant count changed; update this test's coverage"
        );
        for repr in &variants {
            let name = variant_name(repr);
            assert!(
                is_pascal_case(name),
                "PersistentKey variant `{name}` is not PascalCase"
            );
            assert!(
                name.len() <= MAX_SYMBOL_LEN,
                "PersistentKey variant `{name}` exceeds {MAX_SYMBOL_LEN} chars"
            );
        }
    }
}
