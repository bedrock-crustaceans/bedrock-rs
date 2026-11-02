//! A loaded value with its provenance (`Document<T>`), the three collections a stack resolves
//! content into (`Registry`, `Files`, `Merged`), components, and the traits every file kind
//! implements (`Content`, `Definition`, `Placed`, the sealed `Merge`).

use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

use crate::__private::Sealed;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use crate::id::Kind;
use crate::json::{EncodeError, WriteStyle};
use crate::load::LoadContext;
use crate::pack_identity::{PackPath, PackRef, PackSide};
use crate::version::FormatVersion;

mod by_id;
mod by_path;
mod combine;
mod component;
pub mod files;
mod finite;
mod merged;
mod placed;
pub mod registry;

#[doc(hidden)]
pub use by_id::RegistryBuilder;
pub use by_id::{Registry, Resolution};
pub use by_path::Files;
#[doc(hidden)]
pub use by_path::FilesBuilder;
#[doc(hidden)]
pub use combine::{CombineRule, Contribution, DecideFn, EngineVersionDecision, FilesRule, Notice};
pub use component::{Component, ComponentMap, ComponentRef, ComponentSet, CustomComponent};
#[doc(hidden)]
pub use finite::FiniteText;
#[doc(hidden)]
pub use merged::{Layer, MergeCx};
pub use merged::{Merged, RawLayer};
#[doc(hidden)]
pub use placed::default_path;
pub use placed::{default_path_flat, default_path_flat_for_vanilla};

// Defined in `json` beside the walk that fills it and the writer that emits it; this is its
// one public path.
pub use crate::json::extra::Extra;

/// A loaded value with its provenance and findings.
///
/// Dereferences to `T`, so `doc.description.identifier` reads through. The value is always the
/// newest schema; [`version`](Self::version) says what the file was written in.
///
/// Its findings hold no rejecting error; they may hold kept errors (a line the game
/// logs at error level before it keeps the document, [`Diagnostic::kept`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Document<T> {
    value: T,
    origin: Origin,
    version: VersionInfo,
    diagnostics: Diagnostics,
}

impl<T> Document<T> {
    /// A document from its parts (the fields are private, so `bedrock_addon` builds it
    /// here).
    #[doc(hidden)]
    pub fn new(value: T, origin: Origin, version: VersionInfo, diagnostics: Diagnostics) -> Self {
        Self {
            value,
            origin,
            version,
            diagnostics,
        }
    }

    /// The value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// The value, dropping the provenance.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Where the document came from.
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    /// What the file said about its version and what the loader did with it.
    pub fn version(&self) -> &VersionInfo {
        &self.version
    }

    /// Findings for this document only (warnings and infos; a document with an error is rejected).
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Applies `f` to the value and keeps the provenance (for derived views).
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Document<U> {
        Document {
            value: f(self.value),
            origin: self.origin,
            version: self.version,
            diagnostics: self.diagnostics,
        }
    }

    /// The parts, for combine rules that merge documents.
    pub(crate) fn into_parts(self) -> (T, Origin, VersionInfo, Diagnostics) {
        (self.value, self.origin, self.version, self.diagnostics)
    }

    /// The value, mutably, for combine rules that merge into the effective document.
    pub(crate) fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// The origin, mutably (combine rules record contributors).
    pub(crate) fn origin_mut(&mut self) -> &mut Origin {
        &mut self.origin
    }

    /// The findings, mutably (a merge carries the merged-in document's findings).
    pub(crate) fn diagnostics_mut(&mut self) -> &mut Diagnostics {
        &mut self.diagnostics
    }
}

// A deliberate exception to the API guidelines (a document is "a `T` with a label");
// `tests/document_deref.rs` checks that no root type names a method of `Document`.
impl<T> Deref for Document<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

/// Where a document came from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Origin {
    /// The pack (cheap handle: stack index, identity, name). Documents read from
    /// `LoadOptions::app_data` (spawn groups, app-data attachables, features) carry a synthetic
    /// `PackRef` for the app-data layer (`PackRef::is_app_data()`), ordered below every pack.
    pub pack: PackRef,
    /// The stack side.
    pub side: PackSide,
    /// The file, relative to the pack root (subpack folder included when it came from one).
    pub path: PackPath,
    /// The member key inside a multi-definition file (`"animation.zombie.walk"`, `"stone"` in
    /// blocks.json, a scene tag); `None` for one-definition files.
    pub key: Option<Box<str>>,
    /// Other packs that contributed to this value, for combine rules that merge (client
    /// entities by `min_engine_version`, block graphics field by field).
    pub contributors: Vec<PackRef>,
}

impl Origin {
    /// An origin from its parts (the struct is `#[non_exhaustive]`, so `bedrock_addon`
    /// builds it here).
    #[doc(hidden)]
    pub fn new(
        pack: PackRef,
        side: PackSide,
        path: PackPath,
        key: Option<Box<str>>,
        contributors: Vec<PackRef>,
    ) -> Self {
        Self {
            pack,
            side,
            path,
            key,
            contributors,
        }
    }

    /// Where a combine-time finding about this document is: its pack and file.
    pub(crate) fn location(&self) -> Location {
        Location::new(
            Some(self.pack.clone()),
            Some(self.path.clone()),
            JsonPath::new(),
            None,
        )
    }

    /// Records `pack` as a contributor, once.
    pub(crate) fn add_contributor(&mut self, pack: &PackRef) {
        if *pack != self.pack && !self.contributors.contains(pack) {
            self.contributors.push(pack.clone());
        }
    }
}

/// `<pack name>/<path>[#<key>]`.
impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.pack.name(), self.path)?;
        if let Some(key) = &self.key {
            write!(f, "#{key}")?;
        }
        Ok(())
    }
}

/// What the file said about its version and what the loader did with it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VersionInfo {
    /// `format_version` as written; `None` when absent, `"beta"` or the family is unversioned.
    pub written: Option<FormatVersion>,
    /// The file said `"beta"`: the document is a beta struct. A pre-release such as
    /// `1.21.60-beta.3` is an ordinary version, parsed by a stable struct.
    pub beta: bool,
    /// The version gates and the Molang version compare against: `written`, or the family's
    /// default when it is absent.
    pub effective: FormatVersion,
    /// The struct version that parsed the file (`history::v1_19_80`, …).
    pub parsed_as: FormatVersion,
    /// `true` when at least one upgrade step ran.
    pub upgraded: bool,
}

impl VersionInfo {
    /// Version information from its parts (the struct is `#[non_exhaustive]`, so
    /// `bedrock_addon` builds it here).
    #[doc(hidden)]
    pub fn new(
        written: Option<FormatVersion>,
        beta: bool,
        effective: FormatVersion,
        parsed_as: FormatVersion,
        upgraded: bool,
    ) -> Self {
        Self {
            written,
            beta,
            effective,
            parsed_as,
            upgraded,
        }
    }
}

/// A document that could not be loaded, with every finding about it.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{origin}: {}", .diagnostics.summary())]
#[non_exhaustive]
pub struct Rejected {
    /// Where the document came from.
    pub origin: Origin,
    /// Every finding, including the fatal one: an error that is not kept (a kept error alone
    /// rejects nothing).
    pub diagnostics: Diagnostics,
}

impl Rejected {
    /// A rejection from its parts (the struct is `#[non_exhaustive]`, so `bedrock_addon`
    /// builds it here).
    #[doc(hidden)]
    pub fn new(origin: Origin, diagnostics: Diagnostics) -> Self {
        Self {
            origin,
            diagnostics,
        }
    }
}

/// A file kind the crate can read and write.
pub trait Content: Sealed + Sized + Send + Sync + 'static {
    /// `"block"`, `"client entity"`: used in messages.
    const NAME: &'static str;
    /// Sides it is loaded from.
    const SIDES: &'static [PackSide];
    /// Parses one file: version selection, upgrade to the newest struct, validation.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected>;
    /// Writes the newest format version (`format_version` and envelope included).
    /// `EncodeError` (core) rather than `pack::WriteError`, which core cannot name.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError>;
    /// [`write_json`](Self::write_json) into a string, pretty-printed with two spaces. Fallible:
    /// a non-finite number is `EncodeError::NotRepresentable`.
    fn to_json(&self) -> Result<String, EncodeError>;
}

/// A definition stored by identifier in a [`Registry`]. For one-definition files the file and
/// the definition are the same type (`Block`); for multi-definition files they differ
/// (`AnimationFile` holds `Animation`s).
pub trait Definition: Sealed + Sized + Send + Sync + 'static {
    /// The identifier kind the registry is keyed by.
    type Kind: Kind;
    /// The file kind that holds it.
    type File: Content;
    /// The beta struct, or [`NoBeta`] (uninhabited).
    type Beta: Definition<Kind = Self::Kind>;
}

/// The `Beta` of a family without a beta struct: uninhabited.
pub struct NoBeta<K: Kind> {
    never: std::convert::Infallible,
    #[allow(dead_code)] // carries the kind
    kind: PhantomData<fn() -> K>,
}

impl<K: Kind> Sealed for NoBeta<K> {}

/// Never a value; prints nothing.
impl<K: Kind> fmt::Debug for NoBeta<K> {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.never {}
    }
}

impl<K: Kind> Content for NoBeta<K> {
    // No file is ever a `NoBeta`; the name only appears if one is asked to parse.
    const NAME: &'static str = "beta (none)";
    const SIDES: &'static [PackSide] = &[];

    /// Rejects every file: the family has no beta struct, so no `format_version` selects one.
    fn parse(path: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // The origin of a file no loader routes here; the app-data handle stands for
        // "no pack".
        let origin = Origin::new(
            PackRef::app_data(),
            PackSide::Behavior,
            path.clone(),
            None,
            Vec::new(),
        );
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::UnsupportedVersion { text: None },
            Severity::Error,
            Phase::Parse,
            origin.location(),
        ));
        Err(Rejected::new(origin, diagnostics))
    }

    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        match self.never {}
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        match self.never {}
    }
}

impl<K: Kind> Definition for NoBeta<K> {
    type Kind = K;
    type File = NoBeta<K>;
    type Beta = NoBeta<K>;
}

/// `PackWriter::add`'s default path for a root type (`blocks/<namespace>/<name>.json`).
pub trait Placed: Content {
    /// The path the writer puts this value at.
    fn default_path(&self) -> PackPath;
}

/// A merge strategy of a file merged across the stack; sealed, the strategies are crate-private.
pub trait Merge: Sealed {
    /// The key of one entry of the merged value, for [`Merged::origin`].
    type Key: Eq + Hash;
}
