//! Reading [`FogSettings`] from the text of a 1.26.20 payload, as the game's cereal loader
//! reads `FogParametersSrcV0`: every member but `description.identifier` is optional, a member
//! the schema does not know is kept and reported (the game ignores it without a word), and a wrong type or a
//! value outside its constraint fails the document.

use super::{
    ColorNorm, DistanceType, FogColor, FogDescription, FogDistanceCollection, FogDistanceSetting,
    FogSettings, FogTransitionSetting, FogVolumetricCollection, FogVolumetricDensityCollection,
    FogVolumetricDensitySetting, FogVolumetricHenyeyGreensteinGCollection,
    FogVolumetricHenyeyGreensteinGSetting, FogVolumetricMediaCoeffsCollection,
    FogVolumetricMediaCoeffsSetting, FogWaterDistanceSetting, Keyframed, same_zero,
};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FogId;
use bedrock_addon_core::json::kit::{Disposition, Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value, parse_at};
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::color::is_hex;

/// Reads one `minecraft:fog_settings` payload, findings below `at`; `None` (with an error in
/// `d`) when the cereal pass would fail the file.
pub(crate) fn read_settings(
    payload: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<FogSettings> {
    let json = parse_at::<Value>(payload).ok()?;
    let mut found = Diagnostics::default();
    // The cereal pass: a wrong type or a broken constraint is an error that fails the file;
    // an unknown member is kept and told as information (the game says nothing).
    let mut read = Read::new(Reading::DOCUMENT, &mut found).unknown_severity(Severity::Info);
    let settings = settings(&mut read, &Node::with_text(&json, at.clone(), payload));
    let failed = found.has_errors();
    d.extend(found);
    settings.filter(|_| !failed)
}

// ---- the lenient walk ---------------------------------------------------------------------------

/// `code` at `node`, spanning it.
fn tell(read: &mut Read<'_>, code: DiagnosticCode, severity: Severity, node: &Node<'_>) {
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
        DiagnosticCode::InvalidValue { text: None },
        Severity::Error,
        node,
    );
}

/// The token of a node as written.
fn token<'v>(node: &Node<'v>) -> &'v str {
    node.text().map_or("", |text| text.text)
}

/// Walks an object in document order: `on` reads a member it knows and says whether it read
/// (`None` for a member it does not know, which is kept in `extra` with an information: V12;
/// the game prints its parse log only for codes 2 (level 2) and 8 (level 3), never cereal's
/// unknown-member code 4, so vanilla says nothing there). The result is
/// `false` when the node is not an object or a member failed.
fn object<'v>(
    read: &mut Read<'_>,
    node: &Node<'v>,
    extra: &mut UnknownMembers,
    mut on: impl FnMut(&mut Read<'_>, &str, &Node<'v>) -> Option<bool>,
) -> bool {
    let Some(members) = read.object(node) else {
        return false;
    };
    let mut ok = true;
    for (key, value) in members.iter() {
        if let Some(read_it) = on(read, key, &value) {
            ok &= read_it;
        } else {
            read.unknown_member(&value, Disposition::Keep);
            extra.insert(key.to_owned(), value.value().clone());
        }
    }
    ok
}

/// Stores what a member reader gave; the member read when it gave a value.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the walk's callback gives `None` for a member it does not know"
)]
fn put<T>(slot: &mut Option<T>, value: Option<T>) -> Option<bool> {
    let read = value.is_some();
    *slot = value;
    Some(read)
}

fn number(read: &mut Read<'_>, node: &Node<'_>) -> Option<f32> {
    if node.value().as_number().is_none() {
        read.wrong_type(node);
        return None;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the game reads these members as float"
    )]
    let x = token(node).parse::<f64>().ok().map(|x| x as f32);
    if x.is_none_or(|x| !x.is_finite()) {
        read.wrong_type(node);
        return None;
    }
    x
}

/// A number within `[min, max]` (`BasicNumericConstraint`).
fn ranged(read: &mut Read<'_>, node: &Node<'_>, min: f32, max: f32) -> Option<f32> {
    let x = number(read, node)?;
    if x < min || x > max {
        violation(read, node);
        return None;
    }
    Some(x)
}

fn boolean(read: &mut Read<'_>, node: &Node<'_>) -> Option<bool> {
    read.bool(node)
}

fn text(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    read.string(node).map(str::to_owned)
}

/// The string form of a colour, or its constraint violation.
fn hex(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let written = text(read, node)?;
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
fn fog_color(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogColor> {
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
                    DiagnosticCode::ClampedValue { text: None },
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
        match number(read, element) {
            Some(x) => rgb[index] = x,
            None => ok = false,
        }
    }
    ok.then_some(ColorNorm::Array(rgb))
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
    let mut times: Vec<(f32, T)> = Vec::with_capacity(keys.len());
    let mut standing: Option<(usize, Node<'v>)> = None;
    let mut dropped = Vec::new();
    for (time, value, index, member) in keys {
        match times.last_mut() {
            Some(last) if same_zero(last.0).to_bits() == same_zero(time).to_bits() => {
                dropped.extend(standing.take());
                *last = (time, value);
            }
            _ => times.push((time, value)),
        }
        standing = Some((index, member));
    }
    // The keyframes that do not stand, in the file's order.
    dropped.sort_by_key(|(index, _)| *index);
    for (_, member) in dropped {
        tell(
            read,
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            &member,
        );
    }
    ok.then_some(Keyframed::Keys(times))
}

// ---- the schema --------------------------------------------------------------------------------

fn settings(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogSettings> {
    let mut description = None;
    let mut distance = None;
    let mut volumetric = None;
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "description" => put(&mut description, self::description(read, v)),
        "distance" => put(&mut distance, self::distance(read, v)),
        "volumetric" => put(&mut volumetric, self::volumetric(read, v)),
        _ => None,
    });
    if ok && description.is_none() {
        tell(
            read,
            DiagnosticCode::MissingRequiredField { text: None },
            Severity::Error,
            node,
        );
    }
    ok.then_some(FogSettings {
        description: description?,
        distance,
        volumetric,
        extra,
    })
}

fn description(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogDescription> {
    let mut identifier = None;
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        // `SharedTypes::Identifier<9>`: any string, stored as written.
        "identifier" => put(&mut identifier, text(read, v)),
        _ => None,
    });
    if ok && identifier.is_none() {
        tell(
            read,
            DiagnosticCode::MissingRequiredField { text: None },
            Severity::Error,
            node,
        );
    }
    ok.then_some(FogDescription {
        identifier: FogId::parse(identifier?),
        extra,
    })
}

/// Reads member `key` of a distance setting into `setting`; `None` when it is none of the
/// four.
fn base_member(
    read: &mut Read<'_>,
    key: &str,
    v: &Node<'_>,
    setting: &mut FogDistanceSetting,
) -> Option<bool> {
    match key {
        "fog_start" => put(&mut setting.fog_start, number(read, v)),
        // `BasicNumericConstraint<double>::min(0.0)`.
        "fog_end" => put(&mut setting.fog_end, ranged(read, v, 0.0, f32::MAX)),
        "render_distance_type" => put(&mut setting.render_distance_type, distance_type(read, v)),
        "fog_color" => put(&mut setting.fog_color, fog_color(read, v)),
        _ => None,
    }
}

/// The enum `"FogDistanceSetting DistanceType"`: `"fixed"` = 0, `"render"` = 1, as written.
fn distance_type(read: &mut Read<'_>, node: &Node<'_>) -> Option<DistanceType> {
    match text(read, node)?.as_str() {
        "fixed" => Some(DistanceType::Fixed),
        "render" => Some(DistanceType::Render),
        _ => {
            tell(
                read,
                DiagnosticCode::InvalidEnumValue { text: None },
                Severity::Error,
                node,
            );
            None
        }
    }
}

fn distance_setting(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogDistanceSetting> {
    let mut setting = FogDistanceSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| {
        base_member(read, key, v, &mut setting)
    });
    setting.extra = extra;
    ok.then_some(setting)
}

fn water_setting(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogWaterDistanceSetting> {
    let mut base = FogDistanceSetting::default();
    let mut transition_fog = None;
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "transition_fog" => put(&mut transition_fog, transition(read, v)),
        _ => base_member(read, key, v, &mut base),
    });
    ok.then_some(FogWaterDistanceSetting {
        fog_start: base.fog_start,
        fog_end: base.fog_end,
        render_distance_type: base.render_distance_type,
        fog_color: base.fog_color,
        transition_fog,
        extra,
    })
}

fn transition(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogTransitionSetting> {
    let mut t = FogTransitionSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "init_fog" => put(&mut t.init_fog, distance_setting(read, v)),
        "min_percent" => put(&mut t.min_percent, ranged(read, v, 0.0, 1.0)),
        "mid_seconds" => put(&mut t.mid_seconds, ranged(read, v, 0.0, f32::MAX)),
        "mid_percent" => put(&mut t.mid_percent, ranged(read, v, 0.0, 1.0)),
        "max_seconds" => put(&mut t.max_seconds, ranged(read, v, 0.0, f32::MAX)),
        _ => None,
    });
    t.extra = extra;
    ok.then_some(t)
}

fn distance(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogDistanceCollection> {
    let mut c = FogDistanceCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "air" => put(&mut c.air, distance_setting(read, v)),
        "weather" => put(&mut c.weather, distance_setting(read, v)),
        "water" => put(&mut c.water, water_setting(read, v)),
        "lava" => put(&mut c.lava, distance_setting(read, v)),
        "lava_resistance" => put(&mut c.lava_resistance, distance_setting(read, v)),
        "powder_snow" => put(&mut c.powder_snow, distance_setting(read, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

/// A density height; both share one constraint.
fn height(read: &mut Read<'_>, v: &Node<'_>) -> Option<Keyframed<f32>> {
    keyframed(read, v, |read, v| ranged(read, v, -64.0, 320.0))
}

fn density_setting(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogVolumetricDensitySetting> {
    let mut s = FogVolumetricDensitySetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "max_density" => put(
            &mut s.max_density,
            keyframed(read, v, |read, v| ranged(read, v, 0.0, 1.0)),
        ),
        "uniform" => put(&mut s.uniform, boolean(read, v)),
        "zero_density_height" => put(&mut s.zero_density_height, height(read, v)),
        "max_density_height" => put(&mut s.max_density_height, height(read, v)),
        _ => None,
    });
    s.extra = extra;
    ok.then_some(s)
}

fn density(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogVolumetricDensityCollection> {
    let mut c = FogVolumetricDensityCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "air" => put(&mut c.air, density_setting(read, v)),
        "weather" => put(&mut c.weather, density_setting(read, v)),
        "water" => put(&mut c.water, density_setting(read, v)),
        "lava" => put(&mut c.lava, density_setting(read, v)),
        "lava_resistance" => put(&mut c.lava_resistance, density_setting(read, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

fn coefficients(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogVolumetricMediaCoeffsSetting> {
    let mut s = FogVolumetricMediaCoeffsSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "absorption" => put(&mut s.absorption, keyframed_color(read, v)),
        "scattering" => put(&mut s.scattering, keyframed_color(read, v)),
        _ => None,
    });
    s.extra = extra;
    ok.then_some(s)
}

/// A coefficient: a colour, or colours by time key. An object is the keyed form.
fn keyframed_color(read: &mut Read<'_>, node: &Node<'_>) -> Option<Keyframed<ColorNorm>> {
    keyframed(read, node, color_norm)
}

fn media(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogVolumetricMediaCoeffsCollection> {
    let mut c = FogVolumetricMediaCoeffsCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "air" => put(&mut c.air, coefficients(read, v)),
        "water" => put(&mut c.water, coefficients(read, v)),
        "cloud" => put(&mut c.cloud, coefficients(read, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

fn hg_setting(
    read: &mut Read<'_>,
    node: &Node<'_>,
) -> Option<FogVolumetricHenyeyGreensteinGSetting> {
    let mut s = FogVolumetricHenyeyGreensteinGSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "henyey_greenstein_g" => put(
            &mut s.henyey_greenstein_g,
            keyframed(read, v, |read, v| ranged(read, v, -1.0, 1.0)),
        ),
        _ => None,
    });
    s.extra = extra;
    ok.then_some(s)
}

fn hg(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogVolumetricHenyeyGreensteinGCollection> {
    let mut c = FogVolumetricHenyeyGreensteinGCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "air" => put(&mut c.air, hg_setting(read, v)),
        "water" => put(&mut c.water, hg_setting(read, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

fn volumetric(read: &mut Read<'_>, node: &Node<'_>) -> Option<FogVolumetricCollection> {
    let mut c = FogVolumetricCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(read, node, &mut extra, |read, key, v| match key {
        "density" => put(&mut c.density, density(read, v)),
        "media_coefficients" => put(&mut c.media_coefficients, media(read, v)),
        "henyey_greenstein_g" => put(&mut c.henyey_greenstein_g, hg(read, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
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
                finding.code.name(),
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
