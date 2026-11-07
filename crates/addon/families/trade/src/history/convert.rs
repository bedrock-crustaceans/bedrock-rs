//! The conversions of `v0 → v1_21_30` that read a legacy value: the quantity, the trader material
//! expansion and the biome filters.

use super::v0::TradeItem;
use crate::Quantity;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::UpgradeCx;
use facet_value::{VArray, VObject};

/// A legacy quantity in the newest form: read as a range of two ints (a number
/// `a..a`, `"a-b"`, `[a, b]` swapped when `b < a`, `{min, max}`), then `min = max(min, 1)` and
/// `max = max(max, min)`. Vanilla clamps silently; here a bound
/// below 1 is `InvalidValue` and a swap or a raised `max` is `ClampedValue` (informations).
/// Another JSON type reads as 1..1, as vanilla's, and is `None`.
pub(super) fn quantity(json: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<Quantity> {
    let Some(read) = parse_range(json) else {
        // Not taken, and said: the item is written without the member.
        d.push(
            Diagnostic::of(&core_codes::INVALID_TYPE)
                .severity(Severity::Info)
                .at(at.clone())
                .phase(Phase::Upgrade),
        );
        return None;
    };
    let (min, max) = (read.min.max(1), read.max.max(read.min.max(1)));
    let bound = |n: i64| u32::try_from(n).unwrap_or(u32::MAX);
    // A bound that is not the number the file writes: a fraction truncated, a number beyond
    // an `int` or beyond the newest form's `u32`, a value that is no number, a text that is
    // no range (vanilla reads each in silence).
    let beyond = [min, max].iter().any(|n| u32::try_from(*n).is_err());
    if read.min < 1 || read.max < 1 || read.inexact || beyond {
        d.push(
            Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(at.clone())
                .phase(Phase::Upgrade),
        );
    }
    // An object without `max` wrote no bound the raise changes: no warning.
    let wrote_max = !matches!(read.form, Form::Object { max: false, .. });
    // The order is judged on the bounds once each is at least 1 (a bound below 1 has its own
    // finding).
    if read.swapped || (read.max.max(1) < min && wrote_max) {
        d.push(
            Diagnostic::of(&core_codes::CLAMPED_VALUE)
                .at(at.clone())
                .phase(Phase::Upgrade),
        );
    }
    Some(match read.form {
        Form::Number => Quantity::Exact(bound(max)),
        // An absent bound of the object form stays absent unless the clamp changed it (the
        // newest default is 1 as well).
        Form::Object {
            min: has_min,
            max: has_max,
        } => Quantity::Range {
            min: (has_min || min != read.min).then(|| bound(min)),
            max: (has_max || max != read.max).then(|| bound(max)),
        },
        Form::Text | Form::Array => Quantity::Range {
            min: Some(bound(min)),
            max: Some(bound(max)),
        },
    })
}

/// The written form of a quantity.
enum Form {
    Number,
    Text,
    Array,
    /// Which bounds the object writes.
    Object {
        min: bool,
        max: bool,
    },
}

/// What the range reader read, before the clamp.
struct Read {
    min: i64,
    max: i64,
    /// The array form's two values were swapped.
    swapped: bool,
    /// A bound is not the number the file writes (see [`int`]), or the text is no range.
    inexact: bool,
    form: Form,
}

/// Reads a quantity as a range: `None` for a JSON type it does not read.
fn parse_range(json: &Value) -> Option<Read> {
    if json.as_number().is_some() {
        let (n, exact) = int(Some(json));
        return Some(Read {
            min: n,
            max: n,
            swapped: false,
            inexact: !exact,
            form: Form::Number,
        });
    }
    if let Some(text) = json.as_string() {
        let range = text_range(text.as_str());
        let (min, max) = range.unwrap_or((1, 1));
        return Some(Read {
            min,
            max,
            swapped: false,
            inexact: range.is_none(),
            form: Form::Text,
        });
    }
    if let Some(array) = json.as_array() {
        // `[0]..[1]`, a missing element reading 0.
        let ((a, a_exact), (b, b_exact)) = (int(array.get(0)), int(array.get(1)));
        return Some(Read {
            min: a.min(b),
            max: a.max(b),
            swapped: b < a,
            inexact: !(a_exact && b_exact && array.len() == 2),
            form: Form::Array,
        });
    }
    let object = json.as_object()?;
    // An absent bound is an uninitialised int in vanilla; here it is 1.
    let bound = |key: &str| object.get(key).map_or((1, true), |v| int(Some(v)));
    let ((min, min_exact), (max, max_exact)) = (bound("min"), bound("max"));
    Some(Read {
        min,
        max,
        swapped: false,
        inexact: !(min_exact && max_exact),
        form: Form::Object {
            min: object.contains_key("min"),
            max: object.contains_key("max"),
        },
    })
}

/// Reads the text form of a range: `"a-b"` or `"a"`.
fn text_range(text: &str) -> Option<(i64, i64)> {
    let text = text.trim();
    // The separator is the first `-` after the first character (a leading one is a sign).
    let split = text
        .char_indices()
        .skip(1)
        .find(|&(_, c)| c == '-')
        .map(|(i, _)| i);
    if let Some(i) = split {
        return Some((
            text[..i].trim().parse().ok()?,
            text[i + 1..].trim().parse().ok()?,
        ));
    }
    let n = text.parse().ok()?;
    Some((n, n))
}

/// Reads a value as an integer: an integer, a real truncated toward zero; 0 for anything else.
/// With it, whether that is the number the value writes: not for a fraction, a real beyond an
/// `int`, or a
/// value that is no number.
fn int(json: Option<&Value>) -> (i64, bool) {
    let Some(number) = json.and_then(Value::as_number) else {
        return (0, false);
    };
    // A real is asked first: `to_i64` also answers for a real that happens to be a whole
    // number an `i64` holds (−2^63 among them), which would then escape the clamp its
    // neighbours get.
    if !number.is_float()
        && let Some(n) = number.to_i64()
    {
        return (n, true);
    }
    let x = number.to_f64_lossy();
    // Clamped to `i32`, as the game's integer read is: the truncation is the reading.
    #[allow(clippy::cast_possible_truncation)]
    let n = x.clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i64;
    let exact = x.fract() == 0.0 && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&x);
    (n, exact)
}

/// Below this version `trader_material_type` is expanded.
pub(super) const BIOME_FILTERS: FormatVersion = FormatVersion::new(1, 18, 20);

/// The function the expansion applies to.
const TRADER_MATERIAL: &str = "trader_material_type";

/// The five choices of an expansion: the aux value appended to the item and the biomes of the
/// choice.
const MATERIAL_CHOICES: [(u8, &[&str]); 5] = [
    (0, &["plains"]),
    (1, &["taiga", "snow"]),
    (3, &["desert", "jungle"]),
    (4, &["savannah"]),
    (5, &["swamp"]),
];

/// The trader material expansion (the gate below 1.18.20): the five biome-filtered choices
/// `item` expands into, when its `functions` holds `{"function": "trader_material_type"}`.
/// Each takes `item` with `:<n>` appended, the price multiplier, the remaining functions (none
/// when the list is emptied) and its biomes. The source's `filters` is not copied (the choice's
/// biomes become its filter in the conversion that follows), and the quantity is the caller's:
/// it converts the source's once for every choice.
pub(super) fn material_choices(item: &TradeItem) -> Option<Vec<TradeItem>> {
    let functions = item.functions.as_ref()?.as_array()?;
    let is_material = |function: &Value| {
        function
            .as_object()
            .and_then(|object| object.get("function"))
            .and_then(Value::as_string)
            .is_some_and(|name| name.as_str() == TRADER_MATERIAL)
    };
    if !functions.iter().any(is_material) {
        return None;
    }
    let mut remaining = VArray::new();
    for function in functions.iter().filter(|f| !is_material(f)) {
        remaining.push(function.clone());
    }
    let remaining = (!remaining.is_empty()).then(|| Value::from(remaining));
    let name = item.item.as_deref().unwrap_or_default();
    let choices = MATERIAL_CHOICES
        .iter()
        .map(|(aux, biomes)| {
            let mut names = VArray::new();
            for biome in *biomes {
                names.push(Value::from(*biome));
            }
            TradeItem {
                quantity: None,
                price_multiplier: item.price_multiplier,
                item: Some(format!("{name}:{aux}")),
                functions: remaining.clone(),
                filters: None,
                biomes: Some(Value::from(names)),
            }
        })
        .collect();
    Some(choices)
}

/// Below this version `biomes` becomes a filter.
pub(super) const GENERIC_FILTERS: FormatVersion = FormatVersion::new(1, 20, 30);

/// The mark variant of each biome name the biome filter conversion knows.
const MARK_VARIANTS: [(&str, i32); 7] = [
    ("plains", 0),
    ("desert", 1),
    ("jungle", 2),
    ("savannah", 3),
    ("snow", 4),
    ("swamp", 5),
    ("taiga", 6),
];

/// The biome filter conversion (the gate below 1.20.30): each known name of `biomes`
/// (a string or a list) becomes `{"test": "is_mark_variant", "subject": "self", "operator":
/// "=", "value": n}`; when at least one does, `filters` is replaced by `{"any_of": [...]}`.
/// Unknown names add nothing; `biomes` itself is no member of the newest item.
pub(super) fn biomes_to_filters(item: &mut TradeItem) {
    let Some(biomes) = &item.biomes else {
        return;
    };
    let names: Vec<&str> = match biomes.as_array() {
        Some(list) => list
            .iter()
            .filter_map(|v| v.as_string().map(facet_value::VString::as_str))
            .collect(),
        None => biomes
            .as_string()
            .map(facet_value::VString::as_str)
            .into_iter()
            .collect(),
    };
    let mut tests = VArray::new();
    for name in names {
        let Some(&(_, variant)) = MARK_VARIANTS.iter().find(|(known, _)| *known == name) else {
            continue;
        };
        let mut test = VObject::new();
        test.insert("test", Value::from("is_mark_variant"));
        test.insert("subject", Value::from("self"));
        test.insert("operator", Value::from("="));
        test.insert("value", Value::from(variant));
        tests.push(Value::from(test));
    }
    if !tests.is_empty() {
        let mut filters = VObject::new();
        filters.insert("any_of", Value::from(tests));
        item.filters = Some(Value::from(filters));
    }
}

/// What the two gates take from the item at `at` as the file wrote it, each told at the
/// member (the game says nothing, so these are informations): the item's value is what the
/// game ends up with.
///
/// * `filters` beside a `trader_material_type` function below 1.18.20, or beside a `biomes`
///   with a known name below 1.20.30: not copied, or replaced by the `any_of` of the biomes
///   (`DroppedField` at `filters`).
/// * `biomes` beside the function (the choices bring their own) and `biomes` of a file from
///   1.20.30 on, which no gate reads (`DroppedField` at `biomes`).
/// * a `biomes` that is neither a name nor a list (`InvalidType` at `biomes`), an element
///   that is no name (`InvalidType` at the element), a name the gate does not know
///   (`InvalidValue` at the name): each adds nothing.
pub(super) fn tell_lost(item: &TradeItem, expanded: bool, at: &JsonPath, cx: &mut UpgradeCx<'_>) {
    let member = |key: &str| at.child(key);
    let dropped = Code::from(&core_codes::DROPPED_FIELD);
    // The game's legacy parser and slice upgrade read and drop these silently and log nothing
    // of them, so the crate reports an information.
    let note = |code, at| Diagnostic::info(code).at(at).phase(Phase::Upgrade);
    // The expansion reads the function's name alone: any other member of the entry goes
    // with it.
    if expanded {
        let entries = item.functions.as_ref().and_then(Value::as_array);
        for (index, function) in entries.into_iter().flatten().enumerate() {
            let Some(object) = function.as_object() else {
                continue;
            };
            let material = object
                .get("function")
                .and_then(Value::as_string)
                .is_some_and(|name| name.as_str() == TRADER_MATERIAL);
            for key in object
                .keys()
                .filter(|key| material && key.as_str() != "function")
            {
                let path = member("functions").index(index).child(key.as_str());
                cx.diagnostics
                    .push(note(Code::from(&core_codes::UNRECOGNIZED_FIELD), path));
            }
        }
    }
    let Some(biomes) = &item.biomes else {
        if expanded && item.filters.is_some() {
            cx.diagnostics.push(note(dropped, member("filters")));
        }
        return;
    };
    if expanded || cx.original_version >= GENERIC_FILTERS {
        cx.diagnostics.push(note(dropped.clone(), member("biomes")));
        if expanded && item.filters.is_some() {
            cx.diagnostics.push(note(dropped, member("filters")));
        }
        return;
    }
    let known = |name: &str| MARK_VARIANTS.iter().any(|(known, _)| *known == name);
    let at_biomes = member("biomes");
    let mut any = false;
    if let Some(list) = biomes.as_array() {
        if list.is_empty() {
            // A list without a name makes no filter and goes with the member: noted (the
            // game makes none of it either).
            cx.diagnostics.push(note(
                Code::from(&core_codes::DROPPED_FIELD),
                at_biomes.clone(),
            ));
        }
        for (index, element) in list.iter().enumerate() {
            let at_element = at_biomes.index(index);
            match element.as_string() {
                Some(name) if known(name.as_str()) => any = true,
                Some(_) => cx.diagnostics.push(note(
                    Code::from(&core_codes::INVALID_VALUE),
                    at_element.clone(),
                )),
                None => cx.diagnostics.push(note(
                    Code::from(&core_codes::INVALID_TYPE),
                    at_element.clone(),
                )),
            }
        }
    } else if let Some(name) = biomes.as_string() {
        if known(name.as_str()) {
            any = true;
        } else {
            cx.diagnostics.push(note(
                Code::from(&core_codes::INVALID_VALUE),
                at_biomes.clone(),
            ));
        }
    } else {
        cx.diagnostics.push(note(
            Code::from(&core_codes::INVALID_TYPE),
            at_biomes.clone(),
        ));
    }
    if any && item.filters.is_some() {
        cx.diagnostics.push(note(dropped, member("filters")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> i64 {
        let value: Value = bedrock_addon_core::json::from_str(text).unwrap();
        int(Some(&value)).0
    }

    /// A real is truncated and clamped to an `int` whatever its size: the reals at −2^63,
    /// which fit an `i64` exactly, are no exception.
    #[test]
    fn a_real_beyond_an_int_is_clamped_whatever_its_size() {
        for (text, expected) in [("7", 7), ("-3", -3), ("2.9", 2), ("-2.9", -2), ("4e0", 4)] {
            assert_eq!(read(text), expected, "{text}");
        }
        for text in [
            "-9223372036854775808.0",
            "-9.223372036854775808e18",
            // An integer token beyond 64 bits is the double it spells: −2^63.
            "-9223372036854775809",
            "-18446744073709551616",
            "-1e10",
            "-1e300",
        ] {
            assert_eq!(read(text), i64::from(i32::MIN), "{text}");
        }
        for text in [
            "1e10",
            "9223372036854775808.0",
            "18446744073709551616",
            "1e300",
        ] {
            assert_eq!(read(text), i64::from(i32::MAX), "{text}");
        }
        // An integer token that fits 64 bits is itself.
        assert_eq!(read("5000000000"), 5_000_000_000);
        assert_eq!(read("-9223372036854775808"), i64::MIN);
        assert_eq!(int(None), (0, false));
        assert_eq!(read("\"7\""), 0);
    }
}
