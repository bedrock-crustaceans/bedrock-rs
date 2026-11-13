//! Every MCB document of the game's installs through the reader of its family, beside its
//! JSON twin: the loose file of the same path, which each pack ships beside the archive that
//! shadows it. The decode against the twin, member by member, is
//! `bedrock_addon_storage`'s `tests/mcb/vanilla.rs`; the corpus walk is a copy of its own.

use std::path::{Path, PathBuf};

use bytes::Bytes;

use crate::storage::brarchive::Brarchive;
use crate::storage::magic::{FileType, detect};
use crate::storage::mcb::McbHeader;

/// The installs whose archives are read: the folder under the game files, and the label the
/// findings print.
const INSTALLS: [(&str, &str); 3] = [
    ("bds-1.26.36.1", "server-install"),
    ("gdk-retail", "client-install-b"),
    ("gdk-dev", "client-install-a"),
];

/// One MCB document of an install.
struct Baked {
    install: &'static str,
    /// The archive, and the entry's name in it.
    archive: PathBuf,
    name: String,
    bytes: Bytes,
    /// The loose file of the same pack path, when the pack has it as JSON text.
    twin: Option<String>,
}

impl Baked {
    fn place(&self) -> String {
        format!("{}: {}", self.archive.display(), self.name)
    }

    /// The document's path in its pack: `particles/sneeze.json`.
    fn pack_path(&self) -> String {
        let archive = self.archive.with_extension("");
        let archive = archive.to_string_lossy();
        let folder = archive.split("/__brarchive/").nth(1).unwrap();
        format!("{folder}/{}", self.name)
    }

    fn key(&self) -> String {
        McbHeader::parse(&self.bytes)
            .unwrap()
            .payload_key
            .to_owned()
    }
}

/// `<pack>/__brarchive/<folder>.brarchive` holds the files of `<pack>/<folder>/`.
fn twin_path(archive: &Path, name: &str) -> Option<PathBuf> {
    let mut folders = Vec::new();
    let mut at = archive.with_extension("");
    loop {
        let parent = at.parent()?.to_path_buf();
        if at.file_name()? == "__brarchive" {
            let mut path = parent;
            path.extend(folders.iter().rev());
            return Some(path.join(name));
        }
        folders.push(at.file_name()?.to_owned());
        at = parent;
    }
}

/// Every MCB entry of every archive of the installs, or `None` (with a `skipped:` line)
/// without the game's files.
fn baked() -> Option<Vec<Baked>> {
    let mut documents = Vec::new();
    for (folder, install) in INSTALLS {
        for path in game_files(&[folder], ".brarchive")? {
            // the encrypted test pack of one install has archives that are not archives
            let Ok(archive) = Brarchive::decode(Bytes::from(std::fs::read(&path).unwrap())) else {
                continue;
            };
            for entry in archive.entries() {
                let bytes = archive.read(entry);
                if detect(&bytes) != FileType::Mcb {
                    continue;
                }
                let twin = twin_path(&path, &entry.name)
                    .and_then(|twin| std::fs::read(twin).ok())
                    .filter(|twin| detect(twin) != FileType::Mcb)
                    .map(|twin| String::from_utf8(twin).unwrap());
                documents.push(Baked {
                    install,
                    archive: path.clone(),
                    name: entry.name.to_string(),
                    bytes,
                    twin,
                });
            }
        }
    }
    Some(documents)
}

/// Every file whose name ends in `suffix` below `$BEDROCK_ADDON_CORPUS/<top>` for each of
/// `tops`, sorted; `None` (with a message) when the variable is unset or a directory is missing,
/// so the test is skipped.
fn game_files(tops: &[&str], suffix: &str) -> Option<Vec<PathBuf>> {
    use bedrock_addon_test_support::corpus;
    let Some(root) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        corpus::skipped("the game's MCB files", corpus::not_set(corpus::CORPUS));
        return None;
    };
    let mut files = Vec::new();
    for top in tops {
        let dir = PathBuf::from(&root).join(top);
        if !dir.is_dir() {
            corpus::skipped(
                "the game's MCB files",
                format_args!("{} is missing", dir.display()),
            );
            return None;
        }
        let mut stack = vec![dir];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(suffix))
                {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Some(files)
}

/// The family readers on the decoded text and on the twin.
mod readers {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt::Debug;

    use super::{Baked, baked};
    use crate::diagnostic::Severity;
    use crate::document::Content;
    use crate::load::LoadContext;
    use crate::storage::PackPath;
    use crate::storage::mcb::to_json_text;

    /// A value's `Debug` and its findings.
    type Read = (String, Vec<String>);

    /// What the reader of `T` makes of `text` at `document`'s path: the value's `Debug`, and
    /// its findings as `severity code` lines.
    fn read<T: Content + Debug>(document: &Baked, text: &str) -> Read {
        let mut cx = LoadContext::detached();
        cx.is_base_game_pack = true;
        let path = PackPath::new(&document.pack_path()).unwrap();
        let parsed = T::parse(&path, text.as_bytes(), &cx)
            .unwrap_or_else(|rejected| panic!("{}: {:#?}", document.place(), rejected.diagnostics));
        let findings = parsed
            .diagnostics()
            .iter()
            .map(|finding| format!("{:?} {}", finding.severity, finding.code.id()))
            .collect();
        assert!(
            parsed
                .diagnostics()
                .iter()
                .all(|finding| finding.severity < Severity::Error),
            "{}: {:#?}",
            document.place(),
            parsed.diagnostics()
        );
        (format!("{:?}", parsed.value()), findings)
    }

    /// The decoded text and the twin through the reader of the document's family.
    fn both(document: &Baked, text: &str) -> (Read, Option<Read>) {
        let twin = document.twin.as_deref();
        match document.key().as_str() {
            "minecraft:voxel_shape" => {
                use crate::families::voxel_shape::VoxelShape;
                (
                    read::<VoxelShape>(document, text),
                    twin.map(|twin| read::<VoxelShape>(document, twin)),
                )
            }
            "tiers" => {
                use crate::families::trade::TradeTable;
                (
                    read::<TradeTable>(document, text),
                    twin.map(|twin| read::<TradeTable>(document, twin)),
                )
            }
            "minecraft:item" => {
                use crate::families::item::Item;
                (
                    read::<Item>(document, text),
                    twin.map(|twin| read::<Item>(document, twin)),
                )
            }
            "minecraft:camera_entity" => {
                use crate::families::camera::CameraEntity;
                (
                    read::<CameraEntity>(document, text),
                    twin.map(|twin| read::<CameraEntity>(document, twin)),
                )
            }
            "particle_effect" => {
                use crate::families::particle::ParticleEffect;
                (
                    read::<ParticleEffect>(document, text),
                    twin.map(|twin| read::<ParticleEffect>(document, twin)),
                )
            }
            other => panic!("{}: {other}", document.place()),
        }
    }

    /// The documents whose twin its family's reader reads to another value than the decoded
    /// text: the six effects of `EXPECTED_DIFFERENCES` (the `sulfur_cube_particle` of one install
    /// is another file than the other's), and two items that differ in order alone.
    const READ_DIFFERENTLY: &[&str] = &[
        "client-install-a: particles/sulfur_cube_particle.json",
        // The camera's `-0.0` (the body's sign) beside the twin's `-0`, read as 0.
        "client-install-b: cameras/death.json",
        // Equal member by member: the twin has its components in the order its author
        // wrote them, the body by name, and the reader keeps the order it reads.
        "client-install-b: items/red_shrub.json",
        "client-install-b: items/shelf_mushroom.json",
        "client-install-b: particles/geyser_base.particle.json",
        "client-install-b: particles/geyser_plume.particle.json",
        "client-install-b: particles/geyser_poof.particle.json",
        "client-install-b: particles/noxious_gas_particle.json",
        "client-install-b: particles/sulfur_bubble_particle.json",
        "client-install-b: particles/sulfur_cube_particle.json",
    ];

    #[test]
    fn every_vanilla_document_reads_as_its_twin_does() {
        let Some(documents) = baked() else {
            return;
        };
        let mut distinct = BTreeSet::new();
        let mut equal = 0;
        let mut differently = BTreeSet::new();
        let mut findings: BTreeMap<(&str, String), usize> = BTreeMap::new();
        let mut twin_findings: BTreeMap<(&str, String), usize> = BTreeMap::new();
        for document in &documents {
            let text = to_json_text(&document.bytes).unwrap();
            let (here, twin) = both(document, &text);
            for finding in here.1 {
                *findings.entry((document.install, finding)).or_default() += 1;
            }
            let Some(twin) = twin else {
                continue;
            };
            for finding in twin.1 {
                *twin_findings
                    .entry((document.install, finding))
                    .or_default() += 1;
            }
            if !distinct.insert((document.bytes.clone(), document.twin.clone())) {
                continue;
            }
            if here.0 == twin.0 {
                equal += 1;
            } else {
                differently.insert(format!("{}: {}", document.install, document.pack_path()));
            }
        }
        eprintln!(
            "{} distinct pairs, {equal} read to equal values; findings of the decoded text: \
             {findings:?}; of the twins: {twin_findings:?}",
            distinct.len()
        );
        for place in &differently {
            eprintln!("    {place:?},");
        }
        let differently: Vec<&str> = differently.iter().map(String::as_str).collect();
        assert_eq!(differently, READ_DIFFERENTLY);
    }
}
