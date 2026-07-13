# ADR 0018: Test against a mock token contract

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "test against a mock token contract" so the codebase stays consistent and auditable.

## Decision

We test against a mock token contract as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
