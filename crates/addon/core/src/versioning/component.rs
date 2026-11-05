//! Components one at a time: each entry of a `components` object is parsed from its own slice,
//! so a failing one is dropped and the rest survive; and the generic per-component checks on
//! the newest struct (release gates, deprecation, area, superseded versions).

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::payload::{join, read_with_rule, slice_error, span_of};
use super::{
    ComponentTable, ContextArea, ConvertMode, ErrorMask, ExperimentalField, Family, FieldGate,
    FieldMeta,
};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use crate::document::{ComponentMap, ComponentRef, ComponentSet, CustomComponent};
use crate::json::{JsonSlice, NullRule, Value, keep_members, reject_nulls, remove_null_members};
use crate::load::{Experiment, LoadContext, LoadRequirement};
use crate::version::FormatVersion;

/// Parses one typed component from its slice: `T` from the text (strict wrappers decide what
/// coerces, so `mode` is the caller's to honour), its unknown members kept in its `extra` and
/// reported as `UnrecognizedField` warnings (the strict mask turns them into errors after the
/// parse). A failed parse (a member of the wrong type, `0x8`, or a required member missing,
/// `0x80`) is one finding: in a cereal slice of a document-input family
/// ([`LoadContext::strict_components`]) an error, which rejects the whole document under both
/// masks; elsewhere (`JsonCpp` slices, hand-written legacy slices) an error too. A legacy
/// block whose component fails to parse logs "Failed to parse block '%s'" at level 3 and the
/// game refuses the whole block, and the `JsonCpp` readers fail a value of another type with
/// code `0x8`, an error under both masks. Findings are relative to the component; the
/// component-map parser places them. The usual `parse` of a `#[derive(Components)]` row:
/// `parse = versioning::parse_component::<Friction, ItemComponent>`.
pub fn parse_component<T, C>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<C>
where
    T: for<'a> Facet<'a> + Into<C>,
{
    let _ = mode;
    let rule = cx.null_rule;
    let mut value = match read_with_rule::<T>(slice, rule) {
        Ok(value) => value,
        Err(error) => {
            // Both readers fail the slice; `strict_components` only names the input.
            let _ = cx.strict_components;
            d.push(slice_error(
                &error,
                &JsonPath::new(),
                slice,
                Severity::Error,
            ));
            return None;
        }
    };
    // Under the `Rejected` null rule a `null` where the component reads a value is a type
    // error, which rejects the document.
    if rule == Some(NullRule::Rejected) && reject_nulls(&value, slice, &JsonPath::new(), d) {
        return None;
    }
    if let Ok(mut json) = crate::json::from_str_jsonc::<Value>(slice.text) {
        if rule == Some(NullRule::Absent) {
            // Each member read as absent is marked: the write-back writes its default.
            let absent = remove_null_members(T::SHAPE, &mut json);
            crate::json::tell_null_members(&absent, &JsonPath::new(), d);
        }
        let (unknown, misread) = keep_members(&mut value, &json);
        for (path, _) in unknown {
            // The game logs an unknown member at content-log level 2, a warning; a `Strict`
            // slice makes it an error after the parse.
            d.push(Diagnostic::new(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Warning,
                Phase::Parse,
                Location::new(None, None, path, None),
            ));
        }
        // A scalar its member read as another value is told where it is.
        for misread in misread {
            d.push(misread.finding(&JsonPath::new()));
        }
    }
    Some(value.into())
}

/// Parses a `components` object held as raw slices (`entries`, borrowed from `within`, the
/// payload's text) into family `F`'s component map, in file order.
///
/// A key of `F`'s table is parsed by its row's `parse`; the findings it reports are placed below
/// `at/<key>` (`at` is the object's path in the file). A row whose value has no typed key (an
/// entity's dynamic keys) keeps the JSON as a dynamic entry. A key `is_custom` accepts is a
/// custom component, kept as written. Any other key is kept for write-back and reported as an
/// `UnrecognizedComponent` warning.
pub fn parse_component_map<'a, F: Family>(
    entries: &IndexMap<String, RawJson<'a>>,
    within: JsonSlice<'a>,
    at: &JsonPath,
    mode: ConvertMode,
    is_custom: fn(&str) -> bool,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> ComponentMap<F::Component> {
    let table = table::<F>();
    let mut map = ComponentMap::new();
    for (key, raw) in entries {
        let slice = within.nested(raw).unwrap_or(JsonSlice {
            text: raw.as_str(),
            offset: within.offset,
        });
        let mut path = at.clone();
        path.push(key);
        let json = || crate::json::from_str_jsonc::<Value>(slice.text).unwrap_or(Value::NULL);
        if let Some(meta) = table.get(key) {
            let mut found = Diagnostics::default();
            let value = (meta.parse)(slice, mode, cx, &mut found);
            // A component that goes for what is told of a member BELOW it is told at the
            // component too: the member's finding does not say that its siblings go with
            // it. An error refuses the file, and a finding at the component speaks for it.
            let told_at_it = found.has_errors()
                || found
                    .iter()
                    .any(|finding| finding.location.json_path.is_empty());
            for mut diagnostic in found {
                diagnostic.location.json_path = join(&path, &diagnostic.location.json_path);
                d.push(diagnostic);
            }
            match value {
                Some(value) if value.name().is_some() => {
                    map.insert_typed(value, false);
                }
                Some(_) => map.insert_dynamic(key, json()),
                None if told_at_it => {}
                // The game says nothing here, so the crate reports a read that failed without a
                // finding of its own as an information.
                None => d.push(Diagnostic::new(
                    DiagnosticCode::DroppedComponent { text: None },
                    Severity::Info,
                    Phase::Parse,
                    Location::new(None, None, path, Some(span_of(slice))),
                )),
            }
        } else if is_custom(key) {
            map.insert_custom(CustomComponent {
                name: key.clone(),
                parameters: json(),
                from_v1_list: false,
            });
        } else {
            map.insert_unknown(key, json());
            d.push(Diagnostic::new(
                // Cereal's own text for a component the schema does not list: level 2, a
                // warning; a `Strict` slice makes it an error after the parse.
                crate::diagnostic::cereal::component_not_in_schema(),
                Severity::Warning,
                Phase::Parse,
                Location::new(None, None, path, Some(span_of(slice))),
            ));
        }
    }
    map
}

/// One component map of the newest struct, for the generic per-component checks.
pub struct ComponentSection<'a, C: ComponentSet> {
    /// Its path below the payload (`components`, `permutations/2/components`).
    pub at: JsonPath,
    /// A failed requirement skips the section (a permutation) instead of rejecting the
    /// document (the base components).
    pub optional: bool,
    /// The map.
    pub map: &'a mut ComponentMap<C>,
}

impl<'a, C: ComponentSet> ComponentSection<'a, C> {
    /// The section of `map` at `at` (below the payload); `optional` for a permutation.
    pub fn new(at: JsonPath, optional: bool, map: &'a mut ComponentMap<C>) -> Self {
        Self { at, optional, map }
    }
}

/// The generic per-component checks on one map of the newest struct (the game's release-gate
/// check, compared against `original_version`), at `at` in the file. `Custom` entries and
/// `synthesized` ones bypass the table.
///
/// - Gates: allowed when `released` is `Some(r)` and `original_version >= r`; otherwise every
///   entry of `requires` must be satisfied, and an empty `requires` is no gate at all, unless
///   the family says `released` is one (`Family::RELEASED_IS_A_GATE`, [`check_released`]). A
///   failed requirement is `MissingToggles` with the text of its kind, an error (the document
///   is not loaded) for a base section, a warning for an optional one; the function then
///   returns `false`.
/// - Deprecation: at or above `deprecated_since`, `DeprecatedMember` (the row's message, else
///   "%s has been deprecated and is not available in json format %s") and the component is
///   dropped.
/// - Area: a `Vanilla` or `Internal` component outside a base-game pack is an
///   `UnrecognizedComponent` warning and is dropped.
/// - `supersedes`: a map holding both versions of a component is a
///   `DuplicateComponentVersion` warning and the older one is dropped.
pub fn validate_components<F: Family>(
    section: ComponentSection<'_, F::Component>,
    original_version: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let what = F::COMPONENT_NOUN;
    let ComponentSection {
        at: rel,
        optional,
        map,
    } = section;
    let at = join(at, &rel);
    let table = table::<F>();
    // An optional section's gate refuses the document too: a block permutation's toggle
    // lines are logged at level 3 and a failing permutation fails the component load, so the
    // whole block is refused.
    let _ = optional;
    let gate_severity = Severity::Error;
    // A deprecated or out-of-area component is reported like a component missing from the
    // schema: level 2 under the unstrict mask, an error under the strict one.
    let unknown_severity = match unknown_mask::<F>(original_version) {
        ErrorMask::Strict => Severity::Error,
        ErrorMask::Unstrict => Severity::Warning,
    };
    // A family whose `released` is a gate of its own.
    let mut passed =
        !F::RELEASED_IS_A_GATE || check_released::<F>(map, optional, original_version, &at, d);
    let mut dropped: Vec<String> = Vec::new();
    for entry in map.iter() {
        let name = match entry {
            ComponentRef::Typed {
                synthesized: true, ..
            }
            | ComponentRef::Custom(_)
            | ComponentRef::Unknown { .. } => continue,
            ComponentRef::Typed { name, .. } | ComponentRef::Dynamic { name, .. } => name,
        };
        let Some(meta) = table.get(name) else {
            continue;
        };
        let mut path = at.clone();
        path.push(name);
        let released = meta.released.is_some_and(|r| original_version >= r);
        if !released && let Some(failed) = meta.requires.iter().find(|r| !r.satisfied(cx)) {
            passed = false;
            d.push(finding(
                DiagnosticCode::MissingToggles {
                    text: requirement_text(*failed, what, name, meta.released),
                },
                gate_severity,
                path.clone(),
            ));
        }
        if let Some(since) = meta.deprecated_since
            && original_version >= since
        {
            let text = meta.deprecated_message.map_or_else(
                || VanillaText::Deprecated {
                    member: name.to_owned(),
                    version: original_version,
                },
                VanillaText::Fixed,
            );
            // `minecraft:foot_size` at 1.16.100: the game logs the not-in-schema line at
            // level 2 and keeps the actor.
            d.push(finding(
                DiagnosticCode::DeprecatedMember { text: Some(text) },
                unknown_severity,
                path.clone(),
            ));
            dropped.push(name.to_owned());
        }
        if matches!(meta.area, ContextArea::Vanilla | ContextArea::Internal)
            && !cx.is_base_game_pack
        {
            d.push(finding(
                DiagnosticCode::UnrecognizedComponent { text: None },
                unknown_severity,
                path.clone(),
            ));
            dropped.push(name.to_owned());
        }
        if let Some(old) = meta.supersedes
            && map.iter().any(|e| entry_name(&e) == old)
        {
            // The game's "Two versions of the same component" line belongs to the block data
            // fixer, and no block row has `supersedes`; an entity's rename logs nothing. The
            // crate reports the dropped older version as an information.
            d.push(finding(
                DiagnosticCode::DuplicateComponentVersion {
                    old: old.to_owned(),
                    new: name.to_owned(),
                },
                Severity::Info,
                path,
            ));
            dropped.push(old.to_owned());
        }
    }
    for name in &dropped {
        map.remove_named(name);
    }
    passed
}

/// `released` as a gate of its own, for a family whose loaders bind components per
/// format version (`Family::RELEASED_IS_A_GATE`; [`validate_components`] runs it then): a
/// component of `map` whose table row has `released` above `original_version` and an empty
/// `requires` is an `UnsupportedVersion` finding with "To use %s '%s', use json format version
/// %s or higher" at its key below `at`: an error (the document is not loaded), a warning for
/// an `optional` section. `false` when one was found. `Custom`, unknown and synthesized
/// entries are not judged, and a row with requirements is [`validate_components`]' own.
pub fn check_released<F: Family>(
    map: &ComponentMap<F::Component>,
    optional: bool,
    original_version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let table = table::<F>();
    // As `validate_components`' gate: an optional section's refuses the document too.
    let _ = optional;
    let severity = Severity::Error;
    let mut passed = true;
    for entry in map.iter() {
        let name = match entry {
            ComponentRef::Typed {
                synthesized: true, ..
            }
            | ComponentRef::Custom(_)
            | ComponentRef::Unknown { .. } => continue,
            ComponentRef::Typed { name, .. } | ComponentRef::Dynamic { name, .. } => name,
        };
        let Some(meta) = table.get(name) else {
            continue;
        };
        let Some(released) = meta.released.filter(|r| original_version < *r) else {
            continue;
        };
        if !meta.requires.is_empty() {
            continue;
        }
        passed = false;
        let mut path = at.clone();
        path.push(name);
        d.push(finding(
            use_format_version(F::COMPONENT_NOUN, name, released),
            severity,
            path,
        ));
    }
    passed
}

/// The format-version field gate: every member of `fields` that `present` finds and
/// whose `released` is above `original_version` is an `UnsupportedVersion` error with "To use
/// field '%s.%s', use json format version %s or higher" (owner, then member) at the member below
/// `at` (a dotted name continues the path). `false` when one was found: the document is not
/// loaded, as vanilla's strict loaders refuse a member the version's struct does not bind.
pub fn check_fields_released(
    fields: &[FieldGate],
    present: impl Fn(&str) -> bool,
    owner: &str,
    original_version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let mut passed = true;
    for field in fields {
        if original_version >= field.released || !present(field.name) {
            continue;
        }
        passed = false;
        let mut path = at.clone();
        for part in field.name.split('.').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        d.push(finding(
            use_format_version("field", &format!("{owner}.{}", field.name), field.released),
            Severity::Error,
            path,
        ));
    }
    passed
}

/// The experiment-gated fields of a beta struct (the game refuses such a member with result
/// code `0x40` while its experiment is off): every member of `fields`
/// that `present` finds while its experiment is off is a `MissingToggles` error naming the
/// experiment (`To use field '<owner>.<member>', turn on the '<experiment>' toggle in
/// settings`) at the member below `at` (a dotted name continues the path). `false` when one
/// was found: the document is not loaded.
pub fn check_experimental_fields(
    fields: &[ExperimentalField],
    present: impl Fn(&str) -> bool,
    owner: &str,
    at: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> bool {
    let mut passed = true;
    for field in fields {
        if LoadRequirement::Experiment(field.experiment).satisfied(cx) || !present(field.name) {
            continue;
        }
        passed = false;
        let mut path = at.clone();
        for part in field.name.split('.').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        d.push(Diagnostic::new(
            DiagnosticCode::MissingToggles {
                text: Some(VanillaText::TurnOnToggle {
                    what: "field",
                    name: format!("{owner}.{}", field.name),
                    toggle: field.experiment.save_name().unwrap_or_default(),
                }),
            },
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, path, None),
        ));
    }
    passed
}

/// "To use %s '%s', use json format version %s or higher".
fn use_format_version(what: &'static str, name: &str, version: FormatVersion) -> DiagnosticCode {
    DiagnosticCode::UnsupportedVersion {
        text: Some(VanillaText::UseFormatVersion {
            what,
            name: name.to_owned(),
            version,
        }),
    }
}

/// `FieldSinceEngine` warnings for every field of `fields` that `present` finds in a
/// component and that arrived after the pack's `min_engine_version` (an absent one counts as
/// older than every field). `at` is the component's path; a field's dotted name continues it.
pub fn check_fields_since_engine(
    fields: &[FieldMeta],
    present: impl Fn(&str) -> bool,
    at: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) {
    let pack = cx
        .min_engine_version
        .as_ref()
        .map(|m| m.version.format_version());
    for field in fields {
        if pack.is_some_and(|p| p >= field.since_engine) || !present(field.name) {
            continue;
        }
        let mut path = at.clone();
        for part in field.name.split('.').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        // The game has no line for a field newer than the pack, so the crate reports it as an
        // information.
        d.push(finding(
            DiagnosticCode::FieldSinceEngine {
                since: field.since_engine,
            },
            Severity::Info,
            path,
        ));
    }
}

/// The text of a failed requirement: the legacy creator features name the version
/// that needs none; an experiment names its toggle, and the version from which none is
/// needed when the component has one.
fn requirement_text(
    requirement: LoadRequirement,
    what: &'static str,
    name: &str,
    released: Option<FormatVersion>,
) -> Option<VanillaText> {
    let toggle = |experiment: Experiment| experiment.save_name().unwrap_or_default();
    let experiment = match requirement {
        LoadRequirement::LegacyCreatorFeatures => {
            return released.map(|version| VanillaText::UseFormatVersion {
                what,
                name: name.to_owned(),
                version,
            });
        }
        LoadRequirement::BaseGamePack => return None,
        LoadRequirement::BetaFeatures => Experiment::BetaApis,
        LoadRequirement::Experiment(experiment) => experiment,
    };
    Some(match released {
        Some(version) => VanillaText::TurnOnToggleBefore {
            what,
            name: name.to_owned(),
            version,
            toggle: toggle(experiment),
        },
        None => VanillaText::TurnOnToggle {
            what,
            name: name.to_owned(),
            toggle: toggle(experiment),
        },
    })
}

/// The key of a map entry, for the `supersedes` lookup.
fn entry_name<'a, C>(entry: &ComponentRef<'a, C>) -> &'a str {
    match *entry {
        ComponentRef::Typed { name, .. }
        | ComponentRef::Dynamic { name, .. }
        | ComponentRef::Unknown { name, .. } => name,
        ComponentRef::Custom(custom) => &custom.name,
    }
}

/// A finding of the component checks at `path`.
/// `F`'s component table, where the pipeline reads components. A family whose component set
/// is inhabited (not [`NoComponents`](super::NoComponents), which has size 0) and that kept
/// the trait's empty default forgot to override [`Family::components`]: every component of
/// its documents would read as unknown. A debug assertion, since the table is static.
fn table<F: Family>() -> &'static ComponentTable<F> {
    let table = F::components();
    debug_assert!(
        size_of::<F::Component>() == 0 || !table.0.is_empty(),
        "family `{}` has components but an empty table: override `Family::components`",
        F::NAME
    );
    table
}

fn finding(code: DiagnosticCode, severity: Severity, path: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(None, None, path, None),
    )
}

/// The error mask of the struct that reads a document written at `version`: the mask the
/// unknown-component findings of the validate phase follow.
fn unknown_mask<F: Family>(version: FormatVersion) -> ErrorMask {
    let input = crate::version::FormatVersionInput {
        version: Some(version),
        beta_literal: false,
        prerelease: false,
    };
    match super::range::select::<F>(&input) {
        super::range::Selection::Struct(i) => F::error_mask(super::StructIndex::Stable(i)),
        _ => ErrorMask::Unstrict,
    }
}
