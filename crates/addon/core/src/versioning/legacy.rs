//! [`Legacy`]: the struct of an older format version whose payload is one type in every era.
//!
//! A family whose older formats are one payload type read and written by the version (entity's
//! JSON payload, block's legacy document, fog's documents before 1.26.20) declares that payload
//! once, implements [`LegacyPayload`] on it, and names each era `Legacy<Payload, A, B, C>`. The
//! steps are [`LegacyStep`] impls on the payload: the orphan rule refuses a family's
//! `impl Upgrade for Legacy<Local, ..>` (both are core's), so the family implements the step on
//! its own type and the blanket `Upgrade` here runs it.
//!
//! ```
//! # use bedrock_addon_core::diagnostic::Diagnostics;
//! # use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
//! # use bedrock_addon_core::version::FormatVersion;
//! use bedrock_addon_core::versioning::{Legacy, LegacyPayload, LegacyStep, ParseCx, UpgradeCx};
//!
//! /// The payload of every older era.
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct Payload(Value);
//! bedrock_addon_core::impl_no_molang!(Payload);
//!
//! impl LegacyPayload for Payload {
//!     fn parse(payload: JsonSlice<'_>, _: FormatVersion, _: &ParseCx<'_>, _: &mut Diagnostics) -> Option<Self> {
//!         bedrock_addon_core::json::from_str(payload.text).ok().map(Self)
//!     }
//!     fn write(&self, _: FormatVersion) -> Result<Value, EncodeError> {
//!         Ok(self.0.clone())
//!     }
//! }
//!
//! /// The eras.
//! pub type V1 = Legacy<Payload, 1, 0, 0>;
//! pub type V2 = Legacy<Payload, 1, 1, 0>;
//!
//! /// 1.0.0 → 1.1.0: nothing changes but the version.
//! impl LegacyStep<1, 0, 0> for Payload {
//!     type Next = V2;
//!     fn step(this: V1, _: &mut UpgradeCx<'_>) -> V2 {
//!         this.retag()
//!     }
//! }
//! ```

use std::ops::{Deref, DerefMut};

use super::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use crate::diagnostic::Diagnostics;
use crate::document::Unknown;
use crate::json::{EncodeError, JsonSlice, Value};
use crate::molang::{MolangFields, MolangVisitor};
use crate::version::FormatVersion;

/// The payload of a family's older formats: one type read and written by the version of the
/// era that holds it ([`Legacy`]).
pub trait LegacyPayload: MolangFields + Sized {
    /// Parses the payload as the struct of `version` reads it ([`VersionStruct::parse`] with the
    /// era's version).
    fn parse(
        payload: JsonSlice<'_>,
        version: FormatVersion,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self>;

    /// The payload as the struct of `version` writes it ([`VersionStruct::write`]).
    ///
    /// # Errors
    ///
    /// `EncodeError` when the payload has no JSON form.
    fn write(&self, version: FormatVersion) -> Result<Value, EncodeError>;

    /// [`write`](Self::write) with the document's unknown members
    /// ([`VersionStruct::write_with_unknown`]); the plain write by default.
    ///
    /// # Errors
    ///
    /// `EncodeError` when the payload has no JSON form.
    fn write_with_unknown(
        &self,
        version: FormatVersion,
        unknown: &Unknown,
    ) -> Result<Value, EncodeError> {
        let _ = unknown;
        self.write(version)
    }
}

/// The struct of format `A.B.C`: the payload `P` as that era reads and writes it. `Versioned`,
/// `VersionStruct` and `MolangFields` are `P`'s with the era's version; `Upgrade` is `P`'s
/// [`LegacyStep<A, B, C>`].
///
/// A type alias names an era (`pub type Entity = Legacy<Payload, 1, 8, 0>;`); an alias cannot
/// construct or match a tuple struct, so a value is built and matched as `Legacy(payload)`. The
/// struct derefs to its payload.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Legacy<P, const A: u16, const B: u16, const C: u16>(pub P);

impl<P, const A: u16, const B: u16, const C: u16> Legacy<P, A, B, C> {
    /// The same payload in another era: the identity step, and the start of a step that
    /// rewrites the payload in place.
    pub fn retag<const X: u16, const Y: u16, const Z: u16>(self) -> Legacy<P, X, Y, Z> {
        Legacy(self.0)
    }
}

/// The era's payload, as the struct holds it: `legacy.description` reads the payload's member.
impl<P, const A: u16, const B: u16, const C: u16> Deref for Legacy<P, A, B, C> {
    type Target = P;

    fn deref(&self) -> &P {
        &self.0
    }
}

impl<P, const A: u16, const B: u16, const C: u16> DerefMut for Legacy<P, A, B, C> {
    fn deref_mut(&mut self) -> &mut P {
        &mut self.0
    }
}

impl<P, const A: u16, const B: u16, const C: u16> Versioned for Legacy<P, A, B, C> {
    const VERSION: FormatVersion = FormatVersion::new(A, B, C);
}

impl<P: LegacyPayload, const A: u16, const B: u16, const C: u16> VersionStruct
    for Legacy<P, A, B, C>
{
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        P::parse(payload, Self::VERSION, cx, d).map(Self)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.0.write(Self::VERSION)
    }

    fn write_with_unknown(&self, unknown: &Unknown) -> Result<Value, EncodeError> {
        self.0.write_with_unknown(Self::VERSION, unknown)
    }
}

impl<P: MolangFields, const A: u16, const B: u16, const C: u16> MolangFields
    for Legacy<P, A, B, C>
{
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        self.0.visit_molang(visitor);
    }
}

/// The step out of format `A.B.C`, implemented by the family on its payload type (which the
/// orphan rule lets it): `Legacy<P, A, B, C>`'s `Upgrade` runs it.
pub trait LegacyStep<const A: u16, const B: u16, const C: u16>: Sized {
    /// The struct the step gives: the next era, or the newest struct.
    type Next: Versioned;

    /// Carries the struct of `A.B.C` one version forward. An identity step is
    /// `this.retag()`.
    fn step(this: Legacy<Self, A, B, C>, cx: &mut UpgradeCx<'_>) -> Self::Next;
}

impl<P: LegacyStep<A, B, C>, const A: u16, const B: u16, const C: u16> Upgrade
    for Legacy<P, A, B, C>
{
    type Next = P::Next;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> P::Next {
        P::step(self, cx)
    }
}
