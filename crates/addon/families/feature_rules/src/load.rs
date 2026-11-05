//! Loading feature rules: the family's chain parameters, `Content` / `Definition`, the walk of
//! `feature_rules/` and the combine rule (first rule of a `(pass, identifier)` pair wins, packs
//! top-down).

use indexmap::IndexMap;

use super::{FLOOR, FeatureRule, FeatureRules, PlacementPass, validate};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    PackTie, Placed, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, Prepared, WriteStyle};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::versioning::{
    BelowFloor, ErrorMask, Family, InputKind, Loaded, MissingVersion, NoComponents, StructIndex,
    ValidateCx, finish, parse_as_written,
};

/// The feature-rule family of the version-chain machinery.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FeatureRuleFamily;

bedrock_addon_core::version_chain! {
    family = FeatureRuleFamily;
    /// A feature-rule document parsed by the struct its `format_version` selects: there is one.
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) enum Any {
        /// Format 1.12.0 up to the engine.
        V1_12_0(FeatureRule),
    }
}

impl Family for FeatureRuleFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "feature rule";
    const PAYLOAD_KEY: &'static str = "minecraft:feature_rules";
    const DIRECTORIES: &'static [&'static str] = &["feature_rules"];
    const FLOOR: bedrock_addon_core::version::FormatVersion = FLOOR;
    // Below the first slice the versioned loader has no parser at all.
    const BELOW_FLOOR: BelowFloor = BelowFloor::NoParser;
    type Latest = FeatureRule;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // The game reads the document as JSON input: the version as it reads it from the text, the
    // size limits, and `null` members rejected (a `null` where a value is read rejects the
    // rule).
    const INPUT: InputKind = InputKind::Document;
    // The game passes no caller version, so a file without `format_version` fails with "no
    // valid version passed by the caller or found in the input" and is skipped.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

    /// The one slice is registered with the unstrict error mask: an unknown member is a
    /// warning, not an error.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Unstrict
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(rule) = v {
            validate::validate(rule, cx);
        }
    }
}

impl bedrock_addon_core::__private::Sealed for FeatureRule {}

impl Content for FeatureRule {
    const NAME: &'static str = "feature rule";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for FeatureRule {}

impl Definition for FeatureRule {
    type Kind = kind::FeatureRule;
    type File = FeatureRule;
    type Beta = NoBeta<kind::FeatureRule>;
}

impl Placed for FeatureRule {
    /// `feature_rules/<namespace>/<name>.json`: the file name is the identifier's name, as
    /// vanilla requires.
    fn default_path(&self) -> PackPath {
        default_path("feature_rules", &self.description.identifier)
    }
}

/// The finding of an unreadable file: not UTF-8, or nested past the limit (one text for both).
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// The file stem vanilla matches identifiers against: the text after the last `/` or `\` and
/// before the last `.` (the whole name without a `.`).
fn stem(path: &PackPath) -> &str {
    let name = path.as_str().rsplit(['/', '\\']).next().unwrap_or_default();
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// The identifier's namespace syntax, then its name against the file stem. Vanilla logs an
/// error for either and still inserts the rule: a kept error, an error-level finding after
/// which the game keeps the document.
fn identifier_checks(rule: &FeatureRule, path: &PackPath, d: &mut Diagnostics) {
    let identifier = rule.description.identifier.as_str();
    let stem = stem(path);
    if identifier == stem {
        return;
    }
    let at = JsonPath::new().with(&["minecraft:feature_rules", "description", "identifier"]);
    let warn = |template, args: Vec<String>| {
        Diagnostic::error(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Formatted { template, args }),
        })
        .at(at.clone())
        .phase(Phase::Validate)
        .into_kept()
    };
    match identifier.rfind(':') {
        Some(colon) if colon >= 1 => {
            let name = &identifier[colon + 1..];
            if name != stem {
                d.push(warn(
                    "Feature rule identifier '{}' does not match filename '{}'",
                    vec![name.to_owned(), stem.to_owned()],
                ));
            }
        }
        _ => d.push(warn(
            "Feature rule identifier '{}' does not use the appropriate namespace syntax",
            vec![identifier.to_owned()],
        )),
    }
}

/// Loads one feature-rule file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<FeatureRule>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    // A leading byte-order mark is blanked, not cut, so every span is an offset into the file
    // as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        return Err(Rejected::located(origin, diagnostics));
    };
    let parsed = parse_as_written::<FeatureRuleFamily>(prepared.slice(), cx);
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(rule)) => {
            let mut found = Diagnostics::default();
            identifier_checks(&rule, path, &mut found);
            found.locate(&origin);
            diagnostics.extend(found);
            Ok(Document::new(rule, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `feature_rules/**/*.json`, packs top-down (vanilla walks the stack from its highest-priority
/// pack), then the app data's `definitions/feature_rules`. Every pack's matching file is read,
/// subpack-shadowed copies are not, and the family has no text of its own for an unreadable
/// file.
const WALK: Walk = Walk::json_tree_in(&["feature_rules"])
    .top_down()
    .app_data(&["definitions/feature_rules"], AppDataPlace::After);

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<FeatureRule> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The group: one registry per pass, the first rule of a `(pass, identifier)` pair in walk
/// order effective. Across packs the registry's `TopWins` decides (the walk is
/// top-down, so it is the first one) and shadows the lower with `ShadowedDefinition`; within one
/// pack (or the app data) the tie keeps the first file and shadows the later one with the same
/// notice, where `TopWins` alone would keep the later one: the game looks the identifier up in
/// the pass's bucket and a found one drops the new rule.
fn combine(parsed: Vec<Contribution<FeatureRule>>, stack: &StackCx<'_>) -> FeatureRules {
    let builder = || {
        RegistryBuilder::new(
            CombineRule::TopWins {
                shadowed: Some(Notice::shadowed()),
            }
            .within_pack(PackTie::first(Some(Notice::shadowed()))),
            CombineRule::top_wins(),
            stack.options,
        )
    };
    let mut passes: Vec<RegistryBuilder<FeatureRule>> =
        PlacementPass::ALL.iter().map(|_| builder()).collect();
    let mut unknown: IndexMap<String, RegistryBuilder<FeatureRule>> = IndexMap::new();
    let mut rejected = Vec::new();
    let mut diagnostics = Diagnostics::default();
    for contribution in parsed {
        match contribution {
            Contribution::Stable(id, document) => {
                let pass = document.conditions.placement_pass.0.clone();
                let target = match PlacementPass::from_name(&pass) {
                    Some(known) => &mut passes[known as usize],
                    None => unknown.entry(pass).or_insert_with(builder),
                };
                target.push(Contribution::Stable(id, document));
            }
            // A feature rule has no beta struct; nothing produces one.
            Contribution::Beta(..) => {}
            Contribution::Rejected(_, failed) => {
                diagnostics.extend(failed.diagnostics.iter().cloned());
                rejected.push(failed);
            }
        }
    }
    let mut passes = passes.into_iter().map(RegistryBuilder::finish);
    let mut next = || passes.next().unwrap_or_else(|| builder().finish());
    FeatureRules {
        pregeneration_pass: next(),
        first_pass: next(),
        before_underground_pass: next(),
        underground_pass: next(),
        after_underground_pass: next(),
        before_surface_pass: next(),
        surface_pass: next(),
        after_surface_pass: next(),
        before_sky_pass: next(),
        sky_pass: next(),
        after_sky_pass: next(),
        final_pass: next(),
        unknown_passes: unknown
            .into_iter()
            .map(|(pass, builder)| (pass, builder.finish()))
            .collect(),
        rejected,
        diagnostics,
    }
}

/// What a `StackReport` lists of the group: every finding, the failed files' first.
impl bedrock_addon_core::load::Reported for FeatureRules {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

/// The loader of `bp.feature_rules()`.
pub const LOADER: CollectionLoader<FeatureRules, Contribution<FeatureRule>> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    // The combine asks the stack about no other collection and names no extra files.
    needs: &[],
    then: None,
};
