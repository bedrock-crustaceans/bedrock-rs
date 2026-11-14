# Contributing to bedrock-rs

How to set up, change, check, and submit work on this workspace. `README.md` says what the crates
are; `AGENTS.md` is the working method for agents and reads as a good one for people too.

## Before you start

1. Fork the repository and branch from `main`.
2. One branch, one change: a bug fix, a feature, a refactor, or a protocol version bump.
3. Discuss larger changes in an issue or on Discord first.

## Setup

```sh
rustup toolchain install stable
rustup component add rustfmt clippy
cargo install --locked cargo-nextest
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
| `crates/<name>/` | One crate per concern: `network`, `auth`, `form`, `level`, `shared`. `README.md` says what each does. |
| `crates/protocol/` | The protocol: `core` (`bedrock_protocol_core`: the codec traits and primitive codecs), `macros` (`bedrock_macros`: the derive), `def/versions.def.rs` (which version owns which packet, type and enum), `version/<mod>` (`bedrock_protocol_<mod>`, one crate per protocol version: its hand-written packets, types and enums and its generated packet enum), `version/traits` (`bedrock_protocol_version`: the `ProtoVersion*` traits) and `protocol` (`bedrock_protocol`: the facade that re-exports every version, with one feature per version and the integration tests). |
| `crates/addon/` | The add-on toolkit: `core`, `derive`, `manifest` (the pack manifest, on core and no family), `support` (`biome_json` alone: the one piece two biome families need that no shared crate owns), `shared/*` (the types and rules two or more families own together), `vanilla` (the game's tables, no dependencies), `storage` (the pack containers over bytes: zip, marketplace encryption, brarchive, MCB; fixtures under its `tests/fixtures/`), `test_support` (dev-only helpers the tests of the addon crates share), `addon`, `std` and one crate per content family under `families/`. |
| `xtask/` | The code generators: the protocol (`cargo xtask`), the add-on tables built from the game's metadata (`cargo xtask script-meta --samples <dir>`, `actor-types --samples <dir>`, `entity-schemas`), and the family lines of `crates/addon/addon/Cargo.toml` (`cargo xtask addon-features`). |
| `bedrock/examples/server.rs` | A login-flow server against the newest protocol, driving the login explicitly with the `network::login` helpers; the end-to-end check. `client.rs` is its counterpart. |

## Workflow: test first

Every behaviour change starts with a test that fails for the reason you are about to fix. Write
it, watch it fail, make it pass with the smallest change, then clean up with the test guarding
you. The bytes on the wire are the oracle for protocol work: a packet is right when it
deserializes a capture from a real client and serializes back to the same bytes.

`AGENTS.md` walks through the loop step by step and has the test shapes to copy.

## Checks

Run before opening a PR. CI runs the same checks on every push and pull request: formatting
once, clippy and the tests on Linux, Windows and macOS.

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features   # CI sets RUSTFLAGS=-Dwarnings
cargo nextest run --workspace
cargo test --doc --workspace
```

Tests run under [nextest](https://nexte.st), configured in `.config/nextest.toml`; plain
`cargo test` is used only for doctests, which nextest does not run. CI splits the tests into
groups by crate, one job per group and OS (`.github/workflows/ci.yml`), so no job has to build
every test binary of the workspace.

Narrow the loop while you work:

```sh
cargo check -p bedrock_protocol_v2225                   # one version crate and the older ones it uses
cargo nextest run -p bedrock_protocol_v2225 <test_name> # one test beside a packet of that version
cargo nextest run -p bedrock_protocol <test_name>       # one test of the facade's tests/, default feature = newest version only
cargo nextest run -p bedrock_protocol --all-features    # the packet enum of every version
cargo run --example server --features network-tokio,protocol-v2225,protocol-unknown
```

## Protocol changes

Versions are diffs in `crates/protocol/def/versions.def.rs`, expanded by `cargo xtask`. Each
version is a crate, `crates/protocol/version/<mod>/` (`bedrock_protocol_<mod>`), with its
hand-written packets, types and enums under `src/`. After any edit to the def file or to a
version crate:

```sh
cargo xtask
```

Under `crates/protocol/` that writes every `src/generated.rs` (each version crate,
`version/traits`, the facade `protocol`), the whole `Cargo.toml` of each version crate, and the
two marker blocks of `protocol/Cargo.toml`. It formats what it writes with rustfmt and touches
only the files that changed, so nothing is rebuilt that did not change. Never edit those files by
hand; commit them with the change that caused them. A version crate's dependencies come from the
def file and from the crate names its sources use, so a reference to an older version's item
(`bedrock_protocol_v662::packets::Foo`), or a first use of `uuid::` or `bedrock_macros`, compiles
only after `cargo xtask` has run. A version crate names only older version crates, and itself as
`crate::`.

`versions.def.rs` lists, per version, only what was added (`+`), replaced (`%`), or removed (`-`);
a trailing `^` marks an item generic over the version. Unlisted items carry forward. A list ends
with the owner of its items, `] in v662::packets,`: the module name of the version crate, then
the path inside it. The derive attributes are documented in `crates/protocol/macros/src/attr.rs`.

A new version, in the order that works:

1. Add its entry to `versions.def.rs`, its items `in v<N>::packets` (and `::types`, `::enums`).
2. Run `cargo xtask`. It creates `crates/protocol/version/v<N>/` with a `Cargo.toml`, a
   `src/generated.rs` and a `src/lib.rs` to start from, and adds the version to the facade. Do
   not make the directory by hand first: a directory under `version/` without a `Cargo.toml`
   stops every cargo command.
3. Write the modules the `lib.rs` declares; the file is yours from here on.
4. Run `cargo xtask` again, so the manifest lists what the new sources use.
5. Add the `protocol-v<N>` feature to `bedrock/Cargo.toml`.
6. The facade's default feature is now the new version. Move what named the old newest without a
   feature gate to it: the tests in `crates/protocol/protocol/tests/` and `crates/network`, the
   examples and their `required-features`.

The external crates a version crate can use are the list `EXTERNAL` in
`xtask/src/codegen/protocol/manifest.rs`; a new one is added there. The generated manifest has no
`[dev-dependencies]`: a test that needs a crate of its own goes in
`crates/protocol/protocol/tests/`.

To remove a version, delete its entry from `versions.def.rs` and run `cargo xtask`: it takes the
version out of every generated manifest, then fails naming the directory left behind. Delete
`crates/protocol/version/v<N>/` and run it again. Not the directory first: the generated
manifests still name it, and cargo will not start.

## Commits

One change per commit, with a subject that says what the commit does. Recent history uses
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) and new commits should
too: `fix: PlayerAuthInput move_vector should be (f32, f32)`,
`feat: protocol v2169`, `refactor: split up protocol codegen into smaller files`.

- Formatting-only changes are their own commit.
- Regenerated files (`generated.rs`, `generated/`, a version crate's `Cargo.toml`) travel with the
  change that caused them.
- The message names the author's intent and nothing else: no tool or assistant attribution lines.

## Pull requests

1. Rebase onto `main`.
2. Fill in the PR template: what changed, why, how you validated it, breaking changes.
3. Link related issues (`Closes #123`).
4. Address review with follow-up commits.

Before requesting review:

- [ ] `cargo fmt --all` leaves no diff.
- [ ] `cargo clippy --workspace --all-targets --all-features` is warning-free.
- [ ] `cargo nextest run --workspace` and `cargo test --doc --workspace` pass (state any excluded crate).
- [ ] A protocol change has a test that failed before it and passes after.
- [ ] Docs and `bedrock/examples/server.rs` were updated if behaviour changed.

## Reporting bugs and proposing features

Open an issue with reproduction steps, expected versus actual behaviour, your OS, Rust version,
enabled features, the protocol version, and, for a packet bug, a hex dump of the bytes the client
sent.

## Community

Discord: <https://discord.com/invite/VCVcrvt3JC>
