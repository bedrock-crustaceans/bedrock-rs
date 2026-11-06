//! The checks that need every camera document of the stack: the pass that discards invalid
//! presets, the list checks and the duplicate spline names; and the group's combine, which runs
//! them over the parsed files.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use super::{
    ALLOWED_MC_INHERIT, CameraFile, CameraList, CameraPreset, CameraSpline, Cameras, Splines,
    VANILLA_CAMERAS, texts,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase,
};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, Notice, Registry, RegistryBuilder, Rejected,
};
use bedrock_addon_core::id::syntax::split_namespace;
use bedrock_addon_core::id::{CameraPresetId, Kind, kind};
use bedrock_addon_core::load::StackCx;

/// The namespace of vanilla's own presets.
const VANILLA_NAMESPACE: &str = "minecraft";

/// The detail of a reference to the camera preset `id`.
fn preset_reference(id: &str) -> DiagnosticDetail {
    DiagnosticDetail::Reference {
        kind: <kind::CameraPreset as Kind>::NAME,
        id: id.to_owned(),
        hint: None,
    }
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
            if split_namespace(parent).map(|(namespace, _)| namespace) == Some(VANILLA_NAMESPACE) {
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
        return Some(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: texts::formatted(texts::ILLEGAL_OVERRIDE, &[name]),
            })
            .at(JsonPath::of_keys(["minecraft:camera_preset", "radius"]))
            .phase(Phase::Validate)
            .in_file(document.origin()),
        );
    }
    match end {
        End::Valid => None,
        End::Illegal(parent) => Some(
            Diagnostic::error(DiagnosticCode::InvalidReference {
                text: texts::formatted(texts::ILLEGAL_INHERITANCE, &[name, parent]),
            })
            .at(JsonPath::of_keys(inherit_from))
            .phase(Phase::Validate)
            .in_file(document.origin()),
        ),
        End::Circular(repeated) => Some(
            Diagnostic::error(DiagnosticCode::ReferenceCycle {
                text: texts::formatted(texts::CIRCULAR, &[name, repeated]),
            })
            .at(JsonPath::of_keys(inherit_from))
            .phase(Phase::Validate)
            .in_file(document.origin()),
        ),
        End::Unknown(parent) => {
            let unknown = Diagnostic::error(DiagnosticCode::UnresolvedReference {
                text: texts::formatted(texts::UNKNOWN_PARENT, &[name, parent]),
            })
            .at(JsonPath::of_keys(inherit_from))
            .phase(Phase::Validate)
            .in_file(document.origin())
            .detail(preset_reference(parent));
            Some(unknown)
        }
    }
}

/// The finding that keeps `list` from loading, if any: an unknown preset, then an empty list,
/// then a preset named twice (vanilla's order). `defined` holds the presets the stack defined
/// and whether each survived the discard pass; a preset the stack does not define is known when
/// the game has it: the host's provider says so, or without one the generated identifier list.
pub(crate) fn check_list(
    list: &Document<CameraList>,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> Option<Diagnostic> {
    let name = list.identifier.as_str();
    let cameras = ["minecraft:camera_list", "cameras"];
    let known = |id: &CameraPresetId| known_preset(id, defined, stack);
    if let Some(unknown) = list.cameras.iter().find(|id| !known(id)) {
        let finding = Diagnostic::error(DiagnosticCode::UnresolvedReference {
            text: texts::formatted(texts::LIST_UNKNOWN, &[name, unknown.as_str()]),
        })
        .at(JsonPath::of_keys(cameras))
        .phase(Phase::Validate)
        .in_file(list.origin())
        .detail(preset_reference(unknown.as_str()));
        return Some(finding);
    }
    if list.cameras.is_empty() {
        return Some(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: texts::formatted(texts::LIST_EMPTY, &[name]),
            })
            .at(JsonPath::of_keys(cameras))
            .phase(Phase::Validate)
            .in_file(list.origin()),
        );
    }
    let mut seen = HashSet::new();
    let twice = list.cameras.iter().find(|id| !seen.insert(id.as_str()))?;
    Some(
        Diagnostic::error(DiagnosticCode::DuplicateIdentifier {
            text: texts::formatted(texts::LIST_DUPLICATE, &[name, twice.as_str()]),
        })
        .at(JsonPath::of_keys(cameras))
        .phase(Phase::Validate)
        .in_file(list.origin()),
    )
}

/// An information at each spline whose name an earlier spline has. The game gives no
/// diagnostic and keeps both in load order, so this is the crate's own note.
pub(crate) fn duplicate_splines(files: &[Document<CameraSpline>]) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for file in files {
        for (i, spline) in file.splines.iter().enumerate() {
            if !seen.insert(spline.name.as_str()) {
                let mut warning =
                    Diagnostic::info(DiagnosticCode::DuplicateIdentifier { text: None })
                        .at(JsonPath::of_keys([
                            "minecraft:camera_custom_splines",
                            "splines",
                        ]))
                        .phase(Phase::Validate)
                        .in_file(file.origin());
                warning.location.json_path.push_index(i);
                warning.location.json_path.push("name");
                found.push(warning);
            }
        }
    }
    found
}

/// Whether the list entry `id` names a preset: one the stack defined that survived the
/// discard pass, or one of the game's (the host's provider when it gave one, else the
/// tables), as the link pass asks.
fn known_preset(
    id: &CameraPresetId,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> bool {
    match defined.get(id) {
        Some(survived) => *survived,
        None => stack
            .built_ins()
            .contains(<kind::CameraPreset as Kind>::NAME, id.as_str()),
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
    Diagnostic::error(DiagnosticCode::UnresolvedReference { text: Some(text) })
        .at(path.clone())
        .phase(Phase::Validate)
        .in_file(document.origin())
        .detail(preset_reference(name))
}

/// The validator's line of a preset whose parent is unknown.
pub(crate) fn unknown_parent_line(document: &Document<CameraPreset>) -> Option<Diagnostic> {
    let parent = parent(document.value())?;
    let path = JsonPath::of_keys(["minecraft:camera_preset", "inherit_from"]);
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
            let path = JsonPath::new()
                .child("minecraft:camera_list")
                .child("cameras")
                .index(index);
            validator_line(list, &path, id.as_str())
        })
        .collect()
}

// ---- the group's combine ---------------------------------------------------------------------

/// A registry in which the later definition of an identifier replaces the earlier one whole,
/// as vanilla assigns it in place; the replaced one is a `ShadowedDefinition` info.
fn last_loaded_wins<T: Definition>(stack: &StackCx<'_>) -> RegistryBuilder<T> {
    RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::last_loaded_wins(),
        stack.options,
    )
}

/// The rejection of a document the stack stage refuses: its own findings, then `finding`.
fn refuse<T>(document: &Document<T>, finding: Diagnostic) -> Rejected {
    refuse_all(document, std::iter::once(finding))
}

/// [`refuse`] with every finding of `findings`.
fn refuse_all<T>(
    document: &Document<T>,
    findings: impl IntoIterator<Item = Diagnostic>,
) -> Rejected {
    let origin = document.origin().clone();
    let mut diagnostics = document.diagnostics().clone();
    for finding in findings {
        diagnostics.push(finding);
    }
    Rejected::new(origin, diagnostics)
}

/// The presets: last loaded wins per identifier, then one pass over the merged set discards
/// the invalid ones. Returns the registry and the identifiers the
/// stack defined, with whether each survived.
fn presets(
    parsed: Vec<Contribution<CameraPreset>>,
    stack: &StackCx<'_>,
) -> (Registry<CameraPreset>, IndexMap<CameraPresetId, bool>) {
    // The effective definition of each identifier: the last one that loaded.
    let mut effective: IndexMap<CameraPresetId, usize> = IndexMap::new();
    for (index, contribution) in parsed.iter().enumerate() {
        if let Contribution::Stable(id, _) = contribution {
            effective.insert(id.clone(), index);
        }
    }
    let merged: IndexMap<&CameraPresetId, &Document<CameraPreset>> = effective
        .iter()
        .filter_map(|(id, &index)| match parsed.get(index)? {
            Contribution::Stable(_, document) => Some((id, document)),
            _ => None,
        })
        .collect();
    let mut discarded = discard_invalid_presets(&merged);
    let defined: IndexMap<CameraPresetId, bool> = effective
        .keys()
        .map(|id| (id.clone(), !discarded.contains_key(id)))
        .collect();

    let mut builder = last_loaded_wins::<CameraPreset>(stack);
    for (index, contribution) in parsed.into_iter().enumerate() {
        match contribution {
            Contribution::Stable(id, document) => {
                if defined.get(&id) != Some(&false) {
                    builder.push(Contribution::Stable(id, document));
                } else if effective.get(&id) == Some(&index) {
                    let finding = discarded.swap_remove(&id);
                    let rejected = match finding {
                        // An unknown parent also gets the game's own line, besides the
                        // reference finding.
                        Some(finding)
                            // By legacy name: the variant and the definitions that were it.
                            if matches!(finding.code.name(), "UnresolvedReference") =>
                        {
                            // Only where the direct parent is no preset of the stack: a
                            // parent the pass discarded is the chain's, not the validator's.
                            let line = unknown_parent_line(&document).filter(|_| {
                                document
                                    .inherit_from
                                    .as_ref()
                                    .is_some_and(|parent| !effective.contains_key(parent))
                            });
                            refuse_all(&document, std::iter::once(finding).chain(line))
                        }
                        Some(finding) => refuse(&document, finding),
                        None => Rejected::new(document.origin().clone(), Diagnostics::default()),
                    };
                    builder.push(Contribution::Rejected(Some(id), rejected));
                } else {
                    // An earlier definition of a discarded identifier: vanilla replaced it
                    // before the pass ran, so it does not come back.
                    for finding in document.diagnostics().iter().cloned() {
                        builder.push_diagnostic(finding);
                    }
                    builder.push_diagnostic(
                        Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
                            .phase(Phase::Validate)
                            .in_file(document.origin()),
                    );
                }
            }
            failed => builder.push(failed),
        }
    }
    (builder.finish(), defined)
}

/// The lists: each checked against the presets that survived when it loads, then last loaded
/// wins per identifier.
fn lists(
    parsed: Vec<Contribution<CameraList>>,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> Registry<CameraList> {
    let mut builder = last_loaded_wins::<CameraList>(stack);
    for contribution in parsed {
        let contribution = match contribution {
            Contribution::Stable(id, document) => {
                match check_list(&document, defined, stack) {
                    // Each unknown entry also gets the game's own line, besides the
                    // reference finding.
                    Some(finding)
                        // By legacy name: the variant and the definitions that were it.
                        if matches!(finding.code.name(), "UnresolvedReference") =>
                    {
                        let lines = unknown_camera_lines(&document, defined, stack);
                        let all = std::iter::once(finding).chain(lines);
                        Contribution::Rejected(Some(id), refuse_all(&document, all))
                    }
                    Some(finding) => Contribution::Rejected(Some(id), refuse(&document, finding)),
                    None => Contribution::Stable(id, document),
                }
            }
            failed => failed,
        };
        builder.push(contribution);
    }
    builder.finish()
}

/// The spline files: every one kept, in walk order, with the duplicate names reported.
fn splines(parsed: Vec<Result<Document<CameraSpline>, Rejected>>) -> Splines {
    let mut splines = Splines::default();
    for outcome in parsed {
        match outcome {
            Ok(document) => {
                splines
                    .diagnostics
                    .extend(document.diagnostics().iter().cloned());
                splines.files.push(document);
            }
            Err(rejected) => {
                splines
                    .diagnostics
                    .extend(rejected.diagnostics.iter().cloned());
                splines.rejected.push(rejected);
            }
        }
    }
    for finding in duplicate_splines(&splines.files) {
        splines.diagnostics.push(finding);
    }
    splines
}

/// The group, from every parsed file in walk order.
pub(crate) fn combine(parsed: Vec<CameraFile>, stack: &StackCx<'_>) -> Cameras {
    let mut preset_files = Vec::new();
    let mut list_files = Vec::new();
    let mut spline_files = Vec::new();
    for parsed_file in parsed {
        match parsed_file {
            CameraFile::Preset(outcome) => preset_files.push(*outcome),
            CameraFile::List(outcome) => list_files.push(*outcome),
            CameraFile::Spline(outcome) => spline_files.push(*outcome),
        }
    }
    let (presets, defined) = presets(preset_files, stack);
    Cameras {
        presets,
        lists: lists(list_files, &defined, stack),
        splines: splines(spline_files),
    }
}
