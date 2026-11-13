# bedrock-rs agent notes

What the crates are: @README.md
Setup, checks, commits, PRs: @CONTRIBUTING.md

The two `@` files are loaded with this one; nothing here repeats them. This file is the working
method: how to drive a change through any crate test-first, what each crate's oracle is, and the
facts no config confesses.

## The loop

Every change is one pass of red, green, refactor. The test is written before the code it tests,
and it goes **red** for the reason you are about to fix. A test that cannot fail proves nothing;
a compile error is not red.

1. **Locate the file that owns the behaviour.** Each crate section below says where its logic lives and what data it is
   checked against. Done when you can name the file and the item you will change.
2. **Write the red test** in a `#[cfg(test)] mod tests` at the bottom of that file (or in the
   crate's `tests/` for anything that needs a fixture on disk), named for the behaviour:
   `manifest_without_dependencies_imports`, not `test_manifest`. Run only it:
   `cargo nextest run -p <crate> <test_name>`. Done when it fails and the assertion message names the
   missing behaviour.
3. **Go green** with the smallest change that passes. The neighbour you noticed gets its own
   test and its own commit.
4. **Refactor** with the test guarding: names, extracted functions, a derive instead of a hand
   impl. Rerun the one test after each step.
5. **Widen**: the crate's suite, then the checks in `CONTRIBUTING.md`. Done when fmt, clippy and
   the tests are clean, with any excluded crate named in your report.
6. **Commit** the one change, test included.

Code with no test is not a licence to skip step 2. When the behaviour you inherit is unclear,
pin what it does today with a **characterization test**, then change it.

## Test shapes

**Real data is the oracle.** Every crate here mirrors a format Mojang defines, so the strongest
test feeds in something the real game produced and checks the crate reads it and writes it back
unchanged. Prefer, in order:

1. **Round trip a capture**: decode real bytes or JSON, encode, assert identical output, then
   assert the fields you care about. Store short captures inline; larger ones under the crate's
   `tests/`.
2. **Golden value**: build the Rust value by hand and assert it equals the parsed capture
   (`PartialEq` on the type; derive it on anything new).
3. **Error variant**: for bad input, `matches!` on the variant, never on `Display` text.
4. **Two implementations agree**: where a fast path exists beside a plain one (SIMD unpacking in
   `level`), the plain one is the oracle for the fast one over a spread of inputs.

Fixtures are named for what they build (`open_test_db()`, `vanilla_manifest()`), not `setup()`.
Nothing in a test touches the network.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    /// Decode, encode, and require the encoding to match the capture byte for byte.
    fn round_trip(bytes: &[u8]) -> RecordStartedPacket<crate::V2193> {
        let packet = RecordStartedPacket::<crate::V2193>::deserialize(&mut Cursor::new(bytes)).unwrap();
        let mut encoded = Vec::new();
        packet.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the capture");
        packet
    }

    #[test]
    fn record_started_decodes_a_capture() {
        let packet = round_trip(&[/* bytes from a client */]);
        assert_eq!(packet.server_sound_handle, 7);
    }
}
```

The same shape serves JSON (`facet_json::from_str` then `to_string` in `form`) and LevelDB
values (`from_disk` then `to_disk` in `level`).

## Per crate

**`protocol_core` and `macros`**: the `ProtoCodec` traits, primitive codecs in
`protocol_core/src/types/`, and the derive. Red test: a tiny struct using the new piece,
asserting exact bytes both ways. A change here rebuilds every packet; check `bedrock_protocol`
after.

**`protocol`**: one hand-written file per packet, type or enum, in the newest `version/v<N>/`
that changed it; `def/versions.def.rs` says which. Oracle: bytes a real client sent. Get them
from a bug report, a proxy, or the example server, where `Connection::recv_raw` hands you a batch
before decoding. Versions are diffs: `+` added, `%` replaced, `-` removed, `^` generic over `V`;
unlisted items carry forward. After editing `def/` or `version/`, `cargo xtask && cargo fmt --all`.
The default feature is the newest version alone, so also check with `--all-features`.

**`network`**: `codec.rs` (batching), `compression.rs` and `encryption.rs` are pure functions
over `Vec<u8>` and need no socket; test them there. `encode_packets` and `decode_packets` take
compression and encryption as options, so a test can cover one layer at a time. The crate root,
including the `login` helpers (plain-data functions a server or client calls between its own
recv and send; no packet sequencing), is sans-io; everything async lives under `src/tokio/`
(feature `tokio`): connection, listener, client and their errors.
The transport is `raknet-tokio`, a git dependency; `examples/server.rs` is the end-to-end check for it, and for a
login flow.

**`auth`**: does no I/O of its own and must never pull in tokio. `ConnectionRequest::verify` and
`AuthData::validate` check the chain, or the token against the cached JWKS, and never fetch.
Fetching (`AuthOIDC::fetch`, `refresh`, the `*_refreshing` variants) is generic over the
`http::HttpClient` and `http::AsyncHttpClient` traits; the `ureq` and `reqwest` features only add
impls of them. Each fetching function has a sync and an `_async` twin; a change to one is a change
to both. Tests use a key generated in the test, an `AuthOIDC` built by hand around it, and
`test_support::FakeHttp` with `block_on` for anything that fetches.

**`form`**: the JSON forms the client renders, via `facet_json`. Oracle: the JSON the client
sends back or accepts. Tests in `forms/mod.rs` compare a parsed literal against a hand-built
value; add to them.

**`level`**: LevelDB keys (`key.rs`), sub-chunks, biomes, player and settings NBT. The LevelDB
backend itself is `db.rs` and `iter.rs`, pure Rust on top of `rusty-leveldb`;
`Database::keys()` is fallible (`Result<Keys<'_>>`). Oracle: a real world; `tests/level.tar.gz`
is one, and the tests unpack it into a temp dir. Every reader has a writer; a test is
`from_disk`, `to_disk`, `from_disk`, equal. `greedy.rs` has a SIMD unpacker under `unsafe`;
`unpack_nonsimd` is its oracle (`tests/simd.rs` shows the shape). The existing integration tests
are `#[ignore]`d because the readers are incomplete: un-ignore one as your red test. Benches
under `benches/` are Criterion; run one before and after a performance change and put the
numbers in the commit body.

**`shared`**: plain data: newtyped IDs, vectors, world enums. Nothing here reads a disk or a
socket; a unit test beside the type is enough.

**`crates/addon/*`**: the add-on toolkit. `core` holds identifiers, versions, diagnostics and the
version-chain machinery; `manifest` reads and writes `manifest.json` on top of core; `storage`
reads the pack containers (zip, marketplace encryption, brarchive, MCB) over bytes; each
content family (`families/<name>`) owns the types and rules of one kind of game file; `addon`
composes them into packs, stacks, the link pass and the writer, and `std` drives them over the
file system. Oracle: the game's own files. Red tests are small JSON
documents in the family's or `addon`'s `tests/`, asserting the typed value and the findings the
game's content log would print; write the document back and require it to read the same. A
family's document tests live in the family crate's `tests/`; stack, link and cross-family tests in
`addon/tests`; the helpers they share in `bedrock_addon_test_support`.
The tests that load whole trees of the game's packs (`full*` in `std/tests`) and the vanilla
gate (`vanilla_gate`) need files that are not in the repository: they read
`BEDROCK_ADDON_CORPUS` and `BEDROCK_ADDON_VANILLA_1_26_60` and skip without them, which is not a
pass; report which ran. A skipped test prints `SKIPPED (not a pass): <what>: <reason>`, which
nextest shows at the end of the run for the corpus-gated binaries (`.config/nextest.toml`);
count them with `cargo nextest run ... 2>&1 | rg -c 'SKIPPED \(not a pass\)'`. `bedrock_addon` is sans-IO by convention: no file, thread or clock access outside
`std`. The workspace `clippy.toml` keeps every facet-json call behind `core`'s `json` door. The tables
under `core/src/vanilla/generated/`, `vanilla/src/generated/`, `families/script/src/meta/generated/` and the entity
schema tables are generator output (`cargo xtask script-meta`, `actor-types`,
`entity-schemas`): change `xtask/` and regenerate, never edit by hand. So are the family blocks
between the marker comments of `crates/addon/addon/Cargo.toml` (`cargo xtask addon-features`,
whose check test fails when they are stale). The test that
regenerates them reads a checkout of Mojang's `bedrock-samples` repository at a release tag from
`$BEDROCK_ADDON_CORPUS/bedrock-samples`. `addon/benches/skeleton.rs` is the Criterion bench of the
four loader kinds (registry, files, merged, hand-written); run it before and after a change to
`core/src/load` with the command in its header and put the numbers in the commit body.

## Facts no config confesses

- Tests run under `cargo nextest run` (`.config/nextest.toml`); each test is its own process.
  `cargo test` is only for doctests (`cargo test --doc`), which nextest does not run. CI tests
  in crate groups, one job per group and OS; a new crate has to join a group in `ci.yml`.
- `cargo nextest run --workspace` is red on a stale `Cargo.lock` because of `bedrock_network`.
  `CONTRIBUTING.md` has the workaround (`cargo update -p raknet-tokio`). Report what you
  excluded; never present an excluded crate as passing.
- `raknet-tokio` is an unpinned git dependency and the lock file is not committed. An API
  mismatch in `network` may be upstream drift, not your change: `cargo update -p raknet-tokio`
  first, then diagnose. `level`'s `rusty-leveldb` is also a git dependency but pinned by `rev`
  in `crates/level/Cargo.toml`.
- `crates/protocol/src/generated/` is output; regenerate, never edit.
- Feature-gated code compiles only with the feature on: `auth-ureq`, `auth-reqwest`,
  `protocol`'s per-version features, and every crate behind the `bedrock` facade. A change that builds under a crate's
  default features has not been built the way CI builds it (`--all-features`).
- `bedrock_addon_core::test_cost` (the linear-cost measure the addon tests use) and
  `json::kit::fixed_path::testing` build only under core's `test-cost` feature: a crate whose
  tests reach them turns it on in a `[dev-dependencies]` line on core.
- Errors are `thiserror` enums per crate; a new failure is a new variant, never a `String`.
  Library code propagates with `?`; `unwrap` and `expect` belong in tests, benches and `xtask`.
