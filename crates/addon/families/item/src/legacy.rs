//! What vanilla's legacy readers take from an item document: the legacy read of a
//! component-based document (at every version) and the legacy item format of 1.10.

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::version::{FormatVersion, known};
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::{LoadContext, LoadRequirement};

use super::texts;

/// Which reader an item went through (vanilla `ItemVersion`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ItemFormat {
    /// The legacy item format of 1.10: no components, the fixed key set of
    /// [`LegacyFormatItem`].
    Legacy,
    /// A component-based item (1.16.100 and 1.17.0 and later).
    #[default]
    DataDriven,
}

/// What vanilla's legacy reader takes from an item document, besides the typed components.
///
/// Filled once when the document is parsed and carried unchanged by every upgrade step, except
/// that the 1.20.50 step moves `tags` into `minecraft:tags`. Derived from the document; the
/// members no component holds (`block`, `frame_count`, the `tag:` keys of an older file, the
/// keys of `shared` no format binds) are written back from here, and the others are told at
/// their key when the item is loaded.
///
/// Equality and `Debug` include one thing no public member shows: whether the load left a
/// component-based document without a registered component (every component was dropped, or
/// only custom components are left). Such an item is told at `components` when it is loaded,
/// and [`Item::to_json`](bedrock_addon_core::document::Content::to_json) refuses it there
/// (`EncodeError::NotRepresentable`) until it holds a registered component, since the newest
/// format would read its text as a legacy item. Two items equal in every public member can
/// therefore compare unequal, and one of them be written where the other is refused.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LegacyItemData {
    /// Which reader the item went through.
    pub format: ItemFormat,
    /// The whole legacy-format document; `Some` only for [`ItemFormat::Legacy`].
    pub format_item: Option<LegacyFormatItem>,
    /// The load left a component-based document without a registered component (every
    /// component dropped, or custom components alone): the item has no text until it holds
    /// one (the writer refuses it at `components`, where the load told of it).
    pub(crate) left_empty_by_the_load: bool,
    /// The Molang expressions of the legacy members kept as JSON (event and trigger
    /// conditions, `shoot` responses), for the compile pass.
    pub(crate) expressions: super::expressions::Expressions,
    /// `description.category` as written (read by the client only).
    pub category: Option<String>,
    /// `minecraft:block`: the block the item stands for. Written back as that key.
    pub block: Option<String>,
    /// `minecraft:mining_speed` (deprecated in 1.20.30).
    pub mining_speed: Option<f32>,
    /// `minecraft:frame_count`. Written back as that key.
    pub frame_count: Option<i32>,
    /// `minecraft:creative_category` (deprecated in 1.20.20), as written.
    pub creative_category: Option<Value>,
    /// The `tag:<name>` keys, in key order (deprecated in 1.20.50).
    pub tags: Vec<String>,
    /// `minecraft:foil`: read below 1.20.20, and only by packs with the legacy creator
    /// features.
    pub is_glint: Option<bool>,
    /// `minecraft:seed` of a component-based document: nothing reads it.
    pub seed: Option<Value>,
    /// The values of the components the legacy reader also reads (`minecraft:use_duration`,
    /// `minecraft:use_modifiers`, `minecraft:max_stack_size`, …), by key.
    ///
    /// For a document of the newest format, a component the item also holds typed is here in
    /// the form the writer writes it in (`{"value": 16}` for a file's `16`), so the written
    /// text reads back to the same map. For an older document the values are the file's own,
    /// as its legacy read took them, and a written (upgraded) text gives another map.
    pub shared: IndexMap<String, Value>,
    /// The keys of [`shared`](Self::shared) that the item holds nowhere else: components the
    /// legacy reader takes and the file's format binds no typed component for
    /// (`minecraft:use_duration` from 1.20.50, `minecraft:rarity` from 1.20.30 to 1.21.29).
    /// The writers write them from `shared`, as the file had them; a step that makes a
    /// component of one takes it off this list.
    pub(crate) held_in_shared_alone: Vec<String>,
}

impl LegacyItemData {
    /// The legacy use duration in ticks (`(int)(seconds × 20)` of `minecraft:use_duration`),
    /// when the document wrote one.
    pub fn use_duration_ticks(&self) -> Option<i32> {
        let raw = self.shared.get("minecraft:use_duration")?;
        let seconds = number(raw).or_else(|| raw.as_object()?.get("value").and_then(number))?;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "vanilla's (int) conversion truncates"
        )]
        Some((seconds * 20.0) as i32)
    }

    /// The two keys of `components` the legacy reader takes at every version and no format
    /// binds a component for, as every writer of a component-based item writes them: the
    /// block the item stands for and its frame count.
    pub(crate) fn written_keys(&self) -> Vec<(String, Value)> {
        let block = self
            .block
            .as_ref()
            .map(|block| ("minecraft:block".to_owned(), Value::from(block.as_str())));
        let frames = self
            .frame_count
            .map(|frames| ("minecraft:frame_count".to_owned(), Value::from(frames)));
        block.into_iter().chain(frames).collect()
    }

    /// The keys the legacy reader of the format `version` takes and later formats do not,
    /// as the writer of a file of that format writes them: `minecraft:mining_speed` below
    /// 1.20.30, `minecraft:creative_category` and `minecraft:foil` below 1.20.20.
    pub(crate) fn written_keys_of(&self, version: FormatVersion) -> Vec<(String, Value)> {
        let mut keys = Vec::new();
        if version < V1_20_30
            && let Some(speed) = self.mining_speed
        {
            let shortest = speed.to_string().parse().unwrap_or(f64::from(speed));
            keys.push(("minecraft:mining_speed".to_owned(), Value::from(shortest)));
        }
        if version < V1_20_20 {
            if let Some(category) = &self.creative_category {
                keys.push(("minecraft:creative_category".to_owned(), category.clone()));
            }
            if let Some(glint) = self.is_glint {
                keys.push(("minecraft:foil".to_owned(), Value::from(glint)));
            }
        }
        keys
    }

    /// Whether the document's use duration in ticks is beyond an `i32`, so that
    /// [`use_duration_ticks`](Self::use_duration_ticks) is the bound and not the duration.
    pub(crate) fn use_duration_saturates(&self) -> bool {
        let seconds = self
            .shared
            .get("minecraft:use_duration")
            .and_then(|raw| number(raw).or_else(|| raw.as_object()?.get("value").and_then(number)));
        seconds.is_some_and(|seconds| (seconds * 20.0).abs() > f64::from(i32::MAX))
    }
}

/// A JSON number as `f64`.
pub(crate) fn number(value: &Value) -> Option<f64> {
    value.as_number().and_then(facet_value::VNumber::to_f64)
}

/// The keys only the legacy reader takes; they are no component of any format.
pub(crate) const LEGACY_ONLY: &[&str] = &[
    "minecraft:block",
    "minecraft:creative_category",
    "minecraft:foil",
    "minecraft:frame_count",
    "minecraft:mining_speed",
    "minecraft:seed",
];

/// The components whose raw values the legacy reader keeps in [`LegacyItemData::shared`].
const SHARED: &[&str] = &[
    "minecraft:use_duration",
    "minecraft:use_modifiers",
    "minecraft:enchantable",
    "minecraft:render_offsets",
    "minecraft:glint",
    "minecraft:icon",
    "minecraft:rarity",
    "minecraft:interact_button",
    "minecraft:allow_off_hand",
    "minecraft:can_destroy_in_creative",
    "minecraft:damage",
    "minecraft:hand_equipped",
    "minecraft:liquid_clipped",
    "minecraft:max_stack_size",
    "minecraft:should_despawn",
    "minecraft:stacked_by_data",
    "minecraft:use_animation",
    "minecraft:hover_text_color",
];

/// Whether `key` is a `tag:<name>` key (`tag:[a-zA-Z0-9_.:-]+`).
pub(crate) fn is_tag_key(key: &str) -> bool {
    key.strip_prefix("tag:").is_some_and(|name| {
        !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'-'))
    })
}

/// Whether the legacy reader takes `key` (so it is never an unknown component).
pub(crate) fn is_legacy_read(key: &str) -> bool {
    LEGACY_ONLY.contains(&key) || SHARED.contains(&key) || is_tag_key(key)
}

const V1_16_100: FormatVersion = known::V1_16_100;
const V1_20_20: FormatVersion = FormatVersion::new(1, 20, 20);
const V1_20_30: FormatVersion = FormatVersion::new(1, 20, 30);
const V1_20_50: FormatVersion = FormatVersion::new(1, 20, 50);

/// The legacy read of a component-based document's `components` object (`at` is its path),
/// with its deprecations at `original_version`. `glint` says whether `minecraft:glint` is
/// present (it supersedes `minecraft:foil`).
#[allow(
    clippy::too_many_lines,
    reason = "one arm per legacy component, in the schema's order"
)]
pub(crate) fn read_legacy(
    components: Option<&IndexMap<String, Value>>,
    category: Option<String>,
    original_version: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> LegacyItemData {
    let mut data = LegacyItemData {
        category,
        ..LegacyItemData::default()
    };
    let Some(components) = components else {
        return data;
    };
    let path = |key: &str| at.child(key);
    // The game writes a deprecation line for `foil`, `mining_speed`, `tag:` and
    // `creative_category` at the content log's error level, and the item loads on: each is a
    // kept error.
    let deprecated = |d: &mut Diagnostics, key: &str, text: VanillaText| {
        d.push_kept_error(
            Diagnostic::error(DiagnosticCode::DeprecatedMember { text: Some(text) }).at(path(key)),
        );
    };
    for (key, value) in components {
        if SHARED.contains(&key.as_str()) {
            data.shared.insert(key.clone(), value.clone());
        }
        match key.as_str() {
            "minecraft:block" => {
                data.block = value.as_string().map(|s| s.as_str().to_owned());
                if data.block.is_none() {
                    misread(d, DiagnosticCode::InvalidType { text: None }, path(key));
                }
            }
            "minecraft:frame_count" => {
                data.frame_count = number(value).map(truncate);
                match number(value) {
                    None => misread(d, DiagnosticCode::InvalidType { text: None }, path(key)),
                    Some(n) if n.fract() != 0.0 || n.abs() > f64::from(i32::MAX) => {
                        misread(d, DiagnosticCode::InvalidValue { text: None }, path(key));
                    }
                    Some(_) => {}
                }
            }
            "minecraft:mining_speed" => {
                if original_version >= V1_20_30 {
                    deprecated(d, key, VanillaText::Fixed(texts::MINING_SPEED_DEPRECATED));
                } else {
                    #[allow(clippy::cast_possible_truncation, reason = "an f32 member")]
                    let speed = number(value).map(|n| n as f32);
                    match speed {
                        None => misread(d, DiagnosticCode::InvalidType { text: None }, path(key)),
                        Some(speed) if speed.is_infinite() => {
                            misread(d, DiagnosticCode::InvalidValue { text: None }, path(key));
                        }
                        Some(_) => {}
                    }
                    // A number beyond a float is the largest finite float of its sign.
                    data.mining_speed = speed.map(|speed| {
                        if speed.is_infinite() {
                            f32::MAX.copysign(speed)
                        } else {
                            speed
                        }
                    });
                }
            }
            // No slice binds `use_modifiers` before 1.20.50, so what an author wrote below
            // 1.20.50 reaches only the legacy read: it holds it (`shared`, as vanilla's own
            // legacy data does), it is no component of the item, and the item, written in the
            // newest format, does not carry it: told as not written, as an information (as
            // for `creative_category`, which the game holds too).
            "minecraft:use_modifiers" if original_version < V1_20_50 => {
                misread(d, DiagnosticCode::DroppedField { text: None }, path(key));
            }
            // The legacy read takes the number, or `value` of the object form: any other
            // member of the object is read by nothing.
            "minecraft:use_duration" => {
                // From 1.20.50 the legacy read alone takes the duration (below it the
                // typed component does, and says what it finds): a value that is no number
                // gives no duration, told at the member.
                let held = match value.as_object() {
                    Some(object) => object.get("value"),
                    None => Some(value),
                };
                if original_version >= V1_20_50 && held.is_some_and(|held| number(held).is_none()) {
                    let mut at = path(key);
                    if value.is_object() {
                        at.push("value");
                    }
                    misread(d, DiagnosticCode::InvalidType { text: None }, at);
                }
                if let Some(object) = value.as_object() {
                    for (member, _) in object
                        .iter()
                        .filter(|(member, _)| member.as_str() != "value")
                    {
                        let at = path(key).child(member.as_str());
                        misread(d, DiagnosticCode::UnrecognizedField { text: None }, at);
                    }
                }
            }
            "minecraft:creative_category" => {
                if original_version >= V1_20_20 {
                    deprecated(
                        d,
                        key,
                        VanillaText::ItemCreativeCategoryDeprecated {
                            version: original_version,
                        },
                    );
                } else {
                    // Held for the legacy item (the legacy data's `creative_category`). The
                    // file's own format writes it back; the upgrade, which leaves the
                    // formats that read it, tells that it goes.
                    data.creative_category = Some(value.clone());
                }
            }
            "minecraft:foil" => {
                if components.contains_key("minecraft:glint") {
                    // The text is the one the block upgrade writes when a component changes name;
                    // the game writes no such line for an item, so this information is the
                    // crate's own.
                    d.push(
                        Diagnostic::info(DiagnosticCode::DuplicateComponentVersion {
                            old: "minecraft:foil".to_owned(),
                            new: "minecraft:glint".to_owned(),
                        })
                        .at(path("minecraft:glint")),
                    );
                } else if original_version >= V1_20_20 {
                    deprecated(d, key, VanillaText::Fixed(texts::FOIL_DEPRECATED));
                } else if LoadRequirement::LegacyCreatorFeatures.satisfied(cx) {
                    data.is_glint = value.as_bool();
                    if data.is_glint.is_none() {
                        misread(d, DiagnosticCode::InvalidType { text: None }, path(key));
                    }
                } else {
                    // The text is the crate's own and the game says nothing: an information.
                    d.push(
                        Diagnostic::info(DiagnosticCode::DroppedComponent {
                            text: Some(VanillaText::Fixed(texts::FOIL_IGNORED)),
                        })
                        .at(path(key)),
                    );
                }
            }
            "minecraft:seed" => {
                data.seed = Some(value.clone());
                // Nothing reads it in a component-based document, and it is not written.
                // Above 1.16.100 the game's own warning says so (below); at 1.16.100 it is
                // noted here (an information: the game discards it too).
                if original_version <= V1_16_100 {
                    d.push(
                        Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(path(key)),
                    );
                }
            }
            tag if is_tag_key(tag) => {
                if original_version >= V1_20_50 {
                    deprecated(d, key, VanillaText::Fixed(texts::TAG_DEPRECATED));
                } else {
                    data.tags.push(tag["tag:".len()..].to_owned());
                    // The tag is the key; the game does not read the value. The crate
                    // tells a value that is no empty object, so that nothing an author
                    // wrote goes without a word.
                    if !value
                        .as_object()
                        .is_some_and(facet_value::VObject::is_empty)
                    {
                        d.push(
                            Diagnostic::info(DiagnosticCode::InvalidType { text: None })
                                .at(path(key)),
                        );
                    }
                }
            }
            _ => {}
        }
    }
    if components.contains_key("minecraft:seed") && original_version > V1_16_100 {
        seed_warning(at, d);
    }
    data
}

/// The `minecraft:seed` warning of a document above 1.16.100: the game logs it at the content
/// log's warning level and keeps the key.
pub(crate) fn seed_warning(at: &JsonPath, d: &mut Diagnostics) {
    let path = at.child("minecraft:seed");
    d.push(
        Diagnostic::warn(DiagnosticCode::DeprecatedMember {
            text: Some(VanillaText::Fixed(texts::SEED_DEPRECATED)),
        })
        .at(path),
    );
}

/// The information of a key of the legacy reader read as another value than the file writes:
/// the value stays what the reader makes of it. The game is silent, so the finding is the
/// crate's own information.
fn misread(d: &mut Diagnostics, code: DiagnosticCode, path: JsonPath) {
    let said = d
        .iter()
        .any(|finding| finding.location.json_path == path && finding.severity >= Severity::Info);
    if !said {
        d.push(Diagnostic::info(code).at(path));
    }
}

/// A float truncated to an integer, as vanilla's `asInt` does.
#[allow(clippy::cast_possible_truncation, reason = "vanilla's asInt truncates")]
fn truncate(n: f64) -> i32 {
    n as i32
}

/// An item in the legacy item format of 1.10: the fixed key set of `minecraft:item ›
/// components` vanilla's legacy readers take (no format version changes it).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LegacyFormatItem {
    /// `minecraft:max_damage` (default 0).
    pub max_damage: i32,
    /// `minecraft:hand_equipped` (default false).
    pub hand_equipped: bool,
    /// `minecraft:stacked_by_data` (default false).
    pub stacked_by_data: bool,
    /// `minecraft:foil` (default false; no gate in this format).
    pub foil: bool,
    /// `minecraft:block`: the block the item places or stands for.
    pub block: Option<String>,
    /// `minecraft:use_duration` in ticks (an integer; a real number is truncated; default 0).
    pub use_duration: i32,
    /// `minecraft:max_stack_size` (default 64).
    pub max_stack_size: i32,
    /// `minecraft:food`.
    pub food: Option<LegacyFood>,
    /// `minecraft:seed`.
    pub seed: Option<LegacySeed>,
    /// `minecraft:camera`.
    pub camera: Option<LegacyCamera>,
    /// The client keys (`minecraft:icon`, `icon_variant`, `icon_atlas`, `use_animation`,
    /// `hover_text_color`, `rarity`), as written.
    pub client: IndexMap<String, Value>,
    /// Every other key, ignored by vanilla (`minecraft:is_meat`), kept for write-back.
    pub extra: UnknownMembers,
}

impl Default for LegacyFormatItem {
    fn default() -> Self {
        Self {
            max_damage: 0,
            hand_equipped: false,
            stacked_by_data: false,
            foil: false,
            block: None,
            use_duration: 0,
            max_stack_size: 64,
            food: None,
            seed: None,
            camera: None,
            client: IndexMap::new(),
            extra: UnknownMembers::default(),
        }
    }
}

/// `minecraft:food` of the legacy item format.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LegacyFood {
    /// `nutrition`.
    pub nutrition: i32,
    /// `saturation_modifier`: a number, or a name (`poor` 0.1, `low` 0.3, `normal` 0.6, `good`
    /// 0.8, `max` 1.0, `supernatural` 1.2; any other name 0.6), as written.
    pub saturation_modifier: Option<Value>,
    /// `using_converts_to`.
    pub using_converts_to: Option<String>,
    /// `on_use_action` (`chorus_teleport`, `suspicious_stew_effect`).
    pub on_use_action: Option<String>,
    /// `on_use_range`.
    pub on_use_range: Option<Value>,
    /// `cooldown_type`.
    pub cooldown_type: Option<String>,
    /// `cooldown_time`.
    pub cooldown_time: i32,
    /// `can_always_eat`.
    pub can_always_eat: bool,
    /// `effects`.
    pub effects: Vec<LegacyEffect>,
    /// `remove_effects`.
    pub remove_effects: Vec<String>,
    /// Other members.
    pub extra: UnknownMembers,
}

impl LegacyFood {
    /// The saturation factor vanilla uses: the number, or the name's value (lower-cased; an
    /// unknown name is 0.6, silently).
    pub fn saturation(&self) -> f32 {
        match &self.saturation_modifier {
            None => 0.6,
            Some(value) => match value.as_string() {
                Some(name) => saturation_of(&name.as_str().to_ascii_lowercase()).unwrap_or(0.6),
                #[allow(clippy::cast_possible_truncation, reason = "an f32 member")]
                None => number(value).map_or(0.6, |n| n as f32),
            },
        }
    }
}

/// The legacy saturation names (no `high`).
fn saturation_of(name: &str) -> Option<f32> {
    Some(match name {
        "poor" => 0.1,
        "low" => 0.3,
        "normal" => 0.6,
        "good" => 0.8,
        "max" => 1.0,
        "supernatural" => 1.2,
        _ => return None,
    })
}

/// One effect of a [`LegacyFood`].
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LegacyEffect {
    /// `name`.
    pub name: String,
    /// `chance`.
    pub chance: f32,
    /// `duration` in seconds.
    pub duration: i32,
    /// `amplifier`.
    pub amplifier: i32,
}

/// `minecraft:seed` of the legacy item format.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LegacySeed {
    /// `crop_result`: the block planted.
    pub crop_result: Option<String>,
    /// `plant_at`: a descriptor or a list of them, as written.
    pub plant_at: Option<Value>,
    /// `plant_at_any_solid_surface`.
    pub plant_at_any_solid_surface: bool,
    /// `plant_at_face`.
    pub plant_at_face: Option<String>,
    /// Other members.
    pub extra: UnknownMembers,
}

/// `minecraft:camera` of the legacy item format.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LegacyCamera {
    /// `black_bars_duration`.
    pub black_bars_duration: f32,
    /// `black_bars_screen_ratio`.
    pub black_bars_screen_ratio: f32,
    /// `shutter_duration`.
    pub shutter_duration: f32,
    /// `shutter_screen_ratio`.
    pub shutter_screen_ratio: f32,
    /// `picture_duration`.
    pub picture_duration: f32,
    /// `slide_away_duration`.
    pub slide_away_duration: f32,
    /// Other members.
    pub extra: UnknownMembers,
}
