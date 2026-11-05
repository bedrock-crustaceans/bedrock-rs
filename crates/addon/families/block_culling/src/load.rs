//! Loading the block-culling rules: `Content` / `Definition`, the family's chain parameters, the
//! walk, and the combine rule (the highest pack's document per identifier).

use super::history::{Any, CullingFamily};
use super::texts::{FAILED_INTERNAL, FAILED_TO_PARSE, MISSING_OBJECT, formatted};
use super::{CullingRules, validate};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Origin, Placed,
    Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, Prepared, WriteStyle, probe_envelope,
};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BetaPermission, EnvelopeCheck, Family, Loaded, MissingVersion, NoComponents, RangeModel,
    ValidateCx, load_file,
};

/// The directory vanilla walks.
const CULLING_DIRECTORY: &str = "block_culling";

/// The oldest version with a parser (`v1_20_60::BlockCulling::VERSION`).
const CULLING_FLOOR: FormatVersion = FormatVersion::new(1, 20, 60);

impl Family for CullingFamily {
    const NAME: &'static str = "block culling";
    const PAYLOAD_KEY: &'static str = "minecraft:block_culling_rules";
    const DIRECTORIES: &'static [&'static str] = &[CULLING_DIRECTORY];
    const FLOOR: FormatVersion = CULLING_FLOOR;
    // The game's loader registers its last parser up to 1.21.80: the ceiling is 1.21.80,
    // below the engine's version.
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = CullingRules;
    type Beta = std::convert::Infallible;
    // No beta slice: `"beta"` is a version without a parser, not a missing toggle.
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    type Component = NoComponents;
    type Any = Any;
    // The game reads the document with the floor 1.20.60:
    // below it (or without a version) the text is kept raw and the envelope is never looked at
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(CULLING_FLOOR);
    // A missing version is 0.0.0, below the floor: the file is skipped.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

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
        let missing_object = || formatted(MISSING_OBJECT, &[path.as_str()]);
        let found = std::mem::take(diagnostics);
        let mut envelope = false;
        let mut above = false;
        let mut parse = false;
        let mut validate = false;
        for mut finding in found {
            if finding.severity >= Severity::Error {
                match &mut finding.code {
                    DiagnosticCode::UnsupportedVersion { text } => {
                        if matches!(text, Some(VanillaText::UseFormatVersion { .. })) {
                            *text = Some(missing_object());
                        } else {
                            *text = Some(VanillaText::NoParserForVersion {
                                version: document.map(written_version).unwrap_or_default(),
                            });
                            above = true;
                        }
                    }
                    DiagnosticCode::MissingFormatVersion { text }
                    | DiagnosticCode::MalformedFormatVersion { text }
                    | DiagnosticCode::MissingPayload { text } => *text = Some(missing_object()),
                    DiagnosticCode::MalformedInput { .. }
                    | DiagnosticCode::EmptyDocument { .. }
                    | DiagnosticCode::PayloadKeyMismatch
                    | DiagnosticCode::UnknownPayloadKey { .. }
                        if finding.location.json_path.is_empty() =>
                    {
                        envelope = true;
                    }
                    DiagnosticCode::UnrecognizedField { text }
                        if text.is_none() && finding.location.json_path.len() == 1 =>
                    {
                        // A third root member: the envelope is not exactly two members.
                        *text = Some(missing_object());
                    }
                    _ if finding.phase == Phase::Validate => validate = true,
                    _ => parse = true,
                }
            }
            diagnostics.push(finding);
        }
        if envelope {
            diagnostics.push(closing(MISSING_OBJECT, path, Phase::Parse));
        } else if above || parse {
            diagnostics.push(closing(FAILED_TO_PARSE, path, Phase::Parse));
        } else if validate {
            diagnostics.push(closing(FAILED_INTERNAL, path, Phase::Validate));
        }
    }
}

impl bedrock_addon_core::__private::Sealed for CullingRules {}

impl Content for CullingRules {
    const NAME: &'static str = "block culling rules";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_culling(PackRef::app_data(), PackSide::Resource, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for CullingRules {}

impl Definition for CullingRules {
    type Kind = kind::CullingRules;
    type File = CullingRules;
    type Beta = NoBeta<kind::CullingRules>;
}

impl Placed for CullingRules {
    /// `block_culling/<namespace>/<name>.json`; `block_culling/<name>.json` for a document of
    /// the `minecraft` namespace. Only a base-game pack may hold such a document, and the
    /// client walks a base-game pack's folder without its sub-folders (`culling_reached`), so
    /// the file sits in the folder itself, as vanilla's own do. A `/` in such a name becomes
    /// `_`, so `minecraft:a/b` and `minecraft:a_b` get the same path: a pack that holds both
    /// names one of the two files itself.
    fn default_path(&self) -> PackPath {
        let nested = default_path(CULLING_DIRECTORY, &self.description.identifier);
        let vanilla = self
            .description
            .identifier
            .namespace()
            .is_some_and(|namespace| namespace.eq_ignore_ascii_case("minecraft"));
        if !vanilla {
            return nested;
        }
        // The segments after `block_culling/<namespace>/`, joined into one file name.
        let name: Vec<&str> = nested.segments().skip(2).collect();
        PackPath::new(&name.join("_")).map_or(nested, |file| {
            PackPath::from_static(CULLING_DIRECTORY).join(&file)
        })
    }
}

/// A closing line of vanilla's for a culling file that was skipped.
fn closing(template: &'static str, path: &PackPath, phase: Phase) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::DocumentRejected {
        text: Some(formatted(template, &[path.as_str()])),
    })
    .phase(phase)
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

/// Loads one culling-rules file of `pack`.
pub(crate) fn parse_culling(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<CullingRules>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    // Either failure of the gate is the one finding.
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: "not UTF-8 JSON within the nesting limit".to_owned(),
            text: None,
        }));
        <CullingFamily as Family>::close(&mut diagnostics, None, path, true);
        diagnostics.locate(&origin);
        return Err(Rejected::new(origin, diagnostics));
    };
    let document = JsonSlice::new(prepared.as_str());
    let outcome = load_file::<CullingFamily>(document, path, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(rules)) => {
            Ok(Document::new(rules, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
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

/// One culling file's contribution.
fn parse_culling_file(file: &WalkFile, cx: &LoadContext) -> Contribution<CullingRules> {
    match parse_culling(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        // A file that fails registers nothing and leaves an earlier document of its
        // identifier in place.
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The registry: per identifier the highest pack's document; within one pack the file walked
/// last, with a `DuplicateIdentifier` warning (ours; vanilla overwrites silently in directory
/// order).
// The game's culling map is keyed by identifier and written only on success.
fn combine_culling(
    parsed: Vec<Contribution<CullingRules>>,
    stack: &StackCx<'_>,
) -> Registry<CullingRules> {
    RegistryBuilder::combine(
        parsed,
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    )
}

/// The loader of `rp.block_culling()`: `block_culling/**/*.json` of every resource pack, the
/// highest pack's document winning per identifier.
pub const LOADER: CollectionLoader<Registry<CullingRules>, Contribution<CullingRules>> =
    CollectionLoader {
        walk: CULLING_WALK,
        parse: parse_culling_file,
        combine: combine_culling,
        needs: &[],
        then: None,
    };

#[cfg(test)]
mod tests {
    use super::*;

    /// Every finding of loading `text` as `block_culling/a.json` of no pack, which must be
    /// rejected.
    fn rejected_findings(text: &str) -> Diagnostics {
        let path = PackPath::from_static("block_culling/a.json");
        match CullingRules::parse(&path, text.as_bytes(), &LoadContext::detached()) {
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
