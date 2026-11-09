//! Helpers of the `blocks.json` tests: small resource stacks and the registry's findings.

#![allow(
    dead_code,
    reason = "a helper module shared by the test files of this folder; none of them uses every helper"
)]

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Document, Registry};
use bedrock_addon::families::blocks_json::{ClientBlock, FaceTextures};
use bedrock_addon::id::BlockId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    PackOrigin, PackRepository, PackSelection, Resource, StackSet, WorldPacks,
};

use super::pack_common::rp;

/// The fixture folder of this family.
pub const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/block/blocks_json"
);

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{path}"))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Text that lives as long as the test binary (a pack spec holds `'static` bytes).
pub fn leak(text: &str) -> &'static [u8] {
    Box::leak(text.to_owned().into_boxed_str()).as_bytes()
}

/// One pack of a test stack: its `blocks.json` and whether it is the base game's.
#[derive(Clone, Copy)]
pub struct Layer<'a> {
    pub text: &'a str,
    pub base_game: bool,
}

/// A pack that is not the base game's.
pub fn user(text: &str) -> Layer<'_> {
    Layer {
        text,
        base_game: false,
    }
}

/// A base-game pack.
pub fn base(text: &str) -> Layer<'_> {
    Layer {
        text,
        base_game: true,
    }
}

/// The stacks of a world whose resource packs are `layers`, **lowest priority first** (as the
/// folder's fixtures number them: `00_…` is the lowest). Pack `n` is named `rp <n>`.
pub fn stacks(layers: &[Layer<'_>]) -> StackSet {
    let mut repository = PackRepository::new();
    let mut resource = Vec::new();
    for (index, layer) in layers.iter().enumerate() {
        // Named for its place in `layers`; the UUID must not be nil.
        let mut spec = rp(index as u128 + 1).file("blocks.json", leak(layer.text));
        spec.name = format!("rp {index}");
        let origin = if layer.base_game {
            PackOrigin::Package
        } else {
            PackOrigin::User
        };
        repository.add_pack(spec.open(), origin);
        let id = spec.id();
        // The stack is top first.
        resource.insert(
            0,
            PackSelection {
                pack_id: id.uuid,
                version: id.version,
                subpack: None,
            },
        );
    }
    let world = WorldPacks {
        behavior: Vec::new(),
        resource,
    };
    repository.stacks(&world, LoadOptions::default())
}

/// A loaded stack and its registry.
pub struct Loaded {
    set: StackSet,
}

impl Loaded {
    pub fn registry(&self) -> &Registry<ClientBlock> {
        self.set.resource.client_blocks()
    }

    pub fn stack(&self) -> &bedrock_addon::pack::PackStack<Resource> {
        &self.set.resource
    }

    /// The merged block `id` (`minecraft:` may be left out).
    #[track_caller]
    pub fn block(&self, id: &str) -> &Document<ClientBlock> {
        self.get(id)
            .unwrap_or_else(|| panic!("no block {id}: {:#?}", self.found()))
    }

    pub fn get(&self, id: &str) -> Option<&Document<ClientBlock>> {
        self.registry().get(&block_id(id))
    }

    /// The six face textures of `id`, up, down, north, south, west, east.
    #[track_caller]
    pub fn textures(&self, id: &str) -> [String; 6] {
        self.block(id).face_textures().map(str::to_owned)
    }

    /// Every finding of the registry.
    pub fn found(&self) -> Vec<Found> {
        self.registry().diagnostics().map(Found::of).collect()
    }

    /// The findings at warning level or above.
    pub fn problems(&self) -> Vec<Found> {
        self.found()
            .into_iter()
            .filter(|f| f.severity >= Severity::Warning)
            .collect()
    }
}

/// A block identifier; a bare name is a vanilla block's.
pub fn block_id(id: &str) -> BlockId {
    if id.contains(':') {
        BlockId::parse(id)
    } else {
        BlockId::parse(format!("minecraft:{id}"))
    }
}

/// Loads `layers`, lowest priority first.
pub fn load(layers: &[Layer<'_>]) -> Loaded {
    Loaded {
        set: stacks(layers),
    }
}

/// Loads one user pack.
pub fn load_one(text: &str) -> Loaded {
    load(&[user(text)])
}

/// One finding, as the tests compare it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub code: &'static str,
    pub severity: Severity,
    /// The JSON path.
    pub at: String,
    /// Vanilla's text; empty for a finding of ours (`text: None`), which `Display` words itself.
    pub text: String,
    pub pack: String,
}

impl Found {
    pub fn of(f: &Diagnostic) -> Self {
        let shown = f.code.to_string();
        Self {
            code: f.code.id(),
            severity: f.severity,
            at: f.location.json_path.to_string(),
            text: if has_vanilla_text(f) {
                shown
            } else {
                String::new()
            },
            pack: f
                .location
                .pack
                .as_ref()
                .map(|pack| pack.name().to_owned())
                .unwrap_or_default(),
        }
    }
}

/// Whether the finding carries one of vanilla's texts.
fn has_vanilla_text(f: &Diagnostic) -> bool {
    f.code.vanilla().is_some()
}

/// A finding of ours (no vanilla text) in pack 0.
pub fn ours(code: &'static str, severity: Severity, at: &str) -> Found {
    Found {
        code,
        severity,
        at: at.to_owned(),
        text: String::new(),
        pack: "rp 0".to_owned(),
    }
}

/// A finding with vanilla's `text` in pack 0.
pub fn vanilla(code: &'static str, severity: Severity, at: &str, text: &str) -> Found {
    Found {
        text: text.to_owned(),
        ..ours(code, severity, at)
    }
}

impl Found {
    /// The same finding in pack `n`.
    pub fn in_pack(mut self, n: usize) -> Self {
        self.pack = format!("rp {n}");
        self
    }
}

/// A file at 1.21.40 with `entries` (the members after `format_version`).
pub fn file(entries: &str) -> String {
    file_at(r#""1.21.40""#, entries)
}

/// A file whose `format_version` is the JSON `version`, with `entries`.
pub fn file_at(version: &str, entries: &str) -> String {
    if entries.trim().is_empty() {
        format!(r#"{{"format_version": {version}}}"#)
    } else {
        format!(r#"{{"format_version": {version}, {entries}}}"#)
    }
}

/// Six times `name`.
pub fn six(name: &str) -> [String; 6] {
    std::array::from_fn(|_| name.to_owned())
}

/// `FaceTextures::SixFaces` of up, down, north, south, west, east.
pub fn faces(names: [&str; 6]) -> FaceTextures {
    let [up, down, north, south, west, east] = names.map(str::to_owned);
    FaceTextures::SixFaces {
        up,
        down,
        north,
        south,
        west,
        east,
    }
}

pub const W: Severity = Severity::Warning;
pub const E: Severity = Severity::Error;
pub const I: Severity = Severity::Info;
