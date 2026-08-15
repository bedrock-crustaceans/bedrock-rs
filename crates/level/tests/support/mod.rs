//! Shared support for selecting imported test-world fixtures by capability
//! instead of hardcoding a single world. Parses `tests/fixtures/index.json`;
//! see `tests/fixtures/README.md` for what each fixture is and where it
//! came from.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

use bedrock_level::db::Database;
use facet::Facet;
use flate2::read::GzDecoder;
use tar::Archive;

#[derive(Facet, Debug, Clone)]
pub struct FixtureIndex {
    pub fixtures: Vec<Fixture>,
}

/// One row of `tests/fixtures/index.json`: everything known about an
/// imported fixture without opening it. Modeled in full (including fields
/// no current test reads, such as `origin` and `scan`) so a future
/// capability-filtered test can select on any of it without extending this
/// struct first.
#[derive(Facet, Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub tarball: String,
    pub origin: Origin,
    pub game_version: GameVersionFields,
    pub header_storage_version: i32,
    pub storage_version: Option<i32>,
    /// Dimension names (`"overworld"`, `"nether"`, `"end"`), not
    /// `bedrock_shared::world::dimension::Dimension`: the index is JSON, and
    /// that type has no string conversion to key a map with or print as a
    /// value, so plain strings stay the on-disk representation here too.
    pub dimensions: Vec<String>,
    /// Chunk versions actually present, from scanning `0x2c`/`0x76` records.
    pub chunk_versions: Vec<u8>,
    /// Subchunk record versions actually present (1 = legacy single-layer,
    /// 8/9 = paletted), from scanning `0x2f` records.
    pub subchunk_versions: Vec<u8>,
    /// Per-dimension `[min_y, max_y)` block range spanned by the subchunks
    /// actually present, not the dimension's theoretical engine bound.
    pub subchunk_height_range: BTreeMap<String, [i32; 2]>,
    pub capabilities: Capabilities,
    pub scan: ScanInfo,
    /// The upstream `world_test_data.json` sidecar, preserved verbatim,
    /// where the fixture's origin shipped one.
    pub sidecar: Option<Sidecar>,
}

#[derive(Facet, Debug, Clone)]
pub struct Origin {
    pub project: String,
    pub upstream_path: String,
    pub license: String,
    pub permission_note: String,
}

#[derive(Facet, Debug, Clone)]
pub struct GameVersionFields {
    pub last_opened_with: Option<[i32; 5]>,
    pub minimum_compatible_client: Option<[i32; 5]>,
    pub base_game_version: Option<String>,
    pub network_version: Option<i32>,
    pub inventory_version: Option<String>,
    pub world_version: Option<i32>,
}

#[derive(Facet, Debug, Clone)]
pub struct Capabilities {
    pub paletted: bool,
    pub negative_y: bool,
    pub biomes_2d: bool,
    pub biomes_3d: bool,
}

#[derive(Facet, Debug, Clone)]
pub struct ScanInfo {
    pub keys_scanned: u64,
    pub keys_failed: u64,
}

#[derive(Facet, Debug, Clone)]
pub struct Sidecar {
    pub world_data: SidecarWorldData,
    pub dim_height: BTreeMap<String, [[i64; 3]; 2]>,
}

#[derive(Facet, Debug, Clone)]
pub struct SidecarWorldData {
    pub platform: String,
    pub version: String,
    pub origin: String,
}

impl Fixture {
    /// Extracts this fixture's tarball into a fresh `TempDir` and opens its
    /// database, mirroring `open_test_db()` in `test.rs`: the `TempDir`
    /// unlinks its contents on drop and `Database` keeps no reference back
    /// to it, so the caller must hold onto the `TempDir` for as long as the
    /// `Database` is in use -- dropping it early silently truncates whatever
    /// the database can still see to leftover file-descriptor state.
    pub fn open(&self) -> (tempfile::TempDir, Database) {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let tar_gz_path = format!("tests/fixtures/{}", self.tarball);
        let tar_gz = File::open(&tar_gz_path).unwrap_or_else(|e| panic!("{tar_gz_path}: {e}"));
        let tar = GzDecoder::new(tar_gz);
        Archive::new(tar)
            .unpack(tmp.path())
            .unwrap_or_else(|e| panic!("{tar_gz_path}: failed to unpack: {e}"));

        let db_path = tmp.path().join("world/db");
        let db = Database::open(db_path.to_str().unwrap())
            .unwrap_or_else(|e| panic!("{}: failed to open database: {e}", self.name));
        (tmp, db)
    }

    /// Path to this fixture's `level.dat`, given the `TempDir` an earlier
    /// call to [`Self::open`] extracted it into.
    pub fn level_dat_path(&self, tmp: &tempfile::TempDir) -> PathBuf {
        tmp.path().join("world/level.dat")
    }
}

/// Parses `tests/fixtures/index.json`. Panics on any parse failure -- the
/// index is checked-in data generated alongside the fixtures, not user
/// input -- and panics if it parses to zero fixtures, since an empty
/// checked-in fixture set is always a bug and every caller here assumes at
/// least one fixture exists.
pub fn fixtures() -> Vec<Fixture> {
    let path = "tests/fixtures/index.json";
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let index: FixtureIndex =
        facet_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: failed to parse: {e}"));
    assert!(
        !index.fixtures.is_empty(),
        "{path}: parsed to zero fixtures"
    );
    index.fixtures
}
