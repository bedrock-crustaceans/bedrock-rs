# Contributing to bedrock-rs

Thank you for contributing to **bedrock-rs**.
This document explains the expected workflow for code changes, quality checks, and pull requests.

## Before You Start

1. Fork the repository and create a branch from `main`.
2. Keep your branch focused on a single change (bug fix, feature, refactor, etc.).
3. If possible, discuss larger changes in an issue before implementation.

## Development Setup

Install Rust toolchain and required components:

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
```

Clone and enter the project:

```bash
git clone https://github.com/bedrock-crustaceans/bedrock-rs.git
cd bedrock-rs
```

## Code Style and Quality Checks

When making code changes, run these checks before opening a PR:

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If you want a quick local validation pass, you can run:

```bash
cargo fmt --all -- --check
cargo check --workspace
```

## Working with Features and Crates

This workspace is modular; some code paths are behind feature flags.
When your change touches feature-gated logic, validate with appropriate feature sets:

```bash
cargo check --all-features
cargo test --all-features
```

For crate-specific changes, also validate the related package directly:

```bash
cargo test -p bedrockrs_proto
```

## Commit Guidelines

1. Use clear commit messages that explain the intent.
2. Keep commits small and reviewable.
3. Avoid mixing formatting-only changes with behavioral changes in the same commit.
4. Any commit message style is acceptable (Conventional Commit or normal/free style).

Examples:

- `fix(proto): correct packet decode bounds check`
- `Fix packet decode bounds check in proto`
- `feat(server): add connection timeout configuration`
- `Add connection timeout configuration to server builder`
- `Refactor level key parsing`

## Pull Request Process

1. Ensure your branch is up to date with `main`.
2. Push your branch and open a Pull Request.
3. In the PR description, include:
   - What changed.
   - Why the change is needed.
   - How you validated it (commands/results).
   - Any breaking changes or migration notes.
4. Link related issues (for example: `Closes #123`).
5. Respond to review feedback with follow-up commits.

## PR Checklist

Before requesting review, confirm:

- [ ] Code is formatted with `cargo fmt --all`.
- [ ] Build passes with `cargo check --workspace`.
- [ ] Lints pass with `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Tests pass with `cargo test --workspace`.
- [ ] Relevant feature-flag combinations were checked.
- [ ] Documentation/examples were updated if behavior changed.

## Reporting Bugs and Proposing Features

When opening an issue:

1. Provide clear reproduction steps.
2. Include expected vs actual behavior.
3. Share environment details (OS, Rust version, enabled features).
4. Include logs/errors and a minimal example if possible.

## Community

For help or discussion, join the project Discord:
<https://discord.com/invite/VCVcrvt3JC>
