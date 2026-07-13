# ADR 0024: Model storage keys with a DataKey enum

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "model storage keys with a datakey enum" so the codebase stays consistent and auditable.

## Decision

We model storage keys with a DataKey enum as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
