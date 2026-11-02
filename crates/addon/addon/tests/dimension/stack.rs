//! Rows dim11 and dim12 and rules 8 and 9: `bp.dimensions()` reads
//! `dimensions/*.json` (exactly `json`, not recursive) from the last stack entry to the first,
//! and the first file walked wins.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::id::DimensionId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};

use super::pack_common::bp;

const HIGH: &[u8] = br#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:dimension_bounds": {"min": -64, "max": 320}, "minecraft:generation": {"generator_type": "void"}}}}"#;
const LOW: &[u8] = br#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": 256}, "minecraft:generation": {"generator_type": "void"}}}}"#;

fn overworld() -> DimensionId {
    DimensionId::new("minecraft:overworld").unwrap()
}

/// The `ShadowedDefinition` findings of the registry, with the pack of each.
fn shadowed(stack: &PackStack<Behavior>) -> Vec<(Severity, String)> {
    stack
        .dimensions()
        .diagnostics()
        .filter(|f| matches!(f.code, DiagnosticCode::ShadowedDefinition { .. }))
        .map(|f| {
            (
                f.severity,
                f.location
                    .pack
                    .as_ref()
                    .map(|p| p.name().to_owned())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn dim11_the_file_walked_first_from_the_last_stack_entry_wins_deviation() {
    let top = bp(1).file("dimensions/overworld.json", HIGH).open();
    let bottom = bp(2).file("dimensions/overworld.json", LOW).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let dimensions = stack.dimensions();
    assert_eq!(dimensions.len(), 1);
    let kept = dimensions.get(&overworld()).unwrap();
    assert_eq!(
        kept.components.dimension_height.as_ref().unwrap().min_y.0,
        0
    );
    assert_eq!(shadowed(&stack), [(Severity::Info, "bp 1".to_owned())]);
    assert!(std::ptr::eq(dimensions, stack.dimensions()));
}

#[test]
fn rule_8_within_one_pack_the_first_file_walked_wins_deviation() {
    let pack = bp(1)
        .file("dimensions/a.json", LOW)
        .file("dimensions/b.json", HIGH)
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let kept = stack.dimensions().get(&overworld()).unwrap();
    assert_eq!(kept.origin().path.as_str(), "dimensions/a.json");
    assert_eq!(shadowed(&stack), [(Severity::Info, "bp 1".to_owned())]);
}

#[test]
fn dim12_only_json_files_directly_in_dimensions_are_read() {
    let pack = bp(1)
        .file("dimensions/overworld.JSON", HIGH)
        .file("dimensions/notes.txt", b"not a dimension")
        .file("dimensions/sub/overworld.json", HIGH)
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    assert!(stack.dimensions().is_empty());
    assert!(stack.dimensions().rejected().is_empty());
    assert_eq!(stack.dimensions().diagnostics().count(), 0);
}

#[test]
fn a_rejected_file_is_listed_and_claims_nothing() {
    let broken: &[u8] = br#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {}}}"#;
    let top = bp(1).file("dimensions/overworld.json", HIGH).open();
    let bottom = bp(2).file("dimensions/overworld.json", broken).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    assert!(stack.dimensions().get(&overworld()).is_some());
    assert_eq!(stack.dimensions().rejected().len(), 1);
}

/// A script's custom-dimension registry, as the host answers it through `LoadOptions::built_ins`:
/// `test:void`, and the vanilla dimensions the host also knows.
struct Registered;

impl bedrock_addon::load::BuiltIns for Registered {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "dimension"
            && [
                "test:void",
                "minecraft:overworld",
                "minecraft:nether",
                "minecraft:the_end",
            ]
            .contains(&id)
    }
}

#[test]
fn dim18_a_pack_definition_replaces_a_script_registered_dimension() {
    const BROKEN: &[u8] = br#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:void"}, "components": {}}}"#;

    // 1.26.60 `DimensionManager::serverInitialize(…)::$_0`: an identifier the
    // script group has is replaced by the pack's definition (`tryReplaceDimensionDefinition`),
    // then a warning (level 2) names both.
    const TEST_VOID: &[u8] = br#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:void"}, "components": {"minecraft:generation": {"generator_type": "void"}}}}"#;
    let options = || LoadOptions {
        built_ins: Some(std::sync::Arc::new(Registered)),
        ..LoadOptions::default()
    };
    let pack = bp(1)
        .file("dimensions/test_void.json", TEST_VOID)
        .file("dimensions/overworld.json", HIGH)
        .open();
    let stack = PackStack::<Behavior>::of([pack], options()).unwrap();
    let dimensions = stack.dimensions();
    let test_void = DimensionId::new("test:void").unwrap();
    assert!(
        dimensions.get(&test_void).is_some(),
        "{:#?}",
        dimensions.rejected()
    );
    assert!(dimensions.rejected().is_empty());
    // The vanilla dimensions the host knows are not script registrations.
    assert!(dimensions.get(&overworld()).is_some());
    let found: Vec<_> = dimensions.diagnostics().collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(matches!(
        found[0].code,
        DiagnosticCode::ShadowedDefinition { .. }
    ));
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(
        found[0].code.to_string(),
        "Dimension definition identifier 'test:void' registered via scripting was replaced by the definition from pack id '00000000-0000-0000-0000-000000000001'"
    );
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:dimension/description/identifier"
    );
    assert_eq!(
        found[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon_core::pack_identity::PackPath::as_str),
        Some("dimensions/test_void.json")
    );

    // Of two packs' definitions the group keeps the first walked (the lowest pack's), and
    // that one replaces the registration: one warning, naming it.
    let top = bp(1).file("dimensions/test_void.json", TEST_VOID).open();
    let bottom = bp(2).file("dimensions/test_void.json", TEST_VOID).open();
    let stack = PackStack::<Behavior>::of([top, bottom], options()).unwrap();
    let replaced: Vec<String> = stack
        .dimensions()
        .diagnostics()
        .filter(|f| f.severity == Severity::Warning)
        .map(|f| f.code.to_string())
        .collect();
    assert_eq!(
        replaced,
        [
            "Dimension definition identifier 'test:void' registered via scripting was replaced by the definition from pack id '00000000-0000-0000-0000-000000000002'"
        ]
    );
    let kept = stack.dimensions().get(&test_void).unwrap();
    assert_eq!(kept.origin().pack.name(), "bp 2");

    // A pack file that fails to load reaches no registration: rejected as any file, and the
    // script's dimension stays.
    let pack = bp(1).file("dimensions/test_void.json", BROKEN).open();
    let stack = PackStack::<Behavior>::of([pack], options()).unwrap();
    assert_eq!(stack.dimensions().rejected().len(), 1);
    assert!(stack.dimensions().get(&test_void).is_none());
    assert!(
        stack
            .dimensions()
            .diagnostics()
            .all(|f| !matches!(f.code, DiagnosticCode::ShadowedDefinition { .. }))
    );

    // Without the registration the custom dimension loads, and nothing is said.
    let pack = bp(1).file("dimensions/test_void.json", TEST_VOID).open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    assert!(stack.dimensions().get(&test_void).is_some());
    assert_eq!(stack.dimensions().diagnostics().count(), 0);
}

// What a rejected file names is read from the copy of a
// doubled member that stands. Here the last `description` has no identifier, so the file
// names none: it is a rejected file, not a collision with a script registration.
#[test]
fn a_rejected_file_names_the_identifier_of_its_last_copy() {
    const DOUBLED: &[u8] = br#"{"format_version":"1.26.50","minecraft:dimension":{"description":{"identifier":"test:void"},"description":{},"components":{"minecraft:generation":{"generator_type":"void"}}}}"#;
    let pack = bp(1).file("dimensions/test_void.json", DOUBLED).open();
    let options = LoadOptions {
        built_ins: Some(std::sync::Arc::new(Registered)),
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of([pack], options).unwrap();
    let dimensions = stack.dimensions();
    assert_eq!(
        dimensions.rejected().len(),
        1,
        "{:#?}",
        dimensions.rejected()
    );
    let found: Vec<_> = dimensions.diagnostics().collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        matches!(found[0].code, DiagnosticCode::MissingRequiredField { .. }),
        "{found:#?}"
    );
}

// The walk reads `dimensions/*.json` only, so a dimension is written without a namespace
// folder: `<name>.json` for `minecraft:`, as vanilla's own `dimensions/overworld.json`, and
// `<namespace>_<name>.json` otherwise.
#[test]
fn the_default_path_of_a_dimension_is_read_back_by_the_walk() {
    use bedrock_addon::document::{Content, Placed};

    let pack = bp(1).file("dimensions/a.json", HIGH).open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let loaded = stack.dimensions().get(&overworld()).unwrap();
    let path: &'static str = loaded.default_path().as_str().to_owned().leak();
    assert_eq!(path, "dimensions/overworld.json");
    let written: &'static str = loaded.to_json().unwrap().leak();
    let pack = bp(1).file(path, written.as_bytes()).open();
    let again = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let read_back = again.dimensions().get(&overworld()).expect("read back");
    assert_eq!(read_back.origin().path.as_str(), path);
    assert_eq!(
        read_back.components.dimension_height,
        loaded.components.dimension_height
    );
    let mut other = loaded.value().clone();
    other.description.identifier = DimensionId::new("demo:moon/dark").unwrap();
    assert_eq!(
        other.default_path().as_str(),
        "dimensions/demo_moon_dark.json"
    );
}
