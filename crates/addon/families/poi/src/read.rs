//! Reading the POI documents: each struct typed from its slice of the text.
//!
//! Every finding here is a parse finding: an error fails the file, as a parse error of the
//! game's loader does.

use facet::Facet;

use super::{PoiTag, PoiType};
use bedrock_addon_core::diagnostic::{
    Class, Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, parse_at, unknown_members};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

/// The version of both documents' only struct: 1.26.40.
pub(crate) const V1_26_40: FormatVersion = known::V1_26_40;

/// The member `key` of an object.
fn member<'v>(object: &'v Value, key: &str) -> Option<&'v Value> {
    object.as_object()?.get(key)
}

/// Whether `text` is a POI name, `^\w+:\w+$`: a namespace and a name, each one or more of
/// `A-Z`, `a-z`, `0-9` and `_`, around one `:`.
pub(crate) fn is_poi_name(text: &str) -> bool {
    let word = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    };
    text.split_once(':')
        .is_some_and(|(namespace, name)| word(namespace) && word(name))
}

/// Checks the name `value` at `at`: anything but a string is `InvalidType` (and `false`: the
/// typed parse cannot read the member), a string that is no POI name `InvalidValue`.
fn name(value: &Value, at: JsonPath, d: &mut Diagnostics) -> bool {
    match value.as_string() {
        None => {
            d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(at));
            false
        }
        Some(text) => {
            if !is_poi_name(text.as_str()) {
                d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(at));
            }
            true
        }
    }
}

/// Checks `description.identifier` of the payload `object`, when it is there; a missing one is
/// the typed parse's to report.
fn identifier(object: &Value, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let Some(value) = member(object, "description").and_then(|d| member(d, "identifier")) else {
        return true;
    };
    name(value, at.child("description").child("identifier"), d)
}

/// Checks the list of names `key` of the payload `object`, when it is there: an array, each
/// element a POI name.
fn names(object: &Value, key: &str, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let Some(value) = member(object, key) else {
        return true;
    };
    let at = at.child(key);
    let Some(list) = value.as_array() else {
        d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(at));
        return false;
    };
    let mut typed = true;
    for (i, entry) in list.iter().enumerate() {
        typed &= name(entry, at.index(i), d);
    }
    typed
}

/// Checks the byte member `key` of the payload `object`, when it is there: anything but an
/// integer token is `InvalidType`, an integer outside `0..=255` `InvalidValue`. `false` when the
/// member was reported.
fn byte(object: &Value, key: &str, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let Some(value) = member(object, key) else {
        return true;
    };
    let integer = value
        .as_number()
        .filter(|number| number.is_integer())
        .map(|number| number.to_u64().is_some_and(|n| u8::try_from(n).is_ok()));
    let code = match integer {
        Some(true) => return true,
        Some(false) => DiagnosticCode::InvalidValue { text: None },
        None => DiagnosticCode::InvalidType { text: None },
    };
    d.push(Diagnostic::error(code).at(at.child(key)));
    false
}

/// Parses `T` from `payload`, strictly: `check` judges the JSON first (the member types facet
/// would coerce or cannot report, a `null` included (every member's check takes a `null` for
/// the wrong type), and the value constraints; it returns `false` when the typed parse cannot
/// read a member it reported); a member no field takes is an `UnrecognizedField` graded by the
/// slice's mask (`Family::reading`: an error), and a failed typed parse is an error at the
/// failing member. `None` when anything but an unknown member was an error: the value of a
/// document whose only errors are its unknown members is built (the pipeline drops it for
/// them).
fn parse_strict<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    check: impl FnOnce(&Value, &JsonPath, &mut Diagnostics) -> bool,
) -> Option<T> {
    let errors = |d: &Diagnostics| {
        d.iter()
            .filter(|f| {
                f.severity >= Severity::Error
                    && !matches!(f.code, DiagnosticCode::UnrecognizedField { .. })
            })
            .count()
    };
    let before = errors(d);
    if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) {
        let typed = check(&json, &cx.at, d);
        for (path, _) in unknown_members(T::SHAPE, &json) {
            // The game's parser reports an unknown member, which this slice's strict mask makes
            // an error that refuses the file.
            d.push(
                Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
                    .at(cx.at.join(&path))
                    .severity(cx.load.grade(Class::Unknown)),
            );
        }
        if !typed {
            return None;
        }
    }
    let failed = errors(d) > before;
    match parse_at::<T>(payload) {
        Ok(value) => (!failed).then_some(value),
        Err(failure) => {
            d.push(failure.into_diagnostic(&cx.at, |code, _| code));
            None
        }
    }
}

impl Versioned for PoiType {
    const VERSION: FormatVersion = V1_26_40;
}

impl VersionStruct for PoiType {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_strict(payload, cx, d, |json, at, d| {
            let identifier = identifier(json, at, d);
            let tickets = byte(json, "tickets", at, d);
            let distance = byte(json, "usable_distance", at, d);
            let blocks = names(json, "blocks", at, d);
            let tags = names(json, "tags", at, d);
            identifier && tickets && distance && blocks && tags
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl Versioned for PoiTag {
    const VERSION: FormatVersion = V1_26_40;
}

impl VersionStruct for PoiTag {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_strict(payload, cx, d, |json, at, d| {
            let identifier = identifier(json, at, d);
            let children = names(json, "children", at, d);
            let parents = names(json, "parents", at, d);
            identifier && children && parents
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}
