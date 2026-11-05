//! `sounds/sound_definitions.json` of the resource packs: the sound events and the audio each
//! plays. The client merges every pack's copy as JSON before it reads the result, lowest pack
//! first: an object merges into an object member by member, and anything else (an array, a
//! number, `null`) replaces what the lower packs had. The layers of the [`Merged`] are the
//! packs' copies as written.
//!
//! The file has a version chain ([`history`]); the version steps run on each copy before the
//! merge.
//!
//! A finding about a merged sound definition is located in the file of the highest pack that
//! writes that definition, also when the member it is about was written by a lower pack and
//! left as it was above: the merge keeps one origin for each definition, not one for each
//! member.
//!
//! [`Merged`]: bedrock_addon_core::document::Merged

#![cfg_attr(docsrs, feature(doc_cfg))]

mod definitions;
pub mod history;
mod upgrade;

use crate::upgrade::definition_path;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment,
};
use bedrock_addon_core::document::unversioned::lone;
use bedrock_addon_core::document::{
    Content, Document, Merge, Merged, Origin, Placed, RawLayer, Rejected, VersionInfo,
};
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use bedrock_addon_core::versioning::Versioned;
use bedrock_addon_support::sound::fork;
use bedrock_addon_support::sound::merge::{
    SoundCopy, layers, located, not_json, parse_copy, path_member, read_json, readable,
    write_document,
};
pub use definitions::{
    CATEGORIES, ExternalEventParameter, SoundAction, SoundDefinition, SoundDefinitions, SoundEntry,
    SoundEntryForm,
};
use indexmap::IndexMap;

/// Where a pack keeps its sound definitions.
pub(crate) const DEFINITIONS_PATH: &str = "sounds/sound_definitions.json";

impl bedrock_addon_core::__private::Sealed for SoundDefinitions {}

impl Merge for SoundDefinitions {
    /// A sound event name.
    type Key = SoundEventName;
}

/// `finding` without the `sound_definitions` its path begins with: where a file without the
/// wrapper has the member.
fn at_the_root(mut finding: Diagnostic) -> Diagnostic {
    let below = {
        let mut segments = finding.location.json_path.segments();
        (segments.next() == Some(JsonPathSegment::Key("sound_definitions"))).then(|| {
            let mut path = JsonPath::new();
            for segment in segments {
                match segment {
                    JsonPathSegment::Key(key) => path.push(key),
                    JsonPathSegment::Index(index) => path.push_index(index),
                }
            }
            path
        })
    };
    if let Some(path) = below {
        finding.location.json_path = path;
    }
    finding
}

/// Whether the file `json` has no wrapper: its `format_version` is no version from 1.14.0 on,
/// so the versionless struct reads it (`history::Any::read`) and its root's members are the
/// definitions.
fn has_no_wrapper(json: &Value) -> bool {
    upgrade::gate_version(json) < SemVersion::from(upgrade::V1_14_0)
}

/// The findings of one copy of the sound definitions before it is merged: the version steps
/// and what they do to the lower copies in `target` (4, 7, 11, 14, 15,
/// 15a). Returns the upgraded copy.
fn prepare_definitions(json: &Value, target: Option<&Value>, found: &mut Diagnostics) -> Value {
    let mut copy = json.clone();
    if let Some(version) = upgrade::version_member(&copy)
        && version > SemVersion::from(ENGINE_VERSION)
    {
        let at = JsonPath::new().child("format_version");
        found.push(
            Diagnostic::info(DiagnosticCode::FormatVersionAboveEngine {
                version: version.to_string(),
            })
            .at(at),
        );
    }
    let report = upgrade::upgrade(&mut copy, found);
    if report.wrapper_lost {
        found.push(
            Diagnostic::info(DiagnosticCode::SoundDefinitionsWrapperWithoutVersion)
                .at(definition_path("sound_definitions")),
        );
    }
    for name in &report.replaced {
        found.push(
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(definition_path(name)),
        );
    }
    // The definition's own `__use_legacy_max_distance` the 1.14.0 step writes `"true"` over,
    // as the game's does: noted at the member.
    for name in &report.flag_replaced {
        let at = definition_path(name).child("__use_legacy_max_distance");
        found.push(Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(at));
    }
    for (name, member) in &report.zeroed {
        let at = definition_path(name).child(member);
        found.push(Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(at));
    }
    // The entry `pitch` the 1.14.0 step removes. The game gives no message; the crate notes it
    // at the member, so that nothing an author wrote goes without a word.
    for (name, place) in &report.pitch_dropped {
        let mut at = definition_path(name);
        at.push("sounds");
        match place {
            upgrade::EntryPlace::Index(index) => at.push_index(*index),
            upgrade::EntryPlace::Key(key) => at.push(key),
        }
        at.push("pitch");
        found.push(Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(at));
    }
    let lower = target.map(|target| fork::member(target, "sound_definitions"));
    let Some(lower) = lower else {
        return copy;
    };
    if report.not_a_list && lower.as_object().is_some_and(|lower| !lower.is_empty()) {
        let at = JsonPath::new().child("sound_definitions");
        found.push(Diagnostic::info(DiagnosticCode::SoundDefinitionsCleared).at(at));
    }
    let definitions = fork::member(&copy, "sound_definitions");
    if report.distances {
        for name in fork::member_names(definitions) {
            let (own, below) = (fork::member(definitions, &name), fork::member(lower, &name));
            let resets = ["max_distance", "min_distance"].iter().any(|member| {
                fork::member(own, member).is_null() && fork::member(below, member).is_number()
            });
            if own.is_object() && resets {
                found.push(
                    Diagnostic::info(DiagnosticCode::DistanceResetByLegacyOverlay)
                        .at(definition_path(&name)),
                );
            }
        }
    }
    for name in &report.sounds_inserted {
        if !fork::member(fork::member(lower, name), "sounds").is_null() {
            found.push(
                Diagnostic::info(DiagnosticCode::SoundsClearedByLegacyOverlay)
                    .at(definition_path(name)),
            );
        }
    }
    copy
}

/// Merges the copies of `sounds/sound_definitions.json` and reads the result.
fn merge_definitions(
    copies: Vec<SoundCopy>,
    _: &StackCx<'_>,
) -> Merged<SoundDefinitions, RawLayer> {
    let mut diagnostics = Diagnostics::default();
    let copies = readable(DEFINITIONS_PATH, copies, &mut diagnostics);
    let mut origins: IndexMap<SoundEventName, Origin> = IndexMap::new();
    let mut target: Option<Value> = None;
    // The names whose last writer is a copy without the wrapper, and whether the top copy is
    // one: a finding carries the path its own file has.
    let mut bare: std::collections::HashSet<SoundEventName> = std::collections::HashSet::new();
    let mut top_is_bare = false;
    for (origin, json, _) in &copies {
        let mut found = Diagnostics::default();
        let copy = prepare_definitions(json, target.as_ref(), &mut found);
        let is_bare = has_no_wrapper(json);
        // The findings name places of the upgraded copy, where the definitions of a file
        // without the wrapper are below `sound_definitions`.
        let found = if is_bare {
            found.into_iter().map(at_the_root).collect()
        } else {
            found
        };
        located(found, origin, &mut diagnostics);
        for name in fork::member_names(fork::member(&copy, "sound_definitions")) {
            let name = SoundEventName::parse(name);
            if is_bare {
                bare.insert(name.clone());
            } else {
                bare.remove(&name);
            }
            origins.insert(name, origin.clone());
        }
        top_is_bare = is_bare;
        match &mut target {
            None => target = Some(copy),
            Some(target) => fork::recursive_merge(target, &copy, None),
        }
    }
    let mut merged = target.unwrap_or(Value::NULL);
    let mut found = Diagnostics::default();
    // Vanilla runs the steps once more on the merged root; every copy is stamped by now.
    upgrade::upgrade(&mut merged, &mut found);
    let value = SoundDefinitions::read(merged, &mut found);
    let top = copies.last().map(|(origin, _, _)| origin);
    for mut finding in found {
        // `sound_definitions/<name>/…` belongs to the copy that wrote the name last.
        let name = path_member(&finding.location.json_path, 1).map(SoundEventName::parse);
        let named = name.as_ref().and_then(|name| origins.get(name));
        let is_bare = match (named, &name) {
            (Some(_), Some(name)) => bare.contains(name),
            _ => top_is_bare,
        };
        if let Some(origin) = named.or(top) {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
        // The merged document has every definition below `sound_definitions`; the file of
        // a copy without the wrapper has it at the root.
        diagnostics.push(if is_bare {
            at_the_root(finding)
        } else {
            finding
        });
    }
    origins.retain(|name, _| value.sound_definitions.contains_key(name));
    Merged::new(value, origins, layers(&copies), diagnostics)
}

/// The loader of `rp.sound_definitions()`: every resource pack's copy of the file, merged as
/// JSON and read once.
///
/// A hand loader rather than `load::merged`: a copy is the file\'s JSON, merged into the lower
/// copies before the one typed read, not a document read on its own.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<SoundDefinitions, RawLayer>, SoundCopy> =
    CollectionLoader {
        walk: Walk::exact_files(&[DEFINITIONS_PATH]),
        parse: parse_copy,
        combine: merge_definitions,
        needs: &[],
        then: None,
    };

impl Content for SoundDefinitions {
    const NAME: &'static str = "sound definitions";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one pack's `sounds/sound_definitions.json` on its own, as the only copy of a
    /// stack: the version steps, then the typed read. A file that is no JSON is rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let any = Self::parse_as_written(path, bytes, cx)?;
            let (origin, version) = (any.origin().clone(), any.version().clone());
            let mut found = any.diagnostics().clone();
            let mut steps = Diagnostics::default();
            let json = match any.value() {
                history::Any::V1_0_0(value) => &value.definitions,
                history::Any::V1_14_0(value) => &value.document,
                history::Any::V1_20_20(value) => &value.document,
                history::Any::V1_26_50(value) => &value.json,
            };
            // The same findings a stack gives its lowest copy.
            let upgraded = prepare_definitions(json, None, &mut steps);
            let value = Self::read(upgraded, &mut steps);
            // A file without the wrapper has its definitions at the root: a finding of the
            // steps and of the reading, which name them in the upgraded document, is at the
            // member as the file has it.
            let steps = if matches!(any.value(), history::Any::V1_0_0(_)) {
                steps.into_iter().map(at_the_root).collect()
            } else {
                steps
            };
            located(steps, &origin, &mut found);
            let version = VersionInfo::new(
                version.written,
                false,
                version.effective,
                version.parsed_as,
                version.parsed_as != <Self as Versioned>::VERSION,
            );
            Ok(Document::new(value, origin, version, found))
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        // Not core's `write_envelope`: the version steps edit the document in place, so the
        // document read (envelope and all) is what is written.
        write_document(&self.json, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for SoundDefinitions {}

impl SoundDefinitions {
    /// Reads a file without upgrading it: the struct its `format_version` selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        _cx: &LoadContext,
    ) -> Result<Document<history::Any>, Rejected> {
        let origin = lone(path);
        let json = match read_json(bytes) {
            Ok(json) => json,
            Err(message) => return Err(not_json(origin, &message)),
        };
        let written = upgrade::version_member(&json);
        let gate = upgrade::gate_version(&json).format_version();
        // The newest struct's own findings come from `Content::parse`, once.
        let any = history::Any::read(json, &mut Diagnostics::default());
        let version = VersionInfo::new(
            written.map(|version| version.format_version()),
            false,
            gate,
            any.parsed_version(),
            false,
        );
        Ok(Document::new(any, origin, version, Diagnostics::default()))
    }
}

impl Placed for SoundDefinitions {
    /// `sounds/sound_definitions.json`: the one path the client reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(DEFINITIONS_PATH)
    }
}
