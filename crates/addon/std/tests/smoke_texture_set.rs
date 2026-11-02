//! Texture set smoke over the game's own files: the resource packs of the sample packs and of
//! the client installs load through `Driver` into `rp.texture_sets()`, every
//! `*.texture_set.json` of the stack is read, none has an error-level diagnostic, and the load
//! asks for the bytes of no image.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_texture_set`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::texture_file::ImageFormat;
use bedrock_addon::families::texture_set::{LayerKind, Rgba};
use bedrock_addon::id::TexturePath;
use bedrock_addon::load::{
    Collection, CollectionSet, Input, LoadOptions, Loader, LoaderError, Output, Progress, Request,
    Sans, StackLoader, StackOptions,
};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon::version::FormatVersion;
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::base_game_world;

/// The suffix of a texture set.
const SUFFIX: &str = ".texture_set.json";

/// What a stack load asked the host to read.
#[derive(Debug, Default)]
struct Asked {
    /// Every file read.
    files: usize,
    /// The reads of a file with an image extension.
    images: usize,
    /// The sizes the listing reported for those image reads, summed.
    image_bytes: u64,
}

/// A `StackLoader` that counts the reads it asks for.
struct Counting {
    loader: StackLoader,
    asked: Arc<Mutex<Asked>>,
}

impl Sans for Counting {
    type Input = Input;
    type Output = Output<StackSet>;
    type Error = LoaderError;

    fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
        self.loader.handle(input)
    }

    fn poll(&mut self) -> Option<Output<StackSet>> {
        let output = self.loader.poll();
        if let Some(Output::Request(Request::ReadFile {
            location, len_hint, ..
        })) = &output
        {
            let mut asked = self.asked.lock().unwrap();
            asked.files += 1;
            let path = location.path.as_str().to_ascii_lowercase();
            if ImageFormat::from_path(&path).is_some() {
                asked.images += 1;
                asked.image_bytes += len_hint.unwrap_or(0);
            }
        }
        output
    }
}

impl Loader<StackSet> for Counting {
    fn progress(&self) -> Progress {
        self.loader.progress()
    }
}

/// The files below `dir` whose name ends in `suffix`.
fn files_with(dir: &Path, suffix: &str) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .map(|path| {
            if path.is_dir() {
                files_with(&path, suffix)
            } else {
                usize::from(path.to_string_lossy().ends_with(suffix))
            }
        })
        .sum()
}

/// The pack folders directly below `dir` that hold a texture set.
fn packs_with_sets(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|pack| files_with(&pack.join("textures"), SUFFIX) > 0)
                .count()
        })
        .unwrap_or(0)
}

/// The stacks of every pack below `dir` with the texture group loaded, and what the load read.
fn load(dir: &Path, origin: PackOrigin) -> (StackSet, Asked) {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), origin)])
        .unwrap();
    let options = LoadOptions::default();
    let world = base_game_world(&repository, &options);
    let asked = Arc::new(Mutex::new(Asked::default()));
    let loader = Counting {
        loader: StackLoader::new(
            repository.stacks(&world, options),
            StackOptions {
                collections: CollectionSet::only([
                    Collection::TextureSets,
                    Collection::TextureFiles,
                ]),
                ..StackOptions::default()
            },
        ),
        asked: Arc::clone(&asked),
    };
    let stacks = driver.drive(loader).unwrap();
    let asked = std::mem::take(&mut *asked.lock().unwrap());
    (stacks, asked)
}

/// Loads the resource packs below `dir` and checks the texture sets of the stack: none is
/// rejected but the textures of `broken`, each of which is rejected for one layer image its
/// pack lacks. Returns the stacks for the caller's own checks.
fn smoke(source: &str, dir: PathBuf, origin: PackOrigin, broken: &[&str]) -> StackSet {
    let on_disk = files_with(&dir, SUFFIX);
    let packs = packs_with_sets(&dir);
    assert!(
        on_disk > 0,
        "{source}: no texture set below {}",
        dir.display()
    );
    let (stacks, asked) = load(&dir, origin);
    let textures = stacks.resource.texture_sets();
    let loaded = textures.sets().count();
    let mut known = Vec::new();
    let mut rejected = Vec::new();
    for (texture, rejection) in textures.rejected_sets() {
        if broken.contains(&texture) {
            // The data's own defect: exactly one finding, the missing layer image.
            let found: Vec<_> = rejection.diagnostics.iter().collect();
            assert_eq!(found.len(), 1, "{source}: {texture}: {rejection}");
            assert_eq!(
                found[0].code.name(),
                "TextureNotFound",
                "{source}: {texture}"
            );
            // the game's level-1 line, told as an information; the set is still rejected
            assert_eq!(found[0].severity, Severity::Info);
            known.push(texture);
        } else {
            rejected.push(format!("{texture}: {rejection}"));
        }
    }
    assert_eq!(
        known, broken,
        "{source}: the sets known to name a missing image"
    );
    let mut errors = Vec::new();
    let mut by_code: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let set_findings = textures
        .sets()
        .flat_map(|(_, set)| set.diagnostics().iter())
        .chain(
            textures
                .rejected_sets()
                .flat_map(|(_, rejected)| rejected.diagnostics.iter()),
        );
    for finding in set_findings {
        let known = finding
            .location
            .path
            .as_ref()
            .and_then(|path| path.as_str().strip_suffix(SUFFIX))
            .is_some_and(|texture| broken.contains(&texture));
        let level = match finding.severity {
            Severity::Error => {
                if !known {
                    errors.push(finding.to_string());
                }
                "error"
            }
            Severity::Warning => "warning",
            _ => "info",
        };
        *by_code.entry((level, finding.code.name())).or_default() += 1;
    }
    let mut lints: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in textures.diagnostics() {
        let name = finding.code.name();
        if name == "CapabilityHint" || name == "VibrantVisualsDisabledByPack" {
            *lints.entry(name).or_default() += 1;
        }
    }
    let layer_images = stacks.resource.layer_images().len();
    println!(
        "{source}: {on_disk} texture set(s) on disk in {packs} pack(s), {loaded} loaded, {} rejected ({} of them known data defects), \
         {layer_images} layer images named; findings by level and code: {by_code:?}; pack lints: {lints:?}; \
         the load read {} file(s), {} of them images ({} image bytes)",
        rejected.len() + known.len(),
        known.len(),
        asked.files,
        asked.images,
        asked.image_bytes
    );
    assert!(
        errors.is_empty() && rejected.is_empty(),
        "{source}: {} error(s), {} rejected:\n{}\n{}",
        errors.len(),
        rejected.len(),
        errors.join("\n"),
        rejected.join("\n")
    );
    // No texture set of a tree has a warning: the pin is exact.
    let warnings: BTreeMap<&str, usize> = by_code
        .iter()
        .filter(|((level, _), _)| *level == "warning")
        .map(|((_, code), count)| (*code, *count))
        .collect();
    game_files::assert_warnings(source, &warnings, &[]);
    // No image is read: the group needs their names alone.
    assert_eq!(asked.images, 0, "{source}: {asked:?}");
    assert_eq!(asked.image_bytes, 0, "{source}: {asked:?}");
    assert!(
        asked.files >= loaded,
        "{source}: every set was read: {asked:?}"
    );
    // Every vanilla set is the newest format, and each resolves through the one resolver.
    assert!(loaded > 0, "{source}: the stack has texture sets");
    for (texture, set) in textures.sets() {
        assert_eq!(
            set.version().parsed_as,
            FormatVersion::new(1, 21, 30),
            "{source}: {texture}"
        );
        assert!(set.color.is_some(), "{source}: {texture}");
        let file = stacks
            .resource
            .resolve_texture(&TexturePath::parse(texture.to_owned()))
            .unwrap_or_else(|| panic!("{source}: {texture} resolves"));
        assert!(!file.is_missing(), "{source}: {texture}");
    }
    assert!(layer_images >= loaded, "{source}: each set names an image");
    check_layer_images(source, &dir, &stacks);
    stacks
}

/// The identity of a manifest's header as `<uuid> <major>.<minor>.<patch>`: the first `"uuid"`
/// and the first `"version"` of the file (the header comes first in the game's manifests; the
/// version slices of the base game share one UUID and differ in the version).
fn header_identity(manifest: &str) -> Option<String> {
    let after = |key: &str| Some(&manifest[manifest.find(key)? + key.len()..]);
    let uuid = after("\"uuid\"")?;
    let uuid = &uuid[uuid.find('"')? + 1..];
    let uuid = uuid[..uuid.find('"')?].to_ascii_lowercase();
    let version = after("\"version\"")?;
    let version = &version[version.find(':')? + 1..];
    let version = version.trim_start();
    let version = match version.strip_prefix('[') {
        Some(list) => list[..list.find(']')?]
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("."),
        None => {
            let text = version.strip_prefix('"')?;
            text[..text.find('"')?].to_owned()
        }
    };
    Some(format!("{uuid} {version}"))
}

/// The image rules over every layer image of the stack, each read from its
/// pack's folder below `dir`: the game draws its own textures, so none is refused, whatever
/// its format. No format is left out of the count rule: the TGAs (true colour of 24 and 32
/// bits, run-length packed, colour-mapped) and the colour-mapped PNGs are judged like the
/// rest.
fn check_layer_images(source: &str, dir: &Path, stacks: &StackSet) {
    let mut folders: BTreeMap<String, PathBuf> = BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let folder = entry.path();
        let identity = std::fs::read_to_string(folder.join("manifest.json"))
            .ok()
            .and_then(|manifest| header_identity(&manifest));
        if let Some(identity) = identity {
            assert!(
                folders.insert(identity.clone(), folder).is_none(),
                "{source}: two folders of pack {identity}"
            );
        }
    }
    let mut refused = Vec::new();
    let mut by_format: BTreeMap<String, usize> = BTreeMap::new();
    let mut read = 0usize;
    let images = stacks.resource.layer_images();
    for layer in &images {
        let id = layer.pack.id();
        let identity = format!("{} {}", id.uuid, id.version);
        let folder = folders
            .get(&identity)
            .unwrap_or_else(|| panic!("{source}: no folder of pack {identity}"));
        let file = folder.join(layer.image.path.as_str());
        let bytes = std::fs::read(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        read += 1;
        let format = match layer.image.format {
            Some(ImageFormat::Tga) => format!(
                "tga type {} of {} bits",
                bytes.get(2).copied().unwrap_or(0),
                bytes.get(16).copied().unwrap_or(0)
            ),
            Some(ImageFormat::Png) => {
                format!("png colour type {}", bytes.get(25).copied().unwrap_or(0))
            }
            other => format!("{other:?}"),
        };
        *by_format.entry(format).or_default() += 1;
        if let Some(finding) = layer.check(&bytes) {
            refused.push(format!("{}: {:?} {finding}", file.display(), layer.layer));
        }
    }
    println!("{source}: {read} layer images read and checked; by format: {by_format:?}");
    assert_eq!(read, images.len(), "{source}");
    assert!(
        refused.is_empty(),
        "{source}: {} layer image(s) of the game's own packs refused; the first:\n{}",
        refused.len(),
        refused
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn sample_packs_texture_sets_load_without_errors() {
    let Some(samples) = game_files::source(game_files::SAMPLE_PACKS) else {
        eprintln!("{}", game_files::missing(game_files::SAMPLE_PACKS));
        return;
    };
    let on_disk = files_with(&samples.join("resource_pack/textures"), SUFFIX);
    assert_eq!(on_disk, 3130, "the texture sets of the sample packs");
    let stacks = smoke("sample packs", samples, PackOrigin::Package, &[]);
    let textures = stacks.resource.texture_sets();
    // All 3130 load, and `flame_atlas` has the one uniform layer.
    assert_eq!(textures.sets().count(), 3130);
    let flame = stacks
        .resource
        .resolve_texture(&TexturePath::parse("textures/flame_atlas"))
        .and_then(|file| file.set)
        .expect("textures/flame_atlas has a set");
    assert_eq!(
        flame.uniform(LayerKind::Mers),
        Some(Rgba {
            r: 0.0,
            g: 255.0,
            b: 255.0,
            a: 0.0
        })
    );
    let uniform = textures
        .sets()
        .filter(|(_, set)| {
            LayerKind::ALL
                .iter()
                .any(|layer| set.uniform(*layer).is_some())
        })
        .count();
    assert_eq!(uniform, 1, "one array in 3130 files");
    let with_normal = textures
        .sets()
        .filter(|(_, set)| set.normal.is_some())
        .count();
    assert_eq!(with_normal, 4);
}

#[test]
fn client_install_b_texture_sets_load_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_B) else {
        eprintln!("{}", game_files::missing(game_files::CLIENT_INSTALL_B));
        return;
    };
    let dir = client.join("resource_packs");
    assert_eq!(packs_with_sets(&dir), 8, "eight packs ship texture sets");
    smoke("client install B", dir, PackOrigin::Package, &[]);
}

#[test]
fn client_install_a_texture_sets_load_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_A) else {
        eprintln!("{}", game_files::missing(game_files::CLIENT_INSTALL_A));
        return;
    };
    // Client install A ships `wolf_collar_baby.texture_set.json` naming
    // `wolf_collar_baby_mers`, an image no pack of that install has (client install B ships it
    // as `wolf_collar_baby_mers.tga`). The client shows the missing texture for it when it loads
    // the PBR layers; the crate rejects the set with one `TextureNotFound`.
    let stacks = smoke(
        "client install A",
        client.join("resource_packs"),
        PackOrigin::Package,
        &["textures/entity/wolf/wolf_collar_baby"],
    );
    let path = TexturePath::parse("textures/entity/wolf/wolf_collar_baby");
    let file = stacks.resource.resolve_texture(&path).expect("the set");
    assert!(file.is_missing());
}
