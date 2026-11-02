//! 20 R24: the names of a geometry that nothing resolves. (R22 and R23 are the load's:
//! `tests/geometry/stack.rs`, `tests/geometry/gates.rs`.)

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail};
use bedrock_addon::load::LoadOptions;

use super::super::common::{in_file, stacks};

/// 20 R24: a face's `material_instance` names a material instance of the block that uses the
/// geometry, and a texture mesh's `texture` a texture slot of its user: neither the load nor
/// the link pass resolves them.
#[test]
fn r24_a_face_material_instance_and_a_texture_mesh_texture_are_not_checked() {
    const FILE: &str = "models/entity/x.geo.json";
    let text = r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "root", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": {"north": {"uv": [0, 0], "material_instance": "no_such_instance"}}}], "texture_meshes": [{"texture": "no_such_texture", "position": [0, 0, 0]}]}]}]}"#;
    let set = stacks(&[], &[(FILE, text)], LoadOptions::default());
    let geometries = set.resource.geometries();
    assert_eq!(geometries.len(), 1, "the geometry loads");
    let is_reference = |finding: &Diagnostic| {
        matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { .. })
        )
    };
    let of_the_load: Vec<&Diagnostic> = geometries
        .iter()
        .flat_map(|(_, document)| document.diagnostics().iter())
        .chain(geometries.diagnostics())
        .filter(|finding| is_reference(finding))
        .collect();
    assert_eq!(of_the_load, Vec::<&Diagnostic>::new());
    let linked = set.link();
    assert!(!in_file(&linked, FILE).into_iter().any(is_reference));
}
