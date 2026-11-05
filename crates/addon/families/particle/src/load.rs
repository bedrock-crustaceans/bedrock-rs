//! Loading particle effects: the family's version chain parameters, `Content` /
//! `Definition`, the walk of `particles/` and the combine rule.

use super::ParticleEffect;
use super::component::ParticleComponent;
use super::history::{Any, ParticleFamily};
use super::parse::CEREAL;
use super::write::PAYLOAD_KEY;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Origin, Placed,
    Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Prepared, WriteStyle, probe_envelope};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    BelowFloor, ComponentSection, ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity,
    InputKind, Loaded, Parsed, StructIndex, finish, parse_as_written,
};

impl Family for ParticleFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "particle";
    const PAYLOAD_KEY: &'static str = PAYLOAD_KEY;
    const DIRECTORIES: &'static [&'static str] = &["particles"];
    // The legacy slice is the only one below 1.20.80, and data-driven particles start with
    // it.
    const FLOOR: FormatVersion = known::V1_10_0;
    // Below the first slice the versioned loader has no parser at all.
    const BELOW_FLOOR: BelowFloor = BelowFloor::NoParser;
    type Latest = ParticleEffect;
    // Particles have no experimental-build struct, so there is no second text to read.
    type Beta = std::convert::Infallible;
    type Component = ParticleComponent;
    type Any = Any;
    // Below 1.20.80 the game hands the legacy parser the raw text; from 1.20.80 the envelope
    // is checked and has exactly two members.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(CEREAL);
    // The game reads a particle file as a document at every version, taking the version
    // from the first textual `"format_version"`.
    const INPUT: InputKind = InputKind::Document;

    fn components() -> &'static ComponentTable<Self> {
        ParticleComponent::TABLE
    }

    /// The legacy slice `[1.10.0, 1.20.80)` is a hand-written parser, which reads the input
    /// itself (`null` as its readers take it). A 1.20.80 document is read by the family's
    /// own reader (`parse.rs`), which refuses a `null` where it reads a value.
    fn null_rule(index: StructIndex) -> Option<NullRule> {
        match index {
            StructIndex::Stable(0) => None,
            _ => Some(NullRule::Rejected),
        }
    }

    /// An unknown member or component of a 1.20.80 document refuses it: the game loads
    /// that format strictly. The legacy parser's unknown-component line is a content-log
    /// warning that no strict rule acts on, so its slice keeps the warning.
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(0) => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }

    fn component_sections(v: &mut ParticleEffect) -> Vec<ComponentSection<'_, ParticleComponent>> {
        let at = JsonPath::new().child("components");
        vec![ComponentSection::new(at, false, &mut v.components)]
    }
}

impl bedrock_addon_core::__private::Sealed for ParticleEffect {}

impl Content for ParticleEffect {
    const NAME: &'static str = "particle";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Resource, path, bytes, cx)
    }

    /// The effect as a 1.20.80 document, in `style`: `format_version` and `particle_effect`, with
    /// `description` and `components` always (the 1.20.80 reader requires them) and every
    /// other member only where the value has it.
    ///
    /// An effect that was read partially from a file below 1.20.80, because the legacy
    /// parser stopped in it, is written whole at its defaults: `basic_render_parameters`
    /// with an empty `material` or `texture` and an empty `components` are written although
    /// the file had none, and what the parser never reached (components, curves, events
    /// after the stop) is not. The text loads again to an equal value, without the warning
    /// the first load gave, since the written file has the members that warning was about.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for ParticleEffect {}

impl Definition for ParticleEffect {
    type Kind = kind::Particle;
    type File = ParticleEffect;
    type Beta = NoBeta<kind::Particle>;
}

impl Placed for ParticleEffect {
    /// `particles/<identifier>.json`, the identifier's `:` written `_`: a particle
    /// identifier has no namespace of its own, so
    /// `minecraft:basic_flame_particle` is `particles/minecraft_basic_flame_particle.json`.
    fn default_path(&self) -> PackPath {
        default_path("particles", &self.description.identifier)
    }
}

impl ParticleEffect {
    /// Parses one particle file without upgrading it: the struct its `format_version`
    /// selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Resource, path);
        let Ok(prepared) = Prepared::new(bytes) else {
            return Err(reject(origin, malformed()));
        };
        let text = prepared.as_str();
        let document = JsonSlice::new(text);
        let parsed = parse_as_written::<ParticleFamily>(document, cx);
        let mut diagnostics = parsed.diagnostics;
        vanilla_envelope_texts(&mut diagnostics, document);
        diagnostics.locate(&origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// The body of `history::upgrade`: rebuilds the parse stage from the document and finishes
/// it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<ParticleEffect>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let identifier = match &value {
        Any::V1_10_0(effect) => &effect.description.identifier,
        Any::V1_20_80(effect) => &effect.description.identifier,
    };
    // What is found on the upgraded value is below the payload key in both cases.
    let at = JsonPath::new().child(PAYLOAD_KEY);
    let identity = Identity {
        identifier: Some(identifier.as_str().to_owned()),
        runtime_identifier: None,
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<ParticleFamily>::new(
        Some(value),
        version,
        written,
        identity,
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(effect)) => {
            Ok(Document::new(effect, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The finding of an unreadable file.
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// A rejection with one finding.
fn reject(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    Rejected::located(origin, diagnostics)
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// The envelope findings as the particle loader words them, where the shared stage words them
/// otherwise: from 1.20.80, a document with `format_version` alone has no payload, which the
/// game words as "The payload of the Minecraft Document is missing"; and from 1.20.80, a
/// payload under another key is refused at that key, a finding of the crate's own because the
/// game checks the name only in development builds. (Below 1.10.0 there is no slice: the
/// pipeline says no parser is available, `BELOW_FLOOR`.)
fn vanilla_envelope_texts(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    let cleaned = bedrock_addon_core::json::last_wins(document.text);
    let envelope = probe_envelope(JsonSlice::new(&cleaned)).ok();
    // In place, as `Diagnostics::locate`.
    for finding in diagnostics.iter_mut() {
        match &mut finding.code {
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            } if envelope.as_ref().is_some_and(|e| e.members.is_empty()) => {
                finding.code = bedrock_addon_core::diagnostic::cereal::payload_missing();
            }
            _ => {}
        }
    }
}

/// Loads one particle file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<ParticleEffect>, Rejected> {
    let origin = origin(pack, side, path);
    let Ok(prepared) = Prepared::new(bytes) else {
        return Err(reject(origin, malformed()));
    };
    let text = prepared.as_str();
    let document = JsonSlice::new(text);
    let mut parsed = parse_as_written::<ParticleFamily>(document, cx);
    // The 1.10.0 struct is the whole document, the upgraded one its payload: what is found
    // on the upgraded value is below the payload key, as in a 1.20.80 file.
    if matches!(parsed.value, Some(Any::V1_10_0(_))) {
        parsed.at.push(PAYLOAD_KEY);
    }
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    vanilla_envelope_texts(&mut diagnostics, document);
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(effect)) => {
            Ok(Document::new(effect, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `particles/**/*.json`, packs bottom-up: the effect loaded last is bound to its identifier,
/// so the highest pack's is the effective one. Sub-folders are read in every pack, as for
/// the other loaders that list a folder the same way.
const WALK: Walk = Walk::json_tree_in(&["particles"]);

/// One file's contribution. A file that failed binds nothing, whatever it names. A 1.10.0
/// file the legacy parser stops in after its identifier is no failed file: it loads as far as
/// it was read, and that effect is its contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<ParticleEffect> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The registry: per identifier the effect loaded last, each replaced one noted with
/// `ShadowedDefinition` at the information level (the game replaces silently, so the finding
/// is the crate's own). A file that failed is skipped ("Failed to load particle effect"), so
/// an effect below it stays the effective one: a failure is any problem with the document as
/// a whole in either slice, which is every failure from 1.20.80. Below 1.20.80 the legacy
/// parser's own lines fail nothing, and the effect it got to replaces the one below.
fn combine(
    parsed: Vec<Contribution<ParticleEffect>>,
    stack: &StackCx<'_>,
) -> Registry<ParticleEffect> {
    RegistryBuilder::combine(
        parsed,
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    )
}

/// The loader of `rp.particles()`: `particles/**/*.json` of every resource pack, the effect
/// loaded last winning per identifier.
pub const fn load_registry()
-> CollectionLoader<Registry<ParticleEffect>, Contribution<ParticleEffect>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::SINGLE;
    use bedrock_addon_core::document::VersionInfo;
    use bedrock_addon_core::id::ParticleId;
    use bedrock_addon_core::load::LoadOptions;
    use bedrock_addon_core::pack_identity::{PackIdVersion, PackType};
    use bedrock_addon_core::test_cost::{TIMES, assert_linear_consuming_at};
    use bedrock_addon_core::version::SemVersion;

    /// The pack at `index` of a stack (0 is the top).
    fn pack(index: usize, name: &str) -> PackRef {
        let id = PackIdVersion::new(
            uuid::Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        );
        PackRef::new(index, id, name)
    }

    /// The file `particles/<file>.json` of `pack`.
    fn file(pack: &PackRef, file: &str) -> Origin {
        let path = PackPath::new(&format!("particles/{file}.json")).expect("a pack path");
        origin(pack.clone(), PackSide::Resource, &path)
    }

    /// The effect `demo:p<i>` as loaded from `pack`.
    fn stable(pack: &PackRef, name: &str, i: usize) -> Contribution<ParticleEffect> {
        let identifier = ParticleId::parse(format!("demo:p{i}"));
        let effect = ParticleEffect::new(identifier.clone(), "particles_alpha", "textures/t");
        let version = VersionInfo::new(Some(CEREAL), false, CEREAL, CEREAL, false);
        let document = Document::new(
            effect,
            file(pack, &format!("{name}{i}")),
            version,
            Diagnostics::default(),
        );
        Contribution::Stable(identifier, document)
    }

    /// What the walk hands `combine` for a stack of three packs, bottom first: `count`
    /// effects in the bottom pack, every second of them again in the middle pack, and
    /// `count / 2` files of the top pack that failed.
    fn contributions(count: usize) -> Vec<Contribution<ParticleEffect>> {
        let (top, middle, bottom) = (pack(0, "top"), pack(1, "middle"), pack(2, "bottom"));
        let mut parsed: Vec<_> = (0..count).map(|i| stable(&bottom, "p", i)).collect();
        parsed.extend((0..count / 2).map(|i| stable(&middle, "q", i * 2)));
        parsed.extend((0..count / 2).map(|i| {
            let rejected = reject(file(&top, &format!("r{i}")), malformed());
            Contribution::Rejected(None, rejected)
        }));
        parsed
    }

    fn combined(parsed: Vec<Contribution<ParticleEffect>>) -> Registry<ParticleEffect> {
        let options = LoadOptions::default();
        combine(parsed, &StackCx::new(&options, &[]))
    }

    /// `combine` alone on contributions built beforehand: the options and the stack context
    /// are made before the clock starts too, and the registry is looked at after it stops.
    #[test]
    fn combine_takes_100_000_contributed_effects_in_linear_time() {
        let options = LoadOptions::default();
        let stack = StackCx::new(&options, &[]);
        let mut counts = Vec::new();
        assert_linear_consuming_at(
            "combine",
            SINGLE,
            |count| {
                counts.push(count);
                contributions(count)
            },
            |parsed| combine(parsed, &stack),
        );
        // What was timed did the work: checked once more, outside any measurement.
        let registry = combine(contributions(1000), &stack);
        assert_eq!(registry.len(), 1000);
        assert_eq!(registry.rejected().len(), 500);
        assert!(counts.contains(&(TIMES * SINGLE)));
    }

    /// The content of what the timed test combines, at a size a failure can be read at.
    #[test]
    fn combine_keeps_the_effect_loaded_last_and_lists_the_failed_files() {
        let registry = combined(contributions(4));
        let from = |i: usize| {
            let effect = registry.get(&ParticleId::parse(format!("demo:p{i}")));
            effect.map(|effect| effect.origin().pack.name().to_owned())
        };
        assert_eq!(from(0).as_deref(), Some("middle"));
        assert_eq!(from(1).as_deref(), Some("bottom"));
        assert_eq!(from(2).as_deref(), Some("middle"));
        assert_eq!(from(3).as_deref(), Some("bottom"));
        assert_eq!(from(4), None);
        assert_eq!(registry.rejected().len(), 2);
        let shadowed = registry
            .diagnostics()
            .filter(|d| matches!(d.code, DiagnosticCode::ShadowedDefinition { .. }))
            .count();
        assert_eq!(shadowed, 2);
    }
}
