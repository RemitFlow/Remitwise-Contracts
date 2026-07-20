# Contributing to RemitFlow Smart Contracts

First off, thank you for considering contributing to RemitFlow! We welcome contributions from everyone.

## Getting Started

1. Fork the repository and clone it to your local machine.
2. Ensure you have Rust and the `wasm32-unknown-unknown` target installed.
3. Install the Soroban CLI if you plan to optimize or deploy the contract.

## Branch Naming Convention

Please create a new branch from `main` for your changes. Use the following convention for branch names:
- `fix/issue-<number>-<short-description>` for bug fixes or specific issues.
- `feat/<short-description>` for new features.
- `chore/<short-description>` for chores or maintenance tasks.

## Development Workflow

We use `make` to simplify common development tasks.

### Building

To compile the smart contract:

```bash
make build
```

To optimize the contract for deployment (requires Soroban CLI):

```bash
make optimize
```

### Testing

We highly value automated tests. Please ensure that all existing tests pass and that you add tests for any new features or bug fixes.

To run the test suite:

```bash
make test
```

### Formatting and Linting

Before committing your changes, ensure your code adheres to our formatting and linting standards.

To format the code:

```bash
make fmt
```

To check formatting without modifying files:

```bash
make fmt-check
```

To run the linter (Clippy):

```bash
make lint
```

## Pull Requests

1. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/). This helps us auto-generate changelogs.
2. Push your branch to your fork.
3. Open a Pull Request against the `main` branch.
4. Ensure all CI checks pass.
5. A maintainer will review your pull request and merge it once it's ready.

Thank you for contributing!
