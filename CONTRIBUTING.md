# Contributing to bedrock-rs

How to set up, change, check, and submit work on this workspace. `README.md` says what the crates
are; `CLAUDE.md` is the working method for agents and reads as a good one for people too.

## Before you start

1. Fork the repository and branch from `main`.
2. One branch, one change: a bug fix, a feature, a refactor, or a protocol version bump.
3. Discuss larger changes in an issue or on Discord first.

## Setup

```sh
rustup toolchain install stable
rustup component add rustfmt clippy
git clone https://github.com/bedrock-crustaceans/bedrock-rs.git
cd bedrock-rs
cargo build --workspace
```

`raknet-tokio` is an unpinned git dependency, and `Cargo.lock` is not committed. A fresh clone
resolves its `main` branch; an old local lock can lag behind the code. If `bedrock_network` stops
compiling against the RakNet API, run `cargo update -p raknet-tokio`. `bedrock_level`'s
`rusty-leveldb` dependency is a git dependency too, but pinned to a `rev`.

## Where things live

| Path | What |
| --- | --- |
| `bedrock/src/lib.rs` | The `bedrock` facade: one `pub mod` per crate, each behind a feature. |
| `crates/<name>/` | One crate per concern: `protocol_core`, `macros`, `protocol`, `network`, `auth`, `form`, `level`, `shared`. `README.md` says what each does. |
| `xtask/` | The protocol code generator. `cargo xtask` is the whole interface. |
| `bedrock/examples/server.rs` | A login-flow server against the newest protocol; the end-to-end check. |

## Workflow: test first

Every behaviour change starts with a test that fails for the reason you are about to fix. Write
it, watch it fail, make it pass with the smallest change, then clean up with the test guarding
you. The bytes on the wire are the oracle for protocol work: a packet is right when it
deserializes a capture from a real client and serializes back to the same bytes.

`CLAUDE.md` walks through the loop step by step and has the test shapes to copy.

## Checks

Run before opening a PR. CI runs the same three jobs on every push and pull request.

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features   # CI sets RUSTFLAGS=-Dwarnings
cargo test --workspace
```

Narrow the loop while you work:

```sh
cargo test -p bedrock_protocol <test_name>       # one test, default feature = newest version only
cargo test -p bedrock_protocol --all-features    # every version; about five times the compile time
cargo run --example server --features network,protocol-v2225,protocol-unknown
```

## Protocol changes

Versions are diffs in `crates/protocol/def/versions.def.rs`, expanded by `cargo xtask`. After any
edit there or under `crates/protocol/src/version/`:

```sh
cargo xtask && cargo fmt --all
```

Commit the regenerated files with the change that caused them. `versions.def.rs` lists, per
version, only what was added (`+`), replaced (`%`), or removed (`-`); a trailing `^` marks an
item generic over the version. Unlisted items carry forward. The derive attributes are documented
in `crates/macros/src/attr.rs`.

## Commits

One change per commit, with a subject that says what the commit does. Recent history uses
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) and new commits should
too: `fix: PlayerAuthInput move_vector should be (f32, f32)`,
`feat: protocol v2169`, `refactor: split up protocol codegen into smaller files`.

- Formatting-only changes are their own commit.
- Regenerated `generated/` files travel with the change that caused them.
- The message names the author's intent and nothing else: no tool or assistant attribution lines.

## Pull requests

1. Rebase onto `main`.
2. Fill in the PR template: what changed, why, how you validated it, breaking changes.
3. Link related issues (`Closes #123`).
4. Address review with follow-up commits.

Before requesting review:

- [ ] `cargo fmt --all` leaves no diff.
- [ ] `cargo clippy --workspace --all-targets --all-features` is warning-free.
- [ ] `cargo test --workspace` passes (state any excluded crate).
- [ ] A protocol change has a test that failed before it and passes after.
- [ ] Docs and `bedrock/examples/server.rs` were updated if behaviour changed.

## Reporting bugs and proposing features

Open an issue with reproduction steps, expected versus actual behaviour, your OS, Rust version,
enabled features, the protocol version, and, for a packet bug, a hex dump of the bytes the client
sent.

## Community

Discord: <https://discord.com/invite/VCVcrvt3JC>
