# ADR 0009: Use checked arithmetic for all math

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "use checked arithmetic for all math" so the codebase stays consistent and auditable.

## Decision

We use checked arithmetic for all math as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
