//! Sounds smoke over the game's own files: `sounds/sound_definitions.json`, `sounds.json` and
//! `sounds/music_definitions.json` of every resource pack of the client installs and of the
//! sample packs load through `Driver` into the four sound families with no error-level
//! diagnostic; the sound definitions, the `sounds.json` bindings, the music definitions and the
//! name resolution are checked on the vanilla files, and the variant keys against the Molang
//! pairs file.
//!
//! No audio file is read: the names resolve against the packs' file listings.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_sounds`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
#[allow(
    dead_code,
    reason = "the variant keys are listed from the typed bindings, not through a Molang walk"
)]
mod pairs;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Phase, Severity};
use bedrock_addon::document::{Content, Merged, RawLayer};
use bedrock_addon::families::music::MusicDefinitions;
use bedrock_addon::families::sound_definitions::SoundDefinitions;
use bedrock_addon::families::sound_file::{AudioExt, SOUND_FILE_KIND, SoundFiles};
use bedrock_addon::families::sounds_json::SoundBindings;
use bedrock_addon::load::{BuiltIns, Collection, CollectionSet, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// The paths of the three documents of the sound families, where the link pass puts its findings
/// about them.
const DOCUMENTS: [&str; 3] = [
    "sounds/sound_definitions.json",
    "sounds.json",
    "sounds/music_definitions.json",
];

/// What the four sound families of a resource stack hold, with the link findings about their
/// documents: the names a binding, a music definition or a sound entry has of another document
/// or of an audio file.
struct Sounds<'a> {
    definitions: &'a Merged<SoundDefinitions, RawLayer>,
    bindings: &'a Merged<SoundBindings, RawLayer>,
    music: &'a Merged<MusicDefinitions, RawLayer>,
    files: &'a SoundFiles,
    links: Vec<Diagnostic>,
}

impl<'a> Sounds<'a> {
    /// The families of the resource stack of `set`, and the link pass's findings about their
    /// documents.
    fn of(set: &'a StackSet) -> Self {
        let links = set
            .link()
            .iter()
            .filter(|d| {
                d.phase == Phase::Link
                    && d.location
                        .path
                        .as_ref()
                        .is_some_and(|path| DOCUMENTS.contains(&path.as_str()))
            })
            .cloned()
            .collect();
        Self {
            definitions: set.resource.sound_definitions(),
            bindings: set.resource.sounds_json(),
            music: set.resource.music(),
            files: set.resource.sound_files(),
            links,
        }
    }

    /// Every finding of the four families and of the link pass about their documents.
    fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.definitions
            .diagnostics()
            .chain(self.bindings.diagnostics())
            .chain(self.music.diagnostics())
            .chain(self.files.findings())
            .chain(self.links.iter())
    }
}

/// The audio paths the client's own packs ship, as built-ins.
struct ClientFiles(BTreeSet<String>);

impl BuiltIns for ClientFiles {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == SOUND_FILE_KIND && self.0.contains(id)
    }
}

/// Every file below `dir`, as a path relative to `root` with `/`.
fn files_below(root: &Path, dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_below(root, &path, out);
        } else {
            let relative = path.strip_prefix(root).unwrap().to_string_lossy();
            out.insert(relative.replace('\\', "/"));
        }
    }
}

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The audio files below `sounds/` of every pack folder of `packs`, by pack-relative path.
fn client_files(packs: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for root in pack_roots(packs) {
        let mut found = BTreeSet::new();
        files_below(&root, &root.join("sounds"), &mut found);
        out.extend(found.into_iter().filter(|path| {
            [".fsb", ".ogg", ".wav"]
                .iter()
                .any(|ext| path.ends_with(ext))
        }));
    }
    out
}

/// How many pack folders of `packs` have a file at `relative`.
fn copies_on_disk(packs: &Path, relative: &str) -> usize {
    pack_roots(packs)
        .iter()
        .filter(|root| root.join(relative).is_file())
        .count()
}

/// The stacks of the install at `root` (its `behavior_packs` and `resource_packs`), every pack
/// a base-game pack, the resource side in the client's base-game order
/// (`common::world::base_game_world`; the merge of the sound files depends on it), with only
/// the sounds loaded.
fn load_install(root: &Path, built_ins: Option<Arc<dyn BuiltIns>>) -> StackSet {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    let mut driver = Driver::new(options).expect("a driver");
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    let options = LoadOptions {
        built_ins,
        ..LoadOptions::default()
    };
    driver
        .load_stacks(
            repository.stacks(&base_game_world(&repository, &options), options),
            StackOptions {
                collections: CollectionSet::only([
                    Collection::SoundDefinitions,
                    Collection::SoundsJson,
                    Collection::Music,
                    Collection::SoundFiles,
                ]),
                ..StackOptions::default()
            },
        )
        .expect("the stacks load")
}

/// Prints what `sounds` holds and found, by severity and code, and requires the warnings by
/// code to be `warnings`, exactly; returns the error lines.
fn report(source: &str, sounds: &Sounds<'_>, warnings: &[(&str, usize)]) -> Vec<String> {
    let mut by_code: BTreeMap<(Severity, &'static str), usize> = BTreeMap::new();
    let mut errors = Vec::new();
    for finding in sounds.diagnostics() {
        *by_code
            .entry((finding.severity, finding.code.name()))
            .or_insert(0) += 1;
        if finding.severity >= Severity::Error {
            errors.push(finding.to_string());
        }
    }
    eprintln!(
        "{source}: {} sound definitions ({} copies), {} music definitions ({} copies), sounds.json {} copies: {} block sounds, {} entities, {} interactive entities, {} named sounds, {} event sounds",
        sounds.definitions.sound_definitions.len(),
        sounds.definitions.layers().len(),
        sounds.music.definitions.len(),
        sounds.music.layers().len(),
        sounds.bindings.layers().len(),
        sounds.bindings.block_sounds.len(),
        sounds.bindings.entity_sounds.entities.len(),
        sounds
            .bindings
            .interactive_sounds
            .entity_sounds
            .entities
            .len(),
        sounds.bindings.individual_named_sounds.sounds.len(),
        sounds.bindings.individual_event_sounds.events.len(),
    );
    for ((severity, code), count) in &by_code {
        eprintln!("  {count} × {severity:?}: {code}");
    }
    for error in &errors {
        eprintln!("  {error}");
    }
    if std::env::var_os("SMOKE_SOUNDS_VERBOSE").is_some() {
        let mut lines: BTreeMap<String, usize> = BTreeMap::new();
        for finding in sounds.diagnostics() {
            if finding.severity == Severity::Warning {
                *lines.entry(finding.to_string()).or_insert(0) += 1;
            }
        }
        for (line, count) in lines {
            eprintln!("    {count} × {line}");
        }
    }
    game_files::assert_warnings(
        source,
        &game_files::warnings_by_code(sounds.diagnostics()),
        warnings,
    );
    errors
}

/// The distinct sound names of the merged definitions and how many are streamed.
fn names<'a>(sounds: &Sounds<'a>) -> (BTreeSet<&'a str>, usize) {
    let mut all = BTreeSet::new();
    let mut streamed = BTreeSet::new();
    for definition in sounds.definitions.sound_definitions.values() {
        for entry in &definition.sounds {
            all.insert(entry.name.as_str());
            if entry.stream {
                streamed.insert(entry.name.as_str());
            }
        }
    }
    (all, streamed.len())
}

/// The names the group could not resolve to a file, each with how many entries use it.
fn unresolved(sounds: &Sounds<'_>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for finding in &sounds.links {
        if let Some(DiagnosticDetail::Reference { kind, id, .. }) = finding.detail.as_deref()
            && finding.code.name() == "UnresolvedReference"
            && *kind == SOUND_FILE_KIND
        {
            *out.entry(id.clone()).or_insert(0) += 1;
        }
    }
    out
}

#[test]
fn client_install_b_sounds_load_and_every_name_resolves() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let packs = client.join("resource_packs");
    let files = client_files(&packs);
    let (fsb, ogg, wav) = (
        files.iter().filter(|p| p.ends_with(".fsb")).count(),
        files.iter().filter(|p| p.ends_with(".ogg")).count(),
        files.iter().filter(|p| p.ends_with(".wav")).count(),
    );
    eprintln!(
        "client install B: {} client audio paths ({fsb} .fsb, {ogg} .ogg, {wav} .wav)",
        files.len()
    );
    // The listing of the stack's own packs resolves every name: no built-ins, no audio read.
    let stacks = load_install(&client, None);
    let sounds = Sounds::of(&stacks);
    assert_eq!(
        sounds.files.by_name.len() + sounds.files.shadowed.len(),
        files.len()
    );
    // The chemistry pack's `sparkler` category is an information of the crate's own: the game
    // reads the category as a free string.
    let errors = report("client install B", &sounds, &[]);
    assert!(errors.is_empty(), "{errors:#?}");
    // Every copy on disk is a layer of its merge (45, 46 and 9 copies).
    assert_eq!(
        sounds.definitions.layers().len(),
        copies_on_disk(&packs, "sounds/sound_definitions.json")
    );
    assert_eq!(
        sounds.bindings.layers().len(),
        copies_on_disk(&packs, "sounds.json")
    );
    assert_eq!(
        sounds.music.layers().len(),
        copies_on_disk(&packs, "sounds/music_definitions.json")
    );
    assert_eq!(
        (
            sounds.definitions.layers().len(),
            sounds.bindings.layers().len(),
            sounds.music.layers().len()
        ),
        (45, 46, 9)
    );
    // Every name of the merged document resolves; nothing is read of a base-game pack.
    let (all, streamed) = names(&sounds);
    eprintln!("  {} distinct sound names, {streamed} streamed", all.len());
    assert_eq!((all.len(), streamed), (4890, 107), "sound stack counts");
    // Vanilla's own stack keeps its entity table (twelve slices ship no `entity_sounds`, which
    // inserts nothing, while a copy that has the section gets `entities: null`); the sample
    // packs list 100 entities.
    let cleared = sounds
        .bindings
        .diagnostics()
        .filter(|d| d.code.name() == "SoundsJsonTableCleared")
        .count();
    let entities = &sounds.bindings.entity_sounds.entities;
    let keyed = entities
        .values()
        .filter(|e| e.variants.as_ref().is_some_and(|v| v.key.is_some()))
        .count();
    assert_eq!((cleared, entities.len(), keyed), (0, 101, 10));
    // The interactive half of the transform is decided by data too. The sample packs'
    // `sounds.json` is the merged dump and carries `interactive_sounds.entities: null`, which
    // no source file contains; the stack of client install B merged under the crate's own rule
    // has it as well.
    let interactive = |json: &bedrock_addon::json::Value| {
        json.as_object()
            .and_then(|root| root.get("interactive_sounds"))
            .and_then(|section| section.as_object())
            .and_then(|section| section.get("entities"))
            .map(bedrock_addon::json::Value::is_null)
    };
    assert_eq!(interactive(sounds.bindings.json()), Some(true));
    assert!(
        sounds
            .bindings
            .layers()
            .iter()
            .all(|layer| interactive(&layer.json).is_none()),
        "no source file has the member"
    );
    if let Some(samples) = game_files::source(game_files::SAMPLE_PACKS) {
        let text = std::fs::read_to_string(samples.join("resource_pack/sounds.json")).unwrap();
        let dump: bedrock_addon::json::Value = facet_json::from_str_jsonc(&text).unwrap();
        assert_eq!(interactive(&dump), Some(true), "the merged dump has it");
    }
    let dangling = unresolved(&sounds);
    assert!(dangling.is_empty(), "{dangling:#?}");
    // No name has two extensions in one pack, and a base-game pack's files get no finding.
    assert!(sounds.files.shadowed.is_empty() && sounds.files.ignored.is_empty());
    let resolved: Vec<_> = all
        .iter()
        .filter_map(|name| sounds.files.get(name))
        .collect();
    let by = |ext| resolved.iter().filter(|asset| asset.ext == ext).count();
    assert_eq!(
        (
            resolved.len(),
            by(AudioExt::Fsb),
            by(AudioExt::Ogg),
            by(AudioExt::Wav)
        ),
        (4890, 4786, 104, 0),
        "audio extension counts"
    );
    assert!(
        sounds
            .diagnostics()
            .all(|d| d.severity < Severity::Warning || d.code.name() != "UnresolvedReference"),
    );
}

#[test]
fn client_install_a_sounds_load_and_the_dangling_names_are_reported() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_A) else {
        eprintln!(
            "skipped: {}",
            game_files::missing(game_files::CLIENT_INSTALL_A)
        );
        return;
    };
    let packs = client.join("resource_packs");
    // With the client's paths as built-ins instead, the answer is the same.
    let listed = client_files(&packs);
    let stacks = load_install(&client, Some(Arc::new(ClientFiles(listed))));
    let sounds = Sounds::of(&stacks);
    let errors = report("client install A", &sounds, &[]);
    assert!(errors.is_empty(), "{errors:#?}");
    // Vanilla ships names that resolve to no file; each use is one info.
    let dangling = unresolved(&sounds);
    let uses: usize = dangling.values().sum();
    eprintln!(
        "  {uses} unresolved uses of {} distinct names",
        dangling.len()
    );
    assert_eq!((uses, dangling.len()), (40, 27), "dangling names");
    assert!(dangling.contains_key("sounds/mob/shulker/hurt_close1"));
    assert!(dangling.contains_key("music/game/bromeliad"));
    let cleared = sounds
        .bindings
        .diagnostics()
        .filter(|d| d.code.name() == "SoundsJsonTableCleared")
        .count();
    assert_eq!(
        (cleared, sounds.bindings.entity_sounds.entities.len()),
        (0, 100)
    );
    for (name, count) in dangling.iter().take(5) {
        eprintln!("    {count} × {name}");
    }
    // A higher slice replaces the dangling names of `vanilla_1.19.0`.
    let catalyst = sounds
        .definitions
        .get("bloom.sculk_catalyst")
        .expect("bloom.sculk_catalyst");
    assert_eq!(catalyst.sounds[0].name, "sounds/particle/soulspeed1");
    assert!(
        sounds
            .files
            .findings()
            .filter(|d| d.code.name() == "UnresolvedReference")
            .all(|d| d.severity == Severity::Info)
    );
}

#[test]
fn sample_packs_sounds_load_without_diagnostics() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // A published sample of the vanilla pack, not the installed game: its audio listing is
    // partial, so it is opened as a served pack and no name is judged against it: a dangling
    // name is judged only with the client's own files in view.
    let mut driver = Driver::new(Options::default()).expect("a driver");
    let pack = driver
        .pack_dir(samples.join("resource_pack"))
        .expect("the resource pack opens");
    let options = LoadOptions::default();
    let set = StackSet {
        behavior: PackStack::of([], options.clone()).unwrap(),
        resource: PackStack::of([pack], options).unwrap(),
    };
    let stacks = driver
        .load_stacks(
            set,
            StackOptions {
                collections: CollectionSet::only([
                    Collection::SoundDefinitions,
                    Collection::SoundsJson,
                    Collection::Music,
                    Collection::SoundFiles,
                ]),
                ..StackOptions::default()
            },
        )
        .expect("the stacks load");
    let sounds = Sounds::of(&stacks);
    let errors = report("sample packs", &sounds, &[]);
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(
        unresolved(&sounds).is_empty(),
        "no name is judged against a partial listing"
    );
    // 1851 sound definitions and no diagnostics.
    assert_eq!(sounds.definitions.sound_definitions.len(), 1851);
    let found: Vec<String> = sounds
        .definitions
        .diagnostics()
        .map(ToString::to_string)
        .collect();
    assert!(found.is_empty(), "{found:#?}");
    // 11 bases resolved, 10 variant entities, no errors.
    let bindings = &sounds.bindings;
    assert_eq!(
        bindings
            .block_sounds
            .values()
            .filter(|b| !b.base.is_empty())
            .count(),
        11
    );
    let variants: Vec<&str> = bindings
        .entity_sounds
        .entities
        .iter()
        .filter(|(_, entry)| entry.variants.is_some())
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(variants.len(), 10, "{variants:?}");
    assert_eq!(bindings.interactive_sounds.entity_sounds.entities.len(), 6);
    assert!(
        bindings
            .diagnostics()
            .all(|d| d.severity < Severity::Warning)
    );
    // 48 music definitions, delays in ticks.
    assert_eq!(sounds.music.definitions.len(), 48);
    let bamboo = sounds.music.get("bamboo_jungle").expect("bamboo_jungle");
    assert_eq!(bamboo.event_name, "music.overworld.bamboo_jungle");
    assert_eq!(
        (bamboo.min_delay_ticks(), bamboo.max_delay_ticks()),
        (1200, 3600)
    );
    assert_eq!(sounds.music.diagnostics().count(), 0);
}

#[test]
fn the_variant_keys_of_the_client_install_b_packs_match_the_molang_pairs() {
    let pairs_file = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let checked = pairs::check(&pairs_file, &install_b, "sounds.json", |file| {
        let mut cx = file.cx;
        cx.molang_version =
            bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
        let document = SoundBindings::parse(&file.at, file.text.as_bytes(), &cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let mut members = pairs::Members::new();
        for (name, entry) in &document.entity_sounds.entities {
            if let Some(key) = entry.variants.as_ref().and_then(|v| v.key.as_ref()) {
                members.insert(
                    format!("/entity_sounds/entities/{name}/variants/key"),
                    (key.as_str().to_owned(), key.raw_version()),
                );
            }
        }
        let diagnostics = document.diagnostics().clone();
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs sounds.json: {} occurrences in {} files",
        checked.checked, checked.files
    );
    assert!(checked.failures.is_empty(), "{:#?}", checked.failures);
    assert_eq!((checked.checked, checked.files), (18, 9));
}

/// Every `events` key below `value`, lower-cased.
fn event_keys(value: &bedrock_addon::json::Value, out: &mut BTreeSet<String>) {
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            if key.as_str() == "events"
                && let Some(events) = member.as_object()
            {
                out.extend(events.keys().map(|key| key.as_str().to_lowercase()));
            }
            event_keys(member, out);
        }
    }
}

#[test]
fn every_event_key_of_vanillas_sounds_json_files_is_a_known_level_sound_event() {
    // The crate's list is the 570 names of the public level-sound-event schema, not the game's
    // internal table (601 ids in 1.26.20): approximate, so pinned against every file vanilla
    // ships.
    let mut files = Vec::new();
    for tree in [game_files::CLIENT_INSTALL_B, game_files::CLIENT_INSTALL_A] {
        if let Some(root) = game_files::source(tree) {
            files.extend(
                pack_roots(&root.join("resource_packs"))
                    .into_iter()
                    .map(|pack| pack.join("sounds.json"))
                    .filter(|file| file.is_file()),
            );
        }
    }
    if let Some(samples) = game_files::source(game_files::SAMPLE_PACKS) {
        files.push(samples.join("resource_pack/sounds.json"));
    }
    if files.is_empty() {
        eprintln!("skipped: no game tree with a sounds.json");
        return;
    }
    let mut keys = BTreeSet::new();
    let mut unknown = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let json: bedrock_addon::json::Value =
            facet_json::from_str_jsonc(text.trim_start_matches('\u{feff}')).unwrap();
        event_keys(&json, &mut keys);
        let document = SoundBindings::parse(
            &bedrock_addon::storage::PackPath::from_static("sounds.json"),
            text.as_bytes(),
            &bedrock_addon::load::LoadContext::detached(),
        )
        .unwrap_or_else(|rejected| panic!("{}: {rejected}", file.display()));
        unknown.extend(
            document
                .diagnostics()
                .iter()
                .filter(|d| d.code.name() == "UnknownSound")
                .map(|d| format!("{}: {d}", file.display())),
        );
    }
    eprintln!(
        "{} sounds.json files, {} distinct event keys",
        files.len(),
        keys.len()
    );
    assert!(unknown.is_empty(), "{unknown:#?}");
    if game_files::source(game_files::CLIENT_INSTALL_B).is_some()
        && game_files::source(game_files::CLIENT_INSTALL_A).is_some()
        && game_files::source(game_files::SAMPLE_PACKS).is_some()
    {
        assert_eq!((files.len(), keys.len()), (91, 563));
    }
}
