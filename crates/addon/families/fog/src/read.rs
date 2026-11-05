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
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FogId;
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_support::schema_walk::{Kind, Node, Walk};

/// Reads one `minecraft:fog_settings` payload, findings below `at`; `None` (with an error in
/// `d`) when the cereal pass would fail the file.
pub(crate) fn read_settings(
    payload: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<FogSettings> {
    let mut walk = Walk::with_path(at);
    let read = Node::of(payload).and_then(|root| settings(&mut walk, root));
    let failed = walk.diagnostics.has_errors();
    d.extend(walk.diagnostics);
    read.filter(|_| !failed)
}

// ---- the lenient walk ---------------------------------------------------------------------------

/// A type the member does not take.
fn wrong_type(walk: &mut Walk, node: Node<'_>) {
    walk.error(DiagnosticCode::InvalidType { text: None }, node.span());
}

/// A value outside the member's constraint.
fn violation(walk: &mut Walk, node: Node<'_>) {
    walk.error(DiagnosticCode::InvalidValue { text: None }, node.span());
}

/// Walks an object in document order: `on` reads a member it knows and says whether it read
/// (`None` for a member it does not know, which is kept in `extra` with an information: V12;
/// the game prints its parse log only for codes 2 (level 2) and 8 (level 3), never cereal's
/// unknown-member code 4, so vanilla says nothing there). The result is
/// `false` when the node is not an object or a member failed.
fn object<'a>(
    walk: &mut Walk,
    node: Node<'a>,
    extra: &mut UnknownMembers,
    mut on: impl FnMut(&mut Walk, &str, Node<'a>) -> Option<bool>,
) -> bool {
    let Some(members) = walk.members(node) else {
        wrong_type(walk, node);
        return false;
    };
    let mut ok = true;
    for (key, value) in members {
        walk.enter(&key);
        if let Some(read) = on(walk, &key, value) {
            ok &= read;
        } else {
            walk.push(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
                Some(value.span()),
            );
            if let Some(json) = value.value() {
                extra.insert(key.clone(), json);
            }
        }
        walk.leave();
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

fn number(walk: &mut Walk, node: Node<'_>) -> Option<f32> {
    if node.kind() != Kind::Number {
        wrong_type(walk, node);
        return None;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the game reads these members as float"
    )]
    let x = node.text().parse::<f64>().ok().map(|x| x as f32);
    if x.is_none_or(|x| !x.is_finite()) {
        wrong_type(walk, node);
        return None;
    }
    x
}

/// A number within `[min, max]` (`BasicNumericConstraint`).
fn ranged(walk: &mut Walk, node: Node<'_>, min: f32, max: f32) -> Option<f32> {
    let x = number(walk, node)?;
    if x < min || x > max {
        violation(walk, node);
        return None;
    }
    Some(x)
}

fn boolean(walk: &mut Walk, node: Node<'_>) -> Option<bool> {
    match (node.kind(), node.text()) {
        (Kind::Bool, "true") => Some(true),
        (Kind::Bool, "false") => Some(false),
        _ => {
            wrong_type(walk, node);
            None
        }
    }
}

fn text(walk: &mut Walk, node: Node<'_>) -> Option<String> {
    if node.kind() != Kind::String {
        wrong_type(walk, node);
        return None;
    }
    bedrock_addon_core::json::parse_at::<String>(JsonSlice::new(node.text())).ok()
}

/// `SharedTypes::REGEX_RGB_COLOR`: `^#[a-fA-F0-9]{6}$`.
pub(crate) fn is_hex(text: &str) -> bool {
    text.strip_prefix('#')
        .is_some_and(|digits| digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// The string form of a colour, or its constraint violation.
fn hex(walk: &mut Walk, node: Node<'_>) -> Option<String> {
    let written = text(walk, node)?;
    if is_hex(&written) {
        Some(written)
    } else {
        violation(walk, node);
        None
    }
}

/// Exactly three elements (`std::array<_, 3>`).
fn triple<'a>(walk: &mut Walk, node: Node<'a>) -> Option<[Node<'a>; 3]> {
    let elements = walk.elements(node)?;
    if let [r, g, b] = elements.as_slice() {
        Some([*r, *g, *b])
    } else {
        violation(walk, node);
        None
    }
}

/// `Color255RGB`: `#RRGGBB`, or three integers, each clamped to 0–255 as
/// `colorFromIntArray3` clamps it (with a `ClampedValue` info, ours). A real (`0.5`, `1.0`) or a
/// string is refused (ours; what the game's integer reader does with a real is not known).
fn fog_color(walk: &mut Walk, node: Node<'_>) -> Option<FogColor> {
    match node.kind() {
        Kind::String => hex(walk, node).map(FogColor::Hex),
        Kind::Array => {
            let nodes = triple(walk, node)?;
            let mut rgb = [0_u8; 3];
            let mut ok = true;
            for (index, element) in nodes.into_iter().enumerate() {
                walk.enter_index(index);
                let written = (element.kind() == Kind::Number)
                    .then(|| element.text().parse::<i64>().ok())
                    .flatten();
                if let Some(written) = written {
                    let clamped = written.clamp(0, 255);
                    if clamped != written {
                        walk.push(
                            DiagnosticCode::ClampedValue { text: None },
                            Severity::Info,
                            Some(element.span()),
                        );
                    }
                    rgb[index] = u8::try_from(clamped).unwrap_or(u8::MAX);
                } else {
                    violation(walk, element);
                    ok = false;
                }
                walk.leave();
            }
            ok.then_some(FogColor::Rgb255(rgb))
        }
        _ => {
            wrong_type(walk, node);
            None
        }
    }
}

/// `ColorNormRGB`: `#RRGGBB`, or three numbers (clamped to 0–1 when used).
fn color_norm(walk: &mut Walk, node: Node<'_>) -> Option<ColorNorm> {
    match node.kind() {
        Kind::String => hex(walk, node).map(ColorNorm::Hex),
        Kind::Array => {
            let nodes = triple(walk, node)?;
            let mut rgb = [0.0_f32; 3];
            let mut ok = true;
            for (index, element) in nodes.into_iter().enumerate() {
                walk.enter_index(index);
                match number(walk, element) {
                    Some(x) => rgb[index] = x,
                    None => ok = false,
                }
                walk.leave();
            }
            ok.then_some(ColorNorm::Array(rgb))
        }
        _ => {
            wrong_type(walk, node);
            None
        }
    }
}

/// `CerealHelpers::TimeKeyframes<float, T>`: one value, or an object whose keys are numbers
/// written as strings, sorted by key. Each value obeys the member's own rule. Of several keys
/// that are one time the last stands; each of the others is an Info `DroppedField` at its key.
fn keyframed<'a, T>(
    walk: &mut Walk,
    node: Node<'a>,
    mut value: impl FnMut(&mut Walk, Node<'a>) -> Option<T>,
) -> Option<Keyframed<T>> {
    if node.kind() != Kind::Object {
        return value(walk, node).map(Keyframed::Single);
    }
    let members = walk.members(node)?;
    let mut keys = Vec::with_capacity(members.len());
    let mut ok = true;
    for (index, (key, member)) in members.into_iter().enumerate() {
        walk.enter(&key);
        // A key Rust reads as `nan` or `inf`, or one too large for a float, is no time.
        let time = key.trim().parse::<f32>().ok().filter(|t| t.is_finite());
        if time.is_none() {
            violation(walk, member);
        }
        let read = value(walk, member);
        walk.leave();
        match (time, read) {
            (Some(time), Some(value)) => keys.push((time, value, index, key, member.span())),
            _ => ok = false,
        }
    }
    // Two keys that are one time (`"0"` and `"0.0"`, `"-0"` and `"0"`): the later one in the
    // file stands, as of a member written twice. The sort is stable, so the
    // keys of one time stay in the file's order and the last of each run is the later one.
    keys.sort_by(|a, b| same_zero(a.0).total_cmp(&same_zero(b.0)));
    let mut times: Vec<(f32, T)> = Vec::with_capacity(keys.len());
    let mut standing: Option<(usize, String, Span)> = None;
    let mut dropped = Vec::new();
    for (time, value, index, key, span) in keys {
        match times.last_mut() {
            Some(last) if same_zero(last.0).to_bits() == same_zero(time).to_bits() => {
                dropped.extend(standing.take());
                *last = (time, value);
            }
            _ => times.push((time, value)),
        }
        standing = Some((index, key, span));
    }
    // The keyframes that do not stand, in the file's order.
    dropped.sort_by_key(|(index, ..)| *index);
    for (_, key, span) in dropped {
        walk.enter(&key);
        walk.push(
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            Some(span),
        );
        walk.leave();
    }
    ok.then_some(Keyframed::Keys(times))
}

// ---- the schema --------------------------------------------------------------------------------

fn settings(walk: &mut Walk, node: Node<'_>) -> Option<FogSettings> {
    let mut description = None;
    let mut distance = None;
    let mut volumetric = None;
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "description" => put(&mut description, self::description(walk, v)),
        "distance" => put(&mut distance, self::distance(walk, v)),
        "volumetric" => put(&mut volumetric, self::volumetric(walk, v)),
        _ => None,
    });
    if ok && description.is_none() {
        walk.error(
            DiagnosticCode::MissingRequiredField { text: None },
            node.span(),
        );
    }
    ok.then_some(FogSettings {
        description: description?,
        distance,
        volumetric,
        extra,
    })
}

fn description(walk: &mut Walk, node: Node<'_>) -> Option<FogDescription> {
    let mut identifier = None;
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        // `SharedTypes::Identifier<9>`: any string, stored as written.
        "identifier" => put(&mut identifier, text(walk, v)),
        _ => None,
    });
    if ok && identifier.is_none() {
        walk.error(
            DiagnosticCode::MissingRequiredField { text: None },
            node.span(),
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
    walk: &mut Walk,
    key: &str,
    v: Node<'_>,
    setting: &mut FogDistanceSetting,
) -> Option<bool> {
    match key {
        "fog_start" => put(&mut setting.fog_start, number(walk, v)),
        // `BasicNumericConstraint<double>::min(0.0)`.
        "fog_end" => put(&mut setting.fog_end, ranged(walk, v, 0.0, f32::MAX)),
        "render_distance_type" => put(&mut setting.render_distance_type, distance_type(walk, v)),
        "fog_color" => put(&mut setting.fog_color, fog_color(walk, v)),
        _ => None,
    }
}

/// The enum `"FogDistanceSetting DistanceType"`: `"fixed"` = 0, `"render"` = 1, as written.
fn distance_type(walk: &mut Walk, node: Node<'_>) -> Option<DistanceType> {
    match text(walk, node)?.as_str() {
        "fixed" => Some(DistanceType::Fixed),
        "render" => Some(DistanceType::Render),
        _ => {
            walk.error(DiagnosticCode::InvalidEnumValue { text: None }, node.span());
            None
        }
    }
}

fn distance_setting(walk: &mut Walk, node: Node<'_>) -> Option<FogDistanceSetting> {
    let mut setting = FogDistanceSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| {
        base_member(walk, key, v, &mut setting)
    });
    setting.extra = extra;
    ok.then_some(setting)
}

fn water_setting(walk: &mut Walk, node: Node<'_>) -> Option<FogWaterDistanceSetting> {
    let mut base = FogDistanceSetting::default();
    let mut transition_fog = None;
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "transition_fog" => put(&mut transition_fog, transition(walk, v)),
        _ => base_member(walk, key, v, &mut base),
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

fn transition(walk: &mut Walk, node: Node<'_>) -> Option<FogTransitionSetting> {
    let mut t = FogTransitionSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "init_fog" => put(&mut t.init_fog, distance_setting(walk, v)),
        "min_percent" => put(&mut t.min_percent, ranged(walk, v, 0.0, 1.0)),
        "mid_seconds" => put(&mut t.mid_seconds, ranged(walk, v, 0.0, f32::MAX)),
        "mid_percent" => put(&mut t.mid_percent, ranged(walk, v, 0.0, 1.0)),
        "max_seconds" => put(&mut t.max_seconds, ranged(walk, v, 0.0, f32::MAX)),
        _ => None,
    });
    t.extra = extra;
    ok.then_some(t)
}

fn distance(walk: &mut Walk, node: Node<'_>) -> Option<FogDistanceCollection> {
    let mut c = FogDistanceCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "air" => put(&mut c.air, distance_setting(walk, v)),
        "weather" => put(&mut c.weather, distance_setting(walk, v)),
        "water" => put(&mut c.water, water_setting(walk, v)),
        "lava" => put(&mut c.lava, distance_setting(walk, v)),
        "lava_resistance" => put(&mut c.lava_resistance, distance_setting(walk, v)),
        "powder_snow" => put(&mut c.powder_snow, distance_setting(walk, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

/// A density height; both share one constraint.
fn height(walk: &mut Walk, v: Node<'_>) -> Option<Keyframed<f32>> {
    keyframed(walk, v, |walk, v| ranged(walk, v, -64.0, 320.0))
}

fn density_setting(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricDensitySetting> {
    let mut s = FogVolumetricDensitySetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "max_density" => put(
            &mut s.max_density,
            keyframed(walk, v, |walk, v| ranged(walk, v, 0.0, 1.0)),
        ),
        "uniform" => put(&mut s.uniform, boolean(walk, v)),
        "zero_density_height" => put(&mut s.zero_density_height, height(walk, v)),
        "max_density_height" => put(&mut s.max_density_height, height(walk, v)),
        _ => None,
    });
    s.extra = extra;
    ok.then_some(s)
}

fn density(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricDensityCollection> {
    let mut c = FogVolumetricDensityCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "air" => put(&mut c.air, density_setting(walk, v)),
        "weather" => put(&mut c.weather, density_setting(walk, v)),
        "water" => put(&mut c.water, density_setting(walk, v)),
        "lava" => put(&mut c.lava, density_setting(walk, v)),
        "lava_resistance" => put(&mut c.lava_resistance, density_setting(walk, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

fn coefficients(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricMediaCoeffsSetting> {
    let mut s = FogVolumetricMediaCoeffsSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "absorption" => put(&mut s.absorption, keyframed_color(walk, v)),
        "scattering" => put(&mut s.scattering, keyframed_color(walk, v)),
        _ => None,
    });
    s.extra = extra;
    ok.then_some(s)
}

/// A coefficient: a colour, or colours by time key. An object is the keyed form.
fn keyframed_color(walk: &mut Walk, node: Node<'_>) -> Option<Keyframed<ColorNorm>> {
    keyframed(walk, node, color_norm)
}

fn media(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricMediaCoeffsCollection> {
    let mut c = FogVolumetricMediaCoeffsCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "air" => put(&mut c.air, coefficients(walk, v)),
        "water" => put(&mut c.water, coefficients(walk, v)),
        "cloud" => put(&mut c.cloud, coefficients(walk, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

fn hg_setting(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricHenyeyGreensteinGSetting> {
    let mut s = FogVolumetricHenyeyGreensteinGSetting::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "henyey_greenstein_g" => put(
            &mut s.henyey_greenstein_g,
            keyframed(walk, v, |walk, v| ranged(walk, v, -1.0, 1.0)),
        ),
        _ => None,
    });
    s.extra = extra;
    ok.then_some(s)
}

fn hg(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricHenyeyGreensteinGCollection> {
    let mut c = FogVolumetricHenyeyGreensteinGCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "air" => put(&mut c.air, hg_setting(walk, v)),
        "water" => put(&mut c.water, hg_setting(walk, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}

fn volumetric(walk: &mut Walk, node: Node<'_>) -> Option<FogVolumetricCollection> {
    let mut c = FogVolumetricCollection::default();
    let mut extra = UnknownMembers::default();
    let ok = object(walk, node, &mut extra, |walk, key, v| match key {
        "density" => put(&mut c.density, density(walk, v)),
        "media_coefficients" => put(&mut c.media_coefficients, media(walk, v)),
        "henyey_greenstein_g" => put(&mut c.henyey_greenstein_g, hg(walk, v)),
        _ => None,
    });
    c.extra = extra;
    ok.then_some(c)
}
