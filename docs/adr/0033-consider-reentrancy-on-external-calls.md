# ADR 0033: Consider reentrancy on external calls

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "consider reentrancy on external calls" so the codebase stays consistent and auditable.

## Decision

We consider reentrancy on external calls as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
