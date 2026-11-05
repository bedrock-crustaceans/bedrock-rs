//! [`Single`]: the chain of a family with one format version.

use std::fmt;

use super::__macro::{parse_selected_as, parse_with};
use super::{
    Family, Loaded, ParseCx, StructIndex, UpgradeCx, VersionChain, VersionStruct, Versioned,
    write_envelope,
};
use crate::diagnostic::Diagnostics;
use crate::document::Extra;
use crate::json::{EncodeError, JsonSlice};
use crate::load::LoadContext;
use crate::molang::MolangFields;
use crate::version::FormatVersion;

/// The chain of a family with one format version and no beta struct: the newest struct is the
/// only one and nothing upgrades. A family names it as `type Any = Single<Self>;` instead of a
/// `version_chain!` of one variant.
///
/// It reads and writes as that `version_chain!` does: `VERSIONS` is the newest struct's version,
/// the struct is parsed with Molang pass 1, the Molang pass 2 walk visits it, and
/// [`to_json`](Self::to_json) writes the document through [`write_envelope`].
///
/// A chain of one declared `molang = none;` has its struct's empty `MolangFields` impl emitted by
/// `version_chain!`; a family moving onto `Single` keeps that impl with
/// `bedrock_addon_core::impl_no_molang!(Struct);` beside the struct. Today's one-version chains that declare it: camera's list, spline and entity chains,
/// aim assist's categories, jigsaw's structure, pool, structure set and processor list, DDUI's
/// root and composition, POI's type and tag, and voxel shape.
pub struct Single<F: Family>(pub F::Latest);

impl<F: Family> VersionChain<F> for Single<F> {
    const VERSIONS: &'static [FormatVersion] = &[<F::Latest as Versioned>::VERSION];

    fn parse(
        payload: JsonSlice<'_>,
        v: FormatVersion,
        cx: &LoadContext,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        parse_selected_as::<F, Self>(payload, v, cx, d)
    }

    fn parse_struct(
        payload: JsonSlice<'_>,
        index: StructIndex,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        match index {
            StructIndex::Stable(0) => parse_with::<F::Latest>(payload, cx, d).map(Self),
            StructIndex::Stable(_) | StructIndex::Beta => None,
        }
    }

    fn with_molang(loaded: &mut Loaded<F>, visit: &mut dyn FnMut(&mut dyn MolangFields)) {
        // No beta struct: a beta value is never loaded through this chain.
        if let Loaded::Stable(value) = loaded {
            visit(value);
        }
    }

    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<F> {
        Loaded::Stable(self.0)
    }
}

impl<F: Family> Single<F> {
    /// The version of the struct that parsed the file: the newest, the only one.
    pub fn parsed_version(&self) -> FormatVersion {
        <F::Latest as Versioned>::VERSION
    }

    /// The document in the struct's own version, envelope included.
    ///
    /// # Errors
    ///
    /// `EncodeError` when the struct has no JSON form.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let document = write_envelope::<F>(
            self.parsed_version(),
            VersionStruct::write(&self.0)?,
            &Extra::default(),
        );
        crate::json::to_json(&document)
    }
}

impl<F: Family> Clone for Single<F>
where
    F::Latest: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<F: Family> fmt::Debug for Single<F>
where
    F::Latest: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Single").field(&self.0).finish()
    }
}

impl<F: Family> PartialEq for Single<F>
where
    F::Latest: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
