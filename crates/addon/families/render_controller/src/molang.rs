//! The Molang members of a render controller:
//! their specs, the walk over them, pass 1 (versions) and pass 2 (the compile check, with the
//! scope of a rejection: the file for a schema-typed member, the controller for the members the
//! controller parser compiles itself).

use super::{RenderController, RenderControllerFile, Rgba};
use bedrock_addon_core::diagnostic::{Diagnostics, Location};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    MolangContext, MolangFailure, MolangFieldSpec, MolangFields, MolangVisitor, ReaderKind, Step,
    not_checked, pass_one,
};
use bedrock_addon_core::version::FormatVersionInput;

const fn spec(on_failure: MolangFailure) -> MolangFieldSpec {
    // The version is always the pack's `min_engine_version` (the default); the file's
    // version never enters.
    MolangFieldSpec::DEFAULT
        .reader(ReaderKind::SchemaValidated)
        .compile_empty()
        .on_failure(on_failure)
}

/// A member the schema types as an expression (`materials` and `part_visibility` values,
/// colour channels, `uv_anim` elements, `light_color_multiplier`): a rejection fails the
/// schema, and with it the file.
pub(crate) const SCHEMA_VALUE: MolangFieldSpec = spec(MolangFailure::RejectDocument);
/// A member the schema types as a string and the controller parser compiles (`geometry`,
/// `textures`, `arrays` lists): a rejection drops the controller.
pub(crate) const PARSED_SOURCE: MolangFieldSpec = spec(MolangFailure::DropEntry);

fn color(visitor: &mut MolangVisitor<'_>, key: &str, color: &mut Option<Rgba>) {
    let Some(color) = color else { return };
    visitor.enter(Step::Key(key));
    visitor.field("r", &SCHEMA_VALUE, &mut color.r);
    visitor.field("g", &SCHEMA_VALUE, &mut color.g);
    visitor.field("b", &SCHEMA_VALUE, &mut color.b);
    visitor.field("a", &SCHEMA_VALUE, &mut color.a);
    visitor.leave();
}

impl MolangFields for RenderController {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("geometry", &PARSED_SOURCE, &mut self.geometry);
        visitor.field("textures", &PARSED_SOURCE, &mut self.textures);
        // The index is the pair's position in the list, which is the element's index in the
        // file when every element object holds one pattern (as it does when the crate writes it).
        visitor.enter(Step::Key("materials"));
        for (index, (pattern, value)) in self.materials.iter_mut().enumerate() {
            visitor.enter(Step::Index(index));
            visitor.field(pattern.0.as_str(), &SCHEMA_VALUE, value);
            visitor.leave();
        }
        visitor.leave();
        visitor.enter(Step::Key("part_visibility"));
        for (index, (pattern, value)) in self.part_visibility.iter_mut().enumerate() {
            visitor.enter(Step::Index(index));
            visitor.field(pattern.0.as_str(), &SCHEMA_VALUE, value);
            visitor.leave();
        }
        visitor.leave();
        if let Some(arrays) = &mut self.arrays {
            visitor.enter(Step::Key("arrays"));
            visitor.field("geometries", &PARSED_SOURCE, &mut arrays.geometries);
            visitor.field("materials", &PARSED_SOURCE, &mut arrays.materials);
            visitor.field("textures", &PARSED_SOURCE, &mut arrays.textures);
            visitor.leave();
        }
        color(visitor, "color", &mut self.color);
        color(visitor, "overlay_color", &mut self.overlay_color);
        color(visitor, "on_fire_color", &mut self.on_fire_color);
        color(visitor, "is_hurt_color", &mut self.is_hurt_color);
        if let Some(uv_anim) = &mut self.uv_anim {
            visitor.enter(Step::Key("uv_anim"));
            for (key, pair) in [
                ("offset", &mut uv_anim.offset),
                ("scale", &mut uv_anim.scale),
            ] {
                visitor.enter(Step::Key(key));
                for (index, value) in pair.iter_mut().enumerate() {
                    visitor.field(index, &SCHEMA_VALUE, value);
                }
                visitor.leave();
            }
            visitor.leave();
        }
        visitor.field(
            "light_color_multiplier",
            &SCHEMA_VALUE,
            &mut self.light_color_multiplier,
        );
    }
}

impl MolangFields for RenderControllerFile {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::Key("render_controllers"));
        for (key, controller) in &mut self.render_controllers {
            visitor.nested(key.as_str(), controller);
        }
        visitor.leave();
    }
}

// ---- the passes --------------------------------------------------------------------------------

/// How a file's Molang is checked.
#[cfg_attr(
    not(feature = "molang"),
    allow(dead_code, reason = "pass 2 reads the numeric materials")
)]
pub(crate) struct MolangRun<'a> {
    pub(crate) load: &'a LoadContext,
    /// The file's `format_version` (no member takes its version from it).
    pub(crate) written: FormatVersionInput,
    /// The `materials` values that were JSON numbers, by controller key and position.
    pub(crate) numeric_materials: &'a indexmap::IndexSet<(String, usize)>,
}

/// What the passes decided.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct MolangOutcome {
    /// A schema-typed member was rejected: the file is skipped.
    pub(crate) file_rejected: bool,
    /// The controllers a member of theirs dropped, in the file's order.
    pub(crate) dropped: indexmap::IndexSet<String>,
}

/// Pass 1 and pass 2 over `file`.
pub(crate) fn run(
    file: &mut RenderControllerFile,
    run: &MolangRun<'_>,
    diagnostics: &mut Diagnostics,
) -> MolangOutcome {
    let cx = MolangContext::new(
        run.load.min_engine_version.as_ref(),
        run.load.molang_experiments,
        &run.written,
        Some(run.load.molang_version),
    );
    let base = Location::default();
    let pass = pass_one(file, &cx, &base, diagnostics);
    if pass.failed > 0 {
        return MolangOutcome {
            file_rejected: true,
            dropped: indexmap::IndexSet::new(),
        };
    }
    #[cfg(feature = "molang")]
    if let Some(check) = &run.load.molang {
        return check::pass_two(file, check, run, &cx, diagnostics);
    }
    // Without the compile check the texts are stored unchecked, one info per file.
    not_checked(file, &base, diagnostics);
    MolangOutcome::default()
}

#[cfg(feature = "molang")]
mod check {
    use super::{MolangOutcome, MolangRun, PARSED_SOURCE, SCHEMA_VALUE};
    use crate::read::controller_of;
    use bedrock_addon_core::diagnostic::codes as core_codes;
    use bedrock_addon_core::diagnostic::{
        Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Location, Phase, Severity,
    };
    use bedrock_addon_core::molang::{
        MolangCheck, MolangContext, MolangFieldMut, MolangFields, ReportRules, Side, visit_fields,
    };

    /// `Error parsing: %s`: the line the array parser adds after a failing element's own.
    const ERROR_PARSING: &str = "Error parsing: ";

    /// What the controller loader reports beyond the load's rules: molang's argument-count lint
    /// stays for every call. Every member compiles with the client catalogue, so the server-side
    /// rules never apply.
    const RULES: ReportRules = ReportRules {
        vanilla_arities: false,
        ..ReportRules::LOAD
    };

    /// The position of a `materials` pair, when `path` is one's.
    fn material_index(path: &JsonPath) -> Option<usize> {
        let mut segments = path.segments().skip(2);
        match (segments.next(), segments.next()) {
            (Some(JsonPathSegment::Key("materials")), Some(JsonPathSegment::Index(index))) => {
                Some(index)
            }
            _ => None,
        }
    }

    /// The compile check of every member: the
    /// schema-typed sources first, whose rejection skips the file; then the sources the
    /// controller parser compiles, whose rejection drops their controller.
    pub(super) fn pass_two<T: MolangFields>(
        file: &mut T,
        check: &MolangCheck,
        run: &MolangRun<'_>,
        cx: &MolangContext,
        diagnostics: &mut Diagnostics,
    ) -> MolangOutcome {
        let mut outcome = MolangOutcome::default();
        for stage in [SCHEMA_VALUE, PARSED_SOURCE] {
            visit_fields(file, &JsonPath::new(), |field| {
                let source = match field.value {
                    MolangFieldMut::Value(value) => value.source(),
                    MolangFieldMut::Source(source) => Some(&*source),
                    MolangFieldMut::List(_) => None,
                };
                let Some(source) = source else { return };
                let Some(key) = controller_of(field.path) else {
                    return;
                };
                // A `materials` number passed the schema as a constant; the controller parser
                // then compiles the empty text it reads it as (a `materials` given a number).
                let numeric = *field.spec == SCHEMA_VALUE
                    && material_index(field.path).is_some_and(|index| {
                        !run.numeric_materials.is_empty()
                            && run.numeric_materials.contains(&(key.to_owned(), index))
                    });
                let schema_typed = *field.spec == SCHEMA_VALUE && !numeric;
                if schema_typed != (stage == SCHEMA_VALUE) {
                    return;
                }
                let location = Location::new(None, None, field.path.clone(), None);
                // Every member compiles with the client catalogue: render controllers exist
                // only on the client, whatever kind of pack carries the file.
                let start = diagnostics.len();
                let (field_outcome, _) = check.check_source_with(
                    source,
                    source,
                    field.spec,
                    Side::Client,
                    cx,
                    &location,
                    &RULES,
                    diagnostics,
                );
                // `UsesArrays` and `UsesResources` are the normal outcome of `geometry`,
                // `textures`, `materials` and `arrays` sources.
                if !matches!(
                    field_outcome,
                    bedrock_addon_core::molang::MolangOutcome::Failed(_)
                ) {
                    return;
                }
                let reached = diagnostics
                    .iter()
                    .skip(start)
                    .any(|d| d.severity >= Severity::Error);
                let in_array = field
                    .path
                    .segments()
                    .nth(2)
                    .is_some_and(|s| s == JsonPathSegment::Key("arrays"));
                if in_array {
                    diagnostics.push(
                        Diagnostic::of(
                            core_codes::INVALID_MOLANG
                                .with([format!("{ERROR_PARSING}{}", source.as_str())]),
                        )
                        .at(field.path.clone())
                        .phase(Phase::Validate),
                    );
                } else if !reached && !numeric {
                    // A rejection with only an info note (a written empty text below version
                    // 4) takes the member's severity. The empty text a `materials` number is
                    // read as keeps the note alone: the reader has
                    // told the number at the member already.
                    diagnostics.push(
                        Diagnostic::of(
                            core_codes::INVALID_MOLANG.with([source.as_str().to_owned()]),
                        )
                        .at(field.path.clone())
                        .phase(Phase::Validate),
                    );
                }
                if schema_typed {
                    outcome.file_rejected = true;
                } else if !outcome.dropped.contains(key) {
                    outcome.dropped.insert(key.to_owned());
                }
            });
            if outcome.file_rejected {
                // The schema failed: the controller parser never runs.
                outcome.dropped.clear();
                break;
            }
        }
        outcome
    }
}
