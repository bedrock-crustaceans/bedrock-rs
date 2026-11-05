//! `MolangFieldSpec`'s presets and builder reproduce the spec consts the families spell field
//! by field. Each literal below is copied from the file named; the builder form must equal it.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::{
    Admission, BLOCK_STATE_QUERIES, EmptyRule, MolangFailure, MolangFieldSpec, MolangVersion,
    MolangVersionSource, OpsRule, ReaderKind,
};

const C13: MolangVersionSource = MolangVersionSource::Constant(MolangVersion::V13);

#[test]
fn the_presets_are_default_kept_and_default_rejecting() {
    assert_eq!(
        MolangFieldSpec::KEPT,
        MolangFieldSpec {
            error_kept: true,
            ..MolangFieldSpec::DEFAULT
        }
    );
    assert_eq!(
        MolangFieldSpec::REJECTING,
        MolangFieldSpec {
            on_failure: MolangFailure::RejectDocument,
            ..MolangFieldSpec::DEFAULT
        }
    );
}

/// `families/particle/src/read.rs` `KEPT`: the preset itself.
#[test]
fn particle_kept() {
    let literal = MolangFieldSpec {
        error_severity: Severity::Error,
        error_kept: true,
        ..MolangFieldSpec::DEFAULT
    };
    assert_eq!(MolangFieldSpec::KEPT, literal);
}

/// Schema-validated, compiled when empty, rejecting: `support/src/animation/molang.rs`
/// `ANIMATION_VALUE`, `families/render_controller/src/molang.rs` `SCHEMA_VALUE`,
/// `families/geometry/src/molang.rs` `BINDING` (three crates).
#[test]
fn schema_value_rejecting() {
    let binding = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        version: MolangVersionSource::Pack,
        admission: Admission::Default,
        ops: OpsRule::All,
        empty: EmptyRule::Compile,
        on_failure: MolangFailure::RejectDocument,
        error_severity: Severity::Error,
        error_kept: false,
    };
    let built = MolangFieldSpec::REJECTING
        .reader(ReaderKind::SchemaValidated)
        .compile_empty();
    assert_eq!(built, binding);
    // feature's `WORLD_GEN` is the same with the world_gen set.
    assert_eq!(
        built.world_gen(),
        MolangFieldSpec {
            admission: Admission::WorldGen,
            ..binding
        }
    );
}

/// Schema-validated, kept: `families/entity/src/validate.rs` `SET_PROPERTY`,
/// `families/entity/src/component/molang.rs` `LENIENT`, `families/entity/src/dynamic.rs`
/// `MEMBER`.
#[test]
fn schema_member_kept() {
    let member = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        error_severity: Severity::Error,
        error_kept: true,
        ..MolangFieldSpec::DEFAULT
    };
    assert_eq!(
        MolangFieldSpec::KEPT.reader(ReaderKind::SchemaValidated),
        member
    );
}

/// Tags at version 13, kept: `families/entity/src/component/value.rs` `DESCRIPTOR_TAGS`,
/// `families/entity/src/goal/molang.rs` `TAGS`, `families/item/src/component/mod.rs`
/// `BLOCK_TAGS_MOLANG`.
#[test]
fn tags_at_version_13_kept() {
    let descriptor_tags = MolangFieldSpec {
        version: C13,
        admission: Admission::Tags,
        error_severity: Severity::Error,
        error_kept: true,
        ..MolangFieldSpec::DEFAULT
    };
    assert_eq!(MolangFieldSpec::KEPT.version(C13).tags(), descriptor_tags);
}

/// Schema-validated, empty not compiled, rejecting: `families/attachable/src/validate.rs` and
/// `families/client_entity/src/validate.rs` `NAMED_SCRIPT` (and their aliases).
#[test]
fn named_script_rejecting() {
    let named_script = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        on_failure: MolangFailure::RejectDocument,
        ..MolangFieldSpec::DEFAULT
    };
    assert_eq!(
        MolangFieldSpec::REJECTING.reader(ReaderKind::SchemaValidated),
        named_script
    );
}

/// `held_item_scale`: attachable and `client_entity` `HELD_ITEM_SCALE`.
#[test]
fn held_item_scale_rejecting() {
    let held_item_scale = MolangFieldSpec {
        reader: ReaderKind::ScalarOrArray,
        empty: EmptyRule::Compile,
        on_failure: MolangFailure::RejectDocument,
        ..MolangFieldSpec::DEFAULT
    };
    assert_eq!(
        MolangFieldSpec::REJECTING
            .reader(ReaderKind::ScalarOrArray)
            .compile_empty(),
        held_item_scale
    );
}

/// `initialize` / `pre_animation`: attachable and `client_entity` `LOADER_SCRIPT`.
#[test]
fn loader_script_drops_its_entry() {
    let loader_script = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        on_failure: MolangFailure::DropEntry,
        error_severity: Severity::Error,
        error_kept: true,
        ..MolangFieldSpec::DEFAULT
    };
    assert_eq!(
        MolangFieldSpec::KEPT
            .reader(ReaderKind::SchemaValidated)
            .on_failure(MolangFailure::DropEntry),
        loader_script
    );
}

/// The scatter members: `support/src/scatter.rs` and `families/feature/src/walk.rs` `SCATTER`.
#[test]
fn scatter_kept() {
    let scatter = MolangFieldSpec {
        reader: ReaderKind::LenientVersioned,
        version: MolangVersionSource::PackIgnoringObject,
        admission: Admission::WorldGen,
        ops: OpsRule::All,
        empty: EmptyRule::NoExpression,
        on_failure: MolangFailure::KeepLogged,
        error_severity: Severity::Error,
        error_kept: true,
    };
    assert_eq!(
        MolangFieldSpec::KEPT
            .reader(ReaderKind::LenientVersioned)
            .version(MolangVersionSource::PackIgnoringObject)
            .world_gen(),
        scatter
    );
}

/// Legacy item expressions at the item's format version: `families/item/src/expressions.rs`
/// `spec(..)` and `families/item/src/component/mod.rs` `FORMAT_VERSION_MOLANG`.
#[test]
fn item_format_version_kept() {
    let spec = |reader| MolangFieldSpec {
        reader,
        version: MolangVersionSource::FormatVersion,
        error_severity: Severity::Error,
        error_kept: true,
        ..MolangFieldSpec::DEFAULT
    };
    for reader in [
        ReaderKind::StrictVersioned,
        ReaderKind::ScalarOrArray,
        ReaderKind::SchemaValidated,
    ] {
        assert_eq!(
            MolangFieldSpec::KEPT
                .version(MolangVersionSource::FormatVersion)
                .reader(reader),
            spec(reader)
        );
    }
}

/// The block permutation conditions: `families/block/src/parse.rs` `CONDITION` (cereal) and
/// `CONDITION_LEGACY`.
#[test]
fn block_conditions() {
    let condition = MolangFieldSpec {
        reader: ReaderKind::LenientVersioned,
        version: C13,
        admission: Admission::Default,
        ops: OpsRule::All,
        empty: EmptyRule::BlockNoExpression,
        on_failure: MolangFailure::KeepLogged,
        error_severity: Severity::Error,
        error_kept: true,
    };
    assert_eq!(
        MolangFieldSpec::KEPT
            .reader(ReaderKind::LenientVersioned)
            .version(C13)
            .empty(EmptyRule::BlockNoExpression),
        condition
    );
    let legacy = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        version: MolangVersionSource::Pack,
        admission: Admission::Only(BLOCK_STATE_QUERIES),
        ops: OpsRule::NoAssignmentsOrRandom,
        empty: EmptyRule::NoExpression,
        on_failure: MolangFailure::RejectDocument,
        error_severity: Severity::Error,
        error_kept: false,
    };
    assert_eq!(
        MolangFieldSpec::REJECTING
            .reader(ReaderKind::SchemaValidated)
            .only(BLOCK_STATE_QUERIES)
            .ops(OpsRule::NoAssignmentsOrRandom),
        legacy
    );
    // `CONDITION_OLD`: the legacy one, its entry dropped and the error kept.
    assert_eq!(
        MolangFieldSpec::REJECTING
            .reader(ReaderKind::SchemaValidated)
            .only(BLOCK_STATE_QUERIES)
            .ops(OpsRule::NoAssignmentsOrRandom)
            .on_failure(MolangFailure::DropEntry)
            .kept(),
        MolangFieldSpec {
            on_failure: MolangFailure::DropEntry,
            error_kept: true,
            ..legacy
        }
    );
}

/// The two severity and keep setters round-trip.
#[test]
fn severity_and_keep_setters() {
    let spec = MolangFieldSpec::DEFAULT.error_severity(Severity::Warning);
    assert_eq!(spec.error_severity, Severity::Warning);
    assert_eq!(MolangFieldSpec::KEPT.not_kept(), MolangFieldSpec::DEFAULT);
}
