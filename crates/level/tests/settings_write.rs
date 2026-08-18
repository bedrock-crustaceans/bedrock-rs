//! `LevelSettings::write` against every real fixture world plus the 1.26
//! seed world (`tests/level.tar.gz`) -- fourteen level.dats total, the same
//! set `every_fixture_level_dat_parses_strictly` in `fixtures_test.rs`
//! establishes all parse strictly to begin with.
//!
//! What these tests pin: the header's length field, a decode-write-decode
//! round trip is semantically equal (`PartialEq`) under strict decode for
//! every one of the fourteen, and -- since none of them come out
//! byte-identical -- exactly which field classes account for that (key
//! presence *and* key value/tag, not presence alone -- see
//! `compound_key_diff`), so any future improvement (or regression) is
//! visible as a test failure asking to be looked at rather than a silent
//! change in behavior.

mod support;

use std::io::Cursor;

use bedrock_level::settings::LevelSettings;
use flate2::read::GzDecoder;
use tar::Archive;

/// Every field this crate reads under `#[facet(default)]` (a plain, non-
/// `Option` field that tolerates an absent on-disk key by decoding to its
/// type's default) -- the exhaustive set from `settings.rs`, by on-disk
/// path. `LevelSettings::write` re-emits every one of these explicitly
/// (`false` for a bool, `0.0` for `verticalFlySpeed`) whether or not the
/// source world had it, per the fidelity contract stated on
/// `LevelSettings::write`'s own doc comment. This is the *only* class of
/// key a round trip through `write` may add that the source did not have --
/// asserted below for every fixture, so a change that starts adding
/// anything else is caught immediately.
const DEFAULTED_FIELD_PATHS: &[&str] = &[
    "isCreatedInEditor",
    "isExportedFromEditor",
    "isRandomSeedAllowed",
    "recipesunlock",
    "cheatsEnabled",
    "experimentalgameplay",
    "dolimitedcrafting",
    "projectilescanbreakblocks",
    "freezedamage",
    "respawnblocksexplode",
    "showbordereffect",
    "showtags",
    "showdaysplayed",
    "showrecipemessages",
    "locatorbar",
    "tntexplosiondropdecay",
    "isSingleUseWorld",
    "IsHardcore",
    "PlayerHasDied",
    "HasUncompleteWorldFileOnDisk",
    "BiomeOverride",
    "abilities.verticalFlySpeed",
    "experiments.caves_and_cliffs",
    "experiments.caves_and_cliffs_internal",
    "experiments.armadillo",
    "experiments.updateAnnouncedLive2023",
];

/// Fixture-specific exceptions to "no key present in both trees may change
/// value or tag between the original file and a fresh write": `(fixture
/// name, on-disk key path)` pairs where the *source file itself* holds a
/// value that is not the canonical encoding of what it decodes to, so
/// decode-then-write cannot reproduce the original bytes even though the
/// decoded value is exactly what `read` produced from them.
///
/// The one known member, found by extending `compound_key_diff` to catch
/// value/tag changes (not just presence): `v1_16`'s `CenterMapsToOrigin` is
/// a raw on-disk `Byte(15)`, not `Byte(0)`/`Byte(1)`. nbtx decodes a `bool`
/// from a `Byte` tag as `raw_byte == 1` (any other byte, including `15`,
/// decodes to `false`), and `write` always re-encodes `false` as the
/// canonical `Byte(0)` -- a `bool` has exactly two meaningful states, and
/// nothing about reaching one of them from disk promises to remember which
/// non-canonical byte produced it. This is not a `write` bug (nor a
/// `read` one): byte-for-byte fidelity for a value like that is the raw
/// layer's job (architecture decision 3), not this typed struct's.
const NON_CANONICAL_VALUE_EXCEPTIONS: &[(&str, &str)] = &[("v1_16", "CenterMapsToOrigin")];

/// Every fixture's `level.dat` bytes, plus the 1.26 seed world's, labeled by
/// name. Mirrors the `all_databases`/`open_1_26_world` pattern in
/// `finalized_state.rs`, reading raw bytes rather than opening a database
/// since `LevelSettings` has nothing to do with `db/`.
fn all_level_dats() -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = support::fixtures()
        .into_iter()
        .map(|fixture| {
            let (tmp, _db) = fixture.open();
            let data = std::fs::read(fixture.level_dat_path(&tmp))
                .unwrap_or_else(|e| panic!("{}: failed to read level.dat: {e}", fixture.name));
            (fixture.name, data)
        })
        .collect();

    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").expect("seed missing");
    Archive::new(GzDecoder::new(tar_gz))
        .unpack(tmp.path())
        .expect("failed to unpack seed");
    let data = std::fs::read(tmp.path().join("debug/level.dat")).expect("read seed level.dat");
    out.push(("level.tar.gz (1.26)".to_string(), data));

    out
}

/// Recursively diffs two compounds, dot-joined by path (e.g.
/// `experiments.armadillo`), into three buckets: keys added (present in
/// `after` but not `before`), keys removed (present in `before` but not
/// `after`), and keys present on both sides whose *value* differs --
/// covering both an ordinary content change and a tag change on an
/// otherwise-present key (e.g. `List` vs `IntArray`), which key-presence
/// alone cannot see. `Value`'s own `PartialEq` (exact, including which
/// variant a node is) decides "differs"; a differing pair recurses only
/// when both sides are still compounds; otherwise it is reported directly
/// as changed. Values equal at a key are not descended into at all, so an
/// unrelated identical subtree costs nothing.
fn compound_key_diff(
    before: &nbtx::Value,
    after: &nbtx::Value,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    fn walk(
        prefix: &str,
        before: &nbtx::Value,
        after: &nbtx::Value,
        added: &mut Vec<String>,
        removed: &mut Vec<String>,
        changed: &mut Vec<String>,
    ) {
        let (nbtx::Value::Compound(b), nbtx::Value::Compound(a)) = (before, after) else {
            // Reached via a key present on both sides whose values differ
            // (the caller only recurses on inequality) but where at least
            // one side is no longer a compound -- a tag or leaf-value
            // change, not a shape change.
            changed.push(prefix.trim_end_matches('.').to_string());
            return;
        };
        for key in a.keys() {
            if !b.contains_key(key) {
                added.push(format!("{prefix}{key}"));
            }
        }
        for key in b.keys() {
            if !a.contains_key(key) {
                removed.push(format!("{prefix}{key}"));
            }
        }
        for (key, before_child) in b.iter() {
            if let Some(after_child) = a.get(key)
                && before_child != after_child
            {
                walk(
                    &format!("{prefix}{key}."),
                    before_child,
                    after_child,
                    added,
                    removed,
                    changed,
                );
            }
        }
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    walk("", before, after, &mut added, &mut removed, &mut changed);
    (added, removed, changed)
}

/// Decodes the NBT payload of a level.dat buffer (skipping the 8-byte
/// header), for the key-diffing tests, which want the dynamic `Value` tree
/// rather than the typed struct.
fn decode_payload(data: &[u8]) -> nbtx::Value {
    let mut cursor = Cursor::new(&data[8..]);
    nbtx::from_le_bytes(&mut cursor).expect("payload decodes as NBT")
}

/// The header's `i32` payload-length field matches the actual NBT byte
/// count `LevelSettings::write` produced, for every fixture.
#[test]
fn header_length_field_matches_payload_for_every_fixture() {
    for (name, data) in all_level_dats() {
        let settings =
            LevelSettings::read(data.as_slice()).unwrap_or_else(|e| panic!("{name}: read: {e}"));

        let mut out = Vec::new();
        settings
            .write(&mut out)
            .unwrap_or_else(|e| panic!("{name}: write: {e}"));

        assert!(
            out.len() >= 8,
            "{name}: written file shorter than the header"
        );
        let declared_len = i32::from_le_bytes(out[4..8].try_into().unwrap());
        assert_eq!(
            declared_len as usize,
            out.len() - 8,
            "{name}: header payload-length field does not match the actual NBT byte count"
        );
    }
}

/// Decode -> write -> decode reproduces a semantically equal
/// [`LevelSettings`] (`PartialEq`), under strict (`deny-unknown-fields`)
/// decode both times, for every one of the fourteen real level.dats. This
/// is the round-trip fidelity `LevelSettings::write`'s doc comment commits
/// to; byte identity is checked separately below since it is *not* part of
/// that contract.
#[test]
fn round_trip_is_semantically_equal_for_every_fixture() {
    let mut checked = 0usize;
    for (name, data) in all_level_dats() {
        let settings =
            LevelSettings::read(data.as_slice()).unwrap_or_else(|e| panic!("{name}: read: {e}"));

        let mut out = Vec::new();
        settings
            .write(&mut out)
            .unwrap_or_else(|e| panic!("{name}: write: {e}"));

        let round_tripped = LevelSettings::read(out.as_slice())
            .unwrap_or_else(|e| panic!("{name}: strict re-decode of the fresh write failed: {e}"));

        assert_eq!(
            settings, round_tripped,
            "{name}: decode -> write -> decode is not semantically equal"
        );
        checked += 1;
    }
    assert_eq!(
        checked, 14,
        "expected fourteen level.dats (13 fixtures + the 1.26 seed)"
    );
}

/// Every compound in the written output -- the root and every nested one
/// (`abilities`, `experiments`, `world_policies`) -- has its keys in
/// ascending byte order, matching what all fourteen real level.dats were
/// found to use uniformly at every nesting depth (see
/// `settings::sort_compound_keys`'s doc comment for the corpus check this
/// pins).
#[test]
fn written_output_is_globally_key_sorted() {
    fn assert_sorted(name: &str, path: &str, value: &nbtx::Value) {
        if let nbtx::Value::Compound(entries) = value {
            let keys: Vec<&[u8]> = entries.keys().map(|k| k.as_slice()).collect();
            let mut sorted = keys.clone();
            sorted.sort_unstable();
            assert_eq!(
                keys, sorted,
                "{name}: compound at {path:?} is not key-sorted in the written output"
            );
            for (key, child) in entries.iter() {
                assert_sorted(name, &format!("{path}{key}."), child);
            }
        }
    }

    for (name, data) in all_level_dats() {
        let settings =
            LevelSettings::read(data.as_slice()).unwrap_or_else(|e| panic!("{name}: read: {e}"));
        let mut out = Vec::new();
        settings
            .write(&mut out)
            .unwrap_or_else(|e| panic!("{name}: write: {e}"));
        let value = decode_payload(&out);
        assert_sorted(&name, "", &value);
    }
}

/// Byte identity, checked and diagnosed rather than merely asserted false:
/// for every fixture, every key the write adds that the original file
/// lacked is one of [`DEFAULTED_FIELD_PATHS`] (never anything else), zero
/// keys are ever removed -- `Option` fields round-trip absence faithfully
/// (see `LevelSettings::write`'s doc comment) -- and every key present on
/// both sides whose value or tag differs is one of
/// [`NON_CANONICAL_VALUE_EXCEPTIONS`] (never anything else). That third
/// check is what caught a real bug during review: `compound_key_diff`
/// originally only compared key *presence*, so it could not see
/// `lastOpenedWithVersion`/`MinimumCompatibleClientVersion` round-tripping
/// through the wrong NBT tag (`write` emitted `IntArray`; every real
/// level.dat uses `List<Int>`) even though the key was "present on both
/// sides" the whole time -- fixed in `pack_game_versions`, see its doc
/// comment in `settings.rs`. This pins the exact, corpus-wide account of
/// why byte identity is currently 0 of 14: not field reordering
/// (`written_output_is_globally_key_sorted` plus this test's own
/// added/removed/changed accounting together rule that out), not a dropped
/// `Option` field, not an unaccounted-for retagged or revalued field, but
/// exclusively (a) defaulted fields the source world omitted being
/// re-emitted explicit, plus (b) the one documented non-canonical-source
/// exception. If a future change adds presence tracking for the defaulted
/// fields, this test starts failing on the `byte_identical` assertion
/// below and should be updated to expect that higher (or full) identity
/// rate rather than reverting the change.
#[test]
fn byte_identity_report() {
    let mut byte_identical = 0usize;
    let mut total = 0usize;

    for (name, data) in all_level_dats() {
        let settings =
            LevelSettings::read(data.as_slice()).unwrap_or_else(|e| panic!("{name}: read: {e}"));
        let mut out = Vec::new();
        settings
            .write(&mut out)
            .unwrap_or_else(|e| panic!("{name}: write: {e}"));

        total += 1;
        if data == out {
            byte_identical += 1;
            continue;
        }

        let before = decode_payload(&data);
        let after = decode_payload(&out);
        let (added, removed, changed) = compound_key_diff(&before, &after);

        assert!(
            removed.is_empty(),
            "{name}: write dropped key(s) present in the original: {removed:?}"
        );
        for key in &changed {
            assert!(
                NON_CANONICAL_VALUE_EXCEPTIONS.contains(&(name.as_str(), key.as_str())),
                "{name}: write changed key {key:?}'s value/tag, which is not a \
                 documented non-canonical-source exception -- either add \
                 (\"{name}\", {key:?}) to NON_CANONICAL_VALUE_EXCEPTIONS (if the \
                 source file's own bytes are a non-canonical encoding of the \
                 decoded type) or investigate why it diverged"
            );
        }
        for key in &added {
            assert!(
                DEFAULTED_FIELD_PATHS.contains(&key.as_str()),
                "{name}: write added key {key:?}, which is not in the documented \
                 defaulted-field allowlist -- either extend DEFAULTED_FIELD_PATHS \
                 (if this is a genuine new defaulted field) or investigate why it \
                 diverged"
            );
        }
    }

    // Corpus fact as of this task: none of the fourteen real level.dats
    // round-trip byte-identically -- every one of them omits at least one
    // `#[facet(default)]` field that `write` must re-emit explicit, and
    // `v1_16` additionally carries the one documented non-canonical byte
    // (see `NON_CANONICAL_VALUE_EXCEPTIONS`). See this test's doc comment.
    assert_eq!(
        byte_identical, 0,
        "byte-identity rate changed ({byte_identical}/{total}) -- update this pinned \
         count and the doc comment above rather than leaving it stale"
    );
}

/// A freshly written level.dat decodes through [`LevelSettings::read`]
/// under strict (`deny-unknown-fields`) decode -- the crate's default
/// feature set, so this is what every ordinary caller gets. Distinct from
/// `round_trip_is_semantically_equal_for_every_fixture`, which also does
/// this but is about the *value* it decodes to; this test is about decode
/// succeeding at all, closing the loop this task's fixture list opened
/// (`every_fixture_level_dat_parses_strictly` in `fixtures_test.rs` pins
/// every real file parses strictly -- this pins that this crate's own
/// output does too).
#[test]
fn fresh_write_decodes_strictly_for_every_fixture() {
    let mut checked = 0usize;
    for (name, data) in all_level_dats() {
        let settings =
            LevelSettings::read(data.as_slice()).unwrap_or_else(|e| panic!("{name}: read: {e}"));
        let mut out = Vec::new();
        settings
            .write(&mut out)
            .unwrap_or_else(|e| panic!("{name}: write: {e}"));
        LevelSettings::read(out.as_slice())
            .unwrap_or_else(|e| panic!("{name}: fresh write does not decode strictly: {e}"));
        checked += 1;
    }
    assert_eq!(checked, 14);
}
