//! Reading a texture set the way the client does: the file and version checks of
//! the texture set loader, the schema pass, the post-validation and the colour
//! parse. Every finding carries the text the client logs, where it logs one.

use facet_value::{VArray, VObject};

use super::history::{Any, v1_16_100};
use super::set::{LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet};
use super::texts;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::{Value, strip_bom};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_support::texture::jsoncpp::read_lenient;
use bedrock_addon_support::texture::texts::{fixed, formatted};

/// The member that holds the version.
const FORMAT_VERSION: &str = "format_version";
/// The payload member.
pub(super) const PAYLOAD: &str = "minecraft:texture_set";

/// The first schema version.
pub(super) const V1_16_100: FormatVersion = FormatVersion::new(1, 16, 100);
/// The second, and newest, schema version.
pub(super) const V1_21_30: FormatVersion = FormatVersion::new(1, 21, 30);

/// Which of the two schemas reads a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Schema {
    /// Exactly 1.16.100.
    V1_16_100,
    /// Exactly 1.21.30.
    V1_21_30,
}

impl Schema {
    /// The schema's version.
    pub(super) fn version(self) -> FormatVersion {
        match self {
            Self::V1_16_100 => V1_16_100,
            Self::V1_21_30 => V1_21_30,
        }
    }

    /// The schema a version selects: an exact match of the numbers and of the pre-release
    /// text. Build metadata takes no part.
    fn of(version: &SemVersion) -> Option<Schema> {
        if version.any || !version.pre_release.is_empty() {
            return None;
        }
        [Self::V1_16_100, Self::V1_21_30]
            .into_iter()
            .find(|schema| {
                let v = schema.version();
                (version.major, version.minor, version.patch) == (v.major, v.minor, v.patch)
            })
    }

    /// Whether the schema has the layer.
    fn has(self, layer: LayerKind) -> bool {
        layer != LayerKind::Mers || self == Self::V1_21_30
    }
}

/// `at` with `key` below it.
fn child(at: &JsonPath, key: &str) -> JsonPath {
    let mut path = at.clone();
    path.push(key);
    path
}

/// What reading a file gave: the struct its version selects, when every rule passed.
pub(super) struct Read {
    /// The layers as written, by the struct of the file's version.
    pub(super) value: Option<Any>,
    /// The schema the version selected, when it selected one.
    pub(super) schema: Option<Schema>,
}

/// The root value of the file `bytes`, as the client's reader gives it.
fn read_root(bytes: &[u8], file: &str, d: &mut Diagnostics) -> Option<Value> {
    let root_at = JsonPath::new();
    // The client asserts on an empty file and shows the missing texture: an information with
    // the assert's text; the set is not loaded.
    if strip_bom(bytes).is_empty() {
        let code = DiagnosticCode::EmptyDocument {
            text: fixed(texts::SET_EMPTY),
        };
        d.push(Diagnostic::info(code).at(root_at));
        return None;
    }
    match read_lenient(bytes) {
        Ok(read) => {
            if read.trailing {
                // The reader never looks past the root value, and says nothing, so this is an
                // information of the crate's own.
                let code = DiagnosticCode::MalformedInput {
                    message: "content after the JSON value is ignored".to_owned(),
                    text: None,
                };
                d.push(Diagnostic::info(code).at(root_at));
            }
            Some(read.value)
        }
        Err(message) => {
            let code = DiagnosticCode::MalformedInput {
                message: message.to_string(),
                text: formatted(texts::SET_NO_JSON, [file.to_owned()]),
            };
            // The game logs this at the content log's warning level, then shows the image
            // alone: the loader's warning; the set is not loaded.
            d.push(Diagnostic::warn(code).at(root_at));
            None
        }
    }
}

/// The schema the `format_version` member `written` selects.
fn select_schema(written: Option<&Value>, file: &str, d: &mut Diagnostics) -> Option<Schema> {
    let version_at = child(&JsonPath::new(), FORMAT_VERSION);
    // The version is absent, or neither a version string nor three numbers.
    let version = written.and_then(|written| match written.as_array() {
        Some(array) => SemVersion::from_json_array(array.as_slice()),
        None => SemVersion::parse(written.as_string()?.as_str()),
    });
    let Some(version) = version else {
        let code = DiagnosticCode::MalformedFormatVersion {
            text: formatted(texts::SET_BAD_VERSION, [file.to_owned()]),
        };
        // The game logs this at the content log's warning level; the set is not loaded.
        d.push(Diagnostic::warn(code).at(version_at));
        return None;
    };
    // Any version but the two.
    let schema = Schema::of(&version);
    if schema.is_none() {
        let code = DiagnosticCode::UnsupportedVersion {
            text: formatted(
                texts::SET_UNSUPPORTED_VERSION,
                [version.to_string(), file.to_owned()],
            ),
        };
        // The game logs this at the content log's warning level; the set is not loaded.
        d.push(Diagnostic::warn(code).at(version_at));
    }
    schema
}

/// Reads the file `bytes`; `file` is its path, for the client's texts.
pub(super) fn read(bytes: &[u8], file: &str, d: &mut Diagnostics) -> Read {
    let none = |schema| Read {
        value: None,
        schema,
    };
    let root_at = JsonPath::new();
    let Some(root) = read_root(bytes, file, d) else {
        return none(None);
    };
    let members = root.as_object();
    let written = members.and_then(|members| members.get(FORMAT_VERSION));
    let Some(schema) = select_schema(written, file, d) else {
        return none(None);
    };
    let version_at = child(&root_at, FORMAT_VERSION);
    let Some(members) = members else {
        return none(Some(schema));
    };
    // A kept error fails no pass.
    let errors = |d: &Diagnostics| d.iter().filter(|f| f.rejects()).count();
    let before = errors(d);
    // The schema pass: the array form selected a schema, and the schema takes a string.
    if written.is_some_and(|written| !written.is_string()) {
        d.push(unknown_option("'version'", version_at));
    }
    let mut payload = None;
    for (key, value) in members {
        match key.as_str() {
            FORMAT_VERSION => {}
            PAYLOAD => payload = Some(value),
            other => d.push(unknown_member(child(&root_at, other))),
        }
    }
    let payload_at = child(&root_at, PAYLOAD);
    let layers = match payload.map(|payload| (payload, payload.as_object())) {
        None => {
            let code = DiagnosticCode::MissingPayload {
                text: formatted(texts::SET_REQUIRED_CHILD, [PAYLOAD.to_owned()]),
            };
            d.push(Diagnostic::error(code).at(root_at.clone()));
            None
        }
        Some((_, None)) => {
            d.push(unknown_option("'object'", payload_at.clone()));
            None
        }
        Some((_, Some(layers))) => read_layers(layers, schema, &payload_at, d),
    };
    if errors(d) > before {
        d.push(closing(texts::SET_SCHEMA_FAILED, root_at));
        return none(Some(schema));
    }
    let Some(layers) = layers else {
        return none(Some(schema));
    };
    // The post-validation, on the members as written.
    let written_layers = payload.and_then(Value::as_object);
    let names = written_layers
        .into_iter()
        .flatten()
        .map(|(key, value)| (key.as_str(), value.is_null()));
    if let Some(text) = post_validation(names) {
        let code = DiagnosticCode::InvalidValue { text: fixed(text) };
        d.push(Diagnostic::error(code).at(payload_at));
        d.push(closing(texts::SET_POST_FAILED, root_at));
        return none(Some(schema));
    }
    let value = match schema {
        Schema::V1_16_100 => Any::V1_16_100(v1_16_100::TextureSet {
            color: layers.color,
            normal: layers.normal,
            heightmap: layers.heightmap,
            metalness_emissive_roughness: layers.metalness_emissive_roughness,
        }),
        Schema::V1_21_30 => Any::V1_21_30(layers),
    };
    Read {
        value: Some(value),
        schema: Some(schema),
    }
}

/// The line the client logs after a failed pass.
fn closing(text: &'static str, at: JsonPath) -> Diagnostic {
    let code = DiagnosticCode::InvalidValue { text: fixed(text) };
    Diagnostic::error(code).at(at)
}

/// A member the schema has no child for. The client logs nothing of its own for it.
fn unknown_member(at: JsonPath) -> Diagnostic {
    let code = DiagnosticCode::UnrecognizedField { text: None };
    Diagnostic::error(code).at(at)
}

/// A value none of the member's schema children takes; `allowed` is the list of their
/// type names as the client prints it.
fn unknown_option(allowed: &str, at: JsonPath) -> Diagnostic {
    let code = bedrock_addon_core::diagnostic::cereal::unknown_child_option(allowed);
    Diagnostic::error(code).at(at)
}

/// The first rule of the post-validation that `members` (each key of the payload, and whether
/// its value is `null`) breaks, as the client's text.
/// The rules are checked in the game's order.
pub(super) fn post_validation<'a>(
    members: impl Iterator<Item = (&'a str, bool)>,
) -> Option<&'static str> {
    let mut names = Vec::new();
    for (name, null) in members {
        if null {
            return Some(texts::SET_MALFORMED_LAYER);
        }
        names.push(name);
    }
    let has = |name: &str| names.contains(&name);
    let mer = has(LayerKind::Mer.key());
    if !has(LayerKind::Color.key()) {
        return Some(texts::SET_NO_COLOR);
    }
    if mer && has("metalness") && has("emissive") && has("roughness") {
        return Some(texts::SET_MER_AND_SINGLE);
    }
    if mer && has(LayerKind::Mers.key()) {
        return Some(texts::SET_MER_AND_MERS);
    }
    if has(LayerKind::Heightmap.key()) && has(LayerKind::Normal.key()) {
        return Some(texts::SET_HEIGHTMAP_AND_NORMAL);
    }
    None
}

/// The layers of the payload object under `schema`; `None` when a member fails the schema.
/// Colour arrays are parsed as the client parses them.
pub(super) fn read_layers(
    layers: &VObject,
    schema: Schema,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<TextureSet> {
    let mut set = TextureSet::default();
    let mut failed = false;
    for (key, value) in layers {
        let path = child(at, key.as_str());
        let layer = LayerKind::ALL
            .into_iter()
            .find(|layer| layer.key() == key.as_str() && schema.has(*layer));
        let Some(layer) = layer else {
            // `metalness_emissive_roughness_subsurface` at 1.16.100 too.
            d.push(unknown_member(path));
            failed = true;
            continue;
        };
        let read = match layer {
            LayerKind::Color => rgba(value, &path, d).map(|value| set.color = Some(value)),
            LayerKind::Mers => rgba(value, &path, d)
                .map(|value| set.metalness_emissive_roughness_subsurface = Some(value)),
            LayerKind::Mer => {
                rgb(value, &path, d).map(|value| set.metalness_emissive_roughness = Some(value))
            }
            LayerKind::Normal => name_only(value, &path, d).map(|name| set.normal = Some(name)),
            LayerKind::Heightmap => {
                name_only(value, &path, d).map(|name| set.heightmap = Some(name))
            }
        };
        failed |= read.is_none();
    }
    (!failed).then_some(set)
}

/// A string that is a texture name: one that does not start with `#`.
fn texture_name(value: &Value) -> Option<TextureName> {
    let text = value.as_string()?.as_str();
    (!text.starts_with('#')).then(|| TextureName(text.to_owned()))
}

/// A layer with only the name child (`normal`, `heightmap`).
fn name_only(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<TextureName> {
    let name = texture_name(value);
    if name.is_none() {
        d.push(unknown_option("'string'", at.clone()));
    }
    name
}

/// The digits of a hex colour string of `digits` hex digits, as a number; `None` when the
/// string, lower-cased, is not `#` and exactly that many hex digits.
fn hex(text: &str, digits: usize) -> Option<u32> {
    let rest = text.strip_prefix('#')?;
    if rest.len() != digits || !rest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(rest, 16).ok()
}

/// One byte of `value`, `shift` bits up.
fn byte(value: u32, shift: u32) -> f32 {
    f32::from(u8::try_from((value >> shift) & 0xFF).unwrap_or(u8::MAX))
}

/// What a colour child makes of a value, apart from the channel count.
enum Colour<'a> {
    /// A texture name.
    Name(TextureName),
    /// A `#` string the colour child takes: two bytes or more.
    Hex(&'a str),
    /// An array.
    Array(&'a VArray),
    /// An object of four members: the client loads a zero colour; the crate rejects it.
    FourMembers,
    /// A value no child takes.
    Other,
}

/// Sorts a value of a colour layer.
fn colour(value: &Value) -> Colour<'_> {
    if let Some(name) = texture_name(value) {
        return Colour::Name(name);
    }
    if let Some(text) = value.as_string() {
        // The colour child takes a `#` string of two bytes or more.
        return if text.as_str().len() >= 2 {
            Colour::Hex(text.as_str())
        } else {
            Colour::Other
        };
    }
    if let Some(array) = value.as_array() {
        return Colour::Array(array);
    }
    match value.as_object() {
        Some(object) if object.len() == 4 => Colour::FourMembers,
        _ => Colour::Other,
    }
}

/// The types a colour layer's children are registered under.
const COLOUR_TYPES: &str = "'color' 'string'";

/// A four-member object. The crate rejects it; the client logs this text and loads a zero
/// colour.
fn four_members(at: &JsonPath) -> Diagnostic {
    let code = DiagnosticCode::InvalidType {
        text: fixed(texts::BAD_COLOR),
    };
    Diagnostic::error(code).at(at.clone())
}

/// A layer with a four-channel colour child and the name child (`color`,
/// `metalness_emissive_roughness_subsurface`).
fn rgba(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<LayerValue<Rgba>> {
    match colour(value) {
        Colour::Name(name) => Some(LayerValue::Texture(name)),
        Colour::Hex(text) => {
            let Some(v) = hex(text, 8) else {
                let code = DiagnosticCode::InvalidValue {
                    text: fixed(texts::SET_HEX_8),
                };
                d.push(Diagnostic::error(code).at(at.clone()));
                return None;
            };
            // `#AARRGGBB`.
            Some(LayerValue::Hex(Rgba {
                r: byte(v, 16),
                g: byte(v, 8),
                b: byte(v, 0),
                a: byte(v, 24),
            }))
        }
        Colour::Array(array) if array.len() == 4 => {
            let [red, green, blue, alpha] = channels::<4>(array, at, d)?;
            Some(LayerValue::Array(Rgba {
                r: red,
                g: green,
                b: blue,
                a: alpha,
            }))
        }
        Colour::FourMembers => {
            d.push(four_members(at));
            None
        }
        Colour::Array(_) | Colour::Other => {
            d.push(unknown_option(COLOUR_TYPES, at.clone()));
            None
        }
    }
}

/// A layer with a three-channel colour child and the name child
/// (`metalness_emissive_roughness`).
fn rgb(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<LayerValue<Rgb>> {
    match colour(value) {
        Colour::Name(name) => Some(LayerValue::Texture(name)),
        Colour::Hex(text) => {
            let Some(v) = hex(text, 6) else {
                let code = DiagnosticCode::InvalidValue {
                    text: fixed(texts::SET_HEX_6),
                };
                d.push(Diagnostic::error(code).at(at.clone()));
                return None;
            };
            Some(LayerValue::Hex(Rgb {
                r: byte(v, 16),
                g: byte(v, 8),
                b: byte(v, 0),
            }))
        }
        Colour::Array(array) if array.len() == 3 => {
            let [r, g, b] = channels::<3>(array, at, d)?;
            Some(LayerValue::Array(Rgb { r, g, b }))
        }
        Colour::FourMembers => {
            d.push(four_members(at));
            None
        }
        Colour::Array(_) | Colour::Other => {
            d.push(unknown_option(COLOUR_TYPES, at.clone()));
            None
        }
    }
}

/// The `N` channels of a colour array of `N` elements. An element that is no number fails the
/// schema; a number that is no integer is 255 with the client's text; an integer
/// outside 0 to 255 is clamped.
fn channels<const N: usize>(
    array: &VArray,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<[f32; N]> {
    let mut out = [0.0; N];
    let mut failed = false;
    for (index, (element, slot)) in array.iter().zip(&mut out).enumerate() {
        let mut path = at.clone();
        path.push_index(index);
        let Some(number) = element.as_number() else {
            let code = DiagnosticCode::InvalidType { text: None };
            d.push(Diagnostic::error(code).at(path));
            failed = true;
            continue;
        };
        if !number.is_integer() {
            // The game logs this at the content log's error level and the set loads: a kept
            // error.
            let code = DiagnosticCode::InvalidValue {
                text: fixed(texts::NOT_INTEGRAL),
            };
            d.push_kept_error(Diagnostic::error(code).at(path));
            *slot = 255.0;
            continue;
        }
        // An integer too large for an `i64` is far above the range.
        let written = number.to_i64().unwrap_or(i64::MAX);
        let clamped = written.clamp(0, 255);
        if clamped != written {
            let code = DiagnosticCode::ClampedValue { text: None };
            d.push(Diagnostic::info(code).at(path));
        }
        *slot = f32::from(u8::try_from(clamped).unwrap_or(u8::MAX));
    }
    (!failed).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The post-validation of a payload with the members `names`, none `null`.
    fn post(names: &[&'static str]) -> Option<&'static str> {
        post_validation(names.iter().map(|name| (*name, false)))
    }

    const MER: &str = "metalness_emissive_roughness";
    const MERS: &str = "metalness_emissive_roughness_subsurface";

    // Two of the post-validation rules cannot be reached from a file (the schema pass fails first);
    // the rules are implemented with the client's texts all the same.
    #[test]
    fn v9a_a_null_layer_is_malformed_before_any_other_rule() {
        let members = [("normal", true)];
        assert_eq!(
            post_validation(members.into_iter()),
            Some(texts::SET_MALFORMED_LAYER),
            "checked before the missing color"
        );
        let members = [("color", false), ("normal", true), ("heightmap", false)];
        assert_eq!(
            post_validation(members.into_iter()),
            Some(texts::SET_MALFORMED_LAYER)
        );
    }

    #[test]
    fn v9c_the_composite_layer_beside_all_three_single_layers() {
        let all = ["color", MER, "metalness", "emissive", "roughness"];
        assert_eq!(post(&all), Some(texts::SET_MER_AND_SINGLE));
        // Each of the four is needed.
        for leave_out in 1..all.len() {
            let mut some = all.to_vec();
            some.remove(leave_out);
            assert_eq!(post(&some), None, "without {}", all[leave_out]);
        }
        // It is checked before the composite-layer rule.
        let with_mers = ["color", MER, MERS, "metalness", "emissive", "roughness"];
        assert_eq!(post(&with_mers), Some(texts::SET_MER_AND_SINGLE));
    }

    #[test]
    fn the_rules_come_in_the_client_s_order() {
        assert_eq!(post(&[]), Some(texts::SET_NO_COLOR));
        assert_eq!(post(&[MER, MERS]), Some(texts::SET_NO_COLOR));
        assert_eq!(post(&["color"]), None);
        assert_eq!(post(&["color", MER]), None);
        assert_eq!(post(&["color", MERS]), None);
        assert_eq!(post(&["color", MER, MERS]), Some(texts::SET_MER_AND_MERS));
        assert_eq!(post(&["color", "normal"]), None);
        assert_eq!(post(&["color", "heightmap"]), None);
        assert_eq!(
            post(&["color", "heightmap", "normal"]),
            Some(texts::SET_HEIGHTMAP_AND_NORMAL)
        );
        // The composite-layer rule is checked before the heightmap rule.
        assert_eq!(
            post(&["color", MER, MERS, "heightmap", "normal"]),
            Some(texts::SET_MER_AND_MERS)
        );
    }

    #[test]
    fn a_hex_colour_is_the_hash_and_exactly_its_digits() {
        assert_eq!(hex("#AABBCCDD", 8), Some(0xAABB_CCDD));
        assert_eq!(hex("#aabbccdd", 8), Some(0xAABB_CCDD));
        assert_eq!(hex("#AABBCC", 6), Some(0x00AA_BBCC));
        for bad in [
            "#AABBCCD",
            "#AABBCCDDE",
            "#AABBCCDG",
            "#+ABBCCDD",
            "AABBCCDD#",
            "#",
        ] {
            assert_eq!(hex(bad, 8), None, "{bad}");
        }
        assert_eq!(hex("#AABBCCDD", 6), None);
        assert_eq!(hex("#AABBC", 6), None);
    }

    #[test]
    fn a_lone_hash_is_no_colour_and_no_name() {
        // The colour child takes a `#` string of two bytes or more; the name child none.
        let lone: Value = bedrock_addon_core::json::from_str("\"#\"").unwrap();
        assert!(matches!(colour(&lone), Colour::Other));
        let two: Value = bedrock_addon_core::json::from_str("\"#1\"").unwrap();
        assert!(matches!(colour(&two), Colour::Hex("#1")));
        let empty: Value = bedrock_addon_core::json::from_str("\"\"").unwrap();
        assert!(matches!(colour(&empty), Colour::Name(name) if name.0.is_empty()));
        let five: Value =
            bedrock_addon_core::json::from_str(r#"{"a":1,"b":2,"c":3,"d":4,"e":5}"#).unwrap();
        assert!(matches!(colour(&five), Colour::Other));
    }

    #[test]
    fn a_schema_is_selected_by_the_numbers_and_the_pre_release_alone() {
        let of = |text: &str| Schema::of(&SemVersion::parse(text).unwrap());
        assert_eq!(of("1.16.100"), Some(Schema::V1_16_100));
        assert_eq!(of("1.21.30"), Some(Schema::V1_21_30));
        assert_eq!(of("1.21.30+build"), Some(Schema::V1_21_30));
        assert_eq!(of("1.21.30-beta"), None);
        assert_eq!(of("1.16.100-rc.1+build"), None);
        assert_eq!(of("*"), None);
        assert_eq!(of("1.21.31"), None);
        assert_eq!(of("1.21"), None);
        assert_eq!(of("2.21.30"), None);
        assert_eq!(of("1.16.30"), None);
    }
}
