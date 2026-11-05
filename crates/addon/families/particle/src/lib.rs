//! Particles: `ParticleEffect`.
//!
//! Particle effects: `particles/**/*.json` of a resource pack (`particle_effect`).
//! `ParticleEffect` is the newest document (format 1.20.80, which the game reads up to its own
//! version); a file below 1.20.80 is read by the legacy parser's rules and upgraded when it
//! loads, and `history` has that older struct.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod component;
pub mod history;
pub use curve::{BezierChainCurve, BezierChainNode, Curve, SimpleCurve, SimpleCurveType};
pub use event::{
    EventNode, ParticleEffectType, RandomEventNode, SoundEffectEvent, VisualEffectEvent,
};

mod curve;
mod event;
#[cfg(test)]
mod linear;
mod read;
mod texts;
mod write;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, ComponentMap, Contribution, Document, JsonDefinition, JsonDocument, JsonPlaced,
    NoBeta, Notice, Origin, Registry, Rejected, default_path,
};
use bedrock_addon_core::id::{ParticleId, kind};
use bedrock_addon_core::json::{JsonSlice, Prepared, probe_envelope};
use bedrock_addon_core::load::{
    self, CollectionLoader, DocumentSource, LoadContext, StackCx, Walk,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    BelowFloor, ComponentSection, ComponentTable, DocumentRead, Envelope, EnvelopeCheck, Family,
    Identity, InputKind, Loaded, Parsed, Reading, StructIndex, finish, parse_as_written,
};
use component::ParticleComponent;
use history::{Any, ParticleFamily};
use read::CEREAL;
use read::{Opt, object};
use write::PAYLOAD_KEY;

/// A particle effect (`particle_effect`), at the newest format version.
///
/// JSON: `{"format_version": "1.20.80", "particle_effect": {"description", "components",
/// "curves"?, "events"?}}`. The components say how the emitter lives, how many particles it
/// makes, where they appear, and how each particle moves and looks; curves and events are the
/// named parts the components and their Molang refer to.
///
/// Every Molang member is a number, a string (compiled at the Molang version of the pack's
/// `min_engine_version`) or `{"expression", "version"}` (compiled at its own version).
///
/// An effect read from a file below 1.20.80 may be a partial one: where the game's legacy
/// parser stops (no `basic_render_parameters`, no material, no texture, no `components`) the
/// game registers what it had read, and so does this type, with the parser's line as a
/// warning of the load. The value does not say that it is partial: the members never reached
/// are at their defaults (an empty `material` or `texture`, no component, no curve or
/// event), and [`to_json`](bedrock_addon_core::document::Content::to_json) writes the effect whole at
/// those defaults, as a 1.20.80 document that loads again without that warning.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "particle_effect")]
pub struct ParticleEffect {
    /// What the effect is (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`).
    pub components: ComponentMap<ParticleComponent>,
    /// Curves by the Molang variable each sets (JSON `curves`).
    pub curves: Option<IndexMap<String, Curve>>,
    /// Events by name (JSON `events`).
    pub events: Option<IndexMap<String, EventNode>>,
    /// Unknown members of `particle_effect`.
    pub extra: bedrock_addon_core::document::Extra,
}

impl ParticleEffect {
    /// An effect named `identifier` drawn with `material` and `texture`, without components.
    pub fn new(
        identifier: ParticleId,
        material: impl Into<String>,
        texture: impl Into<String>,
    ) -> Self {
        Self {
            description: Description {
                identifier,
                basic_render_parameters: BasicRenderParameters {
                    material: material.into(),
                    texture: texture.into(),
                    extra: bedrock_addon_core::document::Extra::default(),
                },
                extra: bedrock_addon_core::document::Extra::default(),
            },
            components: ComponentMap::new(),
            curves: None,
            events: None,
            extra: bedrock_addon_core::document::Extra::default(),
        }
    }
}

object! {
    /// `particle_effect/description`.
    pub struct Description {
        /// The effect's identifier, as other files name it (JSON `identifier`, required).
        req identifier: ParticleId = "identifier", Opt::NONE;
        /// How the particles are drawn (JSON `basic_render_parameters`, required).
        req basic_render_parameters: BasicRenderParameters = "basic_render_parameters",
            Opt::NONE;
    }
}

object! {
    /// `description/basic_render_parameters`: the material and the texture of every particle
    /// of the effect.
    pub struct BasicRenderParameters finish = read::render_parameters {
        /// The material, a name of the pack stack's material files (JSON `material`,
        /// required).
        req material: String = "material", Opt::NONE;
        /// The texture, a path in the pack without its extension, at most 256 characters
        /// (JSON `texture`, required).
        req texture: String = "texture", Opt::NONE;
    }
}

impl Family for ParticleFamily {
    const NAME: &'static str = "particle";
    const PAYLOAD_KEY: &'static str = PAYLOAD_KEY;
    // The legacy slice is the only one below 1.20.80, and data-driven particles start with
    // it.
    const FLOOR: FormatVersion = known::V1_10_0;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a particle file as a document at every version, taking the version
        // from the first textual `"format_version"`.
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // Below 1.20.80 the game hands the legacy parser the raw text; from 1.20.80 the envelope
        // is checked and has exactly two members.
        check: EnvelopeCheck::From(CEREAL),
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };

    /// The legacy slice `[1.10.0, 1.20.80)` is a hand-written parser, which reads the input
    /// itself (`null` as its readers take it), and its unknown-component line is a content-log
    /// warning that no strict rule acts on. A 1.20.80 document is read by the family's own
    /// reader (`read.rs`), which refuses a `null` where it reads a value, and an unknown member
    /// or component refuses it: the game loads that format strictly.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(0) => Reading::OWN,
            _ => Reading::DOCUMENT,
        }
    }
    type Latest = ParticleEffect;
    // Particles have no experimental-build struct, so there is no second text to read.
    type Beta = NoBeta<kind::Particle>;
    type Component = ParticleComponent;
    type Any = Any;

    fn components() -> &'static ComponentTable<Self> {
        ParticleComponent::TABLE
    }

    fn component_sections(v: &mut ParticleEffect) -> Vec<ComponentSection<'_, ParticleComponent>> {
        let at = JsonPath::new().child("components");
        vec![ComponentSection::new(at, false, &mut v.components)]
    }
}

impl JsonDocument for ParticleEffect {
    type Family = ParticleFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// One particle file. The 1.10.0 struct is the whole document, the upgraded one its
    /// payload: what is found on the upgraded value is below the payload key, as in a 1.20.80
    /// file. The envelope findings are then worded as the particle loader words them.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<ParticleFamily> {
        let Ok(prepared) = Prepared::new(bytes) else {
            return DocumentRead::Rejected(None, reject(origin, malformed()));
        };
        let text = prepared.as_str();
        let document = JsonSlice::new(text);
        let mut parsed = parse_as_written::<ParticleFamily>(document, cx);
        if matches!(parsed.value, Some(Any::V1_10_0(_))) {
            parsed.at.push(PAYLOAD_KEY);
        }
        let identifier = parsed.identity.identifier.clone();
        let outcome = finish(parsed, cx);
        let mut diagnostics = outcome.diagnostics;
        vanilla_envelope_texts(&mut diagnostics, document);
        diagnostics.locate(&origin);
        match outcome.value {
            Some(Loaded::Stable(effect)) => {
                DocumentRead::Stable(Document::new(effect, origin, outcome.version, diagnostics))
            }
            _ => DocumentRead::Rejected(identifier, Rejected::new(origin, diagnostics)),
        }
    }

    /// Parses one particle file without upgrading it: the struct its `format_version`
    /// selects, the envelope findings worded as [`read`](JsonDocument::read) words them.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    fn parse_as_written(
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

impl JsonPlaced for ParticleEffect {
    /// `particles/<identifier>.json`, the identifier's `:` written `_`: a particle
    /// identifier has no namespace of its own, so
    /// `minecraft:basic_flame_particle` is `particles/minecraft_basic_flame_particle.json`.
    fn place(&self) -> PackPath {
        default_path("particles", &self.description.identifier)
    }
}

impl JsonDefinition for ParticleEffect {
    type Kind = kind::Particle;

    fn identifier(&self) -> &ParticleId {
        &self.description.identifier
    }

    /// Per identifier the effect loaded last, each replaced one noted with
    /// `ShadowedDefinition` at the information level (the game replaces silently, so the
    /// finding is the crate's own). A file that failed is skipped ("Failed to load particle
    /// effect"), so an effect below it stays the effective one: a failure is any problem with
    /// the document as a whole in either slice, which is every failure from 1.20.80. Below
    /// 1.20.80 the legacy parser's own lines fail nothing, and the effect it got to replaces
    /// the one below.
    fn combine_rule(_: &StackCx<'_>) -> CombineRule<Self> {
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
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
    let parsed =
        Parsed::<ParticleFamily>::from_value(Some(value), version, written, at, diagnostics)
            .with_identity(identity);
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
/// pipeline says no parser is available, `ENVELOPE.below_floor`.)
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

// ---- the collection ----------------------------------------------------------------------

/// `particles/**/*.json`, packs bottom-up: the effect loaded last is bound to its identifier,
/// so the highest pack's is the effective one. Sub-folders are read in every pack, as for
/// the other loaders that list a folder the same way.
const WALK: Walk = Walk::json_tree_in(&["particles"]);

/// The loader of `rp.particles()`: `particles/**/*.json` of every resource pack, the effect
/// loaded last winning per identifier. A 1.10.0 file the legacy parser stops in after its
/// identifier is no failed file: it loads as far as it was read, and that effect is its
/// contribution.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<ParticleEffect>, Contribution<ParticleEffect>> =
    load::registry::<ParticleEffect>(WALK);

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
        (LOADER.combine)(parsed, &StackCx::new(&options, &[]))
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
            |parsed| (LOADER.combine)(parsed, &stack),
        );
        // What was timed did the work: checked once more, outside any measurement.
        let registry = (LOADER.combine)(contributions(1000), &stack);
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

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn particle_reads_as_pinned() {
        assert_reading_table(
            "ParticleFamily",
            &reading_table::<super::ParticleFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 20, patch: 80 })
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.10.0..=1.10.0 (0..=0): json Own/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
                1.20.80..=1.20.80 (1..=1): json Rejected/Strict/Error/fatal/Refuse, mcb Own/Strict/Error/fatal/Refuse
            ",
        );
    }
}
