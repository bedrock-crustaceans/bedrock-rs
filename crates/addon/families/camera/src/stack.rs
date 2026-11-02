//! The checks that need every camera document of the stack: the pass that discards invalid
//! presets, the list checks and the duplicate spline names.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use super::load::at_origin;
use super::{ALLOWED_MC_INHERIT, CameraList, CameraPreset, CameraSpline, VANILLA_CAMERAS, texts};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::Document;
use bedrock_addon_core::id::{CameraPresetId, Kind, kind};
use bedrock_addon_core::load::StackCx;

/// The namespace of vanilla's own presets.
const VANILLA_NAMESPACE: &str = "minecraft";

/// A stack-stage finding at `path` of `document`'s file.
fn finding<T>(
    code: DiagnosticCode,
    severity: Severity,
    document: &Document<T>,
    path: &[&str],
) -> Diagnostic {
    let mut at = JsonPath::new();
    for key in path {
        at.push(key);
    }
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        at_origin(document.origin(), at),
    )
}

/// The detail of a reference to the camera preset `id`.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills a finding's optional detail"
)]
fn preset_reference(id: &str) -> Option<Box<DiagnosticDetail>> {
    Some(Box::new(DiagnosticDetail::Reference {
        kind: <kind::CameraPreset as Kind>::NAME,
        id: id.to_owned(),
        hint: None,
    }))
}

/// Where the chain of parents of a preset ends.
#[derive(Clone, Copy)]
enum End<'a> {
    /// At a preset without a parent, or at one of `ALLOWED_MC_INHERIT`.
    Valid,
    /// At this vanilla-namespace parent, which is not one of `ALLOWED_MC_INHERIT`.
    Illegal(&'a str),
    /// At this name, which no loaded preset has.
    Unknown(&'a str),
    /// In a cycle: this is the first preset the walk meets a second time.
    Circular(&'a str),
}

/// The presets of the merged set that the game discards,
/// each with its finding, in the set's order. The pass runs once, over the effective preset of
/// each identifier, and judges each preset's chain once (linear in the number of presets).
pub(crate) fn discard_invalid_presets(
    merged: &IndexMap<&CameraPresetId, &Document<CameraPreset>>,
) -> IndexMap<CameraPresetId, Diagnostic> {
    let documents: Vec<&Document<CameraPreset>> = merged.values().copied().collect();
    let ends = chain_ends(&documents);
    merged
        .iter()
        .zip(ends)
        .filter_map(|((id, document), end)| Some(((*id).clone(), invalid(document, end)?)))
        .collect()
}

/// The parent a preset names: `None` for an absent or empty `inherit_from`.
fn parent(preset: &CameraPreset) -> Option<&str> {
    preset
        .inherit_from
        .as_ref()
        .map(bedrock_addon_core::id::Id::as_str)
        .filter(|parent| !parent.is_empty())
}

/// Where each preset's chain of parents ends, by the preset's index.
///
/// Each preset is put on a path once: a walk follows parents from a preset that has no verdict
/// yet until it meets the end of the chain, a preset that already has a verdict, or a preset of
/// its own path (a cycle), and then gives every preset of the path its verdict. A member of a
/// cycle is circular at itself; a preset whose chain leads into a cycle is circular at the
/// cycle's entry, the first preset its own walk would meet twice.
fn chain_ends<'a>(documents: &[&'a Document<CameraPreset>]) -> Vec<End<'a>> {
    /// Not on the current path.
    const OFF: usize = usize::MAX;
    let index: HashMap<&str, usize> = documents
        .iter()
        .enumerate()
        .map(|(i, document)| (document.identifier.as_str(), i))
        .collect();
    let mut ends: Vec<Option<End<'a>>> = vec![None; documents.len()];
    let mut on_path = vec![OFF; documents.len()];
    let mut path: Vec<usize> = Vec::new();
    for start in 0..documents.len() {
        if ends.get(start).copied().flatten().is_some() {
            continue;
        }
        path.clear();
        let mut current = start;
        // Every turn puts a preset without a verdict on the path, so the turns of all walks
        // together are at most the number of presets.
        let (end, cycle_from) = loop {
            if let Some(position) = on_path.get_mut(current) {
                *position = path.len();
            }
            path.push(current);
            let Some(parent) = documents.get(current).and_then(|document| parent(document)) else {
                break (End::Valid, None);
            };
            if parent.split_once(':').map(|(namespace, _)| namespace) == Some(VANILLA_NAMESPACE) {
                let end = if ALLOWED_MC_INHERIT.contains(&parent) {
                    End::Valid
                } else {
                    End::Illegal(parent)
                };
                break (end, None);
            }
            let Some(&next) = index.get(parent) else {
                break (End::Unknown(parent), None);
            };
            if let Some(end) = ends.get(next).copied().flatten() {
                break (end, None);
            }
            match on_path.get(next).copied() {
                Some(position) if position != OFF => break (End::Circular(parent), Some(position)),
                _ => current = next,
            }
        };
        for (position, &node) in path.iter().enumerate() {
            let own = match (cycle_from, documents.get(node)) {
                (Some(from), Some(document)) if position >= from => {
                    End::Circular(document.identifier.as_str())
                }
                _ => end,
            };
            if let Some(slot) = ends.get_mut(node) {
                *slot = Some(own);
            }
            if let Some(slot) = on_path.get_mut(node) {
                *slot = OFF;
            }
        }
    }
    ends.into_iter()
        .map(|end| end.unwrap_or(End::Valid))
        .collect()
}

/// Why `document` is discarded, if it is: the override check, then the end of its chain.
fn invalid(document: &Document<CameraPreset>, end: End<'_>) -> Option<Diagnostic> {
    let name = document.identifier.as_str();
    let inherit_from = ["minecraft:camera_preset", "inherit_from"];
    // Illegal override: a vanilla camera's `radius` is the game's.
    if VANILLA_CAMERAS.contains(&name) && document.radius.is_some() {
        return Some(finding(
            DiagnosticCode::InvalidValue {
                text: texts::formatted(texts::ILLEGAL_OVERRIDE, &[name]),
            },
            Severity::Error,
            document,
            &["minecraft:camera_preset", "radius"],
        ));
    }
    match end {
        End::Valid => None,
        End::Illegal(parent) => Some(finding(
            DiagnosticCode::InvalidReference {
                text: texts::formatted(texts::ILLEGAL_INHERITANCE, &[name, parent]),
            },
            Severity::Error,
            document,
            &inherit_from,
        )),
        End::Circular(repeated) => Some(finding(
            DiagnosticCode::ReferenceCycle {
                text: texts::formatted(texts::CIRCULAR, &[name, repeated]),
            },
            Severity::Error,
            document,
            &inherit_from,
        )),
        End::Unknown(parent) => {
            let mut unknown = finding(
                DiagnosticCode::UnresolvedReference {
                    text: texts::formatted(texts::UNKNOWN_PARENT, &[name, parent]),
                },
                Severity::Error,
                document,
                &inherit_from,
            );
            unknown.detail = preset_reference(parent);
            Some(unknown)
        }
    }
}

/// The finding that keeps `list` from loading, if any: an unknown preset, then an empty list,
/// then a preset named twice (vanilla's order). `defined` holds the presets the stack defined
/// and whether each survived the discard pass; a preset the stack does not define is known when
/// it is a built-in or one of vanilla's (the generated identifier list).
pub(crate) fn check_list(
    list: &Document<CameraList>,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> Option<Diagnostic> {
    let name = list.identifier.as_str();
    let cameras = ["minecraft:camera_list", "cameras"];
    let known = |id: &CameraPresetId| known_preset(id, defined, stack);
    if let Some(unknown) = list.cameras.iter().find(|id| !known(id)) {
        let mut finding = finding(
            DiagnosticCode::UnresolvedReference {
                text: texts::formatted(texts::LIST_UNKNOWN, &[name, unknown.as_str()]),
            },
            Severity::Error,
            list,
            &cameras,
        );
        finding.detail = preset_reference(unknown.as_str());
        return Some(finding);
    }
    if list.cameras.is_empty() {
        return Some(finding(
            DiagnosticCode::InvalidValue {
                text: texts::formatted(texts::LIST_EMPTY, &[name]),
            },
            Severity::Error,
            list,
            &cameras,
        ));
    }
    let mut seen = HashSet::new();
    let twice = list.cameras.iter().find(|id| !seen.insert(id.as_str()))?;
    Some(finding(
        DiagnosticCode::DuplicateIdentifier {
            text: texts::formatted(texts::LIST_DUPLICATE, &[name, twice.as_str()]),
        },
        Severity::Error,
        list,
        &cameras,
    ))
}

/// An information at each spline whose name an earlier spline has. The game gives no
/// diagnostic and keeps both in load order, so this is the crate's own note.
pub(crate) fn duplicate_splines(files: &[Document<CameraSpline>]) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for file in files {
        for (i, spline) in file.splines.iter().enumerate() {
            if !seen.insert(spline.name.0.as_str()) {
                let mut warning = finding(
                    DiagnosticCode::DuplicateIdentifier { text: None },
                    Severity::Info,
                    file,
                    &["minecraft:camera_custom_splines", "splines"],
                );
                warning.location.json_path.push_index(i);
                warning.location.json_path.push("name");
                found.push(warning);
            }
        }
    }
    found
}

/// Whether the list entry `id` names a preset: one the stack defined that survived the
/// discard pass, a built-in or one of vanilla's.
fn known_preset(
    id: &CameraPresetId,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> bool {
    match defined.get(id) {
        Some(survived) => *survived,
        None => {
            stack.options.built_ins.as_ref().is_some_and(|built_ins| {
                built_ins.contains(<kind::CameraPreset as Kind>::NAME, id.as_str())
            }) || bedrock_addon_core::vanilla::ids()
                .camera_presets
                .contains(&id.as_str())
        }
    }
}

// The game's asset validator holds `inherit_from` and each list entry as a camera preset
// reference and logs one it cannot resolve, beside the camera loader's own line. That is a
// second finding, an error with the validator's line and the breadcrumb from the document's
// root (`-> minecraft:camera_preset -> inherit_from`).

/// The validator's line for the preset `name` of `document` at `path`.
fn validator_line<T>(document: &Document<T>, path: &JsonPath, name: &str) -> Diagnostic {
    let text = bedrock_addon_core::diagnostic::validator::validator_text(
        bedrock_addon_core::diagnostic::validator::Validator::Missing,
        path,
        0,
        name,
    );
    let mut line = Diagnostic::new(
        DiagnosticCode::UnresolvedReference { text: Some(text) },
        Severity::Error,
        Phase::Validate,
        at_origin(document.origin(), path.clone()),
    );
    line.detail = preset_reference(name);
    line
}

/// The validator's line of a preset whose parent is unknown.
pub(crate) fn unknown_parent_line(document: &Document<CameraPreset>) -> Option<Diagnostic> {
    let parent = parent(document.value())?;
    let mut path = JsonPath::new();
    path.push("minecraft:camera_preset");
    path.push("inherit_from");
    Some(validator_line(document, &path, parent))
}

/// The validator's line of each entry of `list` that names no preset, at the entry.
pub(crate) fn unknown_camera_lines(
    list: &Document<CameraList>,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> Vec<Diagnostic> {
    list.cameras
        .iter()
        .enumerate()
        // A preset a file defines, even one the pass discarded, is no miss of the validator's
        // (as for a direct parent).
        .filter(|(_, id)| !defined.contains_key(*id) && !known_preset(id, defined, stack))
        .map(|(index, id)| {
            let mut path = JsonPath::new();
            path.push("minecraft:camera_list");
            path.push("cameras");
            path.push_index(index);
            validator_line(list, &path, id.as_str())
        })
        .collect()
}
