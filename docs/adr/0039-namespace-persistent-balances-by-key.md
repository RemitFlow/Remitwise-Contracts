# ADR 0039: Namespace persistent balances by key

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "namespace persistent balances by key" so the codebase stays consistent and auditable.

## Decision

We namespace persistent balances by key as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
