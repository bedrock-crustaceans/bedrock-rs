//! Reading a [`TextureAtlas`] from the merged JSON of the stack, with the coercions of
//! vanilla's JSON library: the derived reader of the types under `Scalars::Lenient` and a
//! silent `null`, and a leaf or a `read_with` fn for each member the game reads by a rule of
//! its own.

use indexmap::IndexMap;

use super::codes;
use super::types::{
    AtlasItem, AtlasKind, AtlasTextures, Color, Face, Isotropic, MAX_WEIGHT, MIN_WEIGHT,
    TextureAtlas, TextureElement, TextureLayer, Tint, TintMapColor, VariationEntry, after_first,
    face_is_read,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::AtlasKey;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::jsoncpp::lenient::{as_float, is_empty, strtouint32_hex, to_text};
use bedrock_addon_core::json::kit::{Node, Profile, Read, Recovery, UnknownAt};
use bedrock_addon_core::schema::{FromJson, UnknownRule, read_document, read_entries};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading, Scalars};
use bedrock_addon_shared_shapes::OneOrMany;

/// The members of the root that vanilla's own files write and nothing reads.
const IGNORED: [&str; 2] = ["resource_pack_name", "format_version"];
/// The members of a tint map.
const CHANNELS: [&str; 4] = ["r_color", "g_color", "b_color", "a_color"];

/// Reads the atlas of `kind` from the merged document; `None`, with vanilla's error, when the
/// document names no `texture_name` (vanilla then builds no atlas).
pub(super) fn read_atlas(
    kind: AtlasKind,
    merged: &Value,
    out: &mut Diagnostics,
) -> Option<TextureAtlas> {
    // A `null` `texture_name` counts as present, and reads as `""`.
    let name = merged.as_object().and_then(|root| root.get("texture_name"));
    let Some(name) = name else {
        out.push(
            Diagnostic::error(codes::NO_TEXTURE_NAME.with([kind.path().to_owned()]))
                .phase(Phase::Validate),
        );
        return None;
    };
    let texture_name = to_text(Some(name));
    let at = JsonPath::new().child("texture_name");
    if texture_name.is_empty() {
        // The game raises a debug assert here; a release build goes on, so this is an
        // information with its text.
        out.push(
            Diagnostic::info(&codes::NO_TEXTURE_PAGE)
                .at(at)
                .phase(Phase::Validate),
        );
    } else if texture_name != kind.expected_name() {
        // The game registers the page under the other name and nothing binds it; the crate
        // reports an information.
        out.push(
            Diagnostic::info(&core_codes::INVALID_VALUE)
                .at(at)
                .phase(Phase::Validate),
        );
    }
    let reading = Reading::OWN
        .with_scalars(Scalars::Lenient)
        .with_null(NullHandling::AbsentSilently)
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut read = Read::new(reading, out)
        .profile(
            Profile::DECLARED_ORDER
                .with_unknown(UnknownAt::Last)
                .with_recovery(Recovery::Value),
        )
        .phase(Phase::Validate)
        .unknown_severity(Severity::Info);
    let mut atlas: TextureAtlas = read_document(&Node::new(merged, JsonPath::new()), &mut read)?;
    atlas.kind = kind;
    atlas.texture_name = texture_name;
    for key in IGNORED {
        if let Some(value) = atlas.extra.remove(key) {
            atlas.ignored.insert(key.to_owned(), value);
        }
    }
    Some(atlas)
}

/// An information of the crate's own at `node`: the game coerces or ignores the value in
/// silence.
fn invalid(read: &mut Read<'_>, at: &JsonPath) {
    read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(at.clone()));
}

/// The game's content-log error after which the atlas loads: a kept error.
fn kept(read: &mut Read<'_>, code: Code, at: &JsonPath) {
    read.findings_mut().push_kept_error(
        Diagnostic::error(code)
            .at(at.clone())
            .phase(Phase::Validate),
    );
}

/// `num_mip_levels`: an integer, a negative count told.
pub(super) fn mip_levels(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let levels = i32::from_json(node, read)?;
    if levels < 0 {
        invalid(read, node.path());
    }
    Some(levels)
}

/// A colour of the leather members, as written: the game reads them as text, whatever they are.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of a `read_with` fn"
)]
pub(super) fn silent_text(node: &Node<'_>, _: &mut Read<'_>) -> Option<String> {
    Some(to_text(Some(node.value())))
}

/// `texture_data`: an object of tiles; the game takes no tile of another value.
pub(super) fn tiles(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<AtlasKey, AtlasItem>> {
    if !node.value().is_object() {
        // The member names of a non-object are empty: no tile, silently.
        invalid(read, node.path());
        return None;
    }
    read_entries(node, read, tile)
}

/// One tile (`_readNode`): an entry that is no object, or has no `textures`, is dropped with
/// vanilla's error.
fn tile(node: &Node<'_>, read: &mut Read<'_>) -> Option<AtlasItem> {
    let key = node.path().last_key().unwrap_or_default();
    let Some(object) = node.value().as_object() else {
        read.push(Diagnostic::error(codes::ENTRY_NOT_OBJECT.with([key])).at(node.path().clone()));
        return None;
    };
    if object.get("textures").is_none_or(Value::is_null) {
        read.push(
            Diagnostic::error(codes::MISSING_TEXTURES.with([key.to_owned()]))
                .at(node.path().clone()),
        );
        return None;
    }
    AtlasItem::from_json(node, read)
}

/// `textures`: one element or an array of them; the game keeps a tile with an empty array and
/// no index, the crate tells it.
pub(super) fn textures(node: &Node<'_>, read: &mut Read<'_>) -> Option<AtlasTextures> {
    let textures = AtlasTextures::from_json(node, read)?;
    if matches!(&textures, OneOrMany::Many(elements) if elements.is_empty()) {
        invalid(read, node.path());
    }
    Some(textures)
}

impl FromJson for TextureElement {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(object) = value.as_object() {
            let variations = object
                .get("variations")
                .filter(|variations| variations.is_array());
            let Some(entries) = variations else {
                return TextureLayer::from_json(node, read)
                    .map(|layer| Self::Layer(Box::new(layer)));
            };
            let entries = Node::new(entries, node.path().child("variations"));
            let variations = Vec::from_json(&entries, read)?;
            let members = read.object(node)?;
            let mut ignored = UnknownMembers::default();
            for (key, value) in read.unknown(&members, &["variations"], UnknownRule::Keep) {
                ignored.insert(key.to_owned(), value.clone());
            }
            return Some(Self::Variations {
                variations,
                ignored,
            });
        }
        if let Some(path) = value.as_string() {
            return Some(Self::Path(path.as_str().to_owned()));
        }
        // The game gives the index no layer, silently; the crate reports an information.
        invalid(read, node.path());
        Some(Self::Invalid(value.clone()))
    }
}

impl FromJson for VariationEntry {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if value.is_object() {
            TextureLayer::from_json(node, read).map(|layer| Self::Layer(Box::new(layer)))
        } else if let Some(path) = value.as_string() {
            Some(Self::Path(path.as_str().to_owned()))
        } else {
            // The game raises a debug assert; the empty layer is kept: an information.
            invalid(read, node.path());
            Some(Self::Invalid(value.clone()))
        }
    }
}

/// `path`: the image; an empty value is none (jsoncpp's `empty()`).
pub(super) fn path(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    if is_empty(Some(node.value())) {
        return None;
    }
    String::from_json(node, read)
}

/// `tint_map_path` and `overlay_color`: text, none when it is empty.
pub(super) fn non_empty_text(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    Some(String::from_json(node, read)?).filter(|text| !text.is_empty())
}

/// `tint_base_color`: the channels' colours; an empty value is none, anything but an object is
/// told.
pub(super) fn tint_base_color(node: &Node<'_>, read: &mut Read<'_>) -> Option<TintMapColor> {
    if is_empty(Some(node.value())) {
        return None;
    }
    if !node.value().is_object() {
        invalid(read, node.path());
        return None;
    }
    TintMapColor::from_json(node, read)
}

/// The weight (default 1.0), clamped to the weight range with vanilla's warnings.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of a `read_with` fn"
)]
pub(super) fn weight(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let weight = as_float(Some(node.value()), 0.0);
    // A NaN is raised to the minimum too.
    let (clamped, text, bound) = if weight >= MIN_WEIGHT {
        if weight > MAX_WEIGHT {
            (MAX_WEIGHT, &codes::WEIGHT_HIGH, "1000000")
        } else {
            return Some(weight);
        }
    } else {
        (MIN_WEIGHT, &codes::WEIGHT_LOW, "0.05")
    };
    // The game logs both texts at the content log's warning level and stores the clamped
    // weight: the game's warning.
    read.push(
        Diagnostic::error(text.with([bound]))
            .severity(Severity::Warning)
            .at(node.path().clone()),
    );
    Some(clamped)
}

impl FromJson for Tint {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().as_object().is_some_and(|map| !map.is_empty()) {
            TintMapColor::from_json(node, read).map(Self::Map)
        } else {
            Color::from_json(node, read).map(Self::Color)
        }
    }
}

/// The four channels of a tint map; the game reads the members it knows and ignores the rest,
/// which the crate tells first.
impl FromJson for TintMapColor {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let members = read.object(node)?;
        read.unknown(&members, &CHANNELS, UnknownRule::Report);
        let mut channel = |key: &str| {
            members
                .get(key)
                .filter(|member| !member.value().is_null())
                .and_then(|member| Color::from_json(&member, read))
        };
        Some(Self {
            r_color: channel("r_color"),
            g_color: channel("g_color"),
            b_color: channel("b_color"),
            a_color: channel("a_color"),
        })
    }
}

/// A colour: an array of three or more channels, or a text read as hex after its first
/// character; anything else is the game's error, and the colour is none.
impl FromJson for Color {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(components) = value.as_array().filter(|array| array.len() >= 3) {
            let mut bytes = [255u8; 4];
            for (index, slot) in bytes.iter_mut().enumerate() {
                if let Some(component) = components.get(index) {
                    *slot = component_byte(component, &node.path().index(index), read);
                }
            }
            let [r, g, b, a] = bytes;
            return Some(if components.len() == 3 {
                Self::Rgb([r, g, b])
            } else {
                Self::Rgba([r, g, b, a])
            });
        }
        match value.as_string().map(facet_value::VString::as_str) {
            // The game throws on `""`; the crate refuses it like any other bad colour.
            Some(text) if !text.is_empty() => Some(Self::Hex(strtouint32_hex(after_first(text)))),
            _ => {
                // The game logs this at the content log's error level; the atlas loads: a kept
                // error.
                kept(read, Code::from(&codes::BAD_COLOR), node.path());
                None
            }
        }
    }
}

/// One colour component: `null` is 255, an integer (or a bool) is clamped to a byte, and
/// anything else is 255 with vanilla's error.
fn component_byte(value: &Value, at: &JsonPath, read: &mut Read<'_>) -> u8 {
    if value.is_null() {
        return 255;
    }
    if let Some(flag) = value.as_bool() {
        return u8::from(flag);
    }
    if let Some(number) = value.as_number().filter(|number| !number.is_float()) {
        // An integer token fits 64 bits; one above `i64` is above a byte.
        let wide = number.to_i64().unwrap_or(i64::MAX);
        return u8::try_from(wide.clamp(0, 255)).unwrap_or(255);
    }
    // The game logs this at the content log's error level (as it does for the other colour
    // texts); the value is 255 and the atlas loads: a kept error.
    kept(read, Code::from(&codes::NOT_INTEGRAL), at);
    255
}

/// The isotropic setting: a bool, or an object of faces.
impl FromJson for Isotropic {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(all) = value.as_bool() {
            return Some(Self::All(all));
        }
        let Some(object) = value.as_object() else {
            // A number, a string or an array leaves the mask alone.
            invalid(read, node.path());
            return None;
        };
        // `side` present and not `null`, whatever its type, selects the key set.
        let with_side = object.get("side").is_some_and(|side| !side.is_null());
        let mut faces = IndexMap::new();
        for (key, on) in object {
            let at = node.path().child(key.as_str());
            let Some(face) = Face::from_name(key.as_str()) else {
                // The game is silent about an unknown face, so this is the crate's own
                // information.
                read.push(Diagnostic::info(&core_codes::UNRECOGNIZED_FIELD).at(at));
                continue;
            };
            if on.is_null() {
                continue;
            }
            match on.as_bool() {
                Some(on) if face_is_read(face, with_side) => {
                    faces.insert(face, on);
                }
                // A face outside the key set, or a value that is no bool: not read.
                _ => invalid(read, &at),
            }
        }
        // The key set is `side`'s even when its own value was no bool.
        if with_side && !faces.contains_key(&Face::Side) {
            faces.insert(Face::Side, false);
        }
        Some(Self::Faces(faces))
    }
}

// Leaves of the derived check and Molang walk: none holds a bound or Molang.
bedrock_addon_core::impl_schema_leaf!(
    TextureElement,
    VariationEntry,
    Tint,
    TintMapColor,
    Color,
    Isotropic
);
bedrock_addon_core::impl_no_molang!(
    TextureElement,
    VariationEntry,
    Tint,
    TintMapColor,
    Color,
    Isotropic
);
