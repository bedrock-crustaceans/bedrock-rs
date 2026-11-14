//! `cargo xtask protocol`: generates the version crates under `crates/protocol/version/`, the
//! traits they implement and the facade over them from `crates/protocol/def/versions.def.rs`.
//!
//! [`render`] is the whole generator as a pure function: the text of the definition and a view
//! of what is hand-written in the tree ([`Tree`]) go in, the files to write ([`Output`]) come
//! out. Reading the tree and writing the files is the caller's job (`main.rs`), so a test can
//! render from a few lines of definition, and the test that the checked-in manifests are current
//! is a comparison of two strings.
//!
//! The definition is resolved into a [`Definition`] first: the versions in order, each with the
//! items it has after its diff is applied to its predecessor's, every item as an [`Item`] that
//! knows its owner crate. `rust.rs` renders the Rust files from that, `manifest.rs` holds the
//! rules of the manifests.

use crate::codegen::protocol::diff::{DefineVersionsDiffEntry, DefineVersionsDiffList, Item};
use crate::codegen::protocol::entry::DefineVersionsEntry;
use crate::codegen::protocol::input::DefineVersionsInput;
use proc_macro2::{Ident, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use syn::LitStr;

pub mod diff;
pub mod entry;
pub mod input;
mod manifest;
mod rust;
#[cfg(test)]
mod tests;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(versions);
    custom_keyword!(packets);
    custom_keyword!(types);
    custom_keyword!(enums);
    custom_keyword!(raknet_version);
}

/// The directory every path of this module is relative to, from the workspace root.
pub const ROOT: &str = "crates/protocol";
/// The definition file.
pub const DEFINITION: &str = "def/versions.def.rs";
/// The directory of the version crates, one per module ident, beside the traits crate.
pub const VERSIONS: &str = "version";
/// The facade's manifest: hand-written but for two blocks between marker lines.
pub const FACADE_MANIFEST: &str = "protocol/Cargo.toml";
/// The facade's re-exports.
const FACADE_GENERATED: &str = "protocol/src/generated.rs";
/// The four `ProtoVersion*` traits.
const TRAITS_GENERATED: &str = "version/traits/src/generated.rs";
/// The directory of the traits crate, the one directory beside the version crates.
const TRAITS_DIRECTORY: &str = "traits";
/// The module idents no version can have: the version's directory would be the traits crate's,
/// or its crate would have the name of the traits crate (`bedrock_protocol_version`) or of
/// `bedrock_protocol_core`.
const RESERVED_MODULES: [&str; 3] = [TRAITS_DIRECTORY, "version", "core"];

/// The `.rs` files under the `src/` of one version crate: the path below `src/`, with `/`
/// between its parts, to the text of the file.
pub type Sources = BTreeMap<String, String>;

/// What the generator reads of the tree besides the definition.
#[derive(Default)]
pub struct Tree {
    /// The sources of each version crate, by the name of its directory (its module ident). A
    /// version with no directory yet has no entry; every directory there is has one, the traits
    /// crate's and one without sources too.
    pub sources: BTreeMap<String, Sources>,
    /// The facade's manifest as it is.
    pub facade_manifest: String,
}

/// One file the generator wants written.
pub struct Output {
    /// The path below [`ROOT`], with `/` between its parts.
    pub path: String,
    pub contents: String,
    /// Whether the file is a starting point and not output: it is rendered only when the tree
    /// lacks it, and it is hand-written from then on.
    pub scaffold: bool,
}

impl Output {
    fn generated(path: impl Into<String>, contents: String) -> Self {
        Self {
            path: path.into(),
            contents,
            scaffold: false,
        }
    }
}

/// What the generator makes of a definition and a tree.
pub struct Rendering {
    /// The files to write.
    pub outputs: Vec<Output>,
    /// What is wrong with the tree that the files do not put right, to be reported once they
    /// are written: a directory that is no version ([`Error::UnknownVersionDirectory`]). It
    /// does not keep the files from being rendered, because the directory of a removed version
    /// can only be deleted once no generated manifest names it: cargo loads no workspace in
    /// which a manifest names a path that is gone, and so could not run this generator.
    pub left_behind: Option<Error>,
}

/// Why nothing was rendered, or what the tree still needs.
#[derive(Debug)]
pub enum Error {
    /// The definition does not parse or contradicts itself.
    Definition(syn::Error),
    /// The facade's manifest lacks a marker line the generated lines go between.
    MissingMarker(&'static str),
    /// A hand-written source of the version crate `module` names the crate `name` of a later
    /// version: the later one depends on the earlier, so this would close a cycle. `file` is
    /// the path of the source below the crate's `src/`.
    SourceNamesLaterVersion {
        module: String,
        file: String,
        name: String,
    },
    /// A hand-written source of the version crate `module` names that crate itself, which
    /// inside it is `crate`.
    SourceNamesOwnCrate { module: String, file: String },
    /// A directory beside the version crates that is no version of the definition: a version
    /// that was removed or renamed leaves its crate behind as a member of the workspace.
    UnknownVersionDirectory { directory: String },
}

impl From<syn::Error> for Error {
    fn from(error: syn::Error) -> Self {
        Self::Definition(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // One line per message of the error, each with the place in the file it points at.
            Self::Definition(error) => {
                for (index, error) in error.into_iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    let start = error.span().start();
                    let (line, column) = (start.line, start.column + 1);
                    write!(f, "{ROOT}/{DEFINITION}:{line}:{column}: {error}")?;
                }
                Ok(())
            }
            Self::MissingMarker(marker) => {
                write!(f, "{ROOT}/{FACADE_MANIFEST}: no marker line `{marker}`")
            }
            Self::SourceNamesLaterVersion { module, file, name } => {
                let own = diff::crate_name(module);
                write!(
                    f,
                    "{ROOT}/{VERSIONS}/{module}/src/{file}: names `{name}`, the crate of a later \
                     version than `{own}`: a version crate can only use the crates of older \
                     versions"
                )
            }
            Self::UnknownVersionDirectory { directory } => {
                write!(
                    f,
                    "{ROOT}/{VERSIONS}/{directory}: `{directory}` is no version of \
                     {ROOT}/{DEFINITION}: delete the directory (no generated file names it any \
                     more), or restore its entry in the definition"
                )
            }
            Self::SourceNamesOwnCrate { module, file } => {
                let own = diff::crate_name(module);
                write!(
                    f,
                    "{ROOT}/{VERSIONS}/{module}/src/{file}: names its own crate `{own}`: inside \
                     the crate, write `crate`"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// One protocol version: what its crate is called and the items it has, those it defines and
/// those it carries forward from the versions before it.
pub struct Version {
    /// The name of the crate's directory and of the facade's module and feature (`v662`).
    pub module: Ident,
    /// The name of the packet enum (`V662`).
    pub enum_ident: Ident,
    pub number: u32,
    pub branch: LitStr,
    pub game_version: LitStr,
    pub raknet_version: u8,
    pub packets: BTreeMap<Ident, Item>,
    pub types: BTreeMap<Ident, Item>,
    pub enums: BTreeMap<Ident, Item>,
    /// The top-level modules of the crate that the definition puts items in, whichever version
    /// lists them.
    pub modules: BTreeSet<Ident>,
}

impl Version {
    /// The package name of the version's crate.
    pub fn crate_name(&self) -> String {
        diff::crate_name(&self.module)
    }

    /// Every item of the version: packets, types and enums.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        let maps = [&self.packets, &self.types, &self.enums];
        maps.into_iter().flat_map(BTreeMap::values)
    }
}

/// The definition file, resolved.
pub struct Definition {
    /// The versions, oldest first: by protocol number, and in the order of the file where two
    /// share a number.
    pub versions: Vec<Version>,
    /// The name of every packet some version adds.
    pub packets: BTreeSet<Ident>,
    /// The name of every type some version adds.
    pub types: BTreeSet<Ident>,
    /// The name of every enum some version adds.
    pub enums: BTreeSet<Ident>,
}

impl DefineVersionsEntry {
    /// The name of the packet enum: what `as` says, or `V<number>`.
    fn enum_ident(&self) -> Ident {
        let default = || Ident::new(&format!("V{}", self.version), Span::call_site());
        self.ident.clone().unwrap_or_else(default)
    }

    /// The sections the entry has.
    fn lists(&self) -> impl Iterator<Item = &DefineVersionsDiffList> {
        let lists = [&self.packets, &self.types, &self.enums];
        lists.into_iter().flatten()
    }
}

/// Collects every identifier that some version adds with `+`, rejecting an identifier that is
/// added twice. `what` names the kind of item for the error message.
fn collect_added(
    versions: &[DefineVersionsEntry],
    what: &str,
    select: impl Fn(&DefineVersionsEntry) -> Option<&DefineVersionsDiffList>,
) -> syn::Result<BTreeSet<Ident>> {
    versions
        .iter()
        .try_fold(BTreeSet::<Ident>::new(), |mut acc, v| {
            if let Some(list) = select(v) {
                for entry in &list.entries {
                    if let DefineVersionsDiffEntry::Added { ident, .. } = entry {
                        if let Some(prev) = acc.get(ident) {
                            let mut err = syn::Error::new(
                                ident.span(),
                                format!("{what} added more than once"),
                            );

                            err.combine(syn::Error::new(ident.span(), "did you mean to use `%`?"));
                            err.combine(syn::Error::new(prev.span(), "previously added here"));

                            return Err(err);
                        }
                        acc.insert(ident.clone());
                    }
                }
            }
            Ok(acc)
        })
}

/// The module ident of every entry, in the order of `entries`: the name of its packet enum in
/// lower case. It is what identifies a version; two versions may share a protocol number.
fn module_idents(entries: &[DefineVersionsEntry]) -> syn::Result<Vec<Ident>> {
    let mut seen = BTreeMap::<String, Span>::new();
    let mut modules = Vec::with_capacity(entries.len());
    for entry in entries {
        let module = entry.enum_ident().to_string().to_lowercase();
        if RESERVED_MODULES.contains(&module.as_str()) {
            // Only an `as` can spell one: the default is `V<number>`.
            let span = entry.ident.as_ref().map_or(entry.span, Ident::span);
            return Err(syn::Error::new(
                span,
                format!(
                    "a version cannot be `{module}`: `version/traits` is the traits crate, and \
                     `bedrock_protocol_version` and `bedrock_protocol_core` are crates already"
                ),
            ));
        }
        if let Some(first) = seen.insert(module.clone(), entry.span) {
            let mut err = syn::Error::new(
                entry.span,
                format!("a second version `{module}`: name one of the two with `as`"),
            );
            err.combine(syn::Error::new(first, "the first is here"));
            return Err(err);
        }
        modules.push(Ident::new(&module, Span::call_site()));
    }
    Ok(modules)
}

/// Checks the owner of every `in` path, and returns for each owner the top-level modules of its
/// crate that the paths name. `modules` is the module ident of every entry, in order.
fn owned_modules(
    entries: &[DefineVersionsEntry],
    modules: &[Ident],
) -> syn::Result<BTreeMap<Ident, BTreeSet<Ident>>> {
    let mut crate_modules = BTreeMap::<Ident, BTreeSet<Ident>>::new();
    for (index, entry) in entries.iter().enumerate() {
        for list in entry.lists() {
            let owner = list.owner();
            let Some(position) = modules.iter().position(|module| module == owner) else {
                return Err(syn::Error::new(
                    owner.span(),
                    format!(
                        "`{owner}` is not a version: an `in` path starts with the module of the \
                         version crate that owns the items, as in `v662::packets`"
                    ),
                ));
            };
            // A version crate depends on the crates of the items it uses, so an item of a later
            // version would close a cycle.
            if position > index {
                let user = &modules[index];
                return Err(syn::Error::new(
                    owner.span(),
                    format!("`{user}` cannot use an item of `{owner}`, a later version"),
                ));
            }
            let named = crate_modules.entry(owner.clone()).or_default();
            named.extend(list.module().into_iter().next());
        }
    }
    Ok(crate_modules)
}

impl Definition {
    /// Resolves the text of the definition file.
    ///
    /// # Errors
    ///
    /// Returns an error if the text does not parse or the definitions are inconsistent: an item
    /// added twice, removed or modified before it was added, a version without a
    /// `raknet_version`, two versions of one module ident, a version with a reserved module
    /// ident, or an `in` path whose owner is not a version or is a later version than the one
    /// that uses it.
    pub fn parse(text: &str) -> syn::Result<Self> {
        let DefineVersionsInput { versions } = syn::parse_str(text)?;
        let mut entries = versions.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|v| v.version);

        let all_packets = collect_added(&entries, "packet", |v| v.packets.as_ref())?;
        let all_types = collect_added(&entries, "type", |v| v.types.as_ref())?;
        let all_enums = collect_added(&entries, "enum", |v| v.enums.as_ref())?;

        let modules = module_idents(&entries)?;
        let mut crate_modules = owned_modules(&entries, &modules)?;

        let mut raknet_version: Option<u8> = None;
        let mut packets = BTreeMap::<Ident, Item>::new();
        let mut types = BTreeMap::<Ident, Item>::new();
        let mut enums = BTreeMap::<Ident, Item>::new();

        let mut versions = Vec::with_capacity(entries.len());
        for (entry, module) in entries.iter().zip(modules) {
            if let Some(list) = &entry.packets {
                list.collapse(&mut packets)?;
            }
            if let Some(list) = &entry.types {
                list.collapse(&mut types)?;
            }
            if let Some(list) = &entry.enums {
                list.collapse(&mut enums)?;
            }

            raknet_version = entry.raknet_version.or(raknet_version);
            let Some(raknet_version) = raknet_version else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "raknet_version not defined",
                ));
            };

            versions.push(Version {
                modules: crate_modules.remove(&module).unwrap_or_default(),
                module,
                enum_ident: entry.enum_ident(),
                number: entry.version,
                branch: entry.branch.clone(),
                game_version: entry.game_version.clone(),
                raknet_version,
                packets: packets.clone(),
                types: types.clone(),
                enums: enums.clone(),
            });
        }

        Ok(Self {
            versions,
            packets: all_packets,
            types: all_types,
            enums: all_enums,
        })
    }
}

/// Renders every file the generator owns: the traits, the manifest and the packet enum of every
/// version crate, and the facade's re-exports and manifest. A version crate with no
/// `src/lib.rs` gets one to start from.
///
/// # Errors
///
/// Returns an error if the definition is inconsistent (see [`Definition::parse`]), a source of a
/// version crate names that crate or the crate of a later version, or the facade's manifest has
/// lost a marker line. A directory that is no version of the definition is not such an error:
/// the files are rendered, and it is [`Rendering::left_behind`].
pub fn render(definition: &str, tree: &Tree) -> Result<Rendering, Error> {
    let definition = Definition::parse(definition)?;
    let no_sources = Sources::new();

    // A version that was removed or renamed leaves its crate behind: still a member of the
    // workspace, with a manifest that says it is generated, and no output would ever touch it.
    let is_version = |directory: &String| definition.versions.iter().any(|v| v.module == directory);
    let stray = |directory: &&String| *directory != TRAITS_DIRECTORY && !is_version(directory);
    let stray = tree.sources.keys().find(stray).cloned();
    let left_behind = stray.map(|directory| Error::UnknownVersionDirectory { directory });

    let mut outputs = vec![Output::generated(
        TRAITS_GENERATED,
        rust::traits(&definition)?,
    )];
    // The version crates come before the facade that names them.
    for version in &definition.versions {
        let dir = format!("{VERSIONS}/{}", version.module);
        let sources = tree
            .sources
            .get(&version.module.to_string())
            .unwrap_or(&no_sources);
        outputs.push(Output::generated(
            format!("{dir}/Cargo.toml"),
            manifest::version(version, &definition.versions, sources)?,
        ));
        if !sources.contains_key("lib.rs") {
            outputs.push(Output {
                path: format!("{dir}/src/lib.rs"),
                contents: rust::lib_scaffold(version),
                scaffold: true,
            });
        }
        outputs.push(Output::generated(
            format!("{dir}/src/generated.rs"),
            rust::version(&definition, version)?,
        ));
    }
    outputs.push(Output::generated(
        FACADE_GENERATED,
        rust::facade(&definition)?,
    ));
    outputs.push(Output::generated(
        FACADE_MANIFEST,
        manifest::facade(&definition.versions, &tree.facade_manifest)?,
    ));
    Ok(Rendering {
        outputs,
        left_behind,
    })
}
