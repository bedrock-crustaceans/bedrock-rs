//! Feature rules: `feature_rules/**/*.json` of a behaviour pack (`minecraft:feature_rules`), the
//! rules that attach a feature to the biomes it generates in. One struct covers every
//! `format_version` from 1.12.0 to the engine.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod read;
mod types;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    CombineRule, Contribution, Document, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Notice,
    Origin, PackTie, RegistryBuilder, default_path,
};
use bedrock_addon_core::id::{FeatureRuleId, kind};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, BeyondWidth, DocumentRead, Envelope, Family, InputKind, Loaded, MissingVersion,
    NoComponents, Reading, Scalars, Single, StructIndex, UnknownMask, ValidateCx, parse_document,
};

/// The reading of the one slice, which every reader of a rule's members takes: an unknown member
/// is a warning, not an error (the unstrict error mask), and the scalars are vanilla's strict
/// cereal reader's (a member takes only a token of its own type, a number beyond its float's
/// width is a type error). Every struct of a rule keeps its unknown members for write-back.
pub(crate) const READING: Reading = Reading::DOCUMENT
    .with_unknown(UnknownMask::Warn)
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error)
    .with_keeps_unknown(true);

pub use bedrock_addon_shared_scatter::{
    CoordinateDistribution, CoordinateEvalOrder, CoordinateRange, RandomDistribution,
    ScatterChance, ScatterFraction, ScatterParams,
};

pub use types::{
    FeatureRule, FeatureRuleConditions, FeatureRuleDescription, FeatureRules, Iter, PlacementPass,
};

/// The first `format_version` vanilla reads feature rules at: 1.12.0, the first schema.
pub(crate) const FLOOR: FormatVersion = FormatVersion::new(1, 12, 0);

/// The feature-rule family of the version-chain machinery.
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct FeatureRuleFamily;

impl Family for FeatureRuleFamily {
    const NAME: &'static str = "feature rule";
    const PAYLOAD_KEY: &'static str = "minecraft:feature_rules";
    const FLOOR: FormatVersion = FLOOR;
    const ENVELOPE: Envelope = Envelope {
        // The game reads the document as JSON input: the version as it reads it from the text, the
        // size limits, and `null` members rejected (a `null` where a value is read rejects the
        // rule).
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // The game passes no caller version, so a file without `format_version` fails with "no
        // valid version passed by the caller or found in the input" and is skipped.
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };

    /// The one slice is registered with the unstrict error mask: an unknown member is a warning,
    /// not an error ([`READING`]).
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        READING
    }
    type Latest = FeatureRule;
    type Beta = NoBeta<kind::FeatureRule>;
    type Component = NoComponents;
    type History = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(rule) = v {
            validate::validate(rule, cx);
        }
    }
}

impl JsonDocument for FeatureRule {
    type Family = FeatureRuleFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// The rule's own version, never below the floor.
    fn write_version(&self) -> FormatVersion {
        self.source_format_version.max(FLOOR)
    }

    /// The family's read, then the identifier checks of a loaded rule (`identifier_checks`),
    /// in the file.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<FeatureRuleFamily> {
        match parse_document::<FeatureRuleFamily>(origin, bytes, cx) {
            DocumentRead::Stable(document) => {
                let origin = document.origin().clone();
                let version = document.version().clone();
                let mut diagnostics = document.diagnostics().clone();
                let unknown = document.unknown().clone();
                let mut found = Diagnostics::default();
                identifier_checks(document.value(), &origin.path, &mut found);
                found.locate(&origin);
                diagnostics.extend(found);
                DocumentRead::Stable(
                    Document::new(document.into_value(), origin, version, diagnostics)
                        .with_unknown_members(unknown),
                )
            }
            read => read,
        }
    }
}

impl JsonPlaced for FeatureRule {
    /// `feature_rules/<namespace>/<name>.json`: the file name is the identifier's name, as
    /// vanilla requires.
    fn place(&self) -> PackPath {
        default_path("feature_rules", &self.description.identifier)
    }
}

impl JsonDefinition for FeatureRule {
    type Kind = kind::FeatureRule;

    fn identifier(&self) -> &FeatureRuleId {
        &self.description.identifier
    }
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
    let warn = |code: Code| Diagnostic::of(code).at(at.clone()).phase(Phase::Validate);
    match identifier.rfind(':') {
        Some(colon) if colon >= 1 => {
            let name = &identifier[colon + 1..];
            if name != stem {
                d.push(warn(codes::IDENTIFIER_MISMATCH.with([name, stem])));
            }
        }
        _ => d.push(warn(codes::IDENTIFIER_NAMESPACE.with([identifier]))),
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
                let pass = document.conditions.placement_pass.clone();
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

/// One file's contribution: the family's `read` in the file's pack. A failed file names no
/// identifier (the combine reads none).
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<FeatureRule> {
    match FeatureRule::read(origin_of(file), &file.bytes, cx).stable() {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The loader of `bp.feature_rules()`.
#[doc(hidden)]
pub const LOADER: CollectionLoader<FeatureRules, Contribution<FeatureRule>> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    // The combine asks the stack about no other collection and names no extra files.
    needs: &[],
    then: None,
};

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn feature_rules_reads_as_pinned() {
        assert_reading_table(
            "FeatureRuleFamily",
            &reading_table::<super::FeatureRuleFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.12.0..=1.12.0 (0..=0): json Rejected/Strict/Error/Warn/keeps/fatal/Refuse, mcb Own/Strict/Error/Warn/keeps/fatal/Refuse
            ",
        );
    }
}
