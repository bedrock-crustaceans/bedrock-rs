//! Dimensions: `dimensions/*.json` of a behaviour pack (`minecraft:dimension`), the height
//! bounds and generator of the vanilla dimensions. The chain is `v1_18_0 → v1_26_50`; the
//! `history` module has the older shape.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
pub mod history;
mod read;
mod types;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath, Phase};
use bedrock_addon_core::document::{
    CombineRule, Contribution, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Notice, Origin,
    Registry, RegistryBuilder, default_path_flat,
};
use bedrock_addon_core::id::{DimensionId, kind};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{self, CollectionLoader, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, BeyondWidth, Envelope, EnvelopeCheck, Family, InputKind, Loaded, MissingVersion,
    NoComponents, Reading, Scalars, StructIndex, UnknownMask, ValidateCx,
};
use history::{DimensionFamily, DimensionHistory};
use indexmap::IndexMap;

pub(crate) use read::{RootV1, constraints, parse_root, slice_null_rule};
pub use types::{
    Clouds, DefaultBiome, Dimension, DimensionBounds, DimensionComponents, DimensionDescription,
    DimensionHeight, Generation, GeneratorType,
};

/// `ShadowedDefinition` (info) at a dropped document: a later file of the same pack that
/// defines a dimension the pack already defined.
fn shadowed(origin: &Origin) -> Diagnostic {
    Diagnostic::of(&core_codes::SHADOWED_DEFINITION)
        .phase(Phase::Validate)
        .in_file(origin)
}

/// The first format version vanilla reads.
pub(crate) const FLOOR: FormatVersion = FormatVersion::new(1, 18, 0);
/// The version of the newest struct (`v1_26_50::DimensionDocument`).
pub(crate) const V1_26_50: FormatVersion = bedrock_addon_core::version::known::V1_26_50;

/// The game reads every slice leniently: an unknown member is a warning, not an error. A
/// scalar member takes its own JSON type alone.
pub(crate) const JSON_READING: Reading = Reading::DOCUMENT
    .with_unknown(UnknownMask::Warn)
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error);

impl Family for DimensionFamily {
    const NAME: &'static str = "dimension";
    const PAYLOAD_KEY: &'static str = "minecraft:dimension";
    const FLOOR: FormatVersion = FLOOR;
    const ENVELOPE: Envelope = Envelope {
        // The game reads the file as a whole document: the version is the one its JSON reader picks
        // from the text, and a `null` where a value is read counts as the wrong JSON type in the
        // cereal slices (the legacy parser below 1.21.60 keeps its own reading:
        // `parse::slice_null_rule`).
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // The game reads the document with no minimum version, so the raw text is read at every
        // version.
        check: EnvelopeCheck::Never,
        // The game passes no version of its own, so a file without `format_version` is rejected
        // with "no valid version passed by the caller or found in the input".
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        JSON_READING
    }
    type Latest = Dimension;
    type Beta = NoBeta<kind::Dimension>;
    type Component = NoComponents;
    type History = DimensionHistory;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(dimension) = v {
            validate::validate(dimension, cx);
        }
    }
}

impl JsonDocument for Dimension {
    type Family = DimensionFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for Dimension {
    /// `dimensions/<name>.json` for a `minecraft:` identifier, `dimensions/<namespace>_<name>.json`
    /// for any other, the `/` of the name as `_`: the walk reads the files directly in the
    /// folder only.
    fn place(&self) -> PackPath {
        default_path_flat("dimensions", &self.description.identifier)
    }
}

impl JsonDefinition for Dimension {
    type Kind = kind::Dimension;

    fn identifier(&self) -> &DimensionId {
        &self.description.identifier
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `dimensions/*.json` (not recursive; an extension exactly `json`, vanilla's byte compare),
/// from the last stack entry to the first: packs bottom-up.
const WALK: Walk = Walk::json_children_in(&["dimensions"]);

/// Whether `id` is a custom dimension registered from script (`DimensionRegistry.
/// registerCustomDimension`): a dimension `LoadOptions::built_ins` knows outside the
/// `minecraft` namespace, which the script path reserves.
fn registered_from_script(id: &DimensionId, stack: &StackCx<'_>) -> bool {
    id.namespace() != Some("minecraft")
        && stack.options.built_ins.as_ref().is_some_and(|built_ins| {
            built_ins.contains(
                <kind::Dimension as bedrock_addon_core::id::Kind>::NAME,
                id.as_str(),
            )
        })
}

/// The warning of a pack definition that replaces a script registration: `ShadowedDefinition`
/// (the registration is the definition shadowed) with vanilla's text, the identifier and the
/// pack id as arguments, at the document's identifier. The game logs it at the content log's
/// warning level.
fn replaced_registration(id: &DimensionId, origin: &Origin) -> Diagnostic {
    let at = JsonPath::new().with(&["minecraft:dimension", "description", "identifier"]);
    // The game logs this at the content log's warning level.
    Diagnostic::of(
        codes::REPLACED_REGISTRATION
            .with([id.as_str().to_owned(), origin.pack.id().uuid.to_string()]),
    )
    .at(at)
    .phase(Phase::Validate)
    .in_file(origin)
}

/// The registry: the first file walked wins (an insert only when the identifier is absent), so
/// the lowest-priority pack's; a later one is `ShadowedDefinition` (info; vanilla turns it into
/// a load error without text). Across packs `BottomWins` decides; within one pack the later file
/// is dropped here. The definition that wins replaces a script registration of its
/// identifier, with a warning; a file that fails to load replaces nothing.
fn combine(parsed: Vec<Contribution<Dimension>>, stack: &StackCx<'_>) -> Registry<Dimension> {
    let mut builder = RegistryBuilder::new(
        CombineRule::BottomWins {
            shadowed: Some(Notice::shadowed()),
        },
        CombineRule::bottom_wins(),
        stack.options,
    );
    let mut first: IndexMap<DimensionId, Option<usize>> = IndexMap::new();
    for contribution in parsed {
        if let Contribution::Stable(id, document) = &contribution {
            let pack = document.origin().pack.index();
            match first.get(id) {
                Some(&claimed) if claimed == pack => {
                    for finding in document.diagnostics().iter().cloned() {
                        builder.push_diagnostic(finding);
                    }
                    builder.push_diagnostic(shadowed(document.origin()));
                    continue;
                }
                Some(_) => {}
                None => {
                    first.insert(id.clone(), pack);
                    if registered_from_script(id, stack) {
                        builder.push_diagnostic(replaced_registration(id, document.origin()));
                    }
                }
            }
        }
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.dimensions()`: the registry's walk and parse with the family's own
/// combine (the within-pack drop and the script-registration warning, findings
/// `RegistryBuilder::combine` does not push).
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Dimension>, Contribution<Dimension>> =
    CollectionLoader {
        combine,
        ..load::registry::<Dimension>(WALK)
    };

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn dimension_reads_as_pinned() {
        assert_reading_table(
            "DimensionFamily",
            &reading_table::<super::DimensionFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Never
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.18.0..=1.26.50 (0..=1): json Rejected/Strict/Error/Warn/reports/fatal/Refuse, mcb Own/Strict/Error/Warn/reports/fatal/Refuse
            ",
        );
    }
}
