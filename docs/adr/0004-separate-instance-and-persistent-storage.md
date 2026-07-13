# ADR 0004: Separate instance and persistent storage

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "separate instance and persistent storage" so the codebase stays consistent and auditable.

## Decision

We separate instance and persistent storage as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
