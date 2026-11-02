//! Stack composition: which registered packs a world's stacks hold, in what order, with
//! which subpack, and which are hidden. Pure over the packs' manifest facts, so the rules are
//! tested without files.
//!
//! Order of a behaviour stack, top first: the world's list (each pack followed by its
//! dependencies, depth first), in an education world the app-bundled chemistry behaviour packs
//! then the base-game layer (enabled experimental packs, the two script libraries, the
//! `vanilla` slices newest first, `vanilla`). A resource stack: the world's list with
//! dependencies, then the resource packs behaviour packs depend on. Duplicates by
//! `(pack, subpack)` keep the first.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::load::LoadOptions;
use bedrock_addon_core::pack_identity::{PackIdVersion, PackOrigin, PackPath, PackSide, PackType};
use bedrock_addon_core::version::BaseGameVersion;

use super::vanilla::{
    IN_PACKAGE_VERSION, SERVER_LIBRARY, SERVER_UI_LIBRARY, VANILLA_BEHAVIOR,
    VANILLA_EXPERIMENTAL_PACKS,
};
use crate::pack::{PackFacts, PackSelection, WorldPacks};

/// A registered pack as composition sees it, in registration order.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Candidate<'a> {
    /// What the manifest says.
    pub(crate) facts: &'a PackFacts,
    /// Where the pack came from.
    pub(crate) origin: PackOrigin,
}

/// One planned stack entry (vanilla's `PackInstance`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Planned {
    /// The registered pack, by its index among the candidates.
    pub(crate) candidate: usize,
    /// The active subpack, by its index in the pack's `subpacks`.
    pub(crate) subpack: Option<usize>,
    /// Added as another pack's dependency.
    pub(crate) is_dependent: bool,
    /// Force-hidden by a missing dependency: kept on the stack, never loaded.
    pub(crate) hidden: bool,
}

/// Both stacks of a world, top first, and the selection findings.
#[derive(Debug, Default)]
pub(crate) struct Composition {
    /// The behaviour stack.
    pub(crate) behavior: Vec<Planned>,
    /// The resource stack.
    pub(crate) resource: Vec<Planned>,
    /// What selection and dependency resolution found, in the order found.
    pub(crate) findings: Diagnostics,
}

/// Composes both stacks of a world from the registered packs.
pub(crate) fn compose(
    candidates: &[Candidate<'_>],
    world: &WorldPacks,
    options: &LoadOptions,
) -> Composition {
    let mut composer = Composer {
        candidates,
        findings: Diagnostics::default(),
    };
    let mut behavior = Vec::new();
    let wanted_resources = composer.level(
        &mut behavior,
        PackSide::Behavior,
        &world.behavior,
        WorldPacks::BEHAVIOR_FILE,
    );
    let chemistry_resources = if options.education {
        composer.education(&mut behavior, options)
    } else {
        Vec::new()
    };
    composer.base_game_behavior(&mut behavior, options);
    let mut resource = Vec::new();
    // A resource pack's dependencies on behaviour packs ask for nothing.
    let _ = composer.level(
        &mut resource,
        PackSide::Resource,
        &world.resource,
        WorldPacks::RESOURCE_FILE,
    );
    composer.auto_add_resources(
        &mut resource,
        &behavior,
        &wanted_resources,
        Announce::Additions,
    );
    // The game's split of the world's stack moves them over without a word.
    composer.auto_add_resources(
        &mut resource,
        &behavior,
        &chemistry_resources,
        Announce::Silent,
    );
    Composition {
        behavior,
        resource,
        findings: composer.findings,
    }
}

/// The subpack the game activates when none is chosen: of the subpacks whose tier is at most
/// `max_memory_tier` (`None` = the highest tier, what a server passes), the highest tier, ties
/// to the later subpack; `None` when no subpack qualifies.
pub(crate) fn default_subpack(facts: &PackFacts, max_memory_tier: Option<u8>) -> Option<usize> {
    let ceiling = max_memory_tier.unwrap_or(HIGHEST_MEMORY_TIER);
    let mut best: Option<(usize, u8)> = None;
    for (index, subpack) in facts.subpacks.iter().enumerate() {
        if subpack.memory_tier <= ceiling
            && best.is_none_or(|(_, tier)| subpack.memory_tier >= tier)
        {
            best = Some((index, subpack.memory_tier));
        }
    }
    best.map(|(index, _)| index)
}

/// The highest hardware memory tier.
const HIGHEST_MEMORY_TIER: u8 = 5;

/// How dependencies of another pack type are treated.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    /// The world's packs: a behaviour pack's resource dependency is auto-added.
    Level,
    /// The base-game layer: dependencies of the other side are dropped, as the game's split of
    /// its scratch stack drops the `vanilla` resource pack the `vanilla` behaviour pack pulls in.
    BaseGame,
    /// The chemistry packs of an education world: added to the world's stack as its own packs
    /// are, but an unmet dependency hides the pack with an information only (the game marks it
    /// in error without a log line).
    Education,
}

/// Why a pack is added to a stack.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reason {
    /// The world's list (or a base-game or education layer) names the pack.
    Listed,
    /// Another pack depends on it.
    Dependency,
}

/// Whether adding a dependency is reported.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Announce {
    /// Each addition is a `DependencyAutoAdded` information.
    Additions,
    /// No finding.
    Silent,
}

struct Composer<'c> {
    candidates: &'c [Candidate<'c>],
    findings: Diagnostics,
}

impl Composer<'_> {
    /// The first registered pack with this identity (the game looks a pack up by exact id and
    /// version).
    fn find(&self, id: &PackIdVersion) -> Option<usize> {
        self.candidates
            .iter()
            .position(|candidate| candidate.facts.id == *id)
    }

    fn facts(&self, candidate: usize) -> &PackFacts {
        self.candidates[candidate].facts
    }

    fn finding(&mut self, code: DiagnosticCode, severity: Severity, location: Location) {
        self.findings
            .push(Diagnostic::new(code, severity, Phase::Validate, location));
    }

    /// One world list: each pack that exists and has a module of `side`'s kind, with its
    /// subpack and its dependencies. Returns the `(requester, resource)` pairs of resource
    /// packs that behaviour packs depend on.
    fn level(
        &mut self,
        stack: &mut Vec<Planned>,
        side: PackSide,
        selections: &[PackSelection],
        file: &'static str,
    ) -> Vec<(usize, usize)> {
        let mut wanted_resources = Vec::new();
        for (index, selection) in selections.iter().enumerate() {
            let id = PackIdVersion::new(
                selection.pack_id,
                selection.version.clone(),
                side.pack_type(),
            );
            let Some(candidate) = self.find(&id) else {
                // The game logs a world-list entry no pack matches only to its developer log,
                // not the content log, so the crate reports it as an information.
                self.finding(
                    DiagnosticCode::PackNotFound { pack: id },
                    Severity::Info,
                    entry_location(file, index, None),
                );
                continue;
            };
            if !self.facts(candidate).has_module(side.pack_type()) {
                // A persona piece pack among the resource packs is not served as a resource
                // pack (`PersonaPiecePack`); any other pack of the wrong kind is a wrong entry.
                let piece = side == PackSide::Resource
                    && self.facts(candidate).has_module(PackType::PersonaPiece);
                let code = if piece {
                    DiagnosticCode::PersonaPiecePack
                } else {
                    DiagnosticCode::InvalidValue { text: None }
                };
                // The game checks no kind and logs nothing for it, so the crate reports it as an
                // information.
                self.finding(
                    code,
                    Severity::Info,
                    entry_location(file, index, Some("pack_id")),
                );
                continue;
            }
            let subpack = self.chosen_subpack(candidate, selection, file, index);
            if stack
                .iter()
                .any(|planned| planned.candidate == candidate && planned.subpack == subpack)
            {
                // A pack is on a stack once, by pack instance, and the first entry wins. The game
                // removes duplicates without a log line, so the crate reports it as an
                // information.
                self.finding(
                    DiagnosticCode::DuplicateIdentifier { text: None },
                    Severity::Info,
                    entry_location(file, index, None),
                );
                continue;
            }
            wanted_resources.extend(self.add(
                stack,
                side,
                candidate,
                subpack,
                Reason::Listed,
                Layer::Level,
            ));
        }
        wanted_resources
    }

    /// The selection's `subpack` (a `folder_name`, else a `name`), or the default one.
    fn chosen_subpack(
        &mut self,
        candidate: usize,
        selection: &PackSelection,
        file: &'static str,
        index: usize,
    ) -> Option<usize> {
        let facts = self.facts(candidate);
        let Some(name) = selection.subpack.as_deref() else {
            return default_subpack(facts, None);
        };
        let by_folder = facts
            .subpacks
            .iter()
            .position(|subpack| &*subpack.folder_name == name);
        let chosen = by_folder.or_else(|| {
            facts
                .subpacks
                .iter()
                .position(|subpack| &*subpack.name == name)
        });
        if chosen.is_some() {
            return chosen;
        }
        let fallback = default_subpack(facts, None);
        // The game falls back to the default subpack without a line, so the crate reports it as
        // an information.
        self.finding(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
            entry_location(file, index, Some("subpack")),
        );
        fallback
    }

    /// Pushes a pack, then each of its dependencies not on the stack yet (depth first, each
    /// after its dependent). A dependency that is not registered hides the pack. Returns the
    /// `(requester, resource)` pairs of resource packs that behaviour packs among them depend on.
    fn add(
        &mut self,
        stack: &mut Vec<Planned>,
        side: PackSide,
        candidate: usize,
        subpack: Option<usize>,
        reason: Reason,
        layer: Layer,
    ) -> Vec<(usize, usize)> {
        let mut wanted_resources = Vec::new();
        let position = stack.len();
        stack.push(Planned {
            candidate,
            subpack,
            is_dependent: reason == Reason::Dependency,
            hidden: false,
        });
        let dependencies = self.facts(candidate).dependencies.clone();
        for dependency in dependencies {
            let Some(found) = self.find(&dependency) else {
                if layer == Layer::Education {
                    stack[position].hidden = true;
                    let pack = self.facts(candidate).id.clone();
                    self.finding(
                        DiagnosticCode::MissingDependency { pack, dependency },
                        Severity::Info,
                        Location::default(),
                    );
                    continue;
                }
                if layer == Layer::BaseGame {
                    // An app-bundled pack is never hidden for a missing dependency: the host may
                    // ship only one side of the vanilla packs.
                    self.finding(
                        DiagnosticCode::PackNotFound { pack: dependency },
                        Severity::Info,
                        Location::default(),
                    );
                    continue;
                }
                stack[position].hidden = true;
                let pack = self.facts(candidate).id.clone();
                // The game marks the pack in error without a log line at load. A third-party
                // pack's missing dependency still reaches the user in the pack screens'
                // activation dialog, so it stays an error; an app-bundled pack's never does, so it
                // is an information.
                let severity = if self.candidates[candidate].origin == PackOrigin::Package {
                    Severity::Info
                } else {
                    Severity::Error
                };
                self.finding(
                    DiagnosticCode::MissingDependency { pack, dependency },
                    severity,
                    Location::default(),
                );
                continue;
            };
            let facts = self.facts(found);
            if facts.has_module(side.pack_type()) {
                if !stack.iter().any(|planned| planned.candidate == found) {
                    let subpack = default_subpack(facts, None);
                    wanted_resources.extend(self.add(
                        stack,
                        side,
                        found,
                        subpack,
                        Reason::Dependency,
                        layer,
                    ));
                }
            } else if side == PackSide::Behavior
                && matches!(layer, Layer::Level | Layer::Education)
                && facts.has_module(PackType::Resource)
            {
                wanted_resources.push((candidate, found));
            }
            // Any other dependency of another type is satisfied by being registered: vanilla
            // splits it off the stack.
        }
        wanted_resources
    }

    /// The chemistry behaviour packs of an education world, appended to the world's stack
    /// when the server starts in education mode: the app-bundled packs of the chemistry
    /// category, walked from the category's end (the newest slice first), each whose header
    /// version is not above the world's base-game version (all for any or no version). The
    /// game assigns the category by the pack's in-package path (`behavior_packs/chemistry` and
    /// `chemistry_<version>`); here it is an app-bundled pack declaring the `chemistry`
    /// capability, as every one of them does.
    fn education(
        &mut self,
        stack: &mut Vec<Planned>,
        options: &LoadOptions,
    ) -> Vec<(usize, usize)> {
        let mut wanted_resources = Vec::new();
        let world = slice_selector(&options.base_game_version);
        let members: Vec<usize> = (0..self.candidates.len())
            .rev()
            .filter(|&index| {
                let candidate = &self.candidates[index];
                candidate.origin == PackOrigin::Package
                    && candidate.facts.id.pack_type == PackType::Behavior
                    && candidate
                        .facts
                        .capabilities
                        .iter()
                        .any(|name| &**name == "chemistry")
                    && world.includes(&candidate.facts.id.version)
            })
            .collect();
        for candidate in members {
            if stack.iter().any(|planned| planned.candidate == candidate) {
                continue;
            }
            let subpack = default_subpack(self.facts(candidate), None);
            wanted_resources.extend(self.add(
                stack,
                PackSide::Behavior,
                candidate,
                subpack,
                Reason::Listed,
                Layer::Education,
            ));
        }
        wanted_resources
    }

    /// The base-game behaviour layer below the world's packs.
    fn base_game_behavior(&mut self, stack: &mut Vec<Planned>, options: &LoadOptions) {
        let at_in_package = |uuid| PackIdVersion::new(uuid, IN_PACKAGE_VERSION, PackType::Behavior);
        let mut layer: Vec<PackIdVersion> = VANILLA_EXPERIMENTAL_PACKS
            .iter()
            .filter(|(_, pack_type, experiment)| {
                *pack_type == PackType::Behavior && options.experiments.contains(*experiment)
            })
            .map(|(uuid, _, _)| at_in_package(*uuid))
            .collect();
        layer.push(at_in_package(SERVER_LIBRARY));
        layer.push(at_in_package(SERVER_UI_LIBRARY));
        layer.extend(self.slices(options));
        layer.push(at_in_package(VANILLA_BEHAVIOR));
        for id in layer {
            let Some(candidate) = self.find(&id) else {
                // Vanilla skips a missing app-bundled pack with a log line.
                self.finding(
                    DiagnosticCode::PackNotFound { pack: id },
                    Severity::Info,
                    Location::default(),
                );
                continue;
            };
            if stack.iter().any(|planned| planned.candidate == candidate) {
                // The base-game layer dedups in silence too, as the game does.
                self.finding(
                    DiagnosticCode::DuplicateIdentifier { text: None },
                    Severity::Info,
                    Location::default(),
                );
                continue;
            }
            let subpack = default_subpack(self.facts(candidate), None);
            // A base-game pack asks for no resource pack.
            let _ = self.add(
                stack,
                PackSide::Behavior,
                candidate,
                subpack,
                Reason::Listed,
                Layer::BaseGame,
            );
        }
    }

    /// The `vanilla` behaviour slices the world's base-game version includes, newest first:
    /// app-bundled packs with `vanilla`'s UUID at a version other than 0.0.1.
    fn slices(&self, options: &LoadOptions) -> Vec<PackIdVersion> {
        let world = slice_selector(&options.base_game_version);
        let mut slices: Vec<PackIdVersion> = self
            .candidates
            .iter()
            .filter(|candidate| {
                let id = &candidate.facts.id;
                candidate.origin == PackOrigin::Package
                    && id.uuid == VANILLA_BEHAVIOR
                    && id.version != IN_PACKAGE_VERSION
                    && world.includes(&id.version)
            })
            .map(|candidate| candidate.facts.id.clone())
            .collect();
        slices.sort_by(|a, b| b.version.cmp(&a.version));
        slices.dedup();
        slices
    }

    /// Appends the resource packs behaviour packs depend on that the resource stack lacks; a
    /// behaviour pack a missing dependency hid asks for nothing. With [`Announce::Additions`]
    /// each addition is a `DependencyAutoAdded` information (the world's packs), else silent
    /// (the education layer).
    fn auto_add_resources(
        &mut self,
        stack: &mut Vec<Planned>,
        behavior: &[Planned],
        wanted: &[(usize, usize)],
        announce: Announce,
    ) {
        for &(requester, resource) in wanted {
            let hidden = behavior
                .iter()
                .any(|planned| planned.candidate == requester && planned.hidden);
            if hidden || stack.iter().any(|planned| planned.candidate == resource) {
                continue;
            }
            let subpack = default_subpack(self.facts(resource), None);
            // A resource pack asks for no behaviour pack.
            let _ = self.add(
                stack,
                PackSide::Resource,
                resource,
                subpack,
                Reason::Dependency,
                Layer::Level,
            );
            if announce == Announce::Silent {
                continue;
            }
            let pack = self.facts(requester).id.clone();
            let dependency = self.facts(resource).id.clone();
            // The game adds a dependency to the stack without a log line, so the crate reports it
            // as an information.
            self.finding(
                DiagnosticCode::DependencyAutoAdded { pack, dependency },
                Severity::Info,
                Location::default(),
            );
        }
    }
}

/// `<file>` at `<index>[/<member>]`.
fn entry_location(file: &'static str, index: usize, member: Option<&str>) -> Location {
    let mut json_path = JsonPath::new();
    json_path.push_index(index);
    if let Some(member) = member {
        json_path.push(member);
    }
    Location::new(None, Some(PackPath::from_static(file)), json_path, None)
}

/// The world's base-game version as slice selection reads it: `*` and every value that is no
/// version (unset, unparsable, the never-compatible constant: `version` is `None`) select
/// every slice; a version selects the slices not above it, whatever its never-compatible flag.
// This is mapped here so the rule does not rest on core's `includes` for a missing version: the
// never-compatible constant, which holds no version, selects every slice too, and the flag
// beside a version is not read.
fn slice_selector(world: &BaseGameVersion) -> BaseGameVersion {
    match &world.version {
        None => BaseGameVersion::ANY,
        Some(version) => BaseGameVersion::new(version.clone()),
    }
}

#[cfg(test)]
mod tests;
