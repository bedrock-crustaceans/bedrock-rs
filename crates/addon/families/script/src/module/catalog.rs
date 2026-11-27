//! `ModuleCatalog`: which versions of which native script modules an engine serves.

use indexmap::IndexMap;
use uuid::Uuid;

use super::ScriptVersion;
use crate::meta;

/// The native script modules an engine serves and their versions: what a pack's
/// dependencies are resolved against.
///
/// The catalog is per engine build. [`ModuleCatalog::shipped`] is the one compiled in from
/// the published script metadata ([`meta::samples_tag`]); a host with another engine builds
/// its own with [`ModuleCatalog::insert`]. An engine serves a whole major from its newest
/// release, so a catalog that lists every release of a major and one that lists only the
/// newest resolve every request alike.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleCatalog {
    modules: IndexMap<String, CatalogModule>,
    /// The name of each module by its UUID, in insertion order.
    names: IndexMap<Uuid, String>,
}

/// One native module of a [`ModuleCatalog`]. The default is a module without a UUID that is
/// its own bindings and serves no version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatalogModule {
    /// The module's UUID; `None` when it is not known.
    pub uuid: Option<Uuid>,
    /// The bindings module the public module stands for (`@minecraft/server-ui-bindings` for
    /// `@minecraft/server-ui`): the name vanilla prints for what this module requires. `None`
    /// for a module that is its own bindings.
    pub bindings: Option<String>,
    /// The versions served, one [`CatalogEntry`] each. [`ModuleCatalog::insert`] sorts them
    /// ascending.
    pub versions: Vec<CatalogEntry>,
}

/// One version of a [`CatalogModule`] as the catalog serves it: the version, whether it is
/// listed, and what it requires.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogEntry {
    /// The version.
    pub version: ScriptVersion,
    /// Served but left out of vanilla's `Available versions:` list (the `-internal`
    /// versions).
    pub hidden: bool,
    /// What this version itself requires: other modules, each at a version.
    pub requires: Vec<(String, ScriptVersion)>,
}

impl CatalogEntry {
    /// A listed version that requires nothing.
    pub fn new(version: ScriptVersion) -> Self {
        Self {
            version,
            hidden: false,
            requires: Vec::new(),
        }
    }
}

impl FromIterator<(String, CatalogModule)> for ModuleCatalog {
    /// Builds a catalog with [`ModuleCatalog::insert`]: a later module replaces an earlier one
    /// of its name.
    fn from_iter<I: IntoIterator<Item = (String, CatalogModule)>>(modules: I) -> Self {
        let mut catalog = Self::new();
        catalog.extend(modules);
        catalog
    }
}

impl Extend<(String, CatalogModule)> for ModuleCatalog {
    fn extend<I: IntoIterator<Item = (String, CatalogModule)>>(&mut self, modules: I) {
        for (name, module) in modules {
            self.insert(&name, module);
        }
    }
}

impl ModuleCatalog {
    /// An empty catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// The catalog compiled in from the published script metadata ([`meta::modules`]).
    ///
    /// On top of the versions the metadata has a file for, it holds what the engine serves
    /// without one: the versions of [`meta::engine_only`] (an `-internal` one hidden) and
    /// the 0.1.0 of `@minecraft/server` and of `@minecraft/server-ui`, which the by-UUID
    /// dependencies stand for.
    ///
    /// It also holds `@minecraft/server-editor`, which no table of the metadata has: its
    /// `0.1.0-beta` and a hidden `0.1.0-internal`, requiring nothing, without a UUID (the
    /// module's UUID is not known). Only an editor world lets a dependency reach it.
    ///
    /// What a version requires is read from the peers of its bindings module: a peer with one
    /// accepted version requires that version; one with several, all in one major, the
    /// newest of them; one whose accepted versions span majors binds to none of them.
    pub fn shipped() -> Self {
        let mut catalog = Self::new();
        for entry in meta::modules() {
            let parent = match entry.shape {
                meta::ModuleShape::Stub { parent } => Some(parent),
                _ => None,
            };
            let mut versions: Vec<CatalogEntry> = entry
                .versions
                .iter()
                .map(|version| {
                    // A stub stands for its bindings module, and vanilla prints that module
                    // as the one that requires.
                    let peers = parent
                        .and_then(meta::module)
                        .zip(version.parent_version)
                        .and_then(|(parent, index)| parent.versions.get(usize::from(index)))
                        .map_or(version.peers, |bound| bound.peers);
                    CatalogEntry {
                        version: module_version(version),
                        hidden: false,
                        requires: peers.iter().filter_map(requirement).collect(),
                    }
                })
                .collect();
            for (_, text) in meta::engine_only()
                .iter()
                .filter(|(module, _)| *module == entry.name)
            {
                if let Some(version) = ScriptVersion::parse(text) {
                    // `Available versions:` leaves the internal version out.
                    let hidden = version.is_internal();
                    versions.push(CatalogEntry {
                        version,
                        hidden,
                        requires: Vec::new(),
                    });
                }
            }
            if UUID_MODULES.contains(&entry.name) {
                // The dedicated server lists 0.1.0 of `@minecraft/server` (the version
                // imported as `mojang-minecraft`) and ships `server-ui-0.1.0.js` beside it.
                versions.push(CatalogEntry::new(ScriptVersion::new(0, 1, 0)));
            }
            catalog.insert(
                entry.name,
                CatalogModule {
                    uuid: Some(Uuid::from_u128(entry.uuid)),
                    bindings: parent.map(str::to_owned),
                    versions,
                },
            );
        }
        // The editor library ships `minecraft-server-editor.0.1.0-beta.js` and
        // `…0.1.0-internal.js`, which is where the module's name and versions come from.
        let mut internal = CatalogEntry::new(ScriptVersion::with_pre(0, 1, 0, "internal"));
        internal.hidden = true;
        catalog.insert(
            EDITOR,
            CatalogModule {
                uuid: None,
                bindings: None,
                versions: vec![
                    CatalogEntry::new(ScriptVersion::with_pre(0, 1, 0, "beta")),
                    internal,
                ],
            },
        );
        catalog
    }

    /// Adds the module called `name`, replacing one of that name; its versions are sorted
    /// ascending. A module without a UUID is found by its name only.
    pub fn insert(&mut self, name: &str, mut module: CatalogModule) {
        module.versions.sort_by(|a, b| {
            a.version
                .cmp(&b.version)
                .then_with(|| a.version.pre.cmp(&b.version.pre))
        });
        if let Some(uuid) = self.modules.get(name).and_then(|old| old.uuid) {
            self.names.shift_remove(&uuid);
        }
        if let Some(uuid) = module.uuid {
            self.names.insert(uuid, name.to_owned());
        }
        self.modules.insert(name.to_owned(), module);
    }

    /// The module called `name`, compared byte for byte (`@Minecraft/Server` is unknown).
    pub fn module(&self, name: &str) -> Option<&CatalogModule> {
        self.modules.get(name)
    }

    /// The name of the module whose UUID is `uuid`.
    pub fn name_of(&self, uuid: Uuid) -> Option<&str> {
        self.names.get(&uuid).map(String::as_str)
    }

    /// Every module with its name, in insertion order.
    pub fn modules(&self) -> impl Iterator<Item = (&str, &CatalogModule)> + '_ {
        self.modules
            .iter()
            .map(|(name, module)| (name.as_str(), module))
    }
}

impl CatalogModule {
    /// The name vanilla prints for a requirement of this module, called `name`: its bindings
    /// module, or itself.
    pub(super) fn requester<'a>(&'a self, name: &'a str) -> &'a str {
        self.bindings.as_deref().unwrap_or(name)
    }

    /// The version equal to `version`.
    pub(super) fn version(&self, version: &ScriptVersion) -> Option<&CatalogEntry> {
        self.versions.iter().find(|v| v.version == *version)
    }
}

/// A shipped version of the generated table as a [`ScriptVersion`].
fn module_version(version: &meta::ModuleVersion) -> ScriptVersion {
    ScriptVersion::with_pre(
        version.major,
        version.minor,
        version.patch,
        version.pre.unwrap_or_default(),
    )
}

/// The modules whose 0.1.0 the engine serves without a metadata file: the two a pack
/// dependency's UUID can stand for.
const UUID_MODULES: [&str; 2] = ["@minecraft/server", "@minecraft/server-ui"];

/// The editor module, served by the engine without a metadata file.
const EDITOR: &str = "@minecraft/server-editor";

/// What a peer of a shipped version makes it require: nothing for a type package or for a
/// module the metadata does not ship; else the one accepted version, or the newest of several
/// in one major.
fn requirement(peer: &meta::PeerReq) -> Option<(String, ScriptVersion)> {
    if peer.types_only || meta::module(peer.module).is_none() {
        return None;
    }
    let accepted: Vec<ScriptVersion> = peer
        .versions
        .iter()
        .filter_map(|text| ScriptVersion::parse(text))
        .collect();
    let newest = accepted.iter().max()?;
    // `server-gametest` accepts `1.17.0`, `2.0.0` and the beta, and runs beside a plugin's
    // `2.0.0` without lifting it to the beta.
    accepted
        .iter()
        .all(|version| version.major == newest.major)
        .then(|| (peer.module.to_owned(), newest.clone()))
}
