//! Reading a [`TextureAtlas`] from the merged JSON of the stack, with the coercions of
//! vanilla's JSON library.

use facet_value::VObject;
use indexmap::IndexMap;

use super::codes;
use super::types::{
    AtlasItem, AtlasKind, AtlasTextures, Color, Face, Isotropic, MAX_WEIGHT, MIN_WEIGHT,
    TextureAtlas, TextureElement, TextureLayer, Tint, TintMapColor, VariationEntry, face_is_read,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::AtlasKey;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::jsoncpp::lenient::{
    as_bool, as_float, as_int, float_is_clamped, int_is_coerced, is_empty, non_null,
    strtouint32_hex, to_text,
};

/// The members of the root that vanilla's own files write and nothing reads.
const IGNORED: [&str; 2] = ["resource_pack_name", "format_version"];
/// The members of the root that `_createAtlas` reads.
const ROOT: [&str; 6] = [
    "padding",
    "num_mip_levels",
    "default_leather_color",
    "default_horse_leather_color",
    "texture_name",
    "texture_data",
];
/// The members of a tile that `_readNode` reads.
const NODE: [&str; 5] = [
    "textures",
    "quad",
    "additive",
    "mip_fade_amount",
    "mip_fade_color",
];
/// The members of a layer that the game reads.
const LAYER: [&str; 9] = [
    "path",
    "tint_map_path",
    "overlay_color",
    "tint_color",
    "tint_base_color",
    "mip_fade_amount",
    "mip_fade_color",
    "weight",
    "isotropic",
];
/// The members of a tint map.
const CHANNELS: [&str; 4] = ["r_color", "g_color", "b_color", "a_color"];

/// The findings of one read, each at its place in the merged document.
struct Reader<'a> {
    out: &'a mut Diagnostics,
    path: JsonPath,
}

impl Reader<'_> {
    /// A finding at the current path.
    fn report(&mut self, code: DiagnosticCode, severity: Severity) {
        self.out.push(
            Diagnostic::error(code)
                .severity(severity)
                .at(self.path.clone())
                .phase(Phase::Validate),
        );
    }

    /// A finding at member `key` of the current path.
    fn report_at(&mut self, key: &str, code: DiagnosticCode, severity: Severity) {
        self.path.push(key);
        self.report(code, severity);
        self.path.pop();
    }

    /// The game's content-log error after which the atlas loads: a kept error.
    fn kept(&mut self, code: DiagnosticCode) {
        self.out.push_kept_error(
            Diagnostic::error(code)
                .at(self.path.clone())
                .phase(Phase::Validate),
        );
    }

    /// A lint of the crate's own at member `key`: vanilla coerces or ignores the value silently, an
    /// information.
    fn invalid(&mut self, key: &str) {
        self.report_at(
            key,
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
    }

    /// Runs `read` one member down.
    fn within<T>(&mut self, key: &str, read: impl FnOnce(&mut Self) -> T) -> T {
        self.path.push(key);
        let value = read(self);
        self.path.pop();
        value
    }

    /// Runs `read` one array element down.
    fn within_index<T>(&mut self, index: usize, read: impl FnOnce(&mut Self) -> T) -> T {
        self.path.push_index(index);
        let value = read(self);
        self.path.pop();
        value
    }

    /// The members of `object` outside `known`, each reported as unrecognized.
    fn unknown(&mut self, object: &VObject, known: &[&str]) -> UnknownMembers {
        let mut extra = UnknownMembers::default();
        for (key, value) in object {
            if !known.contains(&key.as_str()) {
                // The game reads the members it knows and ignores the rest.
                self.report_at(
                    key.as_str(),
                    DiagnosticCode::from(&core_codes::UNRECOGNIZED_FIELD),
                    Severity::Info,
                );
                extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
        extra
    }
}

/// Reads the atlas of `kind` from the merged document; `None`, with vanilla's error, when the
/// document names no `texture_name` (vanilla then builds no atlas).
pub(super) fn read_atlas(
    kind: AtlasKind,
    merged: &Value,
    out: &mut Diagnostics,
) -> Option<TextureAtlas> {
    let mut reader = Reader {
        out,
        path: JsonPath::new(),
    };
    // A `null` `texture_name` counts as present, and reads as `""`.
    let root = merged.as_object();
    let Some((root, name)) = root.and_then(|root| Some((root, root.get("texture_name")?))) else {
        reader.report(
            DiagnosticCode::from(codes::NO_TEXTURE_NAME.with([kind.path().to_owned()])),
            Severity::Error,
        );
        return None;
    };
    let texture_name = to_text(Some(name));
    if texture_name.is_empty() {
        // The game raises a debug assert here; a release build goes on, so this is an
        // information with its text.
        reader.report_at(
            "texture_name",
            DiagnosticCode::from(&codes::NO_TEXTURE_PAGE),
            Severity::Info,
        );
    } else if texture_name != kind.expected_name() {
        // The game registers the page under the other name and nothing binds it; the crate
        // reports an information.
        reader.invalid("texture_name");
    }
    let padding = reader.int(root, "padding");
    let num_mip_levels = reader.int(root, "num_mip_levels");
    if num_mip_levels.is_some_and(|levels| levels < 0) {
        reader.invalid("num_mip_levels");
    }
    let texture_data = match root.get("texture_data") {
        None => None,
        Some(data) if data.is_null() => None,
        Some(data) => {
            let tiles = data
                .as_object()
                .map(|data| reader.within("texture_data", |reader| reader.tiles(data)));
            if tiles.is_none() {
                // The member names of a non-object are empty: no tile, silently.
                reader.invalid("texture_data");
            }
            tiles
        }
    };
    let mut ignored = UnknownMembers::default();
    for key in IGNORED {
        if let Some(value) = root.get(key) {
            ignored.insert(key.to_owned(), value.clone());
        }
    }
    let known: Vec<&str> = ROOT.into_iter().chain(IGNORED).collect();
    Some(TextureAtlas {
        kind,
        texture_name,
        padding,
        num_mip_levels,
        default_leather_color: root.get("default_leather_color").map(|v| to_text(Some(v))),
        default_horse_leather_color: root
            .get("default_horse_leather_color")
            .map(|v| to_text(Some(v))),
        texture_data,
        ignored,
        extra: reader.unknown(root, &known),
    })
}

impl Reader<'_> {
    /// A present member read as an integer defaulting to 0: a value that is no integer of 32
    /// bits is coerced, with an information of the crate's own.
    fn int(&mut self, object: &VObject, key: &str) -> Option<i32> {
        let value = object.get(key)?;
        if value.is_null() || int_is_coerced(value) {
            self.invalid(key);
        }
        Some(as_int(Some(value), 0))
    }

    /// A present member read as a bool defaulting to `false`; anything but a bool or a number is
    /// coerced, with an information of the crate's own.
    fn flag(&mut self, object: &VObject, key: &str) -> Option<bool> {
        let value = non_null(object.get(key))?;
        if !value.is_bool() && !value.is_number() {
            self.invalid(key);
        }
        Some(as_bool(Some(value), false))
    }

    /// `asFloat(default)` of a present member.
    fn float(&mut self, object: &VObject, key: &str, default: f32) -> Option<f32> {
        let value = non_null(object.get(key))?;
        // The crate reports a value that is no number, or a number beyond a float (read as the
        // largest finite one).
        if !value.is_number() || float_is_clamped(value) {
            self.invalid(key);
        }
        Some(as_float(Some(value), default))
    }

    /// The tiles of `texture_data`: an entry that is no object, or has no `textures`, is
    /// dropped with vanilla's error.
    fn tiles(&mut self, data: &VObject) -> IndexMap<AtlasKey, AtlasItem> {
        let mut tiles = IndexMap::new();
        for (key, value) in data {
            let key = key.as_str();
            let Some(node) = value.as_object() else {
                self.report_at(
                    key,
                    DiagnosticCode::from(codes::ENTRY_NOT_OBJECT.with([key])),
                    Severity::Error,
                );
                continue;
            };
            if let Some(item) = self.within(key, |reader| reader.tile(key, node)) {
                tiles.insert(AtlasKey::parse(key), item);
            }
        }
        tiles
    }

    /// One tile (`_readNode`).
    fn tile(&mut self, key: &str, node: &VObject) -> Option<AtlasItem> {
        let Some(textures) = non_null(node.get("textures")) else {
            self.report(
                DiagnosticCode::from(codes::MISSING_TEXTURES.with([key.to_owned()])),
                Severity::Error,
            );
            return None;
        };
        let quad = self.flag(node, "quad");
        let additive = self.flag(node, "additive");
        let mip_fade_amount = self.float(node, "mip_fade_amount", 0.0);
        let mip_fade_color = self.color(node, "mip_fade_color");
        let textures = self.within("textures", |reader| match textures.as_array() {
            Some(elements) => {
                if elements.is_empty() {
                    // The game keeps the tile with no index (a block that names it gets a
                    // content-log warning); the crate reports an information here.
                    reader.report(
                        DiagnosticCode::from(&core_codes::INVALID_VALUE),
                        Severity::Info,
                    );
                }
                AtlasTextures::Many(
                    elements
                        .as_slice()
                        .iter()
                        .enumerate()
                        .map(|(index, element)| {
                            reader.within_index(index, |reader| reader.element(element))
                        })
                        .collect(),
                )
            }
            None => AtlasTextures::One(reader.element(textures)),
        });
        Some(AtlasItem {
            textures,
            quad,
            additive,
            mip_fade_amount,
            mip_fade_color,
            extra: self.unknown(node, &NODE),
        })
    }

    /// One element of `textures`.
    fn element(&mut self, element: &Value) -> TextureElement {
        if let Some(object) = element.as_object() {
            let variations = non_null(object.get("variations")).and_then(Value::as_array);
            let Some(entries) = variations else {
                return TextureElement::Layer(Box::new(self.layer(object)));
            };
            let variations = self.within("variations", |reader| {
                entries
                    .as_slice()
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| reader.within_index(index, |reader| reader.entry(entry)))
                    .collect()
            });
            return TextureElement::Variations {
                variations,
                ignored: self.unknown(object, &["variations"]),
            };
        }
        if let Some(path) = element.as_string() {
            return TextureElement::Path(path.as_str().to_owned());
        }
        // The game gives the index no layer, silently; the crate reports an information.
        self.report(
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
        TextureElement::Invalid(element.clone())
    }

    /// One entry of `variations`.
    fn entry(&mut self, entry: &Value) -> VariationEntry {
        if let Some(object) = entry.as_object() {
            VariationEntry::Layer(Box::new(self.layer(object)))
        } else if let Some(path) = entry.as_string() {
            VariationEntry::Path(path.as_str().to_owned())
        } else {
            // The game raises a debug assert; the empty layer is kept: an information.
            self.report(
                DiagnosticCode::from(&core_codes::INVALID_VALUE),
                Severity::Info,
            );
            VariationEntry::Invalid(entry.clone())
        }
    }

    /// One object layer.
    fn layer(&mut self, object: &VObject) -> TextureLayer {
        let path = object.get("path").filter(|path| !is_empty(Some(path)));
        if path.is_some_and(|path| !path.is_string()) {
            self.invalid("path");
        }
        let tint_map_path = object.get("tint_map_path");
        if tint_map_path.is_some_and(|path| !path.is_string() && !path.is_null()) {
            self.invalid("tint_map_path");
        }
        let overlay_color = object.get("overlay_color");
        if overlay_color.is_some_and(|color| !color.is_string() && !color.is_null()) {
            self.invalid("overlay_color");
        }
        let non_empty = |value: Option<&Value>| Some(to_text(value)).filter(|s| !s.is_empty());
        let tint_color = match object.get("tint_color") {
            Some(tint) if !is_empty(Some(tint)) && tint.is_object() => tint
                .as_object()
                .map(|map| Tint::Map(self.within("tint_color", |reader| reader.tint_map(map)))),
            _ => self.color(object, "tint_color").map(Tint::Color),
        };
        let tint_base_color = match object.get("tint_base_color") {
            Some(base) if is_empty(Some(base)) => None,
            Some(base) => {
                let map = base
                    .as_object()
                    .map(|map| self.within("tint_base_color", |reader| reader.tint_map(map)));
                if map.is_none() {
                    self.invalid("tint_base_color");
                }
                map
            }
            None => None,
        };
        TextureLayer {
            path: path.map(|path| to_text(Some(path))),
            tint_map_path: non_empty(tint_map_path),
            overlay_color: non_empty(overlay_color),
            tint_color,
            tint_base_color,
            mip_fade_amount: self.float(object, "mip_fade_amount", 0.0),
            mip_fade_color: self.color(object, "mip_fade_color"),
            weight: object.get("weight").map(|weight| self.weight(weight)),
            isotropic: self.isotropic(object.get("isotropic")),
            extra: self.unknown(object, &LAYER),
        }
    }

    /// The weight (default 1.0), clamped to the weight range with vanilla's warnings.
    fn weight(&mut self, value: &Value) -> f32 {
        let weight = as_float(Some(value), 0.0);
        // A NaN is raised to the minimum too.
        let (clamped, text, bound) = if weight >= MIN_WEIGHT {
            if weight > MAX_WEIGHT {
                (MAX_WEIGHT, &codes::WEIGHT_HIGH, "1000000")
            } else {
                return weight;
            }
        } else {
            (MIN_WEIGHT, &codes::WEIGHT_LOW, "0.05")
        };
        // The game logs both texts at the content log's warning level and stores the clamped
        // weight: the game's warning.
        self.report_at(
            "weight",
            DiagnosticCode::from(text.with([bound])),
            Severity::Warning,
        );
        clamped
    }

    /// The four channels of a tint map.
    fn tint_map(&mut self, map: &VObject) -> TintMapColor {
        self.unknown(map, &CHANNELS);
        TintMapColor {
            r_color: self.color(map, "r_color"),
            g_color: self.color(map, "g_color"),
            b_color: self.color(map, "b_color"),
            a_color: self.color(map, "a_color"),
        }
    }

    /// The isotropic setting: a bool, or an object of faces.
    fn isotropic(&mut self, value: Option<&Value>) -> Option<Isotropic> {
        let value = non_null(value)?;
        if let Some(all) = value.as_bool() {
            return Some(Isotropic::All(all));
        }
        let Some(object) = value.as_object() else {
            // A number, a string or an array leaves the mask alone.
            self.invalid("isotropic");
            return None;
        };
        self.within("isotropic", |reader| {
            // `side` present and not `null`, whatever its type, selects the key set.
            let with_side = non_null(object.get("side")).is_some();
            let mut faces = IndexMap::new();
            for (key, on) in object {
                let Some(face) = Face::from_name(key.as_str()) else {
                    // The game is silent about an unknown face, so this is the crate's own
                    // information.
                    reader.report_at(
                        key.as_str(),
                        DiagnosticCode::from(&core_codes::UNRECOGNIZED_FIELD),
                        Severity::Info,
                    );
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
                    _ => reader.invalid(key.as_str()),
                }
            }
            // The key set is `side`'s even when its own value was no bool.
            if with_side && !faces.contains_key(&Face::Side) {
                faces.insert(Face::Side, false);
            }
            Some(Isotropic::Faces(faces))
        })
    }

    /// A colour at member `key`: `None` when it is absent, `null` or refused.
    fn color(&mut self, object: &VObject, key: &str) -> Option<Color> {
        let value = non_null(object.get(key))?;
        self.within(key, |reader| reader.color_value(value))
    }

    /// One colour value.
    fn color_value(&mut self, value: &Value) -> Option<Color> {
        if let Some(components) = value.as_array().filter(|array| array.len() >= 3) {
            let mut bytes = [255u8; 4];
            for (index, slot) in bytes.iter_mut().enumerate() {
                if let Some(component) = components.get(index) {
                    *slot = self.within_index(index, |reader| reader.component(component));
                }
            }
            let [r, g, b, a] = bytes;
            return Some(if components.len() == 3 {
                Color::Rgb([r, g, b])
            } else {
                Color::Rgba([r, g, b, a])
            });
        }
        match value.as_string().map(facet_value::VString::as_str) {
            // The game throws on `""`; the crate refuses it like any other bad colour.
            Some(text) if !text.is_empty() => {
                Some(Color::Hex(strtouint32_hex(super::types::after_first(text))))
            }
            _ => {
                // The game logs this at the content log's error level; the atlas loads: a kept
                // error.
                self.kept(DiagnosticCode::from(&codes::BAD_COLOR));
                None
            }
        }
    }

    /// One colour component: `null` is 255, an integer (or a bool) is clamped to a byte, and
    /// anything else is 255 with vanilla's error.
    fn component(&mut self, value: &Value) -> u8 {
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
        self.kept(DiagnosticCode::from(&codes::NOT_INTEGRAL));
        255
    }
}
