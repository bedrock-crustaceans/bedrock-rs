//! The rules on the newest struct: V14 of the cereal parser's
//! validation lambda, which the game runs on a document read at 1.26.20 alone; V15–V20 of that
//! lambda and of the strict schema's typed nodes alike, so of every era; the namespace
//! rule V11 of both eras; and the engine gate of the keyframe forms.
//
// A failed validation refuses the file with "Failed to validate fog schema".

use super::texts::{
    BAD_FORMAT_VERSION, END_BELOW_START, MAX_HEIGHT_REQUIRED, MID_BELOW_MAX, MINECRAFT_NAMESPACE,
    RENDER_END_ABOVE, RENDER_START_ABOVE, RENDER_START_BELOW, UNIFORM_HEIGHTS, ZERO_BELOW_MAX,
    ZERO_HEIGHT_REQUIRED,
};
use super::{
    DistanceType, FogSettings, FogVolumetricDensitySetting, FogVolumetricMediaCoeffsSetting,
    Keyframed, same_zero,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::version::{FormatVersion, SemVersion, known};

/// The first engine that reads the keyframe forms of the volumetric members.
pub(crate) const KEYFRAMES_SINCE: FormatVersion = known::V1_26_30;

fn report(d: &mut Diagnostics, severity: Severity, text: &'static str, at: JsonPath) {
    d.push(
        Diagnostic::error(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed(text)),
        })
        .severity(severity)
        .at(at)
        .phase(Phase::Validate),
    );
}

/// V14: the document's `format_version` text is a version. Unreachable from a file, whose
/// version was read before the cereal parser saw it.
pub(crate) fn format_version(written: &str, at: &JsonPath, d: &mut Diagnostics) {
    if SemVersion::parse(written).is_none() {
        report(
            d,
            Severity::Error,
            BAD_FORMAT_VERSION,
            at.with(&["format_version"]),
        );
    }
}

/// V15, V16, V16a, V16b on one distance entry. Nothing is checked unless both distances are
/// written.
fn distance_entry(
    start: Option<f32>,
    end: Option<f32>,
    kind: Option<DistanceType>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let (Some(start), Some(end)) = (start, end) else {
        return;
    };
    if start > end {
        report(d, Severity::Error, END_BELOW_START, at.clone());
    }
    if kind != Some(DistanceType::Render) {
        return;
    }
    // V16–V16b are level 2 in every era: the 1.26.20 lambda logs them as code 1,
    // which `loadFromString` writes with `_contentLog(1, 2, 26)`; the strict schema's typed
    // nodes write them with `_contentLog(1, 2, …)` (`FogSetting`, `FogDistanceSetting`), and
    // go on.
    if start > 1.0 {
        report(d, Severity::Warning, RENDER_START_ABOVE, at.clone());
    }
    if start < -1.0 {
        report(d, Severity::Warning, RENDER_START_BELOW, at.clone());
    }
    if end > 1.0 {
        report(d, Severity::Warning, RENDER_END_ABOVE, at.clone());
    }
}

/// The pairs of values two keyframed members hold at the same time: two single values, a
/// single value against every key of the other, or the keys both have (ours; the 1.26.20
/// lambda knows single values only).
fn pairs(a: &Keyframed<f32>, b: &Keyframed<f32>) -> Vec<(f32, f32)> {
    match (a, b) {
        (Keyframed::Single(a), Keyframed::Single(b)) => vec![(*a, *b)],
        (Keyframed::Single(a), Keyframed::Keys(b)) => b.iter().map(|b| (*a, b.value)).collect(),
        (Keyframed::Keys(a), Keyframed::Single(b)) => a.iter().map(|a| (a.value, *b)).collect(),
        (Keyframed::Keys(a), Keyframed::Keys(b)) => {
            // `b` by time, for a search per key of `a`. A value built in memory may hold its keys
            // in any order and a time twice: the sort is stable, so the first of `b`'s keys of
            // a time is the one found.
            let mut by_time: Vec<(f32, f32)> =
                b.iter().map(|b| (same_zero(b.time), b.value)).collect();
            by_time.sort_by(|x, y| x.0.total_cmp(&y.0));
            a.iter()
                .filter_map(|a| {
                    let time = same_zero(a.time);
                    let first =
                        by_time.partition_point(|(other, _)| other.total_cmp(&time).is_lt());
                    by_time
                        .get(first)
                        .filter(|(other, _)| other.to_bits() == time.to_bits())
                        .map(|(_, b)| (a.value, *b))
                })
                .collect()
        }
    }
}

/// V18, V19, V19a, V20 on one density entry.
fn density_entry(setting: &FogVolumetricDensitySetting, at: &JsonPath, d: &mut Diagnostics) {
    let max = setting.max_density_height.as_ref();
    let zero = setting.zero_density_height.as_ref();
    if setting.uniform == Some(true) {
        // The lambda returns after this check: V19–V20 never run for a uniform entry.
        if max.is_some() || zero.is_some() {
            report(d, Severity::Error, UNIFORM_HEIGHTS, at.clone());
        }
        return;
    }
    if max.is_none() {
        report(d, Severity::Error, MAX_HEIGHT_REQUIRED, at.clone());
    }
    if zero.is_none() {
        report(d, Severity::Error, ZERO_HEIGHT_REQUIRED, at.clone());
    }
    if let (Some(zero), Some(max)) = (zero, max)
        && pairs(zero, max).iter().any(|(zero, max)| zero < max)
    {
        report(d, Severity::Error, ZERO_BELOW_MAX, at.clone());
    }
}

/// V15–V20 on a document read at 1.26.20, findings below `at` (the payload's path).
pub(crate) fn cereal_rules(settings: &FogSettings, at: &JsonPath, d: &mut Diagnostics) {
    if let Some(distance) = &settings.distance {
        for (name, entry) in [
            ("air", &distance.air),
            ("weather", &distance.weather),
            ("lava", &distance.lava),
            ("lava_resistance", &distance.lava_resistance),
            ("powder_snow", &distance.powder_snow),
        ] {
            if let Some(entry) = entry {
                distance_entry(
                    entry.fog_start,
                    entry.fog_end,
                    entry.render_distance_type,
                    &at.with(&["distance", name]),
                    d,
                );
            }
        }
        if let Some(water) = &distance.water {
            let water_at = at.with(&["distance", "water"]);
            distance_entry(
                water.fog_start,
                water.fog_end,
                water.render_distance_type,
                &water_at,
                d,
            );
            if let Some(transition) = &water.transition_fog {
                let transition_at = water_at.with(&["transition_fog"]);
                if let Some(init) = &transition.init_fog {
                    distance_entry(
                        init.fog_start,
                        init.fog_end,
                        init.render_distance_type,
                        &transition_at.with(&["init_fog"]),
                        d,
                    );
                }
                // V17.
                if let (Some(mid), Some(max)) = (transition.mid_seconds, transition.max_seconds)
                    && mid >= max
                    && mid != 0.0
                {
                    report(d, Severity::Error, MID_BELOW_MAX, transition_at);
                }
            }
        }
    }
    if let Some(density) = settings
        .volumetric
        .as_ref()
        .and_then(|v| v.density.as_ref())
    {
        for (name, entry) in [
            ("air", &density.air),
            ("weather", &density.weather),
            ("water", &density.water),
            ("lava", &density.lava),
            ("lava_resistance", &density.lava_resistance),
        ] {
            if let Some(entry) = entry {
                density_entry(entry, &at.with(&["volumetric", "density", name]), d);
            }
        }
    }
}

/// V11: outside a base-game pack the namespace `minecraft` is refused. The namespace is the
/// lower-cased text before the first `:`, or the whole lower-cased identifier without one.
pub(crate) fn namespace(
    identifier: &str,
    is_base_game_pack: bool,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let namespace = identifier.split(':').next().unwrap_or(identifier);
    if !is_base_game_pack && namespace.eq_ignore_ascii_case("minecraft") {
        d.push(
            Diagnostic::error(DiagnosticCode::NamespaceRule {
                text: Some(VanillaText::Fixed(MINECRAFT_NAMESPACE)),
            })
            .at(at.with(&["description", "identifier"]))
            .phase(Phase::Validate),
        );
    }
}

/// The times of a member written by time key; `None` for one value or no member.
fn times<T>(value: Option<&Keyframed<T>>) -> Option<Vec<f32>> {
    match value {
        Some(Keyframed::Keys(keys)) => Some(keys.iter().map(|key| key.time).collect()),
        _ => None,
    }
}

/// Every member of `volumetric` that is written by time key: its keys below the payload and
/// its times, in the order the members are written.
fn keyed_members(
    volumetric: &super::FogVolumetricCollection,
) -> Vec<(Vec<&'static str>, Vec<f32>)> {
    let mut keyed: Vec<(Vec<&'static str>, Vec<f32>)> = Vec::new();
    let mut note = |times: Option<Vec<f32>>, keys: &[&'static str]| {
        if let Some(times) = times {
            keyed.push((keys.to_vec(), times));
        }
    };
    if let Some(density) = &volumetric.density {
        for (name, entry) in [
            ("air", &density.air),
            ("weather", &density.weather),
            ("water", &density.water),
            ("lava", &density.lava),
            ("lava_resistance", &density.lava_resistance),
        ] {
            let Some(entry) = entry else { continue };
            for (member, value) in [
                ("max_density", &entry.max_density),
                ("zero_density_height", &entry.zero_density_height),
                ("max_density_height", &entry.max_density_height),
            ] {
                note(
                    times(value.as_ref()),
                    &["volumetric", "density", name, member],
                );
            }
        }
    }
    if let Some(media) = &volumetric.media_coefficients {
        let entries: [(&'static str, &Option<FogVolumetricMediaCoeffsSetting>); 3] = [
            ("air", &media.air),
            ("water", &media.water),
            ("cloud", &media.cloud),
        ];
        for (name, entry) in entries {
            let Some(entry) = entry else { continue };
            note(
                times(entry.absorption.as_ref()),
                &["volumetric", "media_coefficients", name, "absorption"],
            );
            note(
                times(entry.scattering.as_ref()),
                &["volumetric", "media_coefficients", name, "scattering"],
            );
        }
    }
    if let Some(hg) = &volumetric.henyey_greenstein_g {
        for (name, entry) in [("air", &hg.air), ("water", &hg.water)] {
            let Some(entry) = entry else { continue };
            note(
                times(entry.henyey_greenstein_g.as_ref()),
                &[
                    "volumetric",
                    "henyey_greenstein_g",
                    name,
                    "henyey_greenstein_g",
                ],
            );
        }
    }
    keyed
}

/// The path of the first keyframe whose time an earlier keyframe of its member has (`-0` and
/// `0` are one time): a value only a caller can build, since the reader keeps one keyframe
/// per time. Written, the two would be one member or two members that read back as one, so
/// the writer refuses the value there.
pub(crate) fn repeated_time(settings: &FogSettings, at: &JsonPath) -> Option<JsonPath> {
    let volumetric = settings.volumetric.as_ref()?;
    for (keys, times) in keyed_members(volumetric) {
        let mut seen = std::collections::HashSet::new();
        for time in times {
            if !seen.insert(super::same_zero(time).to_bits()) {
                let mut path = at.with(&keys);
                path.push(&time.to_string());
                return Some(path);
            }
        }
    }
    None
}

/// The keyframe forms need an engine from 1.26.30: below it each
/// keyed member is `UnsupportedValueForm`.
pub(crate) fn keyframe_gate(
    settings: &FogSettings,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    if engine >= KEYFRAMES_SINCE {
        return;
    }
    let Some(volumetric) = &settings.volumetric else {
        return;
    };
    let keyed = keyed_members(volumetric)
        .into_iter()
        .map(|(keys, _)| at.with(&keys));
    for path in keyed {
        d.push(
            Diagnostic::error(DiagnosticCode::UnsupportedValueForm)
                .at(path)
                .phase(Phase::Validate),
        );
    }
}
