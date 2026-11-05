//! The value types several particle components share.

use facet_value::VArray;
use indexmap::IndexMap;

use crate::read::{Era, Members, Node, Opt, Reader, Writer, object};
use crate::texts;
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{Molang, MolangVisitor};
use bedrock_addon_shared_shapes::OneOrMany;

/// The events a component fires: one event name, or a list of them.
///
/// JSON: a string, or an array of strings. The names are keys of the effect's `events`.
pub type VectorEvents = OneOrMany<String>;

impl Node for VectorEvents {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        if let Some(name) = value.as_string() {
            return Some(Self::One(name.as_str().to_owned()));
        }
        let Some(elements) = value.as_array() else {
            r.wrong_type(value, Some(texts::EVENT_FIELD));
            return None;
        };
        let mut names = Vec::with_capacity(elements.len());
        let mut places = Vec::with_capacity(elements.len());
        for (i, element) in elements.iter().enumerate() {
            match element.as_string() {
                Some(name) => {
                    names.push(name.as_str().to_owned());
                    places.push(i);
                }
                None => r.index(i, |r| r.wrong_type(element, Some(texts::EVENT_ELEMENT))),
            }
        }
        r.kept(&places);
        Some(Self::Many(names))
    }

    fn fallback(_: Opt) -> Self {
        Self::Many(Vec::new())
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(match self {
            Self::One(name) => Value::from(name.as_str()),
            Self::Many(names) => names
                .iter()
                .map(|name| Value::from(name.as_str()))
                .collect::<VArray>()
                .into(),
        })
    }
}

/// The direction particles leave an emitter shape in.
///
/// JSON: `"inwards"`, `"outwards"`, or an array of three Molang values.
#[derive(Clone, Debug, PartialEq)]
pub enum EmitterDirection {
    /// Towards the centre of the shape.
    Inwards,
    /// Away from the centre of the shape.
    Outwards,
    /// A direction of its own, `[x, y, z]`.
    Custom([Molang; 3]),
}

impl Node for EmitterDirection {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        if value.is_array() {
            return <[Molang; 3]>::read(value, r, o).map(Self::Custom);
        }
        match value.as_string().map(facet_value::VString::as_str) {
            Some("inwards") => Some(Self::Inwards),
            Some("outwards") => Some(Self::Outwards),
            Some(_) if !r.legacy() => {
                r.error(DiagnosticCode::InvalidValue { text: None });
                None
            }
            _ => {
                r.wrong_type(value, Some(texts::DIRECTION_INVALID));
                None
            }
        }
    }

    fn fallback(_: Opt) -> Self {
        Self::Outwards
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::Inwards => Ok(Value::from("inwards")),
            Self::Outwards => Ok(Value::from("outwards")),
            Self::Custom(direction) => direction.write(w),
        }
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        if let Self::Custom(direction) = self {
            direction.walk(era, o, v);
        }
    }
}

/// The normal of the plane a disc emitter lies in.
///
/// JSON: `"x"`, `"y"`, `"z"`, or an array of three Molang values.
#[derive(Clone, Debug, PartialEq)]
pub enum PlaneNormal {
    /// The x axis.
    X,
    /// The y axis.
    Y,
    /// The z axis.
    Z,
    /// A normal of its own, `[x, y, z]`.
    Custom([Molang; 3]),
}

impl Node for PlaneNormal {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        if value.is_array() {
            return <[Molang; 3]>::read(value, r, o).map(Self::Custom);
        }
        match value.as_string().map(facet_value::VString::as_str) {
            Some("x") => Some(Self::X),
            Some("y") => Some(Self::Y),
            Some("z") => Some(Self::Z),
            Some(_) if !r.legacy() => {
                r.error(DiagnosticCode::InvalidValue { text: None });
                None
            }
            _ => {
                r.wrong_type(value, Some(texts::PLANE_NORMAL_INVALID));
                None
            }
        }
    }

    fn fallback(_: Opt) -> Self {
        Self::Y
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::X => Ok(Value::from("x")),
            Self::Y => Ok(Value::from("y")),
            Self::Z => Ok(Value::from("z")),
            Self::Custom(normal) => normal.write(w),
        }
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        if let Self::Custom(normal) = self {
            normal.walk(era, o, v);
        }
    }
}

/// A colour.
///
/// JSON: an array of four Molang values (red, green, blue, alpha, each 0 to 1), an array of
/// three (alpha is 1), or a string `"#AARRGGBB"` / `"#RRGGBB"`.
#[derive(Clone, Debug, PartialEq)]
pub enum ColorExpr {
    /// `[red, green, blue, alpha]`.
    Rgba([Molang; 4]),
    /// `[red, green, blue]`; alpha is 1.
    Rgb([Molang; 3]),
    /// `"#AARRGGBB"` or `"#RRGGBB"`, as written.
    Hex(String),
}

/// The components of a channel default: a colour the legacy parser could not read is white.
const CHANNEL: Opt = Opt::with_default(1.0);

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

impl Node for ColorExpr {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        if let Some(text) = value.as_string() {
            let text = text.as_str();
            if hex(text).is_none() {
                // A 1.20.80 document logs the line and keeps the document; the string stays
                // as written.
                r.warn(texts::COLOR_STRING);
                if r.legacy() {
                    return None;
                }
            }
            return Some(Self::Hex(text.to_owned()));
        }
        match value.as_array().map(VArray::len) {
            Some(4) => <[Molang; 4]>::read(value, r, CHANNEL).map(Self::Rgba),
            Some(3) => <[Molang; 3]>::read(value, r, CHANNEL).map(Self::Rgb),
            Some(_) => {
                r.wrong_type(value, Some(texts::COLOR_COUNT));
                None
            }
            None => {
                r.wrong_type(value, Some(texts::COLOR_SHAPE));
                None
            }
        }
    }

    fn fallback(_: Opt) -> Self {
        Self::Rgba(<[Molang; 4]>::fallback(CHANNEL))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::Rgba(channels) => channels.write(w),
            Self::Rgb(channels) => channels.write(w),
            Self::Hex(text) => Ok(Value::from(text.as_str())),
        }
    }

    fn walk(&mut self, era: Era, _: Opt, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Rgba(channels) => channels.walk(era, CHANNEL, v),
            Self::Rgb(channels) => channels.walk(era, CHANNEL, v),
            Self::Hex(_) => {}
        }
    }
}

/// The colours a tint runs through.
///
/// JSON: an array of colours (evenly spaced from 0 to 1), or an object whose keys are the
/// positions as decimal strings (`"0.0"`, `"0.5"`).
#[derive(Clone, Debug, PartialEq)]
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

impl Node for Gradient {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let o = o.wrong(texts::GRADIENT);
        let (read, written) = if let Some(elements) = value.as_array() {
            let read = Vec::<ColorExpr>::read(value, r, o)?;
            (Self::List(read), elements.len())
        } else if let Some(members) = Members::of(value) {
            let read = IndexMap::<String, ColorExpr>::read(value, r, o)?;
            (Self::Keyed(read), members.iter().count())
        } else {
            r.wrong_type(value, o.wrong);
            return None;
        };
        let kept = match &read {
            Self::List(colors) => colors.len(),
            Self::Keyed(colors) => colors.len(),
        };
        // A colour the legacy parser could not read spoils the gradient.
        if kept != written {
            if r.legacy() {
                r.warn(texts::GRADIENT);
            }
            return None;
        }
        Some(read)
    }

    fn fallback(_: Opt) -> Self {
        Self::List(Vec::new())
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::List(colors) => colors.write(w),
            Self::Keyed(colors) => colors.write(w),
        }
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        match self {
            Self::List(colors) => colors.walk(era, o, v),
            Self::Keyed(colors) => colors.walk(era, o, v),
        }
    }
}

object! {
    /// A tint that changes over a particle's life: the gradient form of
    /// `minecraft:particle_appearance_tinting/color`.
    ///
    /// JSON: `{"gradient": …, "interpolant": …}`, both required.
    pub struct ColorGradient {
        /// The colours (JSON `gradient`).
        req gradient: Gradient = "gradient", Opt::NONE.missing(texts::GRADIENT);
        /// Where in the gradient the particle is, 0 to 1 for a list (JSON `interpolant`).
        req interpolant: Molang = "interpolant", Opt::required(0.0);
    }
}

/// `minecraft:particle_appearance_tinting/color`: one colour, or a gradient with the Molang
/// that picks from it.
///
/// JSON: a colour ([`ColorExpr`]), or `{"gradient", "interpolant"}`.
#[derive(Clone, Debug, PartialEq)]
pub enum TintColor {
    /// One colour.
    Color(ColorExpr),
    /// A gradient.
    Gradient(ColorGradient),
}

impl Node for TintColor {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let read = if value.is_object() {
            ColorGradient::read(value, r, o).map(Self::Gradient)
        } else {
            ColorExpr::read(value, r, o).map(Self::Color)
        };
        if read.is_none() && r.legacy() {
            r.warn(texts::INVALID_COLOR);
        }
        read
    }

    /// White: what the legacy parser gives a tint without a usable colour.
    fn fallback(o: Opt) -> Self {
        Self::Color(ColorExpr::fallback(o))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::Color(color) => color.write(w),
            Self::Gradient(gradient) => gradient.write(w),
        }
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Color(color) => color.walk(era, o, v),
            Self::Gradient(gradient) => gradient.walk(era, o, v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::{
        SINGLE, array_of, assert_reads_linear, assert_reads_linear_at, object_of, reading,
    };

    /// The gradient reader alone, in its keyed and its list form, on a value parsed
    /// beforehand.
    #[test]
    fn the_gradient_reader_reads_100_000_stops_in_linear_time() {
        for era in [Era::Legacy, Era::Cereal] {
            assert_reads_linear(
                &format!("keyed gradient ({era:?})"),
                |count| object_of(count, "0.", "[1, 1, 1, 1]"),
                |stops, count| {
                    let (read, found) = reading(era, |r| Gradient::read(stops, r, Opt::NONE));
                    assert!(matches!(&read, Some(Gradient::Keyed(read)) if read.len() == count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
            // A listed stop is read in a quarter of a microsecond: four times the entries.
            assert_reads_linear_at(
                &format!("listed gradient ({era:?})"),
                4 * SINGLE,
                |count| array_of(count, r##""#ff00ff""##),
                |stops, count| {
                    let (read, found) = reading(era, |r| Gradient::read(stops, r, Opt::NONE));
                    assert!(matches!(&read, Some(Gradient::List(read)) if read.len() == count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }
}
