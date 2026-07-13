# ADR 0040: Provide read-only view getters

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "provide read-only view getters" so the codebase stays consistent and auditable.

## Decision

We provide read-only view getters as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
