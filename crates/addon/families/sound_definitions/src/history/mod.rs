//! The older shapes of `sounds/sound_definitions.json` and the three steps that carry them to
//! the newest: `v1_0_0` (the versionless root) → `v1_14_0` (the envelope) → `v1_20_20`
//! (integer distances read) → 1.26.50 ([`SoundDefinitions`], `allow_concurrent_streaming`
//! read).
//!
//! Vanilla's steps are a JSON fixer, so the older structs hold the document as JSON and each
//! step rewrites it. Which struct reads a file is decided by [`SoundDefinitionsHistory::select`]: a
//! `format_version` that is missing or does not read as a version counts
//! as 1.13.0, the versionless root, and there is no newest version.

use super::SoundDefinitions;
use super::upgrade::{self, Upgraded};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// The struct versions of the chain, oldest first.
pub const VERSIONS: &[FormatVersion] = &[
    <v1_0_0::SoundDefinitions as Versioned>::VERSION,
    <v1_14_0::SoundDefinitions as Versioned>::VERSION,
    <v1_20_20::SoundDefinitions as Versioned>::VERSION,
    <SoundDefinitions as Versioned>::VERSION,
];

/// The document of a file, as JSON. A text that is no JSON gives none, with a
/// `MalformedInput` error at the root: the family's loader reads the text first and
/// never comes here with one, a caller of a struct's `parse` can.
fn document(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Value> {
    match bedrock_addon_core::json::from_str_jsonc::<Value>(&bedrock_addon_core::json::last_wins(
        payload.text,
    )) {
        Ok(document) => Some(document),
        Err(error) => {
            d.push(
                bedrock_addon_core::diagnostic::Diagnostic::error(
                    bedrock_addon_core::diagnostic::DiagnosticCode::MalformedInput {
                        message: error.to_string(),
                        text: None,
                    },
                )
                .at(cx.at.clone()),
            );
            None
        }
    }
}

pub mod v1_0_0;

pub mod v1_14_0;

pub mod v1_20_20;

impl Versioned for SoundDefinitions {
    const VERSION: FormatVersion = upgrade::V1_26_50;
}

impl MolangFields for SoundDefinitions {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for SoundDefinitions {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        document(payload, cx, d).map(|document| Self::read(document, d))
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}

/// A sound definitions file as the struct of its own version reads it.
// Hand-written: waits on the `Family` skeleton (not yet adopted by this family).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "the newest struct is the typed value; boxing it would change the variant's public type"
)]
pub enum SoundDefinitionsHistory {
    /// Below 1.14.0, or without a readable `format_version`.
    V1_0_0(v1_0_0::SoundDefinitions),
    /// From 1.14.0 up to 1.20.20.
    V1_14_0(v1_14_0::SoundDefinitions),
    /// From 1.20.20 up to 1.26.50.
    V1_20_20(v1_20_20::SoundDefinitions),
    /// 1.26.50 and every later version.
    V1_26_50(SoundDefinitions),
}

impl SoundDefinitionsHistory {
    /// The version of the struct that reads a file whose gate version is `version` (1.13.0
    /// for a file without a readable `format_version`): the newest struct at or below it.
    /// There is no ceiling.
    pub fn select(version: &SemVersion) -> FormatVersion {
        VERSIONS
            .iter()
            .rev()
            .copied()
            .find(|floor| SemVersion::from(*floor) <= *version)
            .unwrap_or(VERSIONS[0])
    }

    /// Reads `document` with the struct its `format_version` selects.
    pub(crate) fn read(document: Value, diagnostics: &mut Diagnostics) -> Self {
        let selected = Self::select(&upgrade::gate_version(&document));
        if selected == <v1_0_0::SoundDefinitions as Versioned>::VERSION {
            Self::V1_0_0(v1_0_0::SoundDefinitions {
                definitions: document,
            })
        } else if selected == <v1_14_0::SoundDefinitions as Versioned>::VERSION {
            Self::V1_14_0(v1_14_0::SoundDefinitions { document })
        } else if selected == <v1_20_20::SoundDefinitions as Versioned>::VERSION {
            Self::V1_20_20(v1_20_20::SoundDefinitions { document })
        } else {
            Self::V1_26_50(SoundDefinitions::read(document, diagnostics))
        }
    }

    /// The version of the struct that read the file (named so that `Document<SoundDefinitionsHistory>`'s own
    /// `version` does not hide it).
    pub fn parsed_version(&self) -> FormatVersion {
        match self {
            Self::V1_0_0(_) => <v1_0_0::SoundDefinitions as Versioned>::VERSION,
            Self::V1_14_0(_) => <v1_14_0::SoundDefinitions as Versioned>::VERSION,
            Self::V1_20_20(_) => <v1_20_20::SoundDefinitions as Versioned>::VERSION,
            Self::V1_26_50(_) => <SoundDefinitions as Versioned>::VERSION,
        }
    }

    /// Runs the remaining steps, which report to `diagnostics`: the value
    /// [`SoundDefinitions`] parses the same file to.
    pub fn upgrade(self, cx: &LoadContext, diagnostics: &mut Diagnostics) -> SoundDefinitions {
        let cx = &mut UpgradeCx {
            load: cx,
            original_version: self.parsed_version(),
            identifier: "",
            actor_type: ActorType::Mob,
            skip_section_a: false,
            diagnostics,
        };
        match self {
            Self::V1_0_0(value) => value.upgrade(cx).upgrade(cx).upgrade(cx),
            Self::V1_14_0(value) => value.upgrade(cx).upgrade(cx),
            Self::V1_20_20(value) => value.upgrade(cx),
            Self::V1_26_50(value) => value,
        }
    }

    /// The document in the struct's own version, as JSON text: an older struct's document as
    /// read, the newest struct's typed value.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`](bedrock_addon_core::json::EncodeError) when a value has no JSON
    /// spelling, such as a non-finite number.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let document = match self {
            Self::V1_0_0(value) => &value.definitions,
            Self::V1_14_0(value) => &value.document,
            Self::V1_20_20(value) => &value.document,
            Self::V1_26_50(value) => &value.to_value()?,
        };
        bedrock_addon_core::json::to_json(document)
    }
}
