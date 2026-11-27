//! What every link test shares: a world in memory, linked; a host that answers from a list;
//! and selectors over the findings.

#![allow(dead_code, reason = "each group uses part of the helpers")]

use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Kind, Severity};
use bedrock_addon::load::{BuiltIns, LoadOptions};
use bedrock_addon::pack::{
    LinkOptions, Pack, PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks,
};

use super::pack_common::{bp, rp};

/// `text` for the life of the test process: a pack's files are `'static`.
pub fn leak(text: &str) -> &'static [u8] {
    Box::leak(text.to_owned().into_boxed_str()).as_bytes()
}

/// The behaviour pack `n` (`bp <n>`) with `files`, each `(path, text)`.
pub fn behavior_pack(n: u128, files: &[(&'static str, &str)]) -> Pack {
    let mut spec = bp(n);
    for (path, text) in files {
        spec = spec.file(path, leak(text));
    }
    spec.open()
}

/// The resource pack `n` (`rp <n>`) with `files`, each `(path, text)`.
pub fn resource_pack(n: u128, files: &[(&'static str, &str)]) -> Pack {
    let mut spec = rp(n);
    for (path, text) in files {
        spec = spec.file(path, leak(text));
    }
    spec.open()
}

/// The behaviour pack `n` with `files` as bytes: for a file that is no text (a structure).
pub fn behavior_pack_of(n: u128, files: &[(&'static str, &'static [u8])]) -> Pack {
    let mut spec = bp(n);
    for (path, bytes) in files {
        spec = spec.file(path, bytes);
    }
    spec.open()
}

/// The resource pack `n` with `files` as bytes: for an image, a font, a sound.
pub fn resource_pack_of(n: u128, files: &[(&'static str, &'static [u8])]) -> Pack {
    let mut spec = rp(n);
    for (path, bytes) in files {
        spec = spec.file(path, bytes);
    }
    spec.open()
}

/// The pack `n` with one module of `module` (`"data"`, `"resources"`) as a host opens it from
/// a folder and before any content is loaded: the manifest is resident, every file of `files`
/// is indexed and **not resident** (`Pack::is_resident` is `false`), so a collection that
/// walks it reports `FileNotLoaded` and a rule that asks for its bytes gets none.
pub fn unloaded_pack(n: u128, module: &str, files: &[(&'static str, &[u8])]) -> Pack {
    use bedrock_addon::load::{PackLoader, RootId};
    use bedrock_addon::pack::OpenOptions;

    use super::responder::{Order, Responder, manifest};
    let text = manifest(n, module);
    let mut all: Vec<(&str, &[u8])> = vec![("manifest.json", text.as_bytes())];
    all.extend(files.iter().copied());
    let mut responder = Responder::new(Order::Issue);
    responder.pack(1, "", &all);
    let mut loader = PackLoader::directory(RootId(1), OpenOptions::default());
    responder.drive(&mut loader).expect("the folder opens")
}

/// The stacks of a world that selects every given pack, in the given order (the first is the
/// top of its stack), loaded with `options`. The packs are registered behaviour packs first.
pub fn stacks_of(behavior: Vec<Pack>, resource: Vec<Pack>, options: LoadOptions) -> StackSet {
    let select = |packs: &[Pack]| -> Vec<PackSelection> {
        packs
            .iter()
            .map(|pack| PackSelection {
                pack_id: pack.id().uuid,
                version: pack.id().version.clone(),
                subpack: None,
            })
            .collect()
    };
    let world = WorldPacks {
        behavior: select(&behavior),
        resource: select(&resource),
    };
    let mut repository = PackRepository::new();
    for pack in behavior.into_iter().chain(resource) {
        repository.add_pack(pack, PackOrigin::User);
    }
    repository.stacks(&world, options)
}

/// [`stacks_of`] for packs registered each with its own origin: `PackOrigin::Package` makes
/// a pack one the game ships (a base-game pack).
pub fn stacks_from(
    behavior: Vec<(Pack, PackOrigin)>,
    resource: Vec<(Pack, PackOrigin)>,
    options: LoadOptions,
) -> StackSet {
    let select = |packs: &[(Pack, PackOrigin)]| -> Vec<PackSelection> {
        packs
            .iter()
            .map(|(pack, _)| PackSelection {
                pack_id: pack.id().uuid,
                version: pack.id().version.clone(),
                subpack: None,
            })
            .collect()
    };
    let world = WorldPacks {
        behavior: select(&behavior),
        resource: select(&resource),
    };
    let mut repository = PackRepository::new();
    for (pack, origin) in behavior.into_iter().chain(resource) {
        repository.add_pack(pack, origin);
    }
    repository.stacks(&world, options)
}

/// The stacks of a world with one behaviour pack (`bp 1`, the `behavior` files) and one
/// resource pack (`rp 2`, the `resource` files), loaded with `options`.
pub fn stacks(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    options: LoadOptions,
) -> StackSet {
    stacks_of(
        vec![behavior_pack(1, behavior)],
        vec![resource_pack(2, resource)],
        options,
    )
}

/// Whether the resource file `path` is a source of a registry of the game that an add-on
/// does not carry: a file below `materials/`, or one of the two atlas files.
fn is_game_source(path: &str) -> bool {
    path.starts_with("materials/")
        || path == "textures/terrain_texture.json"
        || path == "textures/item_texture.json"
}

/// [`stacks`] for a test of a row's own severity about a material or an atlas tile: the
/// material files and the atlas files of `resource` are a pack the game ships (`rp 3`, a
/// base-game pack below `rp 2`), so the pass has the whole view of those kinds.
/// Every other file is the add-on's, in `rp 2`.
pub fn game_stacks(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    options: LoadOptions,
) -> StackSet {
    let (shipped, own): (Vec<_>, Vec<_>) = resource
        .iter()
        .copied()
        .partition(|(path, _)| is_game_source(path));
    stacks_from(
        vec![(behavior_pack(1, behavior), PackOrigin::User)],
        vec![
            (resource_pack(2, &own), PackOrigin::User),
            (resource_pack(3, &shipped), PackOrigin::Package),
        ],
        options,
    )
}

/// [`link`] over [`game_stacks`].
pub fn game_link(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
) -> Diagnostics {
    game_link_with(
        behavior,
        resource,
        LoadOptions::default(),
        LinkOptions::default(),
    )
}

/// [`link_with`] over [`game_stacks`].
pub fn game_link_with(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    load: LoadOptions,
    link: LinkOptions,
) -> Diagnostics {
    game_stacks(behavior, resource, load).link_with(&link)
}

/// The link findings of a world with one behaviour pack (`behavior` files) and one resource
/// pack (`resource` files): the default options, no host, every rule.
pub fn link(behavior: &[(&'static str, &str)], resource: &[(&'static str, &str)]) -> Diagnostics {
    link_with(
        behavior,
        resource,
        LoadOptions::default(),
        LinkOptions::default(),
    )
}

/// [`link`] with the load options `load` (experiments, a host) and the link options `link`.
pub fn link_with(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    load: LoadOptions,
    link: LinkOptions,
) -> Diagnostics {
    stacks(behavior, resource, load).link_with(&link)
}

/// A host whose game has exactly the listed `(kind, id)` pairs: `kind` is the identifier
/// kind's `Kind::NAME` (`"block"`, `"item"`, `"level sound event"`), `id` the name in the
/// kind's canonical form (`minecraft:stone`).
pub struct Listed(pub &'static [(&'static str, &'static str)]);

impl BuiltIns for Listed {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        self.0.contains(&(kind, id))
    }
}

/// The load options of a world whose host has exactly `list`.
pub fn hosted(list: &'static [(&'static str, &'static str)]) -> LoadOptions {
    LoadOptions {
        built_ins: Some(Arc::new(Listed(list))),
        ..LoadOptions::default()
    }
}

/// What a reference finding says: the kind and the name as written, the severity, and the JSON
/// path of the member.
pub type Reference = (&'static str, String, Severity, String);

/// The reference a finding is about, when it has one.
pub fn reference(finding: &Diagnostic) -> Option<Reference> {
    match finding.detail.as_deref()? {
        DiagnosticDetail::Reference { kind, id, .. } => Some((
            *kind,
            id.clone(),
            finding.severity,
            finding.location.json_path.to_string(),
        )),
        _ => None,
    }
}

/// The references of the findings whose code is called `code` (`"UnresolvedReference"`,
/// `"InvalidReference"`, `"ReferenceCycle"`, `"TextureNotFound"`, `"UnknownSound"`,
/// `"UnresolvedLangKey"`), in order.
pub fn references(findings: &Diagnostics, code: &str) -> Vec<Reference> {
    findings
        .iter()
        .filter(|finding| finding.code.id() == code)
        .filter_map(reference)
        .collect()
}

/// The unresolved-reference findings, plain or worded: `(kind, name as written, severity,
/// JSON path)`.
pub fn unresolved(findings: &Diagnostics) -> Vec<Reference> {
    findings
        .iter()
        .filter(|finding| finding.code.kind() == Kind::UnresolvedReference)
        .filter_map(reference)
        .collect()
}

/// The findings in the file `path`.
pub fn in_file<'a>(findings: &'a Diagnostics, path: &str) -> Vec<&'a Diagnostic> {
    findings
        .iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|file| file.as_str() == path)
        })
        .collect()
}

/// whether `finding` is the line of the game's link pass
/// (`LinkedAssetValidator`) that a family tells beside its own finding of the same miss: the
/// validator's text, at a member another finding of `all` is at too.
pub fn is_second_validator_line(finding: &Diagnostic, all: &Diagnostics) -> bool {
    let text = finding.code.to_string();
    let line =
        text.contains(":  Missing referenced asset ") || text.contains(":  Invalid asset path ");
    line && all.iter().any(|other| {
        !std::ptr::eq(other, finding)
            && other.location.path == finding.location.path
            && other.location.json_path == finding.location.json_path
            && other.code.to_string() != text
    })
}

/// `found` without the second validator lines ([`is_second_validator_line`]): for the tests
/// of a family's own finding.
pub fn family_findings(found: &Diagnostics) -> Diagnostics {
    let mut kept = Diagnostics::default();
    for finding in found {
        if !is_second_validator_line(finding, found) {
            kept.push(finding.clone());
        }
    }
    kept
}

/// `found` without any finding that carries a validator line.
pub fn without_validator_lines(found: &Diagnostics) -> Diagnostics {
    let mut kept = Diagnostics::default();
    for finding in found {
        let text = finding.code.to_string();
        if !(text.contains(":  Missing referenced asset ")
            || text.contains(":  Invalid asset path "))
        {
            kept.push(finding.clone());
        }
    }
    kept
}
