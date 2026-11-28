//! The value types several particle components share.

use facet_value::VArray;
use indexmap::IndexMap;

use crate::codes;
use crate::derived::{
    KEPT, L0, LEGACY, STRICT, elements, entries, legacy, line, molang_array_of, refuse, string,
    walk_each, walk_spec, wrong,
};
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor, Step};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_shared_shapes::OneOrMany;

/// The events a component fires: one event name, or a list of them.
///
/// JSON: a string, or an array of strings. The names are keys of the effect's `events`.
pub type VectorEvents = OneOrMany<String>;

/// A [`VectorEvents`]: an element that is no string is left out with the legacy parser's line.
pub(crate) fn vector_events(node: &Node<'_>, read: &mut Read<'_>) -> Option<VectorEvents> {
    if let Some(name) = node.value().as_string() {
        return Some(VectorEvents::One(name.as_str().to_owned()));
    }
    let names = elements(node, read, Some(&codes::EVENT_FIELD), |element, read| {
        string(element, read, Some(&codes::EVENT_ELEMENT))
    });
    names.map(VectorEvents::Many)
}

/// A map of [`VectorEvents`] by a decimal string, `code` the legacy parser's line for a value
/// that is no object.
pub(crate) fn events_by(
    node: &Node<'_>,
    read: &mut Read<'_>,
    code: &'static bedrock_addon_core::diagnostic::CodeDef<0>,
) -> Option<IndexMap<String, VectorEvents>> {
    entries(node, read, Some(code), |_, entry, read| {
        vector_events(entry, read)
    })
}

/// `timeline` (an emitter's and a particle's): the legacy parser's line for a value that is no
/// object is its own.
pub(crate) fn timeline(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<IndexMap<String, VectorEvents>> {
    events_by(node, read, &codes::TIMELINE)
}

/// The direction particles leave an emitter shape in.
///
/// JSON: `"inwards"`, `"outwards"`, or an array of three Molang values.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum EmitterDirection {
    /// Towards the centre of the shape.
    #[facet(rename = "inwards")]
    Inwards,
    /// Away from the centre of the shape.
    #[facet(rename = "outwards")]
    Outwards,
    /// A direction of its own, `[x, y, z]`.
    Custom([Molang; 3]),
}

impl FromJson for EmitterDirection {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_array() {
            return molang_array_of(node, read, 0.0).map(Self::Custom);
        }
        match node.value().as_string().map(facet_value::VString::as_str) {
            Some("inwards") => Some(Self::Inwards),
            Some("outwards") => Some(Self::Outwards),
            Some(_) if !legacy(read) => {
                refuse(read, node.path(), Code::from(&core_codes::INVALID_VALUE));
                None
            }
            _ => {
                wrong(read, node, Some(&codes::DIRECTION_INVALID));
                None
            }
        }
    }
}

impl MolangFields for EmitterDirection {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        if let Self::Custom(direction) = self {
            walk_each(direction, &walk_spec(v, None), v);
        }
    }
}

bedrock_addon_core::impl_schema_leaf!(
    EmitterDirection,
    PlaneNormal,
    ColorExpr,
    Gradient,
    TintColor
);

/// The normal of the plane a disc emitter lies in.
///
/// JSON: `"x"`, `"y"`, `"z"`, or an array of three Molang values.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum PlaneNormal {
    /// The x axis.
    #[facet(rename = "x")]
    X,
    /// The y axis.
    #[facet(rename = "y")]
    Y,
    /// The z axis.
    #[facet(rename = "z")]
    Z,
    /// A normal of its own, `[x, y, z]`.
    Custom([Molang; 3]),
}

impl FromJson for PlaneNormal {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_array() {
            return molang_array_of(node, read, 0.0).map(Self::Custom);
        }
        match node.value().as_string().map(facet_value::VString::as_str) {
            Some("x") => Some(Self::X),
            Some("y") => Some(Self::Y),
            Some("z") => Some(Self::Z),
            Some(_) if !legacy(read) => {
                refuse(read, node.path(), Code::from(&core_codes::INVALID_VALUE));
                None
            }
            _ => {
                wrong(read, node, Some(&codes::PLANE_NORMAL_INVALID));
                None
            }
        }
    }
}

impl MolangFields for PlaneNormal {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        if let Self::Custom(normal) = self {
            walk_each(normal, &walk_spec(v, None), v);
        }
    }
}

/// A colour.
///
/// JSON: an array of four Molang values (red, green, blue, alpha, each 0 to 1), an array of
/// three (alpha is 1), or a string `"#AARRGGBB"` / `"#RRGGBB"`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum ColorExpr {
    /// `[red, green, blue, alpha]`.
    Rgba([Molang; 4]),
    /// `[red, green, blue]`; alpha is 1.
    Rgb([Molang; 3]),
    /// `"#AARRGGBB"` or `"#RRGGBB"`, as written.
    Hex(String),
}

/// The default of a channel: a colour the legacy parser could not read is white.
const CHANNEL: f32 = 1.0;

impl ColorExpr {
    /// The colour of a [`ColorExpr::Hex`] string as `[alpha, red, green, blue]` bytes: `#`
    /// and one to eight hexadecimal digits. `None` for another variant, and for a string
    /// vanilla logs "Color value string has problems!" for.
    pub fn argb(&self) -> Option<[u8; 4]> {
        let Self::Hex(text) = self else {
            return None;
        };
        hex(text).map(u32::to_be_bytes)
    }
}

/// The text after `#` read as a hexadecimal number.
fn hex(text: &str) -> Option<u32> {
    let digits = text.strip_prefix('#')?;
    // The radix parse refuses an empty text; a sign it would take is no digit.
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(digits, 16).ok()
}

impl FromJson for ColorExpr {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if let Some(text) = node.value().as_string() {
            let text = text.as_str();
            if hex(text).is_none() {
                // A 1.20.80 document logs the line and keeps the document; the string stays
                // as written.
                line(read, node.path(), &codes::COLOR_STRING);
                if legacy(read) {
                    return None;
                }
            }
            return Some(Self::Hex(text.to_owned()));
        }
        match node.value().as_array().map(VArray::len) {
            Some(4) => molang_array_of(node, read, CHANNEL).map(Self::Rgba),
            Some(3) => molang_array_of(node, read, CHANNEL).map(Self::Rgb),
            Some(_) => {
                wrong(read, node, Some(&codes::COLOR_COUNT));
                None
            }
            None => {
                wrong(read, node, Some(&codes::COLOR_SHAPE));
                None
            }
        }
    }
}

impl MolangFields for ColorExpr {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        let spec = walk_spec(v, Some(CHANNEL));
        match self {
            Self::Rgba(channels) => walk_each(channels, &spec, v),
            Self::Rgb(channels) => walk_each(channels, &spec, v),
            Self::Hex(_) => {}
        }
    }
}

/// The colours a tint runs through.
///
/// JSON: an array of colours (evenly spaced from 0 to 1), or an object whose keys are the
/// positions as decimal strings (`"0.0"`, `"0.5"`).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum Gradient {
    /// Evenly spaced colours.
    List(Vec<ColorExpr>),
    /// Colours at positions; the keys are read as floats and the stops sorted by them.
    Keyed(IndexMap<String, ColorExpr>),
}

impl Gradient {
    /// The stops in the order the game uses them: `(position, colour)`. A list's colours are
    /// spaced evenly from 0 to 1; a keyed gradient's are sorted by their key read as a float
    /// (C `atof`: a key that is no number is 0).
    pub fn stops(&self) -> Vec<(f32, &ColorExpr)> {
        match self {
            Self::List(colors) => {
                #[allow(clippy::cast_precision_loss, reason = "a gradient has few stops")]
                let last = colors.len().saturating_sub(1).max(1) as f32;
                colors
                    .iter()
                    .enumerate()
                    .map(|(i, color)| {
                        #[allow(clippy::cast_precision_loss, reason = "a gradient has few stops")]
                        let position = i as f32 / last;
                        (position, color)
                    })
                    .collect()
            }
            Self::Keyed(colors) => {
                let mut stops: Vec<(f32, &ColorExpr)> = colors
                    .iter()
                    .map(|(key, color)| (key.trim().parse().unwrap_or(0.0), color))
                    .collect();
                stops.sort_by(|a, b| a.0.total_cmp(&b.0));
                stops
            }
        }
    }
}

impl FromJson for Gradient {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let code = Some(&codes::GRADIENT);
        let (read_value, written) = match node.value() {
            value if value.is_array() => {
                let colors = elements(node, read, code, ColorExpr::from_json)?;
                (Self::List(colors), value.as_array().map_or(0, VArray::len))
            }
            value if value.is_object() => {
                let colors = entries(node, read, code, |_, entry, read| {
                    ColorExpr::from_json(entry, read)
                })?;
                (
                    Self::Keyed(colors),
                    value.as_object().map_or(0, facet_value::VObject::len),
                )
            }
            _ => {
                wrong(read, node, code);
                return None;
            }
        };
        let kept = match &read_value {
            Self::List(colors) => colors.len(),
            Self::Keyed(colors) => colors.len(),
        };
        // A colour the legacy parser could not read spoils the gradient.
        if kept != written {
            if legacy(read) {
                line(read, node.path(), &codes::GRADIENT);
            }
            return None;
        }
        Some(read_value)
    }
}

impl MolangFields for Gradient {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        match self {
            Self::List(colors) => {
                for (i, color) in colors.iter_mut().enumerate() {
                    v.enter(Step::Index(i));
                    color.visit_molang(v);
                    v.leave();
                }
            }
            Self::Keyed(colors) => {
                for (key, color) in colors.iter_mut() {
                    v.enter(Step::Key(key));
                    color.visit_molang(v);
                    v.leave();
                }
            }
        }
    }
}

/// A tint that changes over a particle's life: the gradient form of
/// `minecraft:particle_appearance_tinting/color`.
///
/// JSON: `{"gradient": …, "interpolant": …}`, both required.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct ColorGradient {
    /// The colours (JSON `gradient`).
    #[facet(
        default = Gradient::List(Vec::new()),
        schema::read_with = Gradient::from_json,
        schema::era(LEGACY, missing = codes::GRADIENT),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub gradient: Gradient,
    /// Where in the gradient the particle is, 0 to 1 for a list (JSON `interpolant`).
    #[facet(
        default = Molang::Const(0.0),
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub interpolant: Molang,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// `minecraft:particle_appearance_tinting/color`: one colour, or a gradient with the Molang
/// that picks from it.
///
/// JSON: a colour ([`ColorExpr`]), or `{"gradient", "interpolant"}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum TintColor {
    /// One colour.
    Color(ColorExpr),
    /// A gradient.
    Gradient(ColorGradient),
}

impl FromJson for TintColor {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let read_value = if node.value().is_object() {
            ColorGradient::from_json(node, read).map(Self::Gradient)
        } else {
            ColorExpr::from_json(node, read).map(Self::Color)
        };
        if read_value.is_none() && legacy(read) {
            line(read, node.path(), &codes::INVALID_COLOR);
        }
        read_value
    }
}

impl MolangFields for TintColor {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Color(color) => color.visit_molang(v),
            Self::Gradient(gradient) => gradient.visit_molang(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::{
        ERAS, SINGLE, array_of, assert_reads_linear, assert_reads_linear_at, object_of, reading,
        root,
    };

    /// The gradient reader alone, in its keyed and its list form, on a value parsed
    /// beforehand.
    #[test]
    fn the_gradient_reader_reads_100_000_stops_in_linear_time() {
        for (era, legacy) in ERAS {
            assert_reads_linear(
                &format!("keyed gradient ({era})"),
                |count| object_of(count, "0.", "[1, 1, 1, 1]"),
                |stops, count| {
                    let (read, found) = reading(legacy, |r| Gradient::from_json(&root(stops), r));
                    assert!(matches!(&read, Some(Gradient::Keyed(read)) if read.len() == count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
            // A listed stop is read in a quarter of a microsecond: four times the entries.
            assert_reads_linear_at(
                &format!("listed gradient ({era})"),
                4 * SINGLE,
                |count| array_of(count, r##""#ff00ff""##),
                |stops, count| {
                    let (read, found) = reading(legacy, |r| Gradient::from_json(&root(stops), r));
                    assert!(matches!(&read, Some(Gradient::List(read)) if read.len() == count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }
}
