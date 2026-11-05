//! `biomes_client.json` (`LegacyClientBiomes`): the legacy per-pack file of water and fog
//! settings by biome. The game merges every pack's copy over the resource stack with a merge
//! strategy of its own (reproduced here) and reads the entries from the merged JSON. The file
//! has no version, is merged over the whole stack, and still overrides the water and fog a
//! client biome sets.
//!
//! The reader stays a hand walk over the merged JSON, as the game reads it after its own merge
//! strategy: no schema stands behind it.

#![cfg_attr(docsrs, feature(doc_cfg))]

use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, RawLayer, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{BiomeId, FogId};
use bedrock_addon_core::json::{
    EncodeError, PrepareError, Prepared, Value, WriteStyle, last_wins, write_json,
};
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::SemVersion;

use bedrock_addon_shared_shapes::color::is_hex;
use bedrock_addon_support::biome_json::child;

/// The file, at the pack root.
const FILE: &str = "biomes_client.json";
/// The member that holds the entries.
const BIOMES: &str = "biomes";
/// The entry every biome without its own takes its water from.
const DEFAULT: &str = "default";
/// From this world base-game version on, the base-game packs' files are not read.
const STOP_LOADING_VANILLA: SemVersion = SemVersion::new(1, 21, 40);
/// Every member the game reads of an entry, in the merge or from the merged value (it looks
/// at no other key).
const MEMBERS: [&str; 10] = [
    "water_surface_color",
    "water_surface_transparency",
    "fog_identifier",
    "inherit_from_prior_fog",
    "remove_all_prior_fog",
    "fog_ids_to_merge",
    "make_back_compat_fog",
    "fog_color",
    "water_fog_color",
    "water_fog_distance",
];
/// The old-style fog members the merge keeps out of an entry that names a fog.
const OLD_FOG: [&str; 3] = ["fog_color", "water_fog_color", "water_fog_distance"];

/// The legacy client-biome settings of a stack: `biomes_client.json` of every pack that takes
/// part, merged lowest pack first.
///
/// From world base-game version 1.21.40 (and for `*`) the base-game packs' files take no part.
/// The game applies these settings after the client-biome documents, so they override the
/// water surface and the fog a client biome sets.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "biomes_client.json")]
#[non_exhaustive]
pub struct LegacyClientBiomes {
    /// The `default` entry: its water surface applies to every biome.
    pub default: Option<LegacyClientBiome>,
    /// The entries by biome. A name written without a namespace is `minecraft:<name>`.
    pub biomes: IndexMap<BiomeId, LegacyClientBiome>,
    /// The merged JSON the entries were read from (`{"biomes": {…}}`), with the members only
    /// the merge reads (`fog_identifier`, `inherit_from_prior_fog`, …).
    pub merged: Value,
}

impl LegacyClientBiomes {
    /// The entry of `biome`: under its full name, else, for a `minecraft` biome, under its
    /// name without the namespace.
    pub fn get(&self, biome: &BiomeId) -> Option<&LegacyClientBiome> {
        self.biomes.get(biome).or_else(|| {
            let (namespace, name) =
                bedrock_addon_core::id::syntax::split_namespace(biome.as_str())?;
            (namespace == "minecraft")
                .then(|| self.biomes.get(&BiomeId::parse(name)))
                .flatten()
        })
    }

    #[must_use]
    #[allow(
        clippy::float_cmp,
        reason = "an override is a value that differs at all from the one it replaces"
    )]
    /// The water surface of `biome` after both documents are applied: the client biome's
    /// `minecraft:water_appearance` (`client_color` as RGBA channels from 0 to 1 and
    /// `client_opacity`, `None` for a member it leaves out or a biome without one), then the
    /// legacy `default` entry (for every biome), then the biome's own legacy entry.
    pub fn effective_water_surface(
        &self,
        client_color: Option<[f32; 4]>,
        client_opacity: Option<f32>,
        biome: &BiomeId,
    ) -> WaterSurface {
        let mut surface = WaterSurface {
            color: client_color,
            opacity: client_opacity,
            ..WaterSurface::default()
        };
        for entry in [self.default.as_ref(), self.get(biome)]
            .into_iter()
            .flatten()
        {
            if let Some(color) = entry.water_surface_color {
                surface.overridden |= surface.color.is_some_and(|set| set != color);
                surface.color = Some(color);
            }
            if let Some(opacity) = entry.water_surface_transparency {
                surface.overridden |= surface.opacity.is_some_and(|set| set != opacity);
                surface.opacity = Some(opacity);
            }
        }
        surface
    }
}

/// One entry of `biomes_client.json` as the game reads it from the merged file (a
/// biome's entry and the `default` entry share this shape).
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LegacyClientBiome {
    /// The old-style fog, present when the merge set `make_back_compat_fog`.
    pub old_fog: Option<OldFog>,
    /// The fog definitions merged into the biome's fog (JSON `fog_ids_to_merge`, normally
    /// built by the merge from the packs' `fog_identifier`s).
    pub fog_ids_to_merge: Vec<FogId>,
    /// JSON `water_surface_color`, as RGBA from 0 to 1.
    pub water_surface_color: Option<[f32; 4]>,
    /// JSON `water_surface_transparency`, clamped to `[0, 1]`.
    pub water_surface_transparency: Option<f32>,
}

/// The fog of an entry written before fog definitions existed.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct OldFog {
    /// JSON `fog_color`, as RGBA with alpha 1.
    pub fog_color: Option<[f32; 4]>,
    /// JSON `water_fog_color`, as RGBA with alpha 1.
    pub water_fog_color: Option<[f32; 4]>,
    /// JSON `water_fog_distance`.
    pub water_fog_distance: Option<f32>,
    /// JSON `fog_identifier`, with `minecraft:` when it has no namespace.
    pub fog_identifier: Option<FogId>,
}

/// The water surface the client ends up with for one biome.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct WaterSurface {
    /// The surface colour, as RGBA from 0 to 1; `None` leaves the game's.
    pub color: Option<[f32; 4]>,
    /// The surface opacity, from 0 to 1; `None` leaves the game's.
    pub opacity: Option<f32>,
    /// `biomes_client.json` replaced a value the client biome's `minecraft:water_appearance`
    /// set.
    pub overridden: bool,
}

/// A channel from 0 to 1.
fn unit(channel: u8) -> f32 {
    f32::from(channel) / 255.0
}

impl Sealed for LegacyClientBiomes {}

impl Merge for LegacyClientBiomes {
    /// The entry's key: a biome, or `default`.
    type Key = BiomeId;
}

impl Content for LegacyClientBiomes {
    const NAME: &'static str = "legacy client biomes";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `biomes_client.json` on its own: the file as the merge leaves it when no
    /// other pack has one. A text that is not JSON is rejected.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
            let layer = match read_copy(origin.clone(), bytes) {
                BiomesCopy::Layer(layer, _) => layer,
                BiomesCopy::Failed(diagnostics) | BiomesCopy::Skipped(diagnostics) => {
                    return Err(Rejected::new(origin, diagnostics));
                }
            };
            let merged = merge(vec![BiomesCopy::Layer(layer, Diagnostics::default())]);
            let mut diagnostics = Diagnostics::default();
            diagnostics.extend(merged.diagnostics().cloned());
            let version = VersionInfo::new(
                None,
                false,
                bedrock_addon_core::version::ENGINE_VERSION,
                bedrock_addon_core::version::ENGINE_VERSION,
                false,
            );
            Ok(Document::new(
                merged.value().clone(),
                origin,
                version,
                diagnostics,
            ))
        })
    }

    /// Writes the merged JSON.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.merged, out, style)
    }
}

impl JsonContent for LegacyClientBiomes {}

impl Placed for LegacyClientBiomes {
    /// `biomes_client.json`, at the pack root.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(FILE)
    }
}

// ---- one pack's copy -----------------------------------------------------------------------

/// What one pack's file gave the merge.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum BiomesCopy {
    /// The file's JSON, and what reading it found: the warnings of the numbers no double
    /// holds, each read as the largest finite one.
    Layer(RawLayer, Diagnostics),
    /// A file that takes no part, and why (a base-game pack's in a world from 1.21.40).
    Skipped(Diagnostics),
    /// A file that is not JSON: skipped, with an information of the crate's own.
    Failed(Diagnostics),
}

/// One finding as a list.
fn only(diagnostic: Diagnostic) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(diagnostic);
    diagnostics
}

/// Reads one file as JSON (comments allowed, the last of a doubled member standing).
// The game parses the text without logging, and the merge skips a file that does not parse
// silently; the crate says so with an information.
fn read_copy(origin: Origin, bytes: &[u8]) -> BiomesCopy {
    let malformed = |message: String| {
        BiomesCopy::Failed(only(
            Diagnostic::info(DiagnosticCode::MalformedInput {
                message,
                text: None,
            })
            .in_file(&origin),
        ))
    };
    let prepared = match Prepared::new(bytes) {
        Ok(prepared) => prepared,
        Err(PrepareError::NotUtf8(_)) => return malformed("the file is not UTF-8".to_owned()),
        // `new` never refuses empty input.
        Err(PrepareError::TooDeep(_) | PrepareError::Empty) => {
            return malformed("the JSON nests too deeply".to_owned());
        }
    };
    let text = prepared.as_str();
    match bedrock_addon_core::json::from_str_jsonc::<Value>(&last_wins(text)) {
        Ok(json) if json.is_object() => {
            BiomesCopy::Layer(RawLayer::new(origin, json), Diagnostics::default())
        }
        Ok(_) => BiomesCopy::Failed(only(
            Diagnostic::info(DiagnosticCode::InvalidType { text: None }).in_file(&origin),
        )),
        Err(error) => malformed(error.kind.to_string()),
    }
}

/// One pack's file. A base-game pack's file takes no part when the world's base-game version
/// includes 1.21.40 (`*` and an absent version do; a never-compatible one does not).
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> BiomesCopy {
    let origin = Origin::file(file.pack.clone(), file.side, file.path.clone());
    if cx.is_base_game_pack && cx.world_base_game_version.includes(&STOP_LOADING_VANILLA) {
        let skipped = Diagnostic::info(DiagnosticCode::VanillaBiomesClientSkipped).in_file(&origin);
        return BiomesCopy::Skipped(only(skipped));
    }
    // The copy is read with the numbers no double holds clamped, so neither its layer
    // nor the merged document keeps an infinity; the warning names this pack's file.
    let finite = bedrock_addon_core::document::FiniteText::of(&file.bytes);
    let mut copy = read_copy(origin.clone(), finite.bytes());
    match &mut copy {
        BiomesCopy::Layer(_, found) | BiomesCopy::Skipped(found) | BiomesCopy::Failed(found) => {
            finite.settle(found, Some(&origin));
        }
    }
    copy
}

// ---- the merge -----------------------------------------------------------------------------

/// The member `key` as a boolean with a default of `false`: absent and `null` are `false`; a
/// value that is no boolean reads `false`, with an information of the crate's own.
fn flag(object: &VObject, key: &str, at: &JsonPath, notes: &mut Notes<'_>) -> bool {
    match object.get(key) {
        None => false,
        Some(value) if value.is_null() => false,
        Some(value) => value.as_bool().unwrap_or_else(|| {
            notes.warning(
                DiagnosticCode::InvalidType { text: None },
                child(at, &[key]),
            );
            false
        }),
    }
}

/// Where the merge of one file reports.
struct Notes<'a> {
    origin: &'a Origin,
    diagnostics: &'a mut Diagnostics,
}

impl Notes<'_> {
    /// A note of the crate's own: the game's merge is silent here, so the finding is an
    /// information.
    fn warning(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.diagnostics
            .push(Diagnostic::info(code).in_file(self.origin).at(at));
    }

    fn info(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.diagnostics
            .push(Diagnostic::info(code).in_file(self.origin).at(at));
    }
}

/// The transform that runs before the merge: every key of `biomes` without a
/// namespace, other than `default`, becomes `minecraft:<key>` unless its value is `null`; the
/// bare key goes. A file with both spellings keeps the bare one's value.
// jsoncpp walks keys sorted, so the bare key is copied last.
fn namespace_keys(json: &mut Value) {
    let Some(biomes) = json
        .as_object_mut()
        .and_then(|root| root.get_mut(BIOMES))
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    let bare: Vec<String> = biomes
        .keys()
        .map(|key| key.as_str().to_owned())
        .filter(|key| key != DEFAULT && !key.contains(':'))
        .collect();
    for key in bare {
        if let Some(value) = biomes.remove(&key)
            && !value.is_null()
        {
            biomes.insert(format!("minecraft:{key}"), value);
        }
    }
}

/// The merge predicate, called on every pair of objects the merge meets: `true` when it merged
/// the level itself.
fn predicate(dst: &mut VObject, src: &VObject, at: &JsonPath, notes: &mut Notes<'_>) -> bool {
    // 1. A file whose `default` says `remove_all_prior_fog` discards everything merged so far.
    if let Some(default) = src.get(DEFAULT).and_then(Value::as_object)
        && flag(
            default,
            "remove_all_prior_fog",
            &child(at, &[DEFAULT]),
            notes,
        )
    {
        dst.clear();
    }
    let has = |key: &str| src.contains_key(key);
    // 2. No fog member: a plain merge.
    if !(OLD_FOG.iter().any(|key| has(key)) || has("fog_identifier")) {
        return false;
    }
    // 3. Old-style fog members without a fog name: the entry becomes a back-compat fog.
    if !has("fog_identifier") {
        dst.insert("fog_ids_to_merge", VArray::new());
        dst.insert("make_back_compat_fog", true);
        return false;
    }
    let fog_identifier = src.get("fog_identifier").cloned().unwrap_or(Value::NULL);
    if flag(src, "inherit_from_prior_fog", at, notes) {
        // 4. The fog name joins the ones below it.
        for (key, value) in src {
            match key.as_str() {
                "fog_identifier" => {
                    if !dst.get("fog_ids_to_merge").is_some_and(Value::is_array) {
                        dst.insert("fog_ids_to_merge", VArray::new());
                    }
                    if let Some(ids) = dst
                        .get_mut("fog_ids_to_merge")
                        .and_then(Value::as_array_mut)
                    {
                        ids.push(value.clone());
                    }
                }
                key if OLD_FOG.contains(&key) => {}
                key => {
                    dst.insert(key, value.clone());
                }
            }
        }
    } else {
        // 5. The fog name replaces the ones below it, and any old-style fog.
        let mut ids = VArray::new();
        ids.push(fog_identifier);
        dst.insert("fog_ids_to_merge", ids);
        dst.remove("make_back_compat_fog");
        for key in OLD_FOG {
            dst.remove(key);
        }
        for (key, value) in src {
            if !OLD_FOG.contains(&key.as_str()) {
                dst.insert(key.as_str(), value.clone());
            }
        }
    }
    true
}

/// The recursive merge: the predicate first; then each member of `src`
/// replaces `dst`'s, unless both are objects, which merge the same way.
fn recursive_merge(dst: &mut VObject, src: &VObject, at: &JsonPath, notes: &mut Notes<'_>) {
    if predicate(dst, src, at, notes) {
        return;
    }
    for (key, value) in src {
        let key = key.as_str();
        match (
            dst.get_mut(key).and_then(Value::as_object_mut),
            value.as_object(),
        ) {
            (Some(below), Some(above)) => recursive_merge(below, above, &child(at, &[key]), notes),
            _ => {
                dst.insert(key, value.clone());
            }
        }
    }
}

/// What a file's entries say that the merge will not act on, judged
/// against what the lower files merged to.
fn check_copy(file: &VObject, below: &VObject, notes: &mut Notes<'_>) {
    for (key, _) in file {
        let at = child(&JsonPath::new(), &[key.as_str()]);
        match key.as_str() {
            BIOMES => {}
            // The file has no version: one written here is read by nothing.
            "format_version" => notes.info(DiagnosticCode::UnrecognizedField { text: None }, at),
            _ => notes.warning(DiagnosticCode::UnrecognizedField { text: None }, at),
        }
    }
    let Some(biomes) = file.get(BIOMES).and_then(Value::as_object) else {
        return;
    };
    let cleared = biomes
        .get(DEFAULT)
        .and_then(Value::as_object)
        .and_then(|default| default.get("remove_all_prior_fog"))
        .is_some_and(Value::is_true);
    let prior = below
        .get(BIOMES)
        .and_then(Value::as_object)
        .filter(|_| !cleared);
    for (key, entry) in biomes {
        let key = key.as_str();
        let at = child(&JsonPath::new(), &[BIOMES, key]);
        let Some(entry) = entry.as_object() else {
            notes.warning(DiagnosticCode::InvalidType { text: None }, at);
            continue;
        };
        for (member, _) in entry {
            if !MEMBERS.contains(&member.as_str()) {
                notes.warning(
                    DiagnosticCode::UnrecognizedField { text: None },
                    child(&at, &[member.as_str()]),
                );
            }
        }
        // only the `default` entry's flag is read.
        if key != DEFAULT
            && entry
                .get("remove_all_prior_fog")
                .is_some_and(Value::is_true)
        {
            notes.warning(
                DiagnosticCode::NoEffect { text: None },
                child(&at, &["remove_all_prior_fog"]),
            );
        }
        // the predicate sees an entry only when a lower file has the key; a first
        // entry is copied as written, and nothing reads its `fog_identifier`.
        let first = !prior.is_some_and(|prior| prior.get(key).is_some_and(Value::is_object));
        if first && entry.contains_key("fog_identifier") && !entry.contains_key("fog_ids_to_merge")
        {
            notes.warning(
                DiagnosticCode::LegacyFogIgnored,
                child(&at, &["fog_identifier"]),
            );
        }
    }
}

/// Merges the copies, lowest pack first, and reads the result.
pub(crate) fn merge(copies: Vec<BiomesCopy>) -> Merged<LegacyClientBiomes, RawLayer> {
    let mut merged = VObject::new();
    let mut origins: IndexMap<BiomeId, Origin> = IndexMap::new();
    let mut diagnostics = Diagnostics::default();
    let mut layers: Vec<RawLayer> = Vec::new();
    for copy in copies {
        let layer = match copy {
            BiomesCopy::Layer(layer, found) => {
                diagnostics.extend(found);
                layer
            }
            BiomesCopy::Skipped(found) | BiomesCopy::Failed(found) => {
                diagnostics.extend(found);
                continue;
            }
        };
        let mut json = layer.json.clone();
        namespace_keys(&mut json);
        if let Some(file) = json.as_object() {
            let mut notes = Notes {
                origin: &layer.origin,
                diagnostics: &mut diagnostics,
            };
            check_copy(file, &merged, &mut notes);
            recursive_merge(&mut merged, file, &JsonPath::new(), &mut notes);
            let biomes = file.get(BIOMES).and_then(Value::as_object);
            let cleared = biomes
                .and_then(|biomes| biomes.get(DEFAULT))
                .and_then(Value::as_object)
                .and_then(|default| default.get("remove_all_prior_fog"))
                .is_some_and(Value::is_true);
            if cleared {
                // every entry of the lower files is gone.
                for lower in &layers {
                    diagnostics.push(
                        Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
                            .in_file(&lower.origin),
                    );
                }
                origins.clear();
            }
            for (key, _) in biomes.into_iter().flat_map(VObject::iter) {
                origins.insert(BiomeId::parse(key.as_str()), layer.origin.clone());
            }
        }
        layers.push(layer);
    }
    let value = read(merged, &origins, !layers.is_empty(), &mut diagnostics);
    // Top-first.
    layers.reverse();
    Merged::new(value, origins, layers, diagnostics)
}

// ---- reading the merged value --------------------------------------------------------------

/// `"#RRGGBB"` as RGBA from 0 to 1 (the form of every vanilla file).
fn color(value: &Value) -> Option<[f32; 4]> {
    let text = value.as_string()?.as_str();
    if !is_hex(text, 6) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(text.get(i..i + 2)?, 16).ok().map(unit);
    Some([channel(1)?, channel(3)?, channel(5)?, 1.0])
}

/// Reads one entry of the merged value.
fn read_entry(entry: &VObject, at: &JsonPath, notes: &mut Notes<'_>) -> LegacyClientBiome {
    let wrong = |notes: &mut Notes<'_>, member: &str| {
        notes.warning(
            DiagnosticCode::InvalidType { text: None },
            child(at, &[member]),
        );
    };
    let color_of = |notes: &mut Notes<'_>, member: &str| {
        let value = entry.get(member).filter(|value| !value.is_null())?;
        let read = color(value);
        if read.is_none() {
            wrong(notes, member);
        }
        read
    };
    let water_surface_color = color_of(notes, "water_surface_color");
    let fog_color = color_of(notes, "fog_color");
    let water_fog_color = color_of(notes, "water_fog_color");
    let number = |notes: &mut Notes<'_>, member: &str| {
        let value = entry.get(member).filter(|value| !value.is_null())?;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the game reads the member as a float"
        )]
        let read = value.as_number().map(|number| number.to_f64_lossy() as f32);
        if read.is_none() {
            notes.warning(
                DiagnosticCode::InvalidType { text: None },
                child(at, &[member]),
            );
        }
        read
    };
    // A value outside [0, 1] is clamped (the game does it silently).
    let water_surface_transparency = number(notes, "water_surface_transparency").map(|value| {
        if !(0.0..=1.0).contains(&value) {
            notes.warning(
                DiagnosticCode::ClampedValue { text: None },
                child(at, &["water_surface_transparency"]),
            );
        }
        value.clamp(0.0, 1.0)
    });
    let water_fog_distance = number(notes, "water_fog_distance");
    let mut fog_ids_to_merge = Vec::new();
    match entry.get("fog_ids_to_merge") {
        None => {}
        Some(value) if value.is_null() => {}
        Some(value) => match value.as_array() {
            Some(ids) => {
                for (index, id) in ids.iter().enumerate() {
                    match id.as_string() {
                        Some(id) => fog_ids_to_merge.push(FogId::parse(id.as_str())),
                        None => notes.warning(
                            DiagnosticCode::InvalidType { text: None },
                            child(at, &["fog_ids_to_merge", &index.to_string()]),
                        ),
                    }
                }
            }
            None => notes.warning(
                DiagnosticCode::InvalidType { text: None },
                child(at, &["fog_ids_to_merge"]),
            ),
        },
    }
    let old_fog = flag(entry, "make_back_compat_fog", at, notes).then(|| OldFog {
        fog_color,
        water_fog_color,
        water_fog_distance,
        // A fog identifier without a namespace gets `minecraft:`.
        fog_identifier: entry
            .get("fog_identifier")
            .and_then(Value::as_string)
            .map(|id| {
                if id.as_str().contains(':') {
                    FogId::parse(id.as_str())
                } else {
                    FogId::parse(format!("minecraft:{}", id.as_str()))
                }
            }),
    });
    LegacyClientBiome {
        old_fog,
        fog_ids_to_merge,
        water_surface_color,
        water_surface_transparency,
    }
}

/// Reads the merged JSON: `default` when it is an object, every other key as a biome's entry.
fn read(
    merged: VObject,
    origins: &IndexMap<BiomeId, Origin>,
    any: bool,
    diagnostics: &mut Diagnostics,
) -> LegacyClientBiomes {
    let mut value = LegacyClientBiomes::default();
    match merged.get(BIOMES).and_then(Value::as_object) {
        Some(biomes) => {
            for (key, entry) in biomes {
                let key = key.as_str();
                let id = BiomeId::parse(key);
                let (Some(entry), Some(origin)) = (entry.as_object(), origins.get(&id)) else {
                    continue;
                };
                let at = child(&JsonPath::new(), &[BIOMES, key]);
                let mut notes = Notes {
                    origin,
                    diagnostics,
                };
                let read = read_entry(entry, &at, &mut notes);
                if key == DEFAULT {
                    value.default = Some(read);
                } else {
                    value.biomes.insert(id, read);
                }
            }
        }
        // without `biomes` nothing is applied.
        None if any => diagnostics
            .push(Diagnostic::info(DiagnosticCode::NoEffect { text: None }).phase(Phase::Validate)),
        None => {}
    }
    value.merged = merged.into();
    value
}

// ---- the collection ------------------------------------------------------------------------

/// `biomes_client.json` at the root of every resource pack, lowest pack first.
const WALK: Walk = Walk::json_children_in(&[""]).accept(Accept::Exact(&[FILE]));

fn combine(copies: Vec<BiomesCopy>, _: &StackCx<'_>) -> Merged<LegacyClientBiomes, RawLayer> {
    merge(copies)
}

/// The loader of `rp.legacy_client_biomes()`, the family's own: a copy may take no part
/// (`BiomesCopy::Skipped`, `Failed`) with findings of its own, which `load::merged`'s
/// `FileCopy` cannot carry.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<LegacyClientBiomes, RawLayer>, BiomesCopy> =
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    };

#[cfg(test)]
mod copy_tests {
    use super::BiomesCopy;
    use std::fmt::Debug;

    fn assert_clone_debug<T: Clone + Debug>() {}

    #[test]
    fn biomes_copy_is_clone_and_debug() {
        assert_clone_debug::<BiomesCopy>();
    }
}
