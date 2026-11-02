//! What the sound tests share: resource packs built from `(path, text)` pairs, stacks given
//! lowest pack first (`p0` … `pN`), selectors over a list of findings, and the link pass over
//! such packs.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Merge, Merged};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    OpenOptions, Pack, PackOrigin, PackRepository, PackSelection, PackStack, Resource, StackSet,
    WorldPacks,
};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

/// The sound definitions path.
pub const DEFINITIONS: &str = "sounds/sound_definitions.json";
/// The bindings path.
pub const BINDINGS: &str = "sounds.json";
/// The music path.
pub const MUSIC: &str = "sounds/music_definitions.json";

/// A format-2 resource-pack manifest; `engine` is the `min_engine_version` member, if any.
pub fn manifest(uuid: u128, engine: Option<&str>) -> String {
    let engine = engine.map_or(String::new(), |v| format!(r#", "min_engine_version": {v}"#));
    format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {uuid}", "uuid": "{}", "version": [1, 0, 0]{engine}}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// A resource pack of `files` (path, bytes) with the given `min_engine_version`.
pub fn pack_bytes(uuid: u128, engine: Option<&str>, files: &[(&str, &[u8])]) -> Pack {
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest(uuid, engine).into_bytes(),
    )];
    all.extend(
        files
            .iter()
            .map(|(path, bytes)| (PackPath::new(path).expect("a pack path"), bytes.to_vec())),
    );
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A resource pack of text files at `min_engine_version` 1.21.100.
pub fn pack(uuid: u128, files: &[(&str, &str)]) -> Pack {
    let files: Vec<(&str, &[u8])> = files.iter().map(|(p, t)| (*p, t.as_bytes())).collect();
    pack_bytes(uuid, Some("[1, 21, 100]"), &files)
}

/// A resource stack of `packs`, given lowest priority first.
pub fn stack_of(mut packs: Vec<Pack>, options: LoadOptions) -> PackStack<Resource> {
    packs.reverse();
    PackStack::<Resource>::of(packs, options).expect("a resource stack")
}

/// A resource stack whose packs each hold one file at `path`, lowest priority first.
pub fn copies(path: &str, texts: &[&str]) -> PackStack<Resource> {
    let packs = texts
        .iter()
        .enumerate()
        .map(|(i, text)| pack(10 * (i as u128 + 1), &[(path, text)]))
        .collect();
    stack_of(packs, LoadOptions::default())
}

/// JSON text as a value.
pub fn json(text: &str) -> Value {
    facet_json::from_str::<Value>(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// Whatever holds findings: a merged document, the audio files, a list.
pub trait Findable {
    /// The findings, in the order the holder keeps them.
    fn findings(&self) -> Vec<&Diagnostic>;
}

impl<T: Findable + ?Sized> Findable for &T {
    fn findings(&self) -> Vec<&Diagnostic> {
        (**self).findings()
    }
}

impl<T: Merge, L> Findable for Merged<T, L> {
    fn findings(&self) -> Vec<&Diagnostic> {
        self.diagnostics().collect()
    }
}

impl Findable for Diagnostics {
    fn findings(&self) -> Vec<&Diagnostic> {
        self.iter().collect()
    }
}

impl Findable for Vec<&Diagnostic> {
    fn findings(&self) -> Vec<&Diagnostic> {
        self.clone()
    }
}

#[cfg(feature = "sound_file")]
impl Findable for bedrock_addon::families::sound_file::SoundFiles {
    fn findings(&self) -> Vec<&Diagnostic> {
        bedrock_addon::families::sound_file::SoundFiles::findings(self).collect()
    }
}

/// The findings of `holder` named `code`.
pub fn found<'a>(holder: &'a impl Findable, code: &str) -> Vec<&'a Diagnostic> {
    holder
        .findings()
        .into_iter()
        .filter(|d| d.code.name() == code)
        .collect()
}

/// The findings as `code severity path | text`, for failure messages.
pub fn describe(holder: &impl Findable) -> Vec<String> {
    holder
        .findings()
        .into_iter()
        .map(|d| {
            format!(
                "{} {:?} {} | {}",
                d.code.name(),
                d.severity,
                d.location.json_path,
                d
            )
        })
        .collect()
}

/// Requires exactly one finding named `code`, at `severity`, whose line contains `text`.
#[track_caller]
pub fn expect_one(holder: &impl Findable, code: &str, severity: Severity, text: &str) {
    let hits: Vec<_> = found(holder, code)
        .into_iter()
        .filter(|d| d.severity == severity && d.to_string().contains(text))
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "one {code} {severity:?} with {text:?} in {:#?}",
        describe(holder)
    );
}

/// Requires no finding named `code`.
#[track_caller]
pub fn expect_none(holder: &impl Findable, code: &str) {
    assert!(
        found(holder, code).is_empty(),
        "no {code} in {:#?}",
        describe(holder)
    );
}

/// Requires no finding at all.
#[track_caller]
pub fn expect_clean(holder: &impl Findable) {
    assert!(holder.findings().is_empty(), "{:#?}", describe(holder));
}

/// The stacks of a world that selects the resource packs `packs` (lowest priority first, each
/// with the origin it is registered under: `PackOrigin::Package` makes a pack the client's
/// own).
pub fn stack_set(packs: Vec<(Pack, PackOrigin)>, options: LoadOptions) -> StackSet {
    let selection: Vec<PackSelection> = packs
        .iter()
        .rev()
        .map(|(pack, _)| PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        })
        .collect();
    let world = WorldPacks {
        behavior: Vec::new(),
        resource: selection,
    };
    let mut repository = PackRepository::new();
    for (pack, origin) in packs {
        repository.add_pack(pack, origin);
    }
    repository.stacks(&world, options)
}

/// The link findings of [`stack_set`].
pub fn linked(packs: Vec<(Pack, PackOrigin)>, options: LoadOptions) -> Diagnostics {
    stack_set(packs, options).link()
}

/// [`linked`] over resource packs that all are served ones (`PackOrigin::User`).
pub fn linked_user(packs: Vec<Pack>) -> Diagnostics {
    linked(
        packs
            .into_iter()
            .map(|pack| (pack, PackOrigin::User))
            .collect(),
        LoadOptions::default(),
    )
}

/// [`linked_user`] over packs that each hold one file at `path`, lowest priority first.
pub fn linked_copies(path: &str, texts: &[&str]) -> Diagnostics {
    linked_user(
        texts
            .iter()
            .enumerate()
            .map(|(i, text)| pack(10 * (i as u128 + 1), &[(path, text)]))
            .collect(),
    )
}

/// The pack index (0 = top) a finding is located in.
pub fn pack_of(finding: &Diagnostic) -> Option<usize> {
    finding.location.pack.as_ref().and_then(|pack| pack.index())
}

/// The vanilla excerpt the merge fixtures start from (bedrock-samples v1.26.50.4
/// `resource_pack/sounds/sound_definitions.json`, `mob.zombie.say`; `max_distance` added as
/// fixtures.md `sd09` says).
pub const ZOMBIE: &str = r#"{
  "format_version": "1.20.20",
  "sound_definitions": {
    "mob.zombie.say": {
      "category": "hostile",
      "max_distance": 16.0,
      "min_distance": 2.0,
      "subtitle": "subtitles.entity.zombie.ambient",
      "sounds": ["sounds/mob/zombie/say1", "sounds/mob/zombie/say2", "sounds/mob/zombie/say3"]
    }
  }
}"#;

/// A resource pack of `origin` with the files, at `min_engine_version` 1.21.100.
pub fn pack_with_origin(uuid: u128, origin: PackOrigin, files: &[(&str, &[u8])]) -> Pack {
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest(uuid, Some("[1, 21, 100]")).into_bytes(),
    )];
    all.extend(
        files
            .iter()
            .map(|(p, b)| (PackPath::new(p).unwrap(), b.to_vec())),
    );
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// An FSB5 header with `subsounds` sounds in FADPCM.
pub fn fsb(subsounds: u32) -> Vec<u8> {
    let mut head = b"FSB5".to_vec();
    for word in [1u32, subsounds, 0, 0, 0, 16] {
        head.extend(word.to_le_bytes());
    }
    head.resize(64, 0);
    head
}

/// The first page of an Ogg stream whose first packet starts with `packet`.
pub fn ogg(packet: &[u8]) -> Vec<u8> {
    let mut head = b"OggS".to_vec();
    head.resize(28, 0);
    head.extend(packet);
    head.resize(64, 0);
    head
}

/// The head of an Ogg Vorbis file.
pub fn vorbis() -> Vec<u8> {
    ogg(b"\x01vorbis")
}

/// The head of a RIFF WAVE file.
pub fn wav() -> Vec<u8> {
    let mut head = b"RIFF\x24\x00\x00\x00WAVEfmt ".to_vec();
    head.resize(64, 0);
    head
}

/// A 1.20.20 document whose definition `a` plays `names`.
pub fn plays(names: &[&str]) -> String {
    let names: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    format!(
        r#"{{"format_version": "1.20.20", "sound_definitions": {{"a": {{"sounds": [{}]}}}}}}"#,
        names.join(", ")
    )
}

/// A pack of binary files, at `min_engine_version` 1.21.100.
pub fn files(uuid: u128, files: &[(&str, Vec<u8>)]) -> Pack {
    let files: Vec<(&str, &[u8])> = files.iter().map(|(p, b)| (*p, b.as_slice())).collect();
    pack_bytes(uuid, Some("[1, 21, 100]"), &files)
}
