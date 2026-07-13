# ADR 0010: Represent token amounts as i128

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "represent token amounts as i128" so the codebase stays consistent and auditable.

## Decision

We represent token amounts as i128 as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
