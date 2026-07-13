# ADR 0008: Require auth on all state-changing entrypoints

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "require auth on all state-changing entrypoints" so the codebase stays consistent and auditable.

## Decision

We require auth on all state-changing entrypoints as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
