//! Fog settings: `fogs/*.json` of a resource pack (`minecraft:fog_settings`). The game has two
//! parsers behind one entry point: a strict schema with three versioned roots below
//! 1.26.20 (`history`), and a cereal loader for exactly 1.26.20, whose shape is the
//! newest struct here.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

pub mod codes;
mod command;
mod gates;
mod jsonutil;
mod read;
mod rules;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FogId;
use bedrock_addon_core::version::known;
use bedrock_addon_shared_shapes::{HexOr, Keyframes};

pub(crate) use jsonutil::{Era, read_document};
pub(crate) use write::{color_ju, float, norm_ju, object_with};

use crate::history::{FogFamily, FogHistory};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath,
};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Document, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Notice,
    Origin, PackTie, Registry, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, PrepareError, Prepared, Value,
    blank_bom, check_depth, last_wins, parse_at, probe_envelope,
};
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    DocumentRead, Envelope, EnvelopeCheck, Family, Loaded, MissingVersion, NoComponents, ParseCx,
    Parsed, RangeModel, StructIndex, VersionChain, finish, parse_as_written,
};

pub use command::{FOG_STACK_LIMIT, FogCommandError, FogStack, is_valid_fog_identifier};
pub use gates::{FOG_FIELD_GATES, FogFieldGate};

/// The `format_version` of the newest fog struct, and the only one it is read at.
pub(crate) const LATEST: bedrock_addon_core::version::FormatVersion = known::V1_26_20;
/// The oldest `format_version` the game reads fog settings at.
pub(crate) const FLOOR: bedrock_addon_core::version::FormatVersion = known::V1_16_100;

/// Fog settings (`minecraft:fog_settings`): how far and in which colour fog is drawn in air,
/// weather and the fluids, and the volumetric fog of the deferred renderer.
///
/// JSON: `{"format_version": "1.26.20", "minecraft:fog_settings": {"description":
/// {"identifier"}, "distance"?: {…}, "volumetric"?: {…}}}`. Only `description.identifier` is
/// required; an absent part is not set by this definition and the game takes it from the next
/// fog on the player's stack. A member the schema does not know is kept in `extra` and
/// reported; the game ignores it. A file from 1.16.100 up to 1.26.20 is upgraded when it loads
/// ([`history`] has the older shapes).
///
/// Written, the settings are a file of 1.26.20, which the game holds to rules the older
/// formats do not have (`fog_start` at most `fog_end`, a density with `uniform` or with
/// both heights, …). Settings that break one of them, as those of an older file may, are
/// written in the older schema instead, at the oldest format version that has their
/// members (1.16.100, 1.21.0 or 1.21.90), where they read back as themselves with the
/// rule's warning. Where the older schema cannot hold them either (a distance entry
/// without one of its four members, an unknown member, a keyframed value, an identifier
/// without a namespace), `to_json` is `NotRepresentable` at the entry that breaks the rule.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:fog_settings")]
#[doc(alias = "fogs")]
pub struct FogSettings {
    /// `description`: the definition's name.
    pub description: FogDescription,
    /// `distance`: the fog of the classic renderer, per medium.
    pub distance: Option<FogDistanceCollection>,
    /// `volumetric`: the volumetric fog of the deferred renderer.
    pub volumetric: Option<FogVolumetricCollection>,
    /// The members the schema does not know, as written.
    pub extra: UnknownMembers,
}

/// `description` of [`FogSettings`].
#[derive(Clone, Debug, PartialEq)]
pub struct FogDescription {
    /// `identifier`: any non-empty text; compared as written, case included. The namespace
    /// `minecraft` (the text before the first `:`, or the whole text, lower-cased) is the base
    /// game's: no other pack may use it.
    pub identifier: FogId,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// `distance`: one setting per medium the camera can be in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogDistanceCollection {
    /// `air`.
    pub air: Option<FogDistanceSetting>,
    /// `weather`: in rain or snow.
    pub weather: Option<FogDistanceSetting>,
    /// `water`; the only medium with a `transition_fog`.
    pub water: Option<FogWaterDistanceSetting>,
    /// `lava`.
    pub lava: Option<FogDistanceSetting>,
    /// `lava_resistance`: in lava with fire resistance.
    pub lava_resistance: Option<FogDistanceSetting>,
    /// `powder_snow`.
    pub powder_snow: Option<FogDistanceSetting>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// One distance setting. Every member is optional; a member a present setting leaves out is 0
/// (`fog_start`, `fog_end`) or `fixed` (`render_distance_type`), which the accessors give.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogDistanceSetting {
    /// `fog_start`: where the fog begins; at most `fog_end`.
    pub fog_start: Option<f32>,
    /// `fog_end`: where the fog is opaque; at least 0.
    pub fog_end: Option<f32>,
    /// `render_distance_type`: what the two distances are measured in.
    pub render_distance_type: Option<DistanceType>,
    /// `fog_color`.
    pub fog_color: Option<FogColor>,
    /// The members the schema does not know (a `transition_fog` outside `water` is one).
    pub extra: UnknownMembers,
}

/// The `water` distance setting: the members of [`FogDistanceSetting`] and `transition_fog`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogWaterDistanceSetting {
    /// `fog_start`: where the fog begins; at most `fog_end`.
    pub fog_start: Option<f32>,
    /// `fog_end`: where the fog is opaque; at least 0.
    pub fog_end: Option<f32>,
    /// `render_distance_type`: what the two distances are measured in.
    pub render_distance_type: Option<DistanceType>,
    /// `fog_color`.
    pub fog_color: Option<FogColor>,
    /// `transition_fog`: how the fog changes while the camera stays under water.
    pub transition_fog: Option<FogTransitionSetting>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// `transition_fog` of the water setting: from `init_fog` towards the setting itself over time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogTransitionSetting {
    /// `init_fog`: the fog on entering the water.
    pub init_fog: Option<FogDistanceSetting>,
    /// `min_percent`: 0 to 1.
    pub min_percent: Option<f32>,
    /// `mid_seconds`: at least 0, and below `max_seconds` unless both are 0.
    pub mid_seconds: Option<f32>,
    /// `mid_percent`: 0 to 1.
    pub mid_percent: Option<f32>,
    /// `max_seconds`: at least 0.
    pub max_seconds: Option<f32>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// `volumetric`: the fog of the deferred renderer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricCollection {
    /// `density`: how dense the fog is, per medium.
    pub density: Option<FogVolumetricDensityCollection>,
    /// `media_coefficients`: how each medium scatters and absorbs light.
    pub media_coefficients: Option<FogVolumetricMediaCoeffsCollection>,
    /// `henyey_greenstein_g`: the scattering direction, per medium.
    pub henyey_greenstein_g: Option<FogVolumetricHenyeyGreensteinGCollection>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// `volumetric.density`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricDensityCollection {
    /// `air`.
    pub air: Option<FogVolumetricDensitySetting>,
    /// `weather`.
    pub weather: Option<FogVolumetricDensitySetting>,
    /// `water`.
    pub water: Option<FogVolumetricDensitySetting>,
    /// `lava`.
    pub lava: Option<FogVolumetricDensitySetting>,
    /// `lava_resistance`.
    pub lava_resistance: Option<FogVolumetricDensitySetting>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// One density setting. Either `uniform` is `true` and neither height is written, or both
/// heights are written with `zero_density_height` at or above `max_density_height`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricDensitySetting {
    /// `max_density`: 0 to 1.
    pub max_density: Option<Keyframed<f32>>,
    /// `uniform`: the same density at every height.
    pub uniform: Option<bool>,
    /// `zero_density_height`: the height at which the fog ends, −64 to 320.
    pub zero_density_height: Option<Keyframed<f32>>,
    /// `max_density_height`: the height below which the fog is at `max_density`, −64 to 320.
    pub max_density_height: Option<Keyframed<f32>>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// `volumetric.media_coefficients`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricMediaCoeffsCollection {
    /// `air`.
    pub air: Option<FogVolumetricMediaCoeffsSetting>,
    /// `water`.
    pub water: Option<FogVolumetricMediaCoeffsSetting>,
    /// `cloud`.
    pub cloud: Option<FogVolumetricMediaCoeffsSetting>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// The coefficients of one medium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricMediaCoeffsSetting {
    /// `absorption`: how much of each colour the medium absorbs.
    pub absorption: Option<Keyframed<ColorNorm>>,
    /// `scattering`: how much of each colour the medium scatters.
    pub scattering: Option<Keyframed<ColorNorm>>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// `volumetric.henyey_greenstein_g`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricHenyeyGreensteinGCollection {
    /// `air`.
    pub air: Option<FogVolumetricHenyeyGreensteinGSetting>,
    /// `water`.
    pub water: Option<FogVolumetricHenyeyGreensteinGSetting>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// The scattering direction of one medium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricHenyeyGreensteinGSetting {
    /// `henyey_greenstein_g`: −1 (backwards) to 1 (forwards).
    pub henyey_greenstein_g: Option<Keyframed<f32>>,
    /// The members the schema does not know.
    pub extra: UnknownMembers,
}

/// What `fog_start` and `fog_end` are measured in (`render_distance_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DistanceType {
    /// `"fixed"`: blocks. The value of a setting that leaves the member out.
    #[default]
    Fixed,
    /// `"render"`: a fraction of the render distance (−1 to 1 for the start, up to 1 for the
    /// end; the game warns outside that).
    Render,
}

/// A fog colour (`fog_color`): `"#RRGGBB"` (as written, hex digits of either case) or
/// `[r, g, b]` with integers 0 to 255 (an integer outside is clamped when read, as the game
/// clamps it). [`HexOr::unit_rgb`] gives the colour as the game uses it.
pub type FogColor = HexOr<[u8; 3]>;

/// A coefficient colour (`absorption`, `scattering`): `"#RRGGBB"` or `[r, g, b]` with numbers
/// as written, which the game clamps to 0 to 1 ([`HexOr::unit_rgb`] gives the clamped components).
pub type ColorNorm = HexOr<[f32; 3]>;

/// A value that may change over the day: one value, or values by time key
/// (`{"0.0": 0.02, "0.5": 0.1}`; shapes' [`Keyframes`]). The keyed form is read by engines
/// from 1.26.30. Keys are sorted by time; two keys of a file that are one time (`"0"` and
/// `"0.0"`, or `"-0"` and `"0"`) are one keyframe, the later one, with its time as written. A
/// list built in memory that holds one time twice is not written: `to_json` refuses it with
/// `NotRepresentable` at the second keyframe of that time.
pub type Keyframed<T> = Keyframes<T>;

/// A keyframe time with `-0` as `0`, so that two finite times are one time exactly when
/// their bits are equal.
pub(crate) fn same_zero(time: f32) -> f32 {
    time + 0.0
}

impl Family for FogFamily {
    const NAME: &'static str = "fog";
    const PAYLOAD_KEY: &'static str = "minecraft:fog_settings";
    const FLOOR: FormatVersion = FLOOR;
    // The cereal parser is built with the 2-argument `CerealParserBase(ctx, VERSION)`: exactly
    // 1.26.20, whatever the engine.
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = FogSettings;
    type Beta = NoBeta<kind::Fog>;
    type Component = NoComponents;
    type History = FogHistory;
    const ENVELOPE: Envelope = Envelope {
        // Below 1.26.20 the strict schema root reads the whole document (a third root member is an
        // unknown member of it); from 1.26.20 `MinecraftDocumentInput` wants exactly two members.
        // A pre-release of 1.26.20 sorts below it.
        check: EnvelopeCheck::From(LATEST),
        missing: MissingVersion::Reject {
            code: &codes::INVALID_VERSION,
        },
        ..Envelope::DEFAULT
    };
}

/// The payload's path in the file.
fn payload_at() -> JsonPath {
    JsonPath::new().child(FogFamily::PAYLOAD_KEY)
}

/// An error of the gates, at the document root.
fn gate(finding: Diagnostic) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    diagnostics
}

/// The reader's messages in the game's line, and again as the finding's syntax detail.
fn malformed(message: String) -> Diagnostics {
    gate(
        Diagnostic::of(codes::PARSE_ERROR.with([message.clone()]))
            .detail(DiagnosticDetail::Syntax(message)),
    )
}

fn empty(code: &'static CodeDef<0>) -> Diagnostics {
    gate(Diagnostic::of(code))
}

/// `parseFogDefinitionIdentifier`: `minecraft:fog_settings.description.identifier` of the raw
/// JSON is a non-empty string.
fn has_identifier(root: &Value) -> bool {
    let identifier = root
        .as_object()
        .and_then(|root| root.get(FogFamily::PAYLOAD_KEY)?.as_object())
        .and_then(|settings| settings.get("description")?.as_object())
        .and_then(|description| description.get("identifier")?.as_string());
    identifier.is_some_and(|text| !text.as_str().is_empty())
}

/// Steps 1 to 7 of `registerFogFromJson` on a file's bytes: the text with duplicate members
/// settled, or the findings of the gate that skipped the file.
fn gates(bytes: &[u8]) -> Result<String, Diagnostics> {
    let too_deep = || malformed("nesting too deep".to_owned());
    // The nesting is checked before the encoding (`Prepared` checks the encoding first): a
    // text both too deep and not UTF-8 is told as too deep. The mark is blanked first, or
    // the scan stops at it.
    let bytes = &*blank_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| too_deep())?;
    let prepared = Prepared::non_empty(bytes).map_err(|error| match error {
        PrepareError::Empty => empty(&codes::EMPTY_RESOURCE),
        PrepareError::TooDeep(_) => too_deep(),
        PrepareError::NotUtf8(error) => malformed(error.to_string()),
    })?;
    let text = prepared.as_str();
    // Of a member written twice the last value stands, before anything looks at the text.
    let cleaned = last_wins(text).into_owned();
    let root =
        parse_at::<Value>(JsonSlice::new(&cleaned)).map_err(|e| malformed(e.error.to_string()))?;
    // `Json::Value::empty()`: null, or an array or object without members.
    let is_empty = root.is_null()
        || root.as_object().is_some_and(facet_value::VObject::is_empty)
        || root.as_array().is_some_and(facet_value::VArray::is_empty);
    if is_empty {
        return Err(empty(&codes::EMPTY_AFTER_PARSE));
    }
    if !has_identifier(&root) {
        return Err(gate(Diagnostic::of(&codes::NO_IDENTIFIER)));
    }
    let invalid = || gate(Diagnostic::of(&codes::INVALID_VERSION));
    let envelope = probe_envelope(JsonSlice::new(&cleaned)).map_err(|_| invalid())?;
    match &envelope.format_version {
        EnvelopeVersion::String(written) => {
            let version = FormatVersionInput::parse(written).ok_or_else(invalid)?;
            // The literal "beta" goes on to the chain, which has no beta struct.
            let below = !version.beta_literal
                && version
                    .version
                    .is_none_or(|v| v < FLOOR || (version.prerelease && v == FLOOR));
            if below {
                return Err(gate(Diagnostic::of(&codes::UNSUPPORTED_VERSION)));
            }
        }
        // V5a (ours): the dispatcher takes a three-element array, and then either parser
        // refuses it; it is refused here.
        EnvelopeVersion::Array(_) => {
            return Err(gate(Diagnostic::of(&core_codes::MALFORMED_FORMAT_VERSION)));
        }
        _ => return Err(invalid()),
    }
    Ok(cleaned)
}

/// The closing line of a document either parser refused.
fn failed_to_validate(diagnostics: &mut Diagnostics) {
    diagnostics.push(Diagnostic::of(&codes::FAILED_TO_VALIDATE));
}

/// The upgrade walk, then what the game checks on the definition: the keyframe gate and, for a
/// document read at 1.26.20, the cereal parser's validation rules, either failing with the
/// closing line; then the namespace rule.
fn finish_parsed(
    parsed: Parsed<FogFamily>,
    cx: &LoadContext,
) -> (Option<FogSettings>, VersionInfo, Diagnostics) {
    let outcome = finish(parsed, cx);
    let version = outcome.version;
    let mut diagnostics = outcome.diagnostics;
    let Some(Loaded::Stable(settings)) = outcome.value else {
        // The literal "beta" never reached a parser: "Invalid 'format_version' …" in the game.
        if !version.beta {
            failed_to_validate(&mut diagnostics);
        }
        return (None, version, diagnostics);
    };
    let at = payload_at();
    rules::keyframe_gate(&settings, cx.engine_version, &at, &mut diagnostics);
    if version.parsed_as == LATEST {
        rules::format_version(
            &version.effective.to_string(),
            &JsonPath::new(),
            &mut diagnostics,
        );
    }
    // V15–V20 are every era's: the 1.26.20 validation lambda's, and the strict schema's
    // typed nodes' own (the same nodes in all three older schemas). There V15 and V17–V20 are
    // level 3 and fail the validation, so the file is refused through "Failed to validate fog
    // schema" below, and V16–V16b are level 2.
    rules::cereal_rules(&settings, &at, &mut diagnostics);
    if diagnostics.has_errors() {
        failed_to_validate(&mut diagnostics);
        return (None, version, diagnostics);
    }
    rules::namespace(
        settings.description.identifier.as_str(),
        cx.is_base_game_pack,
        &at,
        &mut diagnostics,
    );
    let settings = (!diagnostics.has_errors()).then_some(settings);
    (settings, version, diagnostics)
}

/// The struct of the chain that reads the 1.21.90 root.
const V1_21_90: usize = 2;

/// The parse stage of a text the gates let through: the chain's, except for a pre-release of
/// 1.26.20. The game's dispatcher compares with a plain `<`, so `1.26.20-beta.1` is below
/// 1.26.20 and read by the 1.21.90 root; the chain's range model, built for the game's other loaders,
/// covers no pre-release of its newest struct, so that one case is routed here.
fn as_written(text: &str, cx: &LoadContext) -> Parsed<FogFamily> {
    let document = JsonSlice::new(text);
    let written = match probe_envelope(document).map(|envelope| envelope.format_version) {
        Ok(EnvelopeVersion::String(written)) => FormatVersionInput::parse(&written)
            .filter(|input| input.prerelease && input.version == Some(LATEST))
            .map(|input| (written, input)),
        _ => None,
    };
    let Some((written, input)) = written else {
        return parse_as_written::<FogFamily>(document, cx);
    };
    let mut diagnostics = Diagnostics::default();
    let at = JsonPath::new().child("format_version");
    diagnostics.push(Diagnostic::of(core_codes::PRERELEASE_FORMAT_VERSION.with([written])).at(at));
    let versions = <FogHistory as VersionChain<FogFamily>>::VERSIONS;
    let parsed_as = versions.get(V1_21_90).copied().unwrap_or(FLOOR);
    let parse = ParseCx {
        load: cx,
        written: input,
        original_version: LATEST,
        at: JsonPath::new(),
    };
    let value = <FogHistory as VersionChain<FogFamily>>::parse_struct(
        document,
        StructIndex::Stable(V1_21_90),
        &parse,
        &mut diagnostics,
    )
    .filter(|_| !diagnostics.has_errors());
    Parsed::from_value(
        value,
        VersionInfo::new(Some(LATEST), false, LATEST, parsed_as, false),
        input,
        JsonPath::new(),
        diagnostics,
    )
}

impl JsonDocument for FogSettings {
    type Family = FogFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
    const NAME: &'static str = "fog settings";

    /// One fog file: the gates `registerFogFromJson` runs before either parser, the parse
    /// stage (`as_written`), the upgrade and the rules after it (`finish_parsed`).
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<FogFamily> {
        let text = match gates(bytes) {
            Ok(text) => text,
            Err(diagnostics) => {
                return DocumentRead::Rejected(None, Rejected::located(origin, diagnostics));
            }
        };
        let parsed = as_written(&text, cx);
        let identifier = parsed.identity.identifier.clone();
        let (settings, version, diagnostics) = finish_parsed(parsed, cx);
        let diagnostics = diagnostics.located(&origin);
        match settings {
            Some(settings) => {
                DocumentRead::Stable(Document::new(settings, origin, version, diagnostics))
            }
            None => DocumentRead::Rejected(identifier, Rejected::new(origin, diagnostics)),
        }
    }

    /// Parses one fog file without upgrading it: the struct its `format_version` selects,
    /// after the same gates as [`read`](JsonDocument::read);
    /// [`history::upgrade`] finishes it.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<FogHistory>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
        let text = match gates(bytes) {
            Ok(text) => text,
            Err(diagnostics) => {
                return Err(Rejected::located(origin, diagnostics));
            }
        };
        let parsed = as_written(&text, cx);
        let mut diagnostics = parsed.diagnostics;
        if let Some(any) = parsed.value {
            return Ok(Document::new(
                any,
                origin.clone(),
                parsed.version,
                diagnostics.located(&origin),
            ));
        }
        if !parsed.version.beta {
            failed_to_validate(&mut diagnostics);
        }
        Err(Rejected::located(origin, diagnostics))
    }

    /// The document is written at 1.26.20, where the game holds it to the rules V15–V20; so
    /// do the older schemas, so a value that breaks one has no text the game or this loader
    /// reads anywhere, and is refused with the path of the entry. So is a keyed member a
    /// caller built with one time twice (`-0` and `0` are one time), at the second keyframe of
    /// that time: the reader keeps one keyframe per time.
    fn refuse_write(&self) -> Result<(), EncodeError> {
        self.refuse_write_rules()
    }
}

impl JsonPlaced for FogSettings {
    /// `fogs/<identifier>.json`, the identifier's `:` written `_`
    /// (`fogs/minecraft_fog_bamboo_jungle.json`).
    fn place(&self) -> PackPath {
        default_path("fogs", &self.description.identifier)
    }
}

impl JsonDefinition for FogSettings {
    type Kind = kind::Fog;

    fn identifier(&self) -> &FogId {
        &self.description.identifier
    }

    /// Per identifier the highest pack's definition; each lower one is noted with
    /// `ShadowedDefinition` at info (ours; the game skips it silently). Inside one pack the
    /// first file to register an identifier keeps it too (the walk gives a pack's files in
    /// path order): a later one is noted with the same info and keeps the findings of its
    /// parse, which the game ran before it looked the identifier up. A file that failed
    /// registers nothing, so a lower pack's definition of its identifier still does.
    fn combine_rule(_: &StackCx<'_>) -> CombineRule<Self> {
        CombineRule::TopWins {
            shadowed: Some(Notice::shadowed()),
        }
        .within_pack(PackTie::first(Some(Notice::shadowed())))
    }
}

/// `history::upgrade`: the remaining steps and the rules on a document
/// [`FogSettings::parse_as_written`] gave.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
#[doc(hidden)]
pub fn upgrade_document(
    document: Document<FogHistory>,
    cx: &LoadContext,
) -> Result<Document<FogSettings>, Rejected> {
    let (value, origin, version, diagnostics) = document.into_parsed_parts();
    let at = if version.parsed_as == LATEST {
        payload_at()
    } else {
        JsonPath::new()
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<FogFamily>::from_value(Some(value), version, written, at, diagnostics);
    let (settings, version, diagnostics) = finish_parsed(parsed, cx);
    // The findings the document carried are located already; the new ones are not.
    let diagnostics = diagnostics.located(&origin);
    match settings {
        Some(settings) => Ok(Document::new(settings, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------------

/// `fogs/*.json`, packs top-down: the game walks its stack from index 0 and the first file to
/// register an identifier keeps it.
// The game collects them with `getResourcesOfGroup(pack, "fogs")`.
const WALK: Walk = Walk::json_children_in(&["fogs"]).top_down();

/// The loader of `rp.fogs()`: `fogs/*.json` of every resource pack, the highest pack's
/// definition winning per identifier.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<FogSettings>, Contribution<FogSettings>> =
    load::registry::<FogSettings>(WALK);

//
// A present distance entry starts from `FogSetting::ZERO_SETTING` and takes each written member.

impl FogSettings {
    /// Settings named `identifier` that set nothing.
    pub fn new(identifier: FogId) -> Self {
        Self {
            description: FogDescription::new(identifier),
            distance: None,
            volumetric: None,
            extra: UnknownMembers::default(),
        }
    }

    /// The identifier's namespace as the game takes it: the lower-cased text before the first
    /// `:`, or the whole lower-cased identifier when it has none.
    pub fn namespace(&self) -> String {
        let identifier = self.description.identifier.as_str();
        identifier
            .split(':')
            .next()
            .unwrap_or(identifier)
            .to_ascii_lowercase()
    }
}

impl FogDescription {
    /// A description with `identifier`.
    pub fn new(identifier: FogId) -> Self {
        Self {
            identifier,
            extra: UnknownMembers::default(),
        }
    }
}

impl DistanceType {
    /// The JSON spelling: `"fixed"` or `"render"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Render => "render",
        }
    }
}

impl FogDistanceSetting {
    /// `fog_start`; 0 when the setting leaves it out.
    pub fn fog_start(&self) -> f32 {
        self.fog_start.unwrap_or(0.0)
    }

    /// `fog_end`; 0 when the setting leaves it out.
    pub fn fog_end(&self) -> f32 {
        self.fog_end.unwrap_or(0.0)
    }

    /// `render_distance_type`; `fixed` when the setting leaves it out.
    pub fn render_distance_type(&self) -> DistanceType {
        self.render_distance_type.unwrap_or_default()
    }
}

impl FogWaterDistanceSetting {
    /// `fog_start`; 0 when the setting leaves it out.
    pub fn fog_start(&self) -> f32 {
        self.fog_start.unwrap_or(0.0)
    }

    /// `fog_end`; 0 when the setting leaves it out.
    pub fn fog_end(&self) -> f32 {
        self.fog_end.unwrap_or(0.0)
    }

    /// `render_distance_type`; `fixed` when the setting leaves it out.
    pub fn render_distance_type(&self) -> DistanceType {
        self.render_distance_type.unwrap_or_default()
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn fog_reads_as_pinned() {
        assert_reading_table(
            "FogFamily",
            &reading_table::<super::FogFamily>(),
            r#"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 26, patch: 20 })
                missing version: rejected: Fixed("Invalid 'format_version' in fog settings JSON")
                below floor: UseFormatVersion
                empty document: Error
                1.16.100..=1.26.20 (0..=3): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            "#,
        );
    }
}
