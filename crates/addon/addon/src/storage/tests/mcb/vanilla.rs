//! Every MCB document of the game's installs against its JSON twin: the loose file of the
//! same path, which each pack ships beside the archive that shadows it.
//!
//! A twin is not a function of the body: the same bytes have twins that differ in a member
//! written at its default (`"surface_only": false`) or in one the bake does not keep
//! (`velocity` of `emitter_local_space`), and the newest effects of the game ship a twin in the
//! form their author wrote (a gradient of `#AARRGGBB` strings keyed `0.0`) where the body
//! has the baked one. So the comparison is member by member, and what differs is counted by
//! kind and place and pinned here, each line with its reason.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bytes::Bytes;
use facet_value::Value;

use super::super::game_files;
use crate::storage::brarchive::Brarchive;
use crate::storage::magic::{FileType, detect};
use crate::storage::mcb::{McbHeader, to_json_text};

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
    #[cfg(all(
        feature = "voxel_shape",
        feature = "camera",
        feature = "item",
        feature = "particle",
        feature = "trade"
    ))]
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

/// The differences between a decoded document and its twin, as `kind place detail` lines.
#[derive(Default)]
struct Differences {
    lines: Vec<String>,
    /// Numbers equal in value: written as a float by both, or as a float here and without
    /// a fraction in the twin (the twins of the baked effects write `5` for 5.0).
    float_in_both: usize,
    float_here_only: usize,
}

/// A place without what tells one document from another: array positions, gradient times.
fn general(place: &str) -> String {
    let mut out = String::new();
    for part in place.split('/') {
        if part.is_empty() {
            continue;
        }
        out.push('/');
        if part.parse::<f64>().is_ok() {
            out.push('*');
        } else {
            out.push_str(part);
        }
    }
    out
}

fn short(value: &Value) -> String {
    let text = crate::json::to_facet_json(value).unwrap();
    if text.len() > 48 {
        format!("{}…", text.chars().take(48).collect::<String>())
    } else {
        text
    }
}

impl Differences {
    fn note(&mut self, kind: &str, place: &str, detail: &str) {
        self.lines
            .push(format!("{kind} {} {detail}", general(place)));
    }

    fn compare(&mut self, here: &Value, twin: &Value, place: &str) {
        if let (Some(here), Some(twin)) = (here.as_object(), twin.as_object()) {
            for (key, value) in twin {
                let place = format!("{place}/{}", key.as_str());
                match here.get(key.as_str()) {
                    Some(here) => self.compare(here, value, &place),
                    None => self.note("twin-only", &place, &short(value)),
                }
            }
            for (key, value) in here {
                if twin.get(key.as_str()).is_none() {
                    let place = format!("{place}/{}", key.as_str());
                    self.note("here-only", &place, &short(value));
                }
            }
        } else if let (Some(here_items), Some(twin_items)) = (here.as_array(), twin.as_array()) {
            if here_items.len() != twin_items.len() {
                self.note("length", place, &format!("{} {}", short(here), short(twin)));
                return;
            }
            for (index, (here, twin)) in here_items.iter().zip(twin_items.iter()).enumerate() {
                self.compare(here, twin, &format!("{place}/{index}"));
            }
        } else if let (Some(here_number), Some(twin_number)) = (here.as_number(), twin.as_number())
        {
            // A float of the body is 32 bits wide; the twin prints it in up to nine digits.
            #[allow(clippy::cast_possible_truncation)]
            let same = if here_number.is_float() {
                (here_number.to_f64_lossy() as f32).to_bits()
                    == (twin_number.to_f64_lossy() as f32).to_bits()
            } else {
                twin_number.is_integer() && here_number.to_i64() == twin_number.to_i64()
            };
            if !same {
                self.note("value", place, &format!("{} {}", short(here), short(twin)));
            } else if here_number.is_float() && twin_number.is_float() {
                self.float_in_both += 1;
            } else if here_number.is_float() {
                self.float_here_only += 1;
            }
        } else if here != twin {
            self.note("value", place, &format!("{} {}", short(here), short(twin)));
        }
    }
}

fn parse(text: &str) -> Value {
    crate::json::from_str_jsonc(text.trim_start_matches('\u{feff}')).unwrap()
}

/// What the decoded documents and their twins differ in, each with its count over the
/// distinct documents of the three installs, and why.
const EXPECTED_DIFFERENCES: &[(&str, usize)] = &[
    // The 17 bundles (34 distinct pairs): the body has the stack size 1, the
    // value the game reads into `max_stack_size`'s one member; the 1.26.30 twins write `{}`.
    (
        "here-only /minecraft:item/components/minecraft:max_stack_size/value 1",
        34,
    ),
    // The three geyser effects and `sulfur_cube_particle`: the body has the
    // billboard's `uv` and `uv_size` at their defaults, the twin has neither.
    (
        "here-only /particle_effect/components/minecraft:particle_appearance_billboard/uv/uv [0,0]",
        4,
    ),
    (
        "here-only /particle_effect/components/minecraft:particle_appearance_billboard/uv/uv_size [1,1]",
        4,
    ),
    // `sulfur_bubble_particle` and `noxious_gas_particle`: the twin's gradient is keyed
    // `0.0`, `0.5`, `1.0` and its colours are `#AARRGGBB` text; the body has the times as
    // `0.000000` and each colour as red, green, blue, alpha (`#171E3942` is 0x1E, 0x39,
    // 0x42, 0x17 over 255).
    (
        "here-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* [0.11764706,0.22352941,0.25882354,0.09019608]",
        1,
    ),
    (
        "here-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* [0.3372549,0.4862745,0.5176471,1]",
        1,
    ),
    (
        "here-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* [1,1,1,0]",
        1,
    ),
    (
        "here-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* [1,1,1,1]",
        2,
    ),
    // A member the twin writes at its default, which a body does not tell from an absent
    // one (the same bytes have a twin without it).
    (
        "twin-only /particle_effect/components/minecraft:emitter_lifetime_once/active_time 0",
        2,
    ),
    // Not in the body: the bake drops `velocity`.
    (
        "twin-only /particle_effect/components/minecraft:emitter_local_space/velocity true",
        3,
    ),
    (
        "twin-only /particle_effect/components/minecraft:emitter_shape_box/surface_only false",
        1,
    ),
    (
        "twin-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* \"#00FFFFFF\"",
        1,
    ),
    (
        "twin-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* \"#171E3942\"",
        1,
    ),
    (
        "twin-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* \"#FF567C84\"",
        1,
    ),
    (
        "twin-only /particle_effect/components/minecraft:particle_appearance_tinting/color/gradient/* \"#ffffffff\"",
        2,
    ),
    (
        "twin-only /particle_effect/components/minecraft:particle_motion_collision/coefficient_of_restitution 0",
        2,
    ),
    (
        "twin-only /particle_effect/components/minecraft:particle_motion_dynamic/linear_acceleration [0,0,0]",
        2,
    ),
    // The one camera: the body has the float `-0.0` and the decoded text keeps its sign; the
    // twin has `-0`, an integer token, which is read as 0.
    (
        "value /minecraft:camera_entity/components/minecraft:camera_offset/entity/* 0 0",
        1,
    ),
    // `sulfur_cube_particle` of the second install: the body has each expression with its Molang
    // version (13), the twin is the file as its author wrote it, plain text.
    (
        "value /particle_effect/components/minecraft:emitter_shape_point/direction/* {\"expression\":\"Math.Random(-1.0, 1.0)\",\"version\"… \"Math.Random(-1.0, 1.0)\"",
        2,
    ),
    (
        "value /particle_effect/components/minecraft:emitter_shape_point/direction/* {\"expression\":\"Math.Random(0.0, 1.0)\",\"version\":… \"Math.Random(0.0, 1.0)\"",
        1,
    ),
    (
        "value /particle_effect/components/minecraft:emitter_shape_point/offset/* {\"expression\":\"Math.random(-0.25f, 0.25f)\",\"vers… \"Math.random(-0.25f, 0.25f)\"",
        3,
    ),
    (
        "value /particle_effect/components/minecraft:particle_appearance_billboard/size/* {\"expression\":\"(variable.particle_random_1 * 0.0… \"(variable.particle_random_1 * 0.04 + 0.04)\"",
        2,
    ),
    (
        "value /particle_effect/components/minecraft:particle_appearance_billboard/uv/uv/* {\"expression\":\"4 * (variable.particle_random_1*3… \"4 * (variable.particle_random_1*3)\"",
        1,
    ),
    (
        "value /particle_effect/components/minecraft:particle_appearance_billboard/uv/uv/* {\"expression\":\"4 * (variable.particle_random_2*3… \"4 * (variable.particle_random_2*3)\"",
        1,
    ),
    (
        "value /particle_effect/components/minecraft:particle_appearance_billboard/uv/uv_size/* {\"expression\":\"4\",\"version\":13} \"4\"",
        2,
    ),
    (
        "value /particle_effect/components/minecraft:particle_initial_speed {\"expression\":\"Math.random(1, 4)\",\"version\":13} \"Math.random(1, 4)\"",
        1,
    ),
    (
        "value /particle_effect/components/minecraft:particle_lifetime_expression/max_lifetime {\"expression\":\"0.2f / (Math.random(0.0f, 1.0f) *… \"0.2f / (Math.random(0.0f, 1.0f) * 0.9f + 0.1f)\"",
        1,
    ),
];

#[test]
fn every_vanilla_document_decodes_to_what_its_twin_has() {
    let Some(documents) = baked() else {
        return;
    };
    // install, payload key → documents, with a twin
    let mut counts: BTreeMap<(&str, String), (usize, usize)> = BTreeMap::new();
    let mut distinct = BTreeSet::new();
    let mut differences = Differences::default();
    let mut differing = BTreeSet::new();
    for document in &documents {
        let text = to_json_text(&document.bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", document.place()));
        let count = counts
            .entry((document.install, document.key()))
            .or_default();
        count.0 += 1;
        let Some(twin) = &document.twin else {
            eprintln!("no twin: {}", document.place());
            continue;
        };
        count.1 += 1;
        if !distinct.insert((document.bytes.clone(), twin.clone())) {
            continue;
        }
        let before = differences.lines.len();
        differences.compare(&parse(&text), &parse(twin), "");
        if differences.lines.len() > before {
            differing.insert(document.name.clone());
        }
    }
    for ((install, key), (documents, twins)) in &counts {
        eprintln!("{install}: {key}: {documents} documents decoded, {twins} with a twin");
    }
    eprintln!(
        "{} distinct pairs; {} numbers a float in both, {} a float here and without a \
         fraction in the twin; {} documents differ from their twin: {differing:?}",
        distinct.len(),
        differences.float_in_both,
        differences.float_here_only,
        differing.len(),
    );
    let mut found: BTreeMap<&str, usize> = BTreeMap::new();
    for line in &differences.lines {
        *found.entry(line).or_default() += 1;
    }
    for (line, count) in &found {
        eprintln!("    ({line:?}, {count}),");
    }
    let found: Vec<(&str, usize)> = found.into_iter().collect();
    assert_eq!(found, EXPECTED_DIFFERENCES);
}

/// The family readers on the decoded text and on the twin.
#[cfg(all(
    feature = "voxel_shape",
    feature = "camera",
    feature = "item",
    feature = "particle",
    feature = "trade"
))]
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
            .map(|finding| format!("{:?} {}", finding.severity, finding.code.name()))
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
