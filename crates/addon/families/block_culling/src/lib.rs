//! Block culling rules (`CullingRules`): `block_culling/**/*.json` of a resource pack
//! (`minecraft:block_culling_rules`), which parts of a block model are hidden against which
//! neighbour. A behaviour-pack block names a rules document with `minecraft:geometry` `culling`.
//!
//! The reader stays a hand walk in vanilla's strict cereal terms, because every finding carries
//! vanilla's breadcrumb and text for its path, which the reader kit words only through
//! `Read::push`.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

pub mod codes;
mod read;
mod types;
mod validate;
mod write;

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // the clock, to measure
mod cost;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticCode, Diagnostics, Phase, Severity,
};
use bedrock_addon_core::document::{
    Contribution, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Registry,
    default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::{CullingRulesId, kind};
use bedrock_addon_core::json::{EncodeError, EnvelopeVersion, JsonSlice, probe_envelope};
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    Envelope, EnvelopeCheck, Family, Loaded, MissingVersion, NoComponents, RangeModel, ValidateCx,
};
use history::{CullingFamily, CullingHistory};

pub use types::{Condition, CullingRule, CullingRules, Description, Facing, GeometryPart, Rule};

/// The directory vanilla walks.
const CULLING_DIRECTORY: &str = "block_culling";

/// The oldest version with a parser (`v1_20_60::BlockCulling::VERSION`).
const CULLING_FLOOR: FormatVersion = FormatVersion::new(1, 20, 60);

impl Family for CullingFamily {
    const NAME: &'static str = "block culling";
    const PAYLOAD_KEY: &'static str = "minecraft:block_culling_rules";
    const FLOOR: FormatVersion = CULLING_FLOOR;
    // The game's loader registers its last parser up to 1.21.80: the ceiling is 1.21.80,
    // below the engine's version.
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = CullingRules;
    type Beta = NoBeta<kind::CullingRules>;
    type Component = NoComponents;
    type History = CullingHistory;
    const ENVELOPE: Envelope = Envelope {
        // The game reads the document with the floor 1.20.60:
        // below it (or without a version) the text is kept raw and the envelope is never looked at
        check: EnvelopeCheck::From(CULLING_FLOOR),
        // A missing version is 0.0.0, below the floor: the file is skipped.
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(document) = v {
            validate::check(document, cx.load.is_base_game_pack, &cx.at, cx.diagnostics);
        }
    }

    /// Puts vanilla's texts on the findings of a culling file that was skipped:
    /// - a file that never reaches a slice (unparsable, no version or a malformed one, a version
    ///   below 1.20.60, an envelope that is not `format_version` plus the payload) is "Missing
    ///   object", on the finding itself when it is the version's;
    /// - a version above 1.21.80 is "no parser available", then "Failed to parse";
    /// - a strict parse error is followed by "Failed to parse";
    /// - a failed identifier or rule check by "… into internal format".
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        if !failed {
            return;
        }
        let found = std::mem::take(diagnostics);
        let mut envelope = false;
        let mut above = false;
        let mut parse = false;
        let mut validate = false;
        for mut finding in found {
            if finding.severity >= Severity::Error {
                // By legacy name: the variant and the definitions that were it (the pipeline
                // raises on core's definitions); the plain forms by the variant without a text
                // or core's plain definition.
                let root = finding.location.json_path.is_empty();
                let use_format = finding.is(&core_codes::USE_FORMAT_VERSION);
                let plain_unknown = finding.is(&core_codes::UNRECOGNIZED_FIELD);
                match finding.code.name() {
                    "UnsupportedVersion" if use_format => {
                        finding.code = DiagnosticCode::from(
                            codes::MISSING_OBJECT_UNSUPPORTED_VERSION.with([path.as_str()]),
                        );
                    }
                    "UnsupportedVersion" => {
                        finding.code = DiagnosticCode::from(
                            core_codes::NO_PARSER_FOR_VERSION
                                .with([document.map(written_version).unwrap_or_default()]),
                        );
                        above = true;
                    }
                    "MissingFormatVersion" => {
                        finding.code = DiagnosticCode::from(
                            codes::MISSING_OBJECT_MISSING_FORMAT_VERSION.with([path.as_str()]),
                        );
                    }
                    "MalformedFormatVersion" => {
                        finding.code = DiagnosticCode::from(
                            codes::MISSING_OBJECT_MALFORMED_FORMAT_VERSION.with([path.as_str()]),
                        );
                    }
                    "MissingPayload" => {
                        finding.code = DiagnosticCode::from(
                            codes::MISSING_OBJECT_MISSING_PAYLOAD.with([path.as_str()]),
                        );
                    }
                    "MalformedInput" | "EmptyDocument" | "PayloadKeyMismatch"
                    | "UnknownPayloadKey"
                        if root =>
                    {
                        envelope = true;
                    }
                    "UnrecognizedField"
                        if plain_unknown && finding.location.json_path.len() == 1 =>
                    {
                        // A third root member: the envelope is not exactly two members.
                        finding.code = DiagnosticCode::from(
                            codes::MISSING_OBJECT_UNRECOGNIZED_FIELD.with([path.as_str()]),
                        );
                    }
                    _ if finding.phase == Phase::Validate => validate = true,
                    _ => parse = true,
                }
            }
            diagnostics.push(finding);
        }
        if envelope {
            diagnostics.push(closing(
                codes::MISSING_OBJECT_DOCUMENT_REJECTED.with([path.as_str()]),
                Phase::Parse,
            ));
        } else if above || parse {
            diagnostics.push(closing(
                codes::FAILED_TO_PARSE.with([path.as_str()]),
                Phase::Parse,
            ));
        } else if validate {
            diagnostics.push(closing(
                codes::FAILED_INTERNAL.with([path.as_str()]),
                Phase::Validate,
            ));
        }
    }
}

impl JsonDocument for CullingRules {
    type Family = CullingFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
    const NAME: &'static str = "block culling rules";

    fn refuse_write(&self) -> Result<(), EncodeError> {
        self.writable()
    }
}

impl JsonPlaced for CullingRules {
    /// `block_culling/<namespace>/<name>.json`; `block_culling/<name>.json` for a document of
    /// the `minecraft` namespace. Only a base-game pack may hold such a document, and the
    /// client walks a base-game pack's folder without its sub-folders (`culling_reached`), so
    /// the file sits in the folder itself, as vanilla's own do. A `/` in such a name becomes
    /// `_`, so `minecraft:a/b` and `minecraft:a_b` get the same path: a pack that holds both
    /// names one of the two files itself.
    fn place(&self) -> PackPath {
        default_path_flat_for_vanilla(CULLING_DIRECTORY, &self.description.identifier)
    }
}

/// The registry: per identifier the highest pack's document; within one pack the file walked
/// last, with a `DuplicateIdentifier` warning (ours; vanilla overwrites silently in directory
/// order). A file that fails registers nothing and leaves an earlier document of its
/// identifier in place. The rules are the defaults (`top_wins`).
// The game's culling map is keyed by identifier and written only on success.
impl JsonDefinition for CullingRules {
    type Kind = kind::CullingRules;

    fn identifier(&self) -> &CullingRulesId {
        &self.description.identifier
    }
}

/// A closing line of vanilla's for a culling file that was skipped.
fn closing(code: Code, phase: Phase) -> Diagnostic {
    Diagnostic::of(code).phase(phase)
}

/// The `format_version` of a document as vanilla's "no parser available" text prints it:
/// `SemVersion::asString` of what parsed (`1.26` is `1.26.0`, build metadata dropped), a
/// pre-release and the beta literal as written (the framework's
/// `versioning::no_parser_version`).
fn written_version(document: JsonSlice<'_>) -> String {
    match probe_envelope(document).map(|envelope| envelope.format_version) {
        Ok(EnvelopeVersion::String(text)) => {
            match bedrock_addon_core::version::FormatVersionInput::parse(&text) {
                Some(input) => bedrock_addon_core::versioning::no_parser_version(&input, &text),
                None => text,
            }
        }
        Ok(EnvelopeVersion::Array([major, minor, patch])) => format!("{major}.{minor}.{patch}"),
        _ => String::new(),
    }
}

/// `block_culling/**/*.json` (the extension exactly `json`: the game compares the file's
/// extension, without the dot, case-sensitively), packs bottom-up (the task walks the stack from its last
/// `PackInstance` to its first).
// The game walks `block_culling` non-recursively per pack stack entry.
const CULLING_WALK: Walk = Walk::json_tree_in(&[CULLING_DIRECTORY]).in_pack(culling_reached);

/// Whether vanilla's walk reaches `path` in the pack of `cx`: `forEachIn(…, recurseAnyways =
/// false)` walks a pack shipped in the app package flat and every other pack with its
/// sub-folders (the dialogue family's reading of the same call).
fn culling_reached(path: &PackPath, cx: &LoadContext) -> bool {
    let nested = path.segments().count() > 2;
    !(cx.is_base_game_pack && nested)
}

/// The loader of `rp.block_culling()`: `block_culling/**/*.json` of every resource pack, the
/// highest pack's document winning per identifier.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<CullingRules>, Contribution<CullingRules>> =
    load::registry::<CullingRules>(CULLING_WALK);

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackRef;

    /// Every finding of loading `text` as `block_culling/a.json` of no pack, which must be
    /// rejected.
    fn rejected_findings(text: &str) -> Diagnostics {
        let path = PackPath::from_static("block_culling/a.json");
        match <CullingRules as Content>::parse(&path, text.as_bytes(), &LoadContext::detached()) {
            Ok(document) => panic!("loaded: {:#?}", document.diagnostics()),
            Err(rejected) => rejected.diagnostics,
        }
    }

    #[test]
    fn every_finding_of_a_rejected_file_carries_its_pack_and_path() {
        let payload = |description: &str, rule: &str| {
            format!(
                r#"{{"format_version": "1.21.80", "minecraft:block_culling_rules": {{"description": {{"identifier": "{description}"}}, "rules": [{rule}]}}}}"#
            )
        };
        let path = PackPath::from_static("block_culling/a.json");
        for text in [
            // Not JSON: rejected before the chain runs.
            "{".to_owned(),
            // A member no struct has: rejected by the chain.
            payload(
                "test:a",
                r#"{"geometry_part": {"bone": "x"}, "direction": "up", "what": 2}"#,
            ),
            // The vanilla namespace in a user pack: rejected by the checks.
            payload("minecraft:a", ""),
        ] {
            let found = rejected_findings(&text);
            assert!(!found.is_empty(), "{text}");
            for finding in &found {
                assert_eq!(
                    finding.location.pack,
                    Some(PackRef::app_data()),
                    "{finding:#?}"
                );
                assert_eq!(finding.location.path.as_ref(), Some(&path), "{finding:#?}");
            }
        }
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn block_culling_reads_as_pinned() {
        assert_reading_table(
            "CullingFamily",
            &reading_table::<super::CullingFamily>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 20, patch: 60 })
                missing version: rejected: NoValidVersion
                below floor: UseFormatVersion
                empty document: Error
                1.20.60..=1.21.80 (0..=1): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }
}
