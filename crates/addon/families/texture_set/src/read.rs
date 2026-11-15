//! Reading a texture set the way the client does: the file and version checks of
//! the texture set loader, the schema pass, the post-validation and the colour
//! parse. Every finding carries the text the client logs, where it logs one.

use facet_value::VArray;

use super::codes;
use super::history::{TextureSetHistory, v1_16_100};
use super::types::{LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet};
use bedrock_addon_core::diagnostic::cereal::unknown_child_option;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::json::jsoncpp::lenient::read_lenient;
use bedrock_addon_core::json::kit::{Node, Read as KitRead, Versions};
use bedrock_addon_core::json::{Value, strip_bom};
use bedrock_addon_core::schema::{FromJson, read_document};
use bedrock_addon_core::version::{FormatVersion, SemVersion, known};
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::color::parse_hex;

/// The member that holds the version.
const FORMAT_VERSION: &str = "format_version";
/// The payload member.
pub(super) const PAYLOAD: &str = "minecraft:texture_set";

/// The first schema version.
pub(super) const V1_16_100: FormatVersion = known::V1_16_100;
/// The second, and newest, schema version.
pub(super) const V1_21_30: FormatVersion = known::V1_21_30;

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
        if version.is_any() || !version.pre_release.is_empty() {
            return None;
        }
        [Self::V1_16_100, Self::V1_21_30]
            .into_iter()
            .find(|schema| {
                let v = schema.version();
                (version.major, version.minor, version.patch) == (v.major, v.minor, v.patch)
            })
    }
}

/// What reading a file gave: the struct its version selects, when every rule passed.
pub(super) struct Read {
    /// The layers as written, by the struct of the file's version.
    pub(super) value: Option<TextureSetHistory>,
    /// The schema the version selected, when it selected one.
    pub(super) schema: Option<Schema>,
}

/// The root value of the file `bytes`, as the client's reader gives it.
fn read_root(bytes: &[u8], file: &str, d: &mut Diagnostics) -> Option<Value> {
    let root_at = JsonPath::new();
    // The client asserts on an empty file and shows the missing texture: an information with
    // the assert's text; the set is not loaded.
    if strip_bom(bytes).is_empty() {
        let code = Code::from(&codes::SET_EMPTY);
        d.push(Diagnostic::info(code).at(root_at));
        return None;
    }
    match read_lenient(bytes) {
        Ok(read) => {
            if read.trailing {
                // The reader never looks past the root value, and says nothing, so this is an
                // information of the crate's own.
                let code = core_codes::SYNTAX.with(["content after the JSON value is ignored"]);
                d.push(Diagnostic::info(code).at(root_at));
            }
            Some(read.value)
        }
        Err(message) => {
            // The game logs this at the content log's warning level, then shows the image
            // alone: the loader's warning; the set is not loaded.
            d.push(
                Diagnostic::of(codes::SET_NO_JSON.with([file]))
                    .detail(DiagnosticDetail::Syntax(message.to_string()))
                    .at(root_at),
            );
            None
        }
    }
}

/// The schema the `format_version` member `written` selects.
fn select_schema(written: Option<&Value>, file: &str, d: &mut Diagnostics) -> Option<Schema> {
    let version_at = JsonPath::new().child(FORMAT_VERSION);
    // The version is absent, or neither a version string nor three numbers.
    let version = written.and_then(|written| match written.as_array() {
        Some(array) => SemVersion::from_json_array(array.as_slice()),
        None => SemVersion::parse(written.as_string()?.as_str()),
    });
    let Some(version) = version else {
        let code = codes::SET_BAD_VERSION.with([file.to_owned()]);
        // The game logs this at the content log's warning level; the set is not loaded.
        d.push(Diagnostic::warn(code).at(version_at));
        return None;
    };
    // Any version but the two.
    let schema = Schema::of(&version);
    if schema.is_none() {
        let code = codes::SET_UNSUPPORTED_VERSION.with([version.to_string(), file.to_owned()]);
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
    let version_at = root_at.child(FORMAT_VERSION);
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
            other => d.push(unknown_member(root_at.child(other))),
        }
    }
    let payload_at = root_at.child(PAYLOAD);
    let layers = match payload.map(|payload| (payload, payload.as_object())) {
        None => {
            d.push(Diagnostic::of(codes::SET_REQUIRED_CHILD.with([PAYLOAD])).at(root_at.clone()));
            None
        }
        Some((_, None)) => {
            d.push(unknown_option("'object'", payload_at.clone()));
            None
        }
        Some((payload, Some(_))) => read_layers(payload, schema, &payload_at, d),
    };
    if errors(d) > before {
        d.push(closing(&codes::SET_SCHEMA_FAILED, root_at));
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
    if let Some(rule) = post_validation(names) {
        d.push(Diagnostic::of(rule).at(payload_at));
        d.push(closing(&codes::SET_POST_FAILED, root_at));
        return none(Some(schema));
    }
    let value = match schema {
        Schema::V1_16_100 => TextureSetHistory::V1_16_100(v1_16_100::TextureSet {
            color: layers.color,
            normal: layers.normal,
            heightmap: layers.heightmap,
            metalness_emissive_roughness: layers.metalness_emissive_roughness,
        }),
        Schema::V1_21_30 => TextureSetHistory::V1_21_30(layers),
    };
    Read {
        value: Some(value),
        schema: Some(schema),
    }
}

/// The line the client logs after a failed pass.
fn closing(line: &'static CodeDef<0>, at: JsonPath) -> Diagnostic {
    Diagnostic::of(line).at(at)
}

/// A member the schema has no child for. The client logs nothing of its own for it.
fn unknown_member(at: JsonPath) -> Diagnostic {
    let code = Code::from(&core_codes::UNRECOGNIZED_FIELD);
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
) -> Option<&'static CodeDef<0>> {
    let mut names = Vec::new();
    for (name, null) in members {
        if null {
            return Some(&codes::SET_MALFORMED_LAYER);
        }
        names.push(name);
    }
    let has = |name: &str| names.contains(&name);
    let mer = has(LayerKind::Mer.key());
    if !has(LayerKind::Color.key()) {
        return Some(&codes::SET_NO_COLOR);
    }
    if mer && has("metalness") && has("emissive") && has("roughness") {
        return Some(&codes::SET_MER_AND_SINGLE);
    }
    if mer && has(LayerKind::Mers.key()) {
        return Some(&codes::SET_MER_AND_MERS);
    }
    if has(LayerKind::Heightmap.key()) && has(LayerKind::Normal.key()) {
        return Some(&codes::SET_HEIGHTMAP_AND_NORMAL);
    }
    None
}

/// The layers of the payload `layers` under `schema`: the derived read of [`TextureSet`], every
/// failure told, in file order, with the client's texts; the member `metalness_emissive_roughness_subsurface`
/// is `since_format` 1.21.30, so it is an unknown member at 1.16.100. `None` when anything failed
/// (an unknown member fails the schema too, though the read builds the value).
pub(super) fn read_layers(
    layers: &Value,
    schema: Schema,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<TextureSet> {
    let before = d.len();
    let set = {
        let mut read = KitRead::new(Reading::OWN, d)
            .unknown_severity(Severity::Error)
            .versions(Versions {
                format: Some(schema.version()),
                engine: None,
            });
        read_document::<TextureSet>(&Node::new(layers, at.clone()), &mut read)
    };
    set.filter(|_| !d.iter().skip(before).any(Diagnostic::rejects))
}

/// A string that is a texture name: one that does not start with `#`.
fn texture_name(value: &Value) -> Option<TextureName> {
    let text = value.as_string()?.as_str();
    (!text.starts_with('#')).then(|| TextureName(text.to_owned()))
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

/// A colour layer's channels: their count (the array form's length, half the digits of the hex
/// form) and what they are made of.
trait Channels: Sized {
    /// The number of channels.
    const COUNT: usize;
    /// The colour of a hex value (`#AARRGGBB`, or `#RRGGBB`).
    fn hex(value: u32) -> Self;
    /// The colour of the first `COUNT` of `channels`.
    fn array(channels: [f32; 4]) -> Self;
}

impl Channels for Rgba {
    const COUNT: usize = 4;

    fn hex(v: u32) -> Self {
        Self {
            r: byte(v, 16),
            g: byte(v, 8),
            b: byte(v, 0),
            a: byte(v, 24),
        }
    }

    fn array([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl Channels for Rgb {
    const COUNT: usize = 3;

    fn hex(v: u32) -> Self {
        Self {
            r: byte(v, 16),
            g: byte(v, 8),
            b: byte(v, 0),
        }
    }

    fn array([r, g, b, _]: [f32; 4]) -> Self {
        Self { r, g, b }
    }
}

/// A layer with a name child and a colour child of `C`'s channels (`color`,
/// `metalness_emissive_roughness`, `metalness_emissive_roughness_subsurface`): an image name, a hex
/// string or an array of channels. A value no child takes, or a four-member object (the client
/// loads a zero colour; the crate rejects it), is told with the client's text.
fn layer<C: Channels>(node: &Node<'_>, read: &mut KitRead<'_>) -> Option<LayerValue<C>> {
    match colour(node.value()) {
        Colour::Name(name) => Some(LayerValue::Texture(name)),
        Colour::Hex(text) => {
            let hex = if C::COUNT == 4 {
                (8, &codes::SET_HEX_8)
            } else {
                (6, &codes::SET_HEX_6)
            };
            let Some(v) = parse_hex(text, hex.0) else {
                read.refuse(node, Code::from(hex.1));
                return None;
            };
            Some(LayerValue::Hex(C::hex(v)))
        }
        Colour::Array(array) if array.len() == C::COUNT => {
            channels(array, node, read).map(|channels| LayerValue::Array(C::array(channels)))
        }
        Colour::FourMembers => {
            read.refuse(node, Code::from(&codes::BAD_COLOR));
            None
        }
        Colour::Array(_) | Colour::Other => {
            read.refuse(node, unknown_child_option(COLOUR_TYPES));
            None
        }
    }
}

impl FromJson for LayerValue<Rgba> {
    fn from_json(node: &Node<'_>, read: &mut KitRead<'_>) -> Option<Self> {
        layer(node, read)
    }
}

impl FromJson for LayerValue<Rgb> {
    fn from_json(node: &Node<'_>, read: &mut KitRead<'_>) -> Option<Self> {
        layer(node, read)
    }
}

/// A layer with only the name child (`normal`, `heightmap`).
impl FromJson for TextureName {
    fn from_json(node: &Node<'_>, read: &mut KitRead<'_>) -> Option<Self> {
        let name = texture_name(node.value());
        if name.is_none() {
            read.refuse(node, unknown_child_option("'string'"));
        }
        name
    }
}

// Leaves of the derived check and Molang walk: neither holds a bound or Molang.
bedrock_addon_core::impl_schema_leaf!(TextureName, LayerValue<Rgba>, LayerValue<Rgb>);
bedrock_addon_core::impl_no_molang!(TextureName, LayerValue<Rgba>, LayerValue<Rgb>);

/// The first `COUNT` of the channels of the colour array `array`, in an array of four. An
/// element that is no number fails the schema; a number that is no integer is 255 with the
/// client's text; an integer outside 0 to 255 is clamped.
fn channels(array: &VArray, node: &Node<'_>, read: &mut KitRead<'_>) -> Option<[f32; 4]> {
    let mut out = [0.0; 4];
    let mut failed = false;
    for (index, (element, slot)) in array.iter().zip(&mut out).enumerate() {
        let path = node.path().index(index);
        let Some(number) = element.as_number() else {
            let code = Code::from(&core_codes::INVALID_TYPE);
            read.push(Diagnostic::error(code).at(path));
            failed = true;
            continue;
        };
        if !number.is_integer() {
            // The game logs this at the content log's error level and the set loads: a kept
            // error.
            let code = Code::from(&codes::NOT_INTEGRAL);
            read.findings_mut()
                .push_kept_error(Diagnostic::error(code).at(path));
            *slot = 255.0;
            continue;
        }
        // An integer too large for an `i64` is far above the range.
        let written = number.to_i64().unwrap_or(i64::MAX);
        let clamped = written.clamp(0, 255);
        if clamped != written {
            let code = Code::from(&core_codes::CLAMPED_VALUE);
            read.push(Diagnostic::info(code).at(path));
        }
        *slot = f32::from(u8::try_from(clamped).unwrap_or(u8::MAX));
    }
    (!failed).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The post-validation of a payload with the members `names`, none `null`.
    fn post(names: &[&'static str]) -> Option<&'static CodeDef<0>> {
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
            Some(&codes::SET_MALFORMED_LAYER),
            "checked before the missing color"
        );
        let members = [("color", false), ("normal", true), ("heightmap", false)];
        assert_eq!(
            post_validation(members.into_iter()),
            Some(&codes::SET_MALFORMED_LAYER)
        );
    }

    #[test]
    fn v9c_the_composite_layer_beside_all_three_single_layers() {
        let all = ["color", MER, "metalness", "emissive", "roughness"];
        assert_eq!(post(&all), Some(&codes::SET_MER_AND_SINGLE));
        // Each of the four is needed.
        for leave_out in 1..all.len() {
            let mut some = all.to_vec();
            some.remove(leave_out);
            assert_eq!(post(&some), None, "without {}", all[leave_out]);
        }
        // It is checked before the composite-layer rule.
        let with_mers = ["color", MER, MERS, "metalness", "emissive", "roughness"];
        assert_eq!(post(&with_mers), Some(&codes::SET_MER_AND_SINGLE));
    }

    #[test]
    fn the_rules_come_in_the_client_s_order() {
        assert_eq!(post(&[]), Some(&codes::SET_NO_COLOR));
        assert_eq!(post(&[MER, MERS]), Some(&codes::SET_NO_COLOR));
        assert_eq!(post(&["color"]), None);
        assert_eq!(post(&["color", MER]), None);
        assert_eq!(post(&["color", MERS]), None);
        assert_eq!(post(&["color", MER, MERS]), Some(&codes::SET_MER_AND_MERS));
        assert_eq!(post(&["color", "normal"]), None);
        assert_eq!(post(&["color", "heightmap"]), None);
        assert_eq!(
            post(&["color", "heightmap", "normal"]),
            Some(&codes::SET_HEIGHTMAP_AND_NORMAL)
        );
        // The composite-layer rule is checked before the heightmap rule.
        assert_eq!(
            post(&["color", MER, MERS, "heightmap", "normal"]),
            Some(&codes::SET_MER_AND_MERS)
        );
    }

    #[test]
    fn a_hex_colour_is_the_hash_and_exactly_its_digits() {
        assert_eq!(parse_hex("#AABBCCDD", 8), Some(0xAABB_CCDD));
        assert_eq!(parse_hex("#aabbccdd", 8), Some(0xAABB_CCDD));
        assert_eq!(parse_hex("#AABBCC", 6), Some(0x00AA_BBCC));
        for bad in [
            "#AABBCCD",
            "#AABBCCDDE",
            "#AABBCCDG",
            "#+ABBCCDD",
            "AABBCCDD#",
            "#",
        ] {
            assert_eq!(parse_hex(bad, 8), None, "{bad}");
        }
        assert_eq!(parse_hex("#AABBCCDD", 6), None);
        assert_eq!(parse_hex("#AABBC", 6), None);
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
