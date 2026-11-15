//! Reading [`FogSettings`] from the text of a 1.26.20 payload, as the game's cereal loader
//! reads `FogParametersSrcV0`: every member but `description.identifier` is optional, a member
//! the schema does not know is kept and reported (the game ignores it without a word), and a wrong type or a
//! value outside its constraint fails the document. The settings types are the schema and read
//! on the derived read (an object fails by an error below it, and only then is a missing member
//! not told); what has a syntax of its own reads by a `read_with` here.

use super::{ColorNorm, FogColor, FogSettings, Keyframed, same_zero};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, parse_slice};
use bedrock_addon_core::json::{BeyondWidth, JsonSlice};
use bedrock_addon_core::schema::{FromJson, read_document};
use bedrock_addon_core::versioning::{Reading, Scalars};
use bedrock_addon_shared_shapes::Keyframe;
use bedrock_addon_shared_shapes::color::is_hex;

/// Reads one `minecraft:fog_settings` payload, findings below `at`; `None` (with an error in
/// `d`) when the cereal pass would fail the file.
pub(crate) fn read_settings(
    payload: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<FogSettings> {
    let text = parse_slice(payload, Options::default()).ok()?;
    let mut found = Diagnostics::default();
    // The cereal pass: a wrong type or a broken constraint is an error that fails the file;
    // an unknown member is kept and told as information (the game says nothing).
    let reading = Reading::DOCUMENT
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Error);
    let mut read = Read::new(reading, &mut found)
        .unknown_severity(Severity::Info)
        .profile(Profile::TREE);
    let settings = read_document::<FogSettings>(&Node::parsed(&text, at.clone()), &mut read);
    drop(read);
    let failed = found.has_errors();
    d.extend(found);
    settings.filter(|_| !failed)
}

/// `code` at `node`, spanning it.
fn tell(read: &mut Read<'_>, code: Code, severity: Severity, node: &Node<'_>) {
    let mut finding = Diagnostic::error(code)
        .severity(severity)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// A value outside the member's constraint.
fn violation(read: &mut Read<'_>, node: &Node<'_>) {
    tell(
        read,
        Code::from(&core_codes::INVALID_VALUE),
        Severity::Error,
        node,
    );
}

/// The token of a node as written.
fn token<'v>(node: &Node<'v>) -> &'v str {
    node.text().map_or("", |text| text.text)
}

/// A number within `[min, max]` (`BasicNumericConstraint`).
fn ranged(read: &mut Read<'_>, node: &Node<'_>, min: f32, max: f32) -> Option<f32> {
    let x = f32::from_json(node, read)?;
    if x < min || x > max {
        violation(read, node);
        return None;
    }
    Some(x)
}

/// A number of at least 0 (`BasicNumericConstraint<double>::min(0.0)`): `fog_end`, the seconds.
pub(crate) fn positive(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    ranged(read, node, 0.0, f32::MAX)
}

/// A number of 0 to 1: the percents.
pub(crate) fn percent(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    ranged(read, node, 0.0, 1.0)
}

/// `max_density`: 0 to 1, or by time key.
pub(crate) fn density(node: &Node<'_>, read: &mut Read<'_>) -> Option<Keyframed<f32>> {
    keyframed(read, node, |read, v| ranged(read, v, 0.0, 1.0))
}

/// A density height (both share one constraint): −64 to 320, or by time key.
pub(crate) fn height(node: &Node<'_>, read: &mut Read<'_>) -> Option<Keyframed<f32>> {
    keyframed(read, node, |read, v| ranged(read, v, -64.0, 320.0))
}

/// `henyey_greenstein_g`: −1 to 1, or by time key.
pub(crate) fn scattering_g(node: &Node<'_>, read: &mut Read<'_>) -> Option<Keyframed<f32>> {
    keyframed(read, node, |read, v| ranged(read, v, -1.0, 1.0))
}

/// The string form of a colour, or its constraint violation.
fn hex(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let written = String::from_json(node, read)?;
    // `SharedTypes::REGEX_RGB_COLOR`: `^#[a-fA-F0-9]{6}$`.
    if is_hex(&written, 6) {
        Some(written)
    } else {
        violation(read, node);
        None
    }
}

/// Exactly three elements (`std::array<_, 3>`).
fn triple<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<[Node<'v>; 3]> {
    let elements = read.array(node)?;
    let all: Vec<Node<'v>> = elements.iter().collect();
    if let Ok(three) = <[Node<'v>; 3]>::try_from(all) {
        Some(three)
    } else {
        violation(read, node);
        None
    }
}

/// `Color255RGB`: `#RRGGBB`, or three integers, each clamped to 0–255 as
/// `colorFromIntArray3` clamps it (with a `ClampedValue` info, ours). A real (`0.5`, `1.0`) or a
/// string is refused (ours; what the game's integer reader does with a real is not known).
pub(crate) fn fog_color(node: &Node<'_>, read: &mut Read<'_>) -> Option<FogColor> {
    let value = node.value();
    if value.is_string() {
        return hex(read, node).map(FogColor::Hex);
    }
    if !value.is_array() {
        read.wrong_type(node);
        return None;
    }
    let nodes = triple(read, node)?;
    let mut rgb = [0_u8; 3];
    let mut ok = true;
    for (index, element) in nodes.iter().enumerate() {
        let written = element
            .value()
            .as_number()
            .and_then(|_| token(element).parse::<i64>().ok());
        if let Some(written) = written {
            let clamped = written.clamp(0, 255);
            if clamped != written {
                tell(
                    read,
                    Code::from(&core_codes::CLAMPED_VALUE),
                    Severity::Info,
                    element,
                );
            }
            rgb[index] = u8::try_from(clamped).unwrap_or(u8::MAX);
        } else {
            violation(read, element);
            ok = false;
        }
    }
    ok.then_some(FogColor::Array(rgb))
}

/// `ColorNormRGB`: `#RRGGBB`, or three numbers (clamped to 0–1 when used).
fn color_norm(read: &mut Read<'_>, node: &Node<'_>) -> Option<ColorNorm> {
    let value = node.value();
    if value.is_string() {
        return hex(read, node).map(ColorNorm::Hex);
    }
    if !value.is_array() {
        read.wrong_type(node);
        return None;
    }
    let nodes = triple(read, node)?;
    let mut rgb = [0.0_f32; 3];
    let mut ok = true;
    for (index, element) in nodes.iter().enumerate() {
        match f32::from_json(element, read) {
            Some(x) => rgb[index] = x,
            None => ok = false,
        }
    }
    ok.then_some(ColorNorm::Array(rgb))
}

/// A coefficient: a colour, or colours by time key. An object is the keyed form.
pub(crate) fn coefficient(node: &Node<'_>, read: &mut Read<'_>) -> Option<Keyframed<ColorNorm>> {
    keyframed(read, node, color_norm)
}

/// `CerealHelpers::TimeKeyframes<float, T>`: one value, or an object whose keys are numbers
/// written as strings, sorted by key. Each value obeys the member's own rule. Of several keys
/// that are one time the last stands; each of the others is an Info `DroppedField` at its key.
fn keyframed<'v, T>(
    read: &mut Read<'_>,
    node: &Node<'v>,
    mut value: impl FnMut(&mut Read<'_>, &Node<'v>) -> Option<T>,
) -> Option<Keyframed<T>> {
    if !node.value().is_object() {
        return value(read, node).map(Keyframed::Single);
    }
    let members = read.object(node)?;
    let mut keys = Vec::with_capacity(members.len());
    let mut ok = true;
    for (index, (key, member)) in members.iter().enumerate() {
        // A key Rust reads as `nan` or `inf`, or one too large for a float, is no time.
        let time = key.trim().parse::<f32>().ok().filter(|t| t.is_finite());
        if time.is_none() {
            violation(read, &member);
        }
        let read_value = value(read, &member);
        match (time, read_value) {
            (Some(time), Some(value)) => keys.push((time, value, index, member)),
            _ => ok = false,
        }
    }
    // Two keys that are one time (`"0"` and `"0.0"`, `"-0"` and `"0"`): the later one in the
    // file stands, as of a member written twice. The sort is stable, so the
    // keys of one time stay in the file's order and the last of each run is the later one.
    keys.sort_by(|a, b| same_zero(a.0).total_cmp(&same_zero(b.0)));
    let mut times: Vec<Keyframe<T>> = Vec::with_capacity(keys.len());
    let mut standing: Option<(usize, Node<'v>)> = None;
    let mut dropped = Vec::new();
    for (time, value, index, member) in keys {
        match times.last_mut() {
            Some(last) if same_zero(last.time).to_bits() == same_zero(time).to_bits() => {
                dropped.extend(standing.take());
                *last = Keyframe { time, value };
            }
            _ => times.push(Keyframe { time, value }),
        }
        standing = Some((index, member));
    }
    // The keyframes that do not stand, in the file's order.
    dropped.sort_by_key(|(index, _)| *index);
    for (_, member) in dropped {
        tell(
            read,
            Code::from(&core_codes::DROPPED_FIELD),
            Severity::Info,
            &member,
        );
    }
    ok.then_some(Keyframed::Keys(times))
}

#[cfg(test)]
mod tests {
    //! Characterization of the 1.26.20 reader: every finding (severity, path, code, text and
    //! span) and the value read, for payloads that reach every member and every rule, and for
    //! the payloads of the crate's fog fixtures. `read.pins` holds what the reader gave before
    //! it moved onto the reader kit; `BEDROCK_ADDON_FOG_PIN=write` writes it instead.

    use super::*;

    /// Payloads that reach every reader and every rule of this file.
    #[allow(
        clippy::needless_raw_string_hashes,
        reason = "one delimiter for every payload, the colours' `#` included"
    )]
    const PAYLOADS: &[&str] = &[
        r##"{"description": {"identifier": "a:b"}}"##,
        r##"{"description": {"identifier": "a:b", "x": 1}, "y": [1, {"z": null}]}"##,
        r##"{"description": {}}"##,
        r##"{}"##,
        r##"[]"##,
        r##"5"##,
        r##"{"description": {"identifier": 5}}"##,
        r##"{"description": [], "distance": {}}"##,
        r##"{"description": {"identifier": "a:b"}, "description": {"identifier": "c:d"}}"##,
        r##"{"description": {"identifier": "a:b", "identifier": 1}}"##,
        r##"{"description": {"identifier": "a\u0062"}} // a comment"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_start": 1, "fog_end": 2.5, "render_distance_type": "fixed", "fog_color": "#aabbcc"}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_start": "1", "fog_end": -1, "render_distance_type": "far", "fog_color": "#abc"}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_start": true, "fog_end": null, "render_distance_type": 3, "fog_color": 7}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_color": [1, 300, -4]}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_color": [1.0, 2, 3]}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_color": [1, 2]}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_color": [1, 2, 3, 4], "q": 1}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_color": ["1", 2, 1e2]}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"air": {"fog_start": 1e39, "fog_end": 3.4028235e38}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"weather": {}, "lava": {"fog_start": 0}, "lava_resistance": {"fog_end": 0}, "powder_snow": {"render_distance_type": "render"}, "other": {}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"water": {"fog_start": 1, "transition_fog": {"init_fog": {"fog_start": 2}, "min_percent": 0.5, "mid_seconds": 1, "mid_percent": 2, "max_seconds": -1, "w": 0}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"water": {"transition_fog": 5}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": {"water": {"transition_fog": {"init_fog": []}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "distance": []}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": {"air": {"max_density": 0.5, "uniform": true, "zero_density_height": 0, "max_density_height": 300}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": {"air": {"max_density": 2, "uniform": 1, "zero_density_height": -100, "max_density_height": "x"}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": {"weather": {"max_density": {"0": 0.1, "0.5": 0.2, "1": 0.3}}, "water": {}, "lava": {}, "lava_resistance": {}, "fire": {}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": {"air": {"max_density": {"0": 0.1, "0.0": 0.2, "-0": 0.3, "x": 0.4, "nan": 0.1, "inf": 0.2, "1e40": 0.3, " 2 ": 0.5}}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": {"air": {"max_density": {"0": 2, "1": "a"}}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": {"air": {"max_density": {}}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"media_coefficients": {"air": {"absorption": [0.1, 0.2, 0.3], "scattering": "#000000"}, "water": {"absorption": {"0": [1, 1, 1], "1": "#ffffff"}}, "cloud": {"scattering": [1, 2]}, "fog": {}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"media_coefficients": {"air": {"absorption": ["a", 0.2, null], "scattering": 5, "e": 1}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": -0.5}, "water": {"henyey_greenstein_g": {"0": 2, "1": -1}}, "cloud": {}}}}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": true, "k": 1}}}, "volumetric": 5}"##,
        r##"{"description": {"identifier": "a:b"}, "volumetric": {"density": 1, "media_coefficients": [], "henyey_greenstein_g": "x", "v": 1}}"##,
        r##"{"distance": {"air": {"fog_start": "x"}}, "description": {"identifier": "a:b"}}"##,
        r##"{"distance": {"air": {"fog_start": 1}}}"##,
    ];

    /// What the reader made of `payload`: its findings, then the value.
    fn outcome(payload: &str) -> String {
        use std::fmt::Write as _;
        let mut d = Diagnostics::default();
        let at = JsonPath::of_keys(["minecraft:fog_settings"]);
        let read = read_settings(JsonSlice::new(payload), &at, &mut d);
        let mut out = format!("== {payload}\n");
        for finding in &d {
            writeln!(
                out,
                "{:?}|{}|{}|{}|{:?}|kept={}",
                finding.severity,
                finding.location.json_path,
                finding.code.id(),
                finding.code,
                finding.location.span.map(|span| (span.offset, span.len)),
                finding.kept,
            )
            .expect("a String takes it");
        }
        writeln!(out, "{read:?}").expect("a String takes it");
        out
    }

    /// The payloads of the crate's fog fixtures, as JSON texts.
    fn fixture_payloads() -> Vec<String> {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../addon/tests/fixtures");
        let mut files = Vec::new();
        let mut dirs = vec![root];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).expect("fixtures") {
                let path = entry.expect("entry").path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "json") {
                    files.push(path);
                }
            }
        }
        files.sort();
        files
            .iter()
            .filter_map(|path| {
                let text = std::fs::read_to_string(path).ok()?;
                let json: bedrock_addon_core::json::Value =
                    bedrock_addon_core::json::from_str_jsonc(&text).ok()?;
                let payload = json.as_object()?.get("minecraft:fog_settings")?;
                bedrock_addon_core::json::to_json(payload).ok()
            })
            .collect()
    }

    #[test]
    fn the_1_26_20_reader_reads_as_pinned() {
        let mut out = String::new();
        for payload in PAYLOADS {
            out.push_str(&outcome(payload));
        }
        for payload in fixture_payloads() {
            out.push_str(&outcome(&payload));
        }
        let pins = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/read.pins");
        if std::env::var("BEDROCK_ADDON_FOG_PIN").as_deref() == Ok("write") {
            std::fs::write(&pins, &out).expect("write the pins");
            return;
        }
        let pinned = std::fs::read_to_string(&pins).expect("read.pins");
        for (line, (got, want)) in out.lines().zip(pinned.lines()).enumerate() {
            assert_eq!(got, want, "line {}", line + 1);
        }
        assert_eq!(out.lines().count(), pinned.lines().count());
    }
}
