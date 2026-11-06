//! What reading an animation file and an animation controller file share: the text check, the
//! `format_version` member, the parsed-file record, the particle and sound events, and the
//! document a parsed file becomes.

use super::types::{ParticleEffect, SoundEffect};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::node_tree::{Group, Hit, Kind, Node, PARSE_ERROR, Walk, text};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, parse_at, strip_bom,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};

/// A parsed file: the value when it loads, the version as written, every finding.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed<T> {
    /// The value, when the file loads.
    pub value: Option<T>,
    /// The `format_version` as written, when the file has one.
    pub written: Option<SemVersion>,
    /// Every finding of the read.
    pub diagnostics: Diagnostics,
}

/// The text of a file after the BOM, the depth check and the JSON syntax check; the finding
/// when it is not readable (vanilla "parse error:\n%s").
///
/// # Errors
///
/// The `MalformedInput` finding (boxed) when the text is nested deeper than `MAX_JSON_DEPTH`, is
/// not UTF-8, or is not valid JSON.
pub fn text_of(bytes: &[u8]) -> Result<&str, Box<Diagnostic>> {
    let malformed = |message: String| {
        Box::new(Diagnostic::error(DiagnosticCode::MalformedInput {
            text: text(PARSE_ERROR, &[&message]),
            message,
        }))
    };
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| malformed("nesting too deep".to_owned()))?;
    let text = std::str::from_utf8(bytes).map_err(|e| malformed(e.to_string()))?;
    // `Json::Reader::parse` of the whole text first: a syntax error anywhere drops the file.
    parse_at::<Value>(JsonSlice::new(text)).map_err(|e| malformed(e.error.to_string()))?;
    Ok(text)
}

/// A `format_version` member of a schema: a version string (`NoWildcards`), at least `minimum`.
pub fn format_version(walk: &mut Walk, node: Node<'_>, minimum: &SemVersion) -> Option<SemVersion> {
    let written = if node.kind() == Kind::String {
        walk.string(node)
    } else {
        None
    };
    let version = written
        .as_deref()
        .filter(|t| *t != "beta")
        .and_then(SemVersion::parse)
        .filter(|v| !v.is_any());
    let Some(version) = version else {
        walk.bad_type("SemVersion", node);
        return None;
    };
    if version < *minimum {
        walk.error(
            bedrock_addon_core::diagnostic::cereal::version_below_min(
                written.as_deref().unwrap_or(""),
                &minimum.to_string(),
            ),
            node.span(),
        );
        return None;
    }
    Some(version)
}

/// The version input of a parsed `SemVersion`.
pub fn input_of(version: &SemVersion) -> FormatVersionInput {
    FormatVersionInput {
        version: Some(version.format_version()),
        beta_literal: false,
        prerelease: version.is_pre_release(),
    }
}

const PARTICLE_MEMBERS: [Group; 4] = [
    Group::required("effect"),
    Group::name("locator"),
    Group::name("pre_effect_script"),
    Group::name("bind_to_actor"),
];

/// A particle event (animation and controller state): `bind_to_actor` accepts only `false`.
pub fn read_particle(walk: &mut Walk, node: Node<'_>) -> Option<ParticleEffect> {
    let mut effect = ParticleEffect::default();
    let ok = walk.object(node, &PARTICLE_MEMBERS, |walk, group, _, v| match group {
        0 => walk.string(v).map(|s| effect.effect = s).into(),
        1 => walk.string(v).map(|s| effect.locator = Some(s)).into(),
        2 => walk
            .molang(v)
            .map(|m| effect.pre_effect_script = Some(m))
            .into(),
        _ => match walk.bool(v) {
            Some(false) => {
                effect.bind_to_actor = Some(false);
                Hit::Ok
            }
            Some(true) => {
                // The allowed-value set is `{false}`.
                walk.not_allowed("true", v);
                Hit::Failed
            }
            None => Hit::Failed,
        },
    });
    ok.then_some(effect)
}

const SOUND_MEMBERS: [Group; 2] = [Group::required("effect"), Group::name("locator")];

/// A sound event (animation and controller state).
pub fn read_sound(walk: &mut Walk, node: Node<'_>) -> Option<SoundEffect> {
    let mut sound = SoundEffect::default();
    let ok = walk.object(node, &SOUND_MEMBERS, |walk, group, _, v| match group {
        0 => walk.string(v).map(|s| sound.effect = s).into(),
        _ => walk.string(v).map(|s| sound.locator = Some(s)).into(),
    });
    ok.then_some(sound)
}

/// The side a context's pack is on (a resource pack is the client).
pub fn side_of(cx: &LoadContext) -> PackSide {
    if cx.pack_type == PackType::Resource {
        PackSide::Resource
    } else {
        PackSide::Behavior
    }
}

/// Gives every finding the file's pack and path, pushed one by one: the places the findings
/// held are not carried over.
// TODO(phase 1 step 4): this loop becomes the existing `Diagnostics::locate`, as core's
// `unversioned` one does. That switch is behaviour-bearing: the repush here drops the moves
// and places, `locate` keeps them in place.
fn locate(diagnostics: Diagnostics, origin: &Origin) -> Diagnostics {
    let mut out = Diagnostics::default();
    for mut finding in diagnostics {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        out.push(finding);
    }
    out
}

/// The version record of a file whose `format_version` was written as `written`, parsed as
/// `parsed_as`, and whether it was upgraded to get there.
pub fn version_info(written: &SemVersion, parsed_as: FormatVersion, upgraded: bool) -> VersionInfo {
    let written = written.format_version();
    VersionInfo::new(Some(written), false, written, parsed_as, upgraded)
}

/// One file's result as a document of the whole file.
///
/// # Errors
///
/// `Rejected` (with the file's located diagnostics) when the parse produced no value or the file's
/// `format_version` was not read.
pub fn whole<T>(
    parsed: Parsed<T>,
    origin: Origin,
    parsed_as: impl FnOnce(&SemVersion) -> (FormatVersion, bool),
) -> Result<Document<T>, Rejected> {
    let diagnostics = locate(parsed.diagnostics, &origin);
    match (parsed.value, parsed.written) {
        (Some(value), Some(written)) => {
            let (as_, upgraded) = parsed_as(&written);
            Ok(Document::new(
                value,
                origin,
                version_info(&written, as_, upgraded),
                diagnostics,
            ))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}
