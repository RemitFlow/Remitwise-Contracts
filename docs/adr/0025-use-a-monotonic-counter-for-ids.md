# ADR 0025: Use a monotonic counter for ids

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "use a monotonic counter for ids" so the codebase stays consistent and auditable.

## Decision

We use a monotonic counter for ids as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
