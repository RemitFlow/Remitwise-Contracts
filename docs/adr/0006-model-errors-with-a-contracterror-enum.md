# ADR 0006: Model errors with a contracterror enum

- Status: Accepted
- Deciders: arisu6804

## Context

The RemitFlow smart contract needs a clear, documented approach to "model errors with a contracterror enum" so the codebase stays consistent and auditable.

## Decision

We model errors with a contracterror enum as the standard for this contract, in line with Soroban best practices.

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
