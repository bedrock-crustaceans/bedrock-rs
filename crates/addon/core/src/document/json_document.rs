//! [`JsonDocument`] and [`JsonDefinition`]: the three or four items a versioned JSON file kind
//! states, from which core supplies the mechanical quartet (`Sealed`, [`Content`],
//! [`JsonContent`], [`Placed`], and [`Definition`] for a definition) by blanket impls.

use super::{
    CombineRule, Content, Definition, Document, JsonContent, Placed, Rejected, Unknown,
    UnknownMembers,
};
use crate::__private::Sealed;
use crate::diagnostic::Diagnostic;
use crate::id::{Id, Kind};
use crate::json::{EncodeError, Value, WriteStyle};
use crate::load::{LoadContext, StackCx};
use crate::pack_identity::{PackPath, PackRef, PackSide};
use crate::version::FormatVersion;
use crate::versioning::{
    DocumentRead, Family, VersionStruct, Versioned, parse_document, parse_document_as_written,
    write_envelope,
};

use super::Origin;

/// A versioned JSON file kind whose newest struct is `Self`: one document per file, read
/// through its family's chain and written back with the family's envelope.
///
/// Core implements `Sealed`, [`Content`] (parse = [`read`](Self::read), a beta document refused
/// with "no parser available" for `beta` and closed as failed, write =
/// [`write_envelope`]) and [`JsonContent`] for every `JsonDocument`, and [`Placed`] for every
/// [`JsonPlaced`], so a type that implements them implements none of those by hand: a family
/// moving onto them deletes its own impls in the same change (they would overlap).
/// A file kind that is not this (lang, a multi-definition file read its own way, a beta struct
/// whose family's `Latest` is another type) keeps its hand impls beside the blanket ones.
///
/// `NAME` and `SIDES` are also `Content`'s: where both traits are in scope, a concrete path names
/// one (`<VoxelShape as Content>::NAME`).
pub trait JsonDocument: VersionStruct + Send + Sync + 'static {
    /// The chain the file kind reads and writes; its newest struct is this type.
    type Family: Family<Latest = Self>;
    /// The sides it is loaded from ([`Content::SIDES`]); the first is the side of a file read
    /// on its own ([`Content::parse`]).
    const SIDES: &'static [PackSide];
    /// [`Content::NAME`]: the family's name unless the file kind is named otherwise in messages
    /// (`"fog settings"` of the `"fog"` family).
    const NAME: &'static str = <Self::Family as Family>::NAME;

    /// [`Content::UNKNOWN_PRECEDENCE`] of the kind: which value
    /// [`write_root_with`](Self::write_root_with) keeps where an unknown member has the key of
    /// a written one.
    const UNKNOWN_PRECEDENCE: crate::json::Precedence = crate::json::Precedence::UnknownWins;

    /// The `format_version` the value is written at. Default: the family's newest struct's,
    /// this type's own. A family that writes the version the file was read at overrides it
    /// (`source_format_version.max(FLOOR)`).
    ///
    /// Ignored on the raw-text path (`Family::ENVELOPE.check` is `EnvelopeCheck::Never`, or
    /// `From(m)` with this version below `m`), where the struct writes its own envelope: its
    /// `VersionStruct::write` is the whole document, version member included (dimension,
    /// attachable, `client_entity`).
    fn write_version(&self) -> FormatVersion {
        <Self as Versioned>::VERSION
    }

    /// Adds the root members written after the payload (a document's `root_extra`); none by
    /// default. Ignored on the raw-text path (`EnvelopeCheck::Never`, see
    /// [`write_version`](Self::write_version)), where the struct writes its own root, root
    /// members included.
    #[allow(unused_variables)]
    fn write_root_members(&self, root: &mut UnknownMembers) {}

    /// Refuses to write a value the game would not read back (fog's rules, block culling's
    /// cube-index check), before anything is written. Default: every value is written.
    ///
    /// # Errors
    ///
    /// `EncodeError` when the value must not be written.
    fn refuse_write(&self) -> Result<(), EncodeError> {
        Ok(())
    }

    /// The whole document the value writes: by default [`write_envelope`] over
    /// [`write_version`](Self::write_version), the struct's payload and
    /// [`write_root_members`](Self::write_root_members), after [`refuse_write`](Self::refuse_write).
    ///
    /// Overridden for a value written in a format other than the newest envelope: one the
    /// newest format cannot hold, written as an older document whose payload is not
    /// [`VersionStruct::write`] (block, item, trade's legacy blobs). The three hooks above are
    /// then the override's to call or not.
    ///
    /// # Errors
    ///
    /// `EncodeError` when the value has no text.
    fn write_root(&self) -> Result<Value, EncodeError> {
        document_of(self)
    }

    /// [`write_root`](Self::write_root) with the document's unknown members in it, each
    /// after the members of its object ([`json::merge_unknown`](crate::json::merge_unknown)),
    /// and the findings of the members left out. What `Document::write_json` writes.
    ///
    /// Overridden by a writer whose choice of format looks at the members the document has
    /// (block writes an older document for an unknown member the newest one refuses): it
    /// puts the members into the document it chooses.
    ///
    /// # Errors
    ///
    /// As [`write_root`](Self::write_root).
    #[doc(hidden)]
    fn write_root_with(&self, unknown: &Unknown) -> Result<(Value, Vec<Diagnostic>), EncodeError> {
        let mut root = self.write_root()?;
        let told = crate::json::merge_unknown_with(
            &mut root,
            unknown,
            <Self as JsonDocument>::UNKNOWN_PRECEDENCE,
        );
        Ok((root, told))
    }

    /// Reads the file at `origin`: the stable document, a beta one or the rejection. Default:
    /// [`parse_document`] of the family. [`Content::parse`] (a beta document refused) and
    /// `load::registry` read through it, so a family with a check of its own around the
    /// generic read (a gate before the parse, an admit step, an identifier or vanilla check
    /// after it) overrides it and wraps the default, the one place both paths see.
    ///
    /// The default's stable document and rejection are closed (`Family::close`) and their
    /// findings located; an override that adds a finding after it adds it after the close,
    /// put in the file itself (`Rejected::located`, `OpenBeta::push`). It does not reach
    /// [`parse_as_written`](Self::parse_as_written), which gives the struct before the upgrade
    /// and the checks: a family whose override gates the text overrides that too.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        parse_document::<Self::Family>(origin, bytes, cx)
    }

    /// The struct the file's `format_version` selects, not upgraded, for the file `path` read on
    /// its own: [`parse_document_as_written`] on the first of [`SIDES`](Self::SIDES). Kept by
    /// name for the families' inherent `parse_as_written`, which this replaces.
    ///
    /// # Errors
    ///
    /// `Rejected` when the text is not readable JSON or the file does not parse into a value.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<<Self::Family as Family>::History>, Rejected> {
        parse_document_as_written::<Self::Family>(first_side(Self::SIDES), path, bytes, cx)
    }
}

/// A [`JsonDocument`] with a path of its own in a pack, where `PackWriter::add` puts it: core
/// implements [`Placed`] for it (`default_path` = [`place`](Self::place)). A file kind the
/// writer is not given by value (trade's tables) implements none.
pub trait JsonPlaced: JsonDocument {
    /// Where `PackWriter::add` puts this value ([`Placed::default_path`]): one of
    /// `document::{default_path, default_path_flat, default_path_flat_for_vanilla}` of the
    /// identifier, or a fixed path.
    fn place(&self) -> PackPath;
}

/// A [`JsonDocument`] that is one definition of a registry, its file holding only it
/// (`Definition::File = Self`): core implements [`Definition`] for it, its beta the family's
/// `Beta` (`NoBeta<Kind>` for a family without one, which the bound on `Family` here requires
/// to be of this kind).
///
/// A file of several definitions (animation, render controller, geometry, dialogue, server
/// sound definitions, `blocks.json`) is a `JsonDocument` at most for the file; its entry type
/// keeps a hand `Definition` with `File = <the file>`, and its loader takes the file's
/// entries, `fn entries(&File) -> Vec<(Id<Kind>, Def)>`, which core does not generalise yet:
/// how a file's findings and rejection are shared among its entries differs by family.
pub trait JsonDefinition:
    JsonDocument<Family: Family<Beta: Definition<Kind = <Self as JsonDefinition>::Kind>>>
{
    /// The identifier kind the registry is keyed by.
    type Kind: Kind;

    /// The definition's identifier (`description.identifier`, `identifier` or `name`, as the
    /// family has it).
    fn identifier(&self) -> &Id<<Self as JsonDefinition>::Kind>;

    /// How the stable definitions of one identifier from several packs combine
    /// (`load::registry`'s combine). Default `CombineRule::top_wins()`.
    #[allow(unused_variables)]
    fn combine_rule(stack: &StackCx<'_>) -> CombineRule<Self> {
        CombineRule::top_wins()
    }

    /// How the beta definitions combine. Default `CombineRule::top_wins()`.
    #[allow(unused_variables)]
    fn beta_rule(stack: &StackCx<'_>) -> CombineRule<<Self::Family as Family>::Beta> {
        CombineRule::top_wins()
    }
}

/// The side of a file read on its own: the first the kind is loaded from.
fn first_side(sides: &[PackSide]) -> PackSide {
    sides.first().copied().unwrap_or(PackSide::Behavior)
}

/// The whole document of `value`, after its refusal check.
fn document_of<T: JsonDocument>(value: &T) -> Result<Value, EncodeError> {
    value.refuse_write()?;
    let mut extras = UnknownMembers::default();
    value.write_root_members(&mut extras);
    Ok(write_envelope::<T::Family>(
        value.write_version(),
        VersionStruct::write(value)?,
        &extras,
    ))
}

impl<T: JsonDocument> Sealed for T {}

impl<T: JsonDocument> Content for T {
    const NAME: &'static str = <T as JsonDocument>::NAME;
    const SIDES: &'static [PackSide] = <T as JsonDocument>::SIDES;
    const UNKNOWN_PRECEDENCE: crate::json::Precedence = <T as JsonDocument>::UNKNOWN_PRECEDENCE;

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(
            PackRef::app_data(),
            first_side(<T as JsonDocument>::SIDES),
            path.clone(),
        );
        T::read(origin, bytes, cx).stable()
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        crate::json::write_json(&self.write_root()?, out, style)
    }

    fn write_json_with(
        &self,
        unknown: &Unknown,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<Vec<Diagnostic>, EncodeError> {
        if unknown.is_empty() {
            return self.write_json(out, style).map(|()| Vec::new());
        }
        let (root, told) = self.write_root_with(unknown)?;
        crate::json::write_json(&root, out, style)?;
        Ok(told)
    }
}

impl<T: JsonDocument> JsonContent for T {}

impl<T: JsonPlaced> Placed for T {
    fn default_path(&self) -> PackPath {
        self.place()
    }
}

impl<T: JsonDefinition> Definition for T {
    type Kind = <T as JsonDefinition>::Kind;
    type File = T;
    type Beta = <T::Family as Family>::Beta;
}
