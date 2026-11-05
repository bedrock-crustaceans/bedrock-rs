//! The repository of every pack found under some sources, and the world's pack selection.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, PackReport, Phase, Severity,
};
use bedrock_addon_core::document::{Origin, Rejected};
use bedrock_addon_core::json::{self, JsonSlice, MAX_JSON_DEPTH, Value};
use bedrock_addon_core::load::{LoadOptions, Place};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackOrigin, PackPath, PackRef, PackSide};
use bedrock_addon_core::version::SemVersion;
use facet::Facet;
use indexmap::IndexMap;
use uuid::Uuid;

use super::stack::compose::{Candidate, Composition, compose};
use super::stack::{Behavior, PackInstance, PackStack, Resource, Side, StackSet};
use super::{OpenError, Pack};

/// Every pack found under some sources, valid or not (vanilla's `ResourcePackRepository`).
#[derive(Default)]
pub struct PackRepository {
    /// Registered packs, in registration order.
    packs: Vec<Registered>,
    invalid: Vec<InvalidPack>,
    source_reports: Vec<SourceReport>,
    /// Registrations dropped by first-wins, as findings.
    registration: Diagnostics,
}

/// Prints the registered pack ids, the invalid packs and the source reports.
impl std::fmt::Debug for PackRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<_> = self
            .packs
            .iter()
            .map(|registered| registered.pack.id())
            .collect();
        f.debug_struct("PackRepository")
            .field("packs", &ids)
            .field("invalid", &self.invalid)
            .field("source_reports", &self.source_reports)
            .finish_non_exhaustive()
    }
}

/// One registered pack.
struct Registered {
    pack: Pack,
    origin: PackOrigin,
}

impl PackRepository {
    /// An empty repository.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one pack already in memory, first registration wins. Scanning sources
    /// is `RepositoryLoader::new(sources, options)`, whose result carries one
    /// [`SourceReport`] per source ([`PackRepository::source_reports`]); an unreadable root or
    /// an invalid pack is recorded there and in [`PackRepository::invalid`].
    ///
    /// A second pack with the same `(uuid, version)` and origin is dropped with a
    /// `DuplicateIdentifier` information in [`StackSet::selection`] of every stack set built from
    /// the repository.
    pub fn add_pack(&mut self, pack: Pack, origin: PackOrigin) {
        if let Err(dropped) = self.register(pack, origin) {
            // The game skips a duplicate registration without a log line, so the crate reports it
            // as an information.
            self.registration.push(Diagnostic::new(
                DiagnosticCode::DuplicateIdentifier { text: None },
                Severity::Info,
                Phase::Validate,
                Location::new(None, Some(manifest_path()), JsonPath::new(), None),
            ));
            drop(dropped);
        }
    }

    /// Registers `pack` unless a pack with its `(uuid, version)` and `origin` is registered;
    /// gives the pack back then (the game's dedup is silent; the caller reports it).
    // The registration key is `(PackIdVersion, PackOrigin)`; the first pack registered wins.
    pub(crate) fn register(&mut self, pack: Pack, origin: PackOrigin) -> Result<(), Pack> {
        let taken = self
            .packs
            .iter()
            .any(|registered| registered.origin == origin && registered.pack.id() == pack.id());
        if taken {
            return Err(pack);
        }
        self.packs.push(Registered { pack, origin });
        Ok(())
    }

    /// Records a pack that could not be opened.
    #[allow(dead_code, reason = "the pack loader calls it")]
    pub(crate) fn push_invalid(&mut self, invalid: InvalidPack) {
        self.invalid.push(invalid);
    }

    /// Records what one source yielded.
    #[allow(dead_code, reason = "the pack loader calls it")]
    pub(crate) fn push_source_report(&mut self, report: SourceReport) {
        self.source_reports.push(report);
    }

    /// Registered packs; the first registration of a `(PackIdVersion, PackOrigin)` wins.
    pub fn packs(&self) -> impl Iterator<Item = &Pack> + '_ {
        self.packs.iter().map(|registered| &registered.pack)
    }

    /// The pack with this identity: the first registered, whatever its origin.
    pub fn get(&self, id: &PackIdVersion) -> Option<&Pack> {
        self.packs().find(|pack| pack.id() == id)
    }

    /// Packs that could not be opened.
    pub fn invalid(&self) -> &[InvalidPack] {
        &self.invalid
    }

    /// One report per source of the `RepositoryLoader` that built this repository.
    pub fn source_reports(&self) -> &[SourceReport] {
        &self.source_reports
    }

    /// Builds both stacks of a world: dependencies, subpacks, base-game slices and the
    /// vanilla layer. Missing dependencies hide a pack and are diagnostics,
    /// kept in [`StackSet::selection`] (a collection operation: never fails).
    ///
    /// The behaviour stack is the world's list, each pack followed by its dependencies, then
    /// the base-game layer (the experimental packs whose experiment is on, the script libraries,
    /// the `vanilla` slices up to the world's base-game version, newest first, and `vanilla`).
    /// The resource stack is the world's list with its dependencies, then the resource packs
    /// behaviour packs depend on. A subpack the world does not choose is the highest memory
    /// tier the pack offers.
    pub fn stacks(&self, world: &WorldPacks, options: LoadOptions) -> StackSet {
        let candidates: Vec<Candidate<'_>> = self
            .packs
            .iter()
            .map(|registered| Candidate {
                facts: registered.pack.facts(),
                origin: registered.origin,
            })
            .collect();
        let Composition {
            behavior,
            resource,
            findings,
        } = compose(&candidates, world, &options);
        let mut selection = self.registration.clone();
        selection.extend(findings);
        StackSet {
            behavior: self.stack::<Behavior>(&behavior, options.clone(), selection),
            resource: self.stack::<Resource>(&resource, options, Diagnostics::default()),
        }
    }

    /// One stack from its plan.
    fn stack<S: Side>(
        &self,
        plan: &[super::stack::compose::Planned],
        options: LoadOptions,
        selection: Diagnostics,
    ) -> PackStack<S> {
        let mut instances = Vec::new();
        let mut hidden = Vec::new();
        for planned in plan {
            let registered = &self.packs[planned.candidate];
            let instance = PackInstance {
                pack: registered.pack.clone(),
                origin: registered.origin,
                subpack: planned.subpack,
                is_dependent: planned.is_dependent,
            };
            if planned.hidden {
                hidden.push(instance);
            } else {
                instances.push(instance);
            }
        }
        PackStack::new(instances, hidden, options, selection)
    }
}

/// `manifest.json`.
fn manifest_path() -> PackPath {
    PackPath::from_static("manifest.json")
}

/// What one source of a `RepositoryLoader` yielded: the packs found, an unreadable root, the
/// invalid packs.
#[derive(Debug)]
#[non_exhaustive]
pub struct SourceReport {
    /// The source's root (a directory of packs, an archive; `RootId::MEMORY` for packs given
    /// in memory).
    pub location: Place,
    /// The origin the source's packs were registered with.
    pub origin: PackOrigin,
    /// Every pack found, by identity, with its report (vanilla's `PackSourceReport`), in
    /// registration order; a pack first-wins dropped is listed too.
    pub packs: IndexMap<PackIdVersion, PackReport>,
    /// The root itself could not be read: the driver's error kind (`NotFound` for a missing
    /// root).
    pub unreadable: Option<std::io::ErrorKind>,
    /// Where the source's invalid packs are; the errors are in [`PackRepository::invalid`].
    pub invalid: Vec<Place>,
}

impl SourceReport {
    /// An empty report for the source at `location`.
    #[allow(dead_code, reason = "the pack loader calls it")]
    pub(crate) fn new(location: Place, origin: PackOrigin) -> Self {
        Self {
            location,
            origin,
            packs: IndexMap::new(),
            unreadable: None,
            invalid: Vec::new(),
        }
    }
}

/// A pack that could not be opened, with why.
#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidPack {
    /// Where the pack was found.
    pub location: Place,
    /// The origin of its source.
    pub origin: PackOrigin,
    /// Why there is no pack (its manifest findings in `OpenError::InvalidManifest`).
    pub error: OpenError,
}

impl InvalidPack {
    /// A pack at `location` that failed to open.
    #[allow(dead_code, reason = "the pack loader calls it")]
    pub(crate) fn new(location: Place, origin: PackOrigin, error: OpenError) -> Self {
        Self {
            location,
            origin,
            error,
        }
    }
}

/// The world's choice of packs: `world_behavior_packs.json` and `world_resource_packs.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldPacks {
    /// `world_behavior_packs.json`.
    pub behavior: Vec<PackSelection>,
    /// `world_resource_packs.json`.
    pub resource: Vec<PackSelection>,
}

impl WorldPacks {
    /// The behaviour-pack list file.
    pub const BEHAVIOR_FILE: &'static str = "world_behavior_packs.json";
    /// The resource-pack list file.
    pub const RESOURCE_FILE: &'static str = "world_resource_packs.json";

    /// Parses the world's pack lists; `None` for a file that does not exist (= empty list).
    /// Reading: `bedrock_addon_std::read_world_packs`.
    ///
    /// Each file is a JSON array of `{ "pack_id", "version", "subpack"? }`; `version` is an
    /// array of three numbers or a version string; `pack_id` a UUID of 32 hex digits or 36
    /// hyphenated characters, any case; other members are ignored. A file that is not such an
    /// array, or an entry with a member missing, of the wrong type or invalid, rejects the parse.
    ///
    /// # Errors
    ///
    /// [`Rejected`] when a file is not a JSON array of valid pack entries; see above.
    pub fn parse(behavior: Option<&[u8]>, resource: Option<&[u8]>) -> Result<Self, Rejected> {
        Ok(Self {
            behavior: parse_list(behavior, Self::BEHAVIOR_FILE, PackSide::Behavior)?,
            resource: parse_list(resource, Self::RESOURCE_FILE, PackSide::Resource)?,
        })
    }
}

/// One entry of a world pack list: `{ pack_id, version, subpack? }`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackSelection {
    /// `pack_id`.
    pub pack_id: Uuid,
    /// `version`.
    pub version: SemVersion,
    /// `subpack`.
    pub subpack: Option<String>,
}

/// An entry as written; unknown members are ignored.
#[derive(Facet)]
struct Entry {
    pack_id: Value,
    version: Value,
    subpack: Option<Value>,
}

/// A pack UUID in one of the two forms a manifest accepts: 32 hex digits, or 36 characters
/// hyphenated, any case; braced and `urn:uuid:` forms are not UUIDs here.
fn pack_uuid(text: &str) -> Option<Uuid> {
    matches!(text.len(), 32 | 36)
        .then(|| Uuid::parse_str(text).ok())
        .flatten()
}

/// One world pack list.
fn parse_list(
    bytes: Option<&[u8]>,
    file: &'static str,
    side: PackSide,
) -> Result<Vec<PackSelection>, Rejected> {
    let Some(bytes) = bytes else {
        return Ok(Vec::new());
    };
    let reject = |code: DiagnosticCode, json_path: JsonPath, span| {
        let origin = Origin::file(PackRef::app_data(), side, PackPath::from_static(file));
        let location = Location::new(None, Some(PackPath::from_static(file)), json_path, span);
        let diagnostics = std::iter::once(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            location,
        ))
        .collect();
        Rejected::new(origin, diagnostics)
    };
    let bytes = &*crate::json::blank_bom(bytes);
    let bytes = json::strip_bom(bytes);
    let malformed = |message: String| DiagnosticCode::MalformedInput {
        message,
        text: None,
    };
    if let Err(too_deep) = json::check_depth(bytes, MAX_JSON_DEPTH) {
        return Err(reject(
            malformed(too_deep.to_string()),
            JsonPath::new(),
            None,
        ));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|error| reject(malformed(error.to_string()), JsonPath::new(), None))?;
    let entries: Vec<Entry> = json::parse_at(JsonSlice::new(text)).map_err(|error| {
        let blank = text.trim().is_empty();
        let code = bedrock_addon_core::diagnostic::code_for_deserialize_error_kind(
            &error.error.kind,
            error.error.path.is_some(),
            blank,
        );
        reject(code, error.path, Some(error.span))
    })?;
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let at = |member: &str| {
                let mut path = JsonPath::new();
                path.push_index(index);
                path.push(member);
                path
            };
            let text = entry.pack_id.as_string().ok_or_else(|| {
                reject(
                    DiagnosticCode::InvalidType { text: None },
                    at("pack_id"),
                    None,
                )
            })?;
            let pack_id = pack_uuid(text.as_str()).ok_or_else(|| {
                reject(
                    DiagnosticCode::InvalidValue { text: None },
                    at("pack_id"),
                    None,
                )
            })?;
            let version = SemVersion::from_json(&entry.version).ok_or_else(|| {
                reject(
                    DiagnosticCode::InvalidValue { text: None },
                    at("version"),
                    None,
                )
            })?;
            let subpack = match entry.subpack {
                None => None,
                Some(value) => Some(
                    value
                        .as_string()
                        .map(|text| text.as_str().to_owned())
                        .ok_or_else(|| {
                            reject(
                                DiagnosticCode::InvalidType { text: None },
                                at("subpack"),
                                None,
                            )
                        })?,
                ),
            };
            Ok(PackSelection {
                pack_id,
                version,
                subpack,
            })
        })
        .collect()
}

impl StackSet {
    /// Pack selection and dependency findings of [`PackRepository::stacks`]; also in `load_all`'s
    /// report.
    pub fn selection(&self) -> &Diagnostics {
        // Both sides' findings are kept on the behaviour stack, so `selection` has one place to
        // read them.
        self.behavior.selection()
    }

    /// Runs the link pass over both stacks: every rule of the enabled modules that resolves a
    /// reference between definitions, of two families or of one, lints included. The same as
    /// [`link_with`](Self::link_with) with the default options.
    ///
    /// The pass reads the collections through the stacks' accessors, so it loads, from
    /// resident bytes, what it reads and is not loaded yet; it issues no request and performs
    /// no I/O (all it consults of the host is the `BuiltIns` of the `LoadOptions`). It
    /// never fails and never panics, whatever the stacks hold, and every finding is of
    /// [`Phase::Link`]. The findings come in a fixed order: by rule group, and inside a group
    /// as its rules tell them.
    ///
    /// The pass returns findings and changes nothing: a definition whose reference does not
    /// resolve stays in its registry as the load left it, also where the game would drop the
    /// document; the finding then has the severity of an error.
    pub fn link(&self) -> Diagnostics {
        self.link_with(&super::LinkOptions::default())
    }

    /// [`link`](Self::link) with `options`: with `LinkOptions { lints: false, ..Default::default() }` the pass
    /// returns no finding of information severity: the lints', and also a miss under a
    /// partial view and the note of a file that is not loaded.
    pub fn link_with(&self, options: &super::LinkOptions) -> Diagnostics {
        super::link::link(self, *options)
    }
}

#[cfg(test)]
mod tests {
    //! The repository and `PackStack::of` over packs built in-crate (the public-API twins in
    //! `tests/pack_repository.rs` build their packs through `Pack::from_files`).

    use bedrock_addon_core::pack_identity::PackType;

    use super::*;
    use crate::manifest::{Manifest, Module, ModuleType};
    use crate::pack::WrongPackType;
    use crate::pack::tests::{pack_from, pack_of};

    /// A pack with UUID `n`, version 1.0.0, the given module types and dependency texts.
    fn manifest(n: u8, modules: &[&str], dependencies: &[u8]) -> String {
        let modules: Vec<String> = modules
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                format!(
                    r#"{{ "type": "{kind}", "uuid": "00000000-0000-4000-9{i:03}-0000000000{n:02}", "version": [1, 0, 0] }}"#
                )
            })
            .collect();
        let dependencies: Vec<String> = dependencies
            .iter()
            .map(|d| {
                format!(
                    r#"{{ "uuid": "00000000-0000-4000-8000-0000000000{d:02}", "version": [1, 0, 0] }}"#
                )
            })
            .collect();
        format!(
            r#"{{ "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "00000000-0000-4000-8000-0000000000{n:02}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{}], "dependencies": [{}] }}"#,
            modules.join(", "),
            dependencies.join(", ")
        )
    }

    fn bp(n: u8, dependencies: &[u8], file: &'static str) -> Pack {
        pack_from(
            &manifest(n, &["data"], dependencies),
            &[(file, Some(b"x"))],
            PackOrigin::World,
        )
    }

    fn rp(n: u8) -> Pack {
        pack_from(&manifest(n, &["resources"], &[]), &[], PackOrigin::World)
    }

    fn select(pack: &Pack) -> PackSelection {
        PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        }
    }

    fn ids<S: Side>(stack: &PackStack<S>) -> Vec<PackIdVersion> {
        stack.packs().map(|pack| pack.id().clone()).collect()
    }

    #[test]
    fn the_first_registration_of_an_identity_and_origin_wins_with_an_information() {
        let first = bp(1, &[], "first.txt");
        let mut repository = PackRepository::new();
        repository.add_pack(first.clone(), PackOrigin::World);
        repository.add_pack(bp(1, &[], "second.txt"), PackOrigin::World);
        repository.add_pack(bp(1, &[], "global.txt"), PackOrigin::User);
        assert_eq!(repository.packs().count(), 2);
        let found = repository.get(first.id()).expect("registered");
        assert!(found.paths().any(|path| path.as_str() == "first.txt"));
        let world = WorldPacks {
            behavior: vec![select(&first)],
            resource: Vec::new(),
        };
        let set = repository.stacks(&world, LoadOptions::default());
        assert_eq!(ids(&set.behavior), [first.id().clone()]);
        // The game dedups in silence, so the crate reports it as an information.
        let warnings: Vec<_> = set
            .selection()
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .filter(|d| matches!(d.code, DiagnosticCode::DuplicateIdentifier { .. }))
            .collect();
        assert!(matches!(
            &warnings[..],
            [d] if matches!(d.code, DiagnosticCode::DuplicateIdentifier { .. })
        ));
    }

    #[test]
    fn stacks_hide_a_pack_whose_dependency_is_missing_and_auto_add_a_resource_dependency() {
        let resource = rp(10);
        let needs_resource = bp(1, &[10], "a");
        let needs_ghost = bp(2, &[9], "b");
        let mut repository = PackRepository::new();
        for pack in [&needs_resource, &needs_ghost, &resource] {
            repository.add_pack(pack.clone(), PackOrigin::World);
        }
        let world = WorldPacks {
            behavior: vec![select(&needs_resource), select(&needs_ghost)],
            resource: Vec::new(),
        };
        let set = repository.stacks(&world, LoadOptions::default());
        assert_eq!(ids(&set.behavior), [needs_resource.id().clone()]);
        assert_eq!(set.behavior.hidden().len(), 1);
        assert_eq!(ids(&set.resource), [resource.id().clone()]);
        let codes: Vec<&DiagnosticCode> = set
            .selection()
            .iter()
            .map(|d| &d.code)
            .filter(|code| {
                matches!(
                    code,
                    DiagnosticCode::MissingDependency { .. }
                        | DiagnosticCode::DependencyAutoAdded { .. }
                )
            })
            .collect();
        assert!(matches!(
            codes[..],
            [
                DiagnosticCode::MissingDependency { .. },
                DiagnosticCode::DependencyAutoAdded { .. }
            ]
        ));
    }

    #[test]
    fn a_stack_of_packs_refuses_one_without_a_module_of_its_kind() {
        let resource = rp(3);
        let error =
            PackStack::<Behavior>::of([bp(1, &[], "a"), resource.clone()], LoadOptions::default())
                .expect_err("a resource pack on the behaviour stack");
        assert_eq!(
            error,
            WrongPackType {
                pack: resource.id().clone(),
                expected: PackType::Behavior,
                found: PackType::Resource,
            }
        );
        // Data and resources modules in one pack: the manifest readers refuse that mix
        // (`MultipleModules`), so the manifest is built by hand.
        let mut manifest = Manifest::with_ids(
            "pack 4",
            PackType::Behavior,
            Uuid::from_u128(4),
            Uuid::from_u128(5),
        );
        manifest.modules.push(Module::new(
            ModuleType::Resources,
            Uuid::from_u128(6),
            SemVersion::new(1, 0, 0),
        ));
        let both = pack_of(manifest, &[], PackOrigin::World);
        assert!(PackStack::<Behavior>::of([both.clone()], LoadOptions::default()).is_ok());
        let stack =
            PackStack::<Resource>::of([both.clone()], LoadOptions::default()).expect("accepted");
        let refs: Vec<_> = stack
            .packs()
            .map(|pack| (pack.index(), pack.name().to_owned()))
            .collect();
        assert_eq!(refs, [(Some(0), "pack 4".to_owned())]);
    }

    /// A pack with a data and a resources module is on both stacks. Each stack's own
    /// report has the pack's icon lints; the report of both lists them once per pack, the
    /// behaviour stack's being the ones kept.
    #[test]
    fn the_icon_lints_of_a_pack_on_both_stacks_are_reported_once_by_the_set() {
        // The manifest readers refuse the mix (`MultipleModules`): built by hand.
        let both = |n: u128, name: &str, files: &[(&'static str, Option<&'static [u8]>)]| {
            let mut manifest = Manifest::with_ids(
                name,
                PackType::Behavior,
                Uuid::from_u128(n),
                Uuid::from_u128(n + 1),
            );
            manifest.modules.push(Module::new(
                ModuleType::Resources,
                Uuid::from_u128(n + 2),
                SemVersion::new(1, 0, 0),
            ));
            pack_of(manifest, files, PackOrigin::World)
        };
        let without = both(0x40, "without", &[]);
        let not_png = both(0x50, "not png", &[("pack_icon.png", Some(b"GIF89a"))]);
        let set = StackSet {
            behavior: PackStack::of(
                [bp(1, &[], "a"), without.clone(), not_png.clone()],
                LoadOptions::default(),
            )
            .expect("behaviour packs"),
            resource: PackStack::of([without, rp(3), not_png], LoadOptions::default())
                .expect("resource packs"),
        };
        let icons = |report: &crate::pack::StackReport<'_>| {
            let mut found: Vec<(String, &'static str, Severity)> = report
                .iter()
                .filter(|finding| {
                    finding
                        .location
                        .path
                        .as_ref()
                        .is_some_and(|path| path.as_str() == "pack_icon.png")
                })
                .map(|finding| {
                    let pack = finding.location.pack.as_ref().expect("its pack");
                    (
                        pack.name().to_owned(),
                        finding.code.name(),
                        finding.severity,
                    )
                })
                .collect();
            found.sort();
            found
        };
        let missing = |name: &str| (name.to_owned(), "PackIconMissing", Severity::Info);
        let refused = ("not png".to_owned(), "InvalidValue", Severity::Info);
        // Each stack says it of its own packs.
        assert_eq!(
            icons(&set.behavior.report()),
            [refused.clone(), missing("pack 1"), missing("without")]
        );
        assert_eq!(
            icons(&set.resource.report()),
            [refused.clone(), missing("pack 3"), missing("without")]
        );
        // Both together: once per pack.
        let expected = [
            refused,
            missing("pack 1"),
            missing("pack 3"),
            missing("without"),
        ];
        assert_eq!(icons(&set.report()), expected);
        let report = set.load_all();
        assert_eq!(icons(&report), expected);
        // The finding kept is the behaviour stack's: it names the pack as that stack does.
        for name in ["without", "not png"] {
            let on_behavior = set
                .behavior
                .packs()
                .find(|pack| pack.name() == name)
                .expect("on the behaviour stack");
            let kept = report
                .iter()
                .find(|finding| {
                    finding
                        .location
                        .pack
                        .as_ref()
                        .is_some_and(|pack| pack.name() == name)
                })
                .expect("reported");
            assert_eq!(kept.location.pack.as_ref(), Some(&on_behavior), "{name}");
        }
    }
}

#[cfg(test)]
mod debug_tests {
    use super::PackRepository;

    #[test]
    fn repository_prints_its_contents() {
        let text = format!("{:?}", PackRepository::new());
        assert!(text.starts_with("PackRepository"), "{text}");
        assert!(text.contains("packs") && text.contains("invalid"), "{text}");
    }
}

#[cfg(test)]
mod common_trait_tests {
    use std::collections::HashSet;

    use super::{PackSelection, WorldPacks};
    use bedrock_addon_core::version::SemVersion;
    use uuid::Uuid;

    #[test]
    fn world_packs_default_is_two_empty_lists() {
        let empty = WorldPacks::default();
        assert!(empty.behavior.is_empty() && empty.resource.is_empty());
        assert_eq!(empty, WorldPacks::parse(None, None).unwrap());
    }

    #[test]
    fn a_selection_compares_and_hashes() {
        let selection = PackSelection {
            pack_id: Uuid::from_u128(1),
            version: SemVersion::new(1, 0, 0),
            subpack: None,
        };
        let set: HashSet<PackSelection> = [selection.clone(), selection.clone()].into();
        assert_eq!(set.len(), 1);
        assert_ne!(
            selection,
            PackSelection {
                subpack: Some("x".into()),
                ..selection.clone()
            }
        );
    }
}
