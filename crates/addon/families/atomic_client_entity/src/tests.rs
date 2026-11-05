//! Every scalar member of an atomic client entity, read from every JSON type: each line pins what the load says
//! (code, severity, path, span) and, when the document loads, the document written back. The
//! members were strict wrappers; this table is what they read, and the oracle of the change
//! that makes them plain.

use std::fmt::Write as _;

use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;

use super::AtomicClientEntity;

/// Loads `text` as an atomic client entity file.
fn load(text: &str) -> Result<Document<AtomicClientEntity>, Rejected> {
    AtomicClientEntity::parse(
        &PackPath::from_static("atomic_client_entities/wisp.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
}

/// The values each member is read from.
const VALUES: &[&str] = &[
    "\"x\"",
    "\"5\"",
    "5",
    "0.5",
    "-1",
    "true",
    "null",
    "1e39",
    "18446744073709551616",
    "[]",
    "{}",
];

/// The documents the table reads, `@@` standing for the value.
const PLACES: &[(&str, &str)] = &[
    (
        "aabb.width",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:aabb_dimensions": {"width": @@, "height": 1}}}}"#,
    ),
    (
        "aabb.height",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:aabb_dimensions": {"width": 1, "height": @@}}}}"#,
    ),
    (
        "avoidance.offsets[0][0]",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[@@, 0, 0]]}}}}"#,
    ),
    (
        "avoidance.require_hidden_before_fallback",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [], "require_hidden_before_fallback": @@}}}}"#,
    ),
    (
        "avoidance.anchor_switch_cooldown",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [], "anchor_switch_cooldown": @@}}}}"#,
    ),
    (
        "clipping.min_hidden_seconds",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:companion_block_clipping": {"min_hidden_seconds": @@}}}}"#,
    ),
    (
        "clipping.reveal_boundary.margin",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:companion_block_clipping": {"reveal_boundary": {"margin": @@}}}}}"#,
    ),
    (
        "scripts.initialize[0]",
        r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp", "scripts": {"initialize": [@@]}}}}"#,
    ),
];

/// `text` on one line: each line trimmed, the lines joined.
fn compact(text: &str) -> String {
    text.lines().map(str::trim).collect()
}

/// One line of the table: what loading `text` gave.
fn line(text: &str) -> String {
    let (loaded, diagnostics) = match load(text) {
        Ok(document) => (
            format!(
                "loaded {}",
                compact(&document.to_json().unwrap_or_default())
            ),
            document.diagnostics().clone(),
        ),
        Err(rejected) => ("rejected".to_owned(), rejected.diagnostics),
    };
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                d.code.name(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!("{loaded} [{}]", findings.join("; "))
}

#[test]
fn every_scalar_member_reads_as_pinned() {
    let mut table = String::new();
    for (name, place) in PLACES {
        for value in VALUES {
            let _ = writeln!(
                table,
                "{name} {value} => {}",
                line(&place.replace("@@", value))
            );
        }
    }
    assert!(table == EXPECTED, "--- actual ---\n{table}--- end ---");
}

/// The table as the strict wrappers read it.
const EXPECTED: &str = r#"aabb.width "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width 151+7]
aabb.width "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width 151+7]
aabb.width 5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 5.0,"height": 1.0}}}} []
aabb.width 0.5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 0.5,"height": 1.0}}}} []
aabb.width -1 => rejected [InvalidValue Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width -]
aabb.width true => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width 151+7]
aabb.width null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width 151+7]
aabb.width 1e39 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 3.4028235e38,"height": 1.0}}}} [InvalidValue Info minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width 160+4]
aabb.width 18446744073709551616 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 1.8446744e19,"height": 1.0}}}} []
aabb.width [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width/0 161+1]
aabb.width {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width 161+1]
aabb.height "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height 163+8]
aabb.height "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height 163+8]
aabb.height 5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 1.0,"height": 5.0}}}} []
aabb.height 0.5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 1.0,"height": 0.5}}}} []
aabb.height -1 => rejected [InvalidValue Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height -]
aabb.height true => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height 163+8]
aabb.height null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height 163+8]
aabb.height 1e39 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 1.0,"height": 3.4028235e38}}}} [InvalidValue Info minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height 173+4]
aabb.height 18446744073709551616 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:aabb_dimensions": {"width": 1.0,"height": 1.8446744e19}}}} []
aabb.height [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height/0 174+1]
aabb.height {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height 174+1]
avoidance.offsets[0][0] "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 188+3]
avoidance.offsets[0][0] "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 188+3]
avoidance.offsets[0][0] 5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[5.0,0.0,0.0]]}}}} []
avoidance.offsets[0][0] 0.5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[0.5,0.0,0.0]]}}}} []
avoidance.offsets[0][0] -1 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[-1.0,0.0,0.0]]}}}} []
avoidance.offsets[0][0] true => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 188+4]
avoidance.offsets[0][0] null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 188+4]
avoidance.offsets[0][0] 1e39 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 188+4]
avoidance.offsets[0][0] 18446744073709551616 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[1.8446744e19,0.0,0.0]]}}}} []
avoidance.offsets[0][0] [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 189+1]
avoidance.offsets[0][0] {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/backup_anchor_offsets/0 189+1]
avoidance.require_hidden_before_fallback "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback 5 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback 0.5 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback -1 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback true => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [],"require_hidden_before_fallback": true}}}} []
avoidance.require_hidden_before_fallback null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 224+4]
avoidance.require_hidden_before_fallback 1e39 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback 18446744073709551616 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 190+32]
avoidance.require_hidden_before_fallback [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback/0 225+1]
avoidance.require_hidden_before_fallback {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/require_hidden_before_fallback 225+1]
avoidance.anchor_switch_cooldown "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown 5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [],"anchor_switch_cooldown": 5}}}} []
avoidance.anchor_switch_cooldown 0.5 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown -1 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown true => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 216+4]
avoidance.anchor_switch_cooldown 1e39 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown 18446744073709551616 => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 190+24]
avoidance.anchor_switch_cooldown [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown/0 217+1]
avoidance.anchor_switch_cooldown {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_avoidance/anchor_switch_cooldown 217+1]
clipping.min_hidden_seconds "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds 160+20]
clipping.min_hidden_seconds "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds 160+20]
clipping.min_hidden_seconds 5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"min_hidden_seconds": 5.0}}}} []
clipping.min_hidden_seconds 0.5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"min_hidden_seconds": 0.5}}}} []
clipping.min_hidden_seconds -1 => rejected [InvalidValue Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds -]
clipping.min_hidden_seconds true => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds 160+20]
clipping.min_hidden_seconds null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds 182+4]
clipping.min_hidden_seconds 1e39 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"min_hidden_seconds": 3.4028235e38}}}} [InvalidValue Info minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds 182+4]
clipping.min_hidden_seconds 18446744073709551616 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"min_hidden_seconds": 1.8446744e19}}}} []
clipping.min_hidden_seconds [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds/0 183+1]
clipping.min_hidden_seconds {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/min_hidden_seconds 183+1]
clipping.reveal_boundary.margin "x" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin 180+8]
clipping.reveal_boundary.margin "5" => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin 180+8]
clipping.reveal_boundary.margin 5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"reveal_boundary": {"margin": 5.0}}}}} []
clipping.reveal_boundary.margin 0.5 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"reveal_boundary": {"margin": 0.5}}}}} []
clipping.reveal_boundary.margin -1 => rejected [InvalidValue Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin -]
clipping.reveal_boundary.margin true => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin 180+8]
clipping.reveal_boundary.margin null => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin 190+4]
clipping.reveal_boundary.margin 1e39 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"reveal_boundary": {"margin": 3.4028235e38}}}}} [InvalidValue Info minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin 190+4]
clipping.reveal_boundary.margin 18446744073709551616 => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"},"components": {"minecraft:companion_block_clipping": {"reveal_boundary": {"margin": 1.8446744e19}}}}} []
clipping.reveal_boundary.margin [] => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin/0 191+1]
clipping.reveal_boundary.margin {} => rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary/margin 191+1]
scripts.initialize[0] "x" => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp","scripts": {"initialize": ["x"]}}}} []
scripts.initialize[0] "5" => loaded {"format_version": "beta","minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp","scripts": {"initialize": ["5"]}}}} []
scripts.initialize[0] 5 => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+1]
scripts.initialize[0] 0.5 => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+3]
scripts.initialize[0] -1 => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+2]
scripts.initialize[0] true => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+4]
scripts.initialize[0] null => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+4]
scripts.initialize[0] 1e39 => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+4]
scripts.initialize[0] 18446744073709551616 => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 132+20]
scripts.initialize[0] [] => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 133+1]
scripts.initialize[0] {} => rejected [InvalidType Error minecraft:atomic_client_entity/description/scripts/initialize/0 133+1]
"#;

/// Documents with a struct member written `null`: what the slice's null rule makes of it,
/// told once (rejected where the input rejects nulls, read as its defaults elsewhere).
const NULL_STRUCTS: &[&str] = &[
    r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:aabb_dimensions": null}}}"#,
    r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}, "components": {"minecraft:companion_block_clipping": {"reveal_boundary": null}}}}"#,
];

#[test]
fn a_null_struct_member_reads_as_pinned() {
    let mut table = String::new();
    for text in NULL_STRUCTS {
        let _ = writeln!(table, "{}", line(text));
    }
    assert!(
        table == NULL_STRUCTS_EXPECTED,
        "--- actual ---\n{table}--- end ---"
    );
}

/// The null struct table.
const NULL_STRUCTS_EXPECTED: &str = r"rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:aabb_dimensions 150+4]
rejected [InvalidType Error minecraft:atomic_client_entity/components/minecraft:companion_block_clipping/reveal_boundary 179+4]
";

/// The family compiles no Molang: its render controllers are plain names (the object form is
/// refused on read) and its scripts plain strings, so the walk of even a hand-built entity
/// with a condition visits nothing.
#[test]
fn the_molang_walk_visits_no_field() {
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::id::AtomicClientEntityId;
    use bedrock_addon_core::molang::{Molang, MolangFieldSpec, visit_fields_under};
    use bedrock_addon_core::visual::ControllerUse;

    let mut entity = AtomicClientEntity::new(AtomicClientEntityId::parse("demo:wisp"));
    let mut condition = indexmap::IndexMap::new();
    condition.insert("controller.render.wisp".to_owned(), Molang::Const(1.0));
    entity.description.render_controllers = Some(vec![ControllerUse::Conditional(condition)]);
    entity.description.scripts = Some(super::AtomicScripts {
        initialize: Some(vec!["v.x = 1;".to_owned()]),
        pre_animation: None,
        animate: None,
    });
    let mut seen = Vec::new();
    visit_fields_under(
        &mut entity,
        &JsonPath::new(),
        &MolangFieldSpec::DEFAULT,
        |field| seen.push(field.path.to_string()),
    );
    assert!(seen.is_empty(), "{seen:?}");
}
