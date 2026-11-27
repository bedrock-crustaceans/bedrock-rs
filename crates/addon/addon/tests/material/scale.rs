//! Long chains through the stack accessor: twenty thousand links load, resolve, print and
//! drop without recursion. The cost of each step is measured in the crate
//! (`src/families/material/tests.rs`), on inputs built beforehand; this file is about depth.

use bedrock_addon::families::material::MaterialGroupKind;

use crate::common::all;
use crate::stacks::stack;

const LINKS: usize = 20_000;

/// A wrapped file of `LINKS` materials whose keys `key` gives, each with `body`.
fn file(key: impl Fn(usize) -> String, body: &str) -> String {
    let members: Vec<String> = (0..LINKS)
        .map(|i| format!("\"{}\": {body}", key(i)))
        .collect();
    format!(
        "{{\"materials\": {{\"version\": \"1.0.0\", {}}}}}",
        members.join(",")
    )
}

const LISTING: &str = r#"[{"path": "materials/a.material", "+defines": ["BASE"]}]"#;

#[test]
fn a_chain_of_twenty_thousand_materials_loads_resolves_prints_and_drops() {
    let chain = file(
        |i| match i {
            0 => "m000000".to_owned(),
            _ => format!("m{i:06}:m{:06}", i - 1),
        },
        r#"{"+defines": ["A"], "+states": ["Blending"]}"#,
    );
    let stack = stack(&[&[
        ("materials/common.json", LISTING),
        ("materials/a.material", &chain),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    assert_eq!(materials.common.len(), LINKS);
    let last = format!("m{:06}", LINKS - 1);
    let material = materials.common.get(&last).expect("the last link");
    assert_eq!(material.parent.as_deref(), Some("m019998"));
    // The whole chain applied, from the list entry's define down.
    let resolved = materials
        .resolve(MaterialGroupKind::Common, &last)
        .expect("resolves");
    assert_eq!(resolved.defines, ["A", "BASE"]);
    assert_eq!(resolved.states, ["Blending"]);
    // Printing the group walks no chain.
    let printed = format!("{materials:?}");
    assert!(printed.contains("m019999"));
    // Two materials compare as their own links.
    assert_ne!(material, materials.common.get("m000000").unwrap());
}

#[test]
fn a_chain_in_which_every_link_adds_a_variant_gives_the_last_them_all() {
    let members: Vec<String> = (0..LINKS)
        .map(|i| {
            let key = match i {
                0 => "m000000".to_owned(),
                _ => format!("m{i:06}:m{:06}", i - 1),
            };
            format!("\"{key}\": {{\"+variants\": [{{\"v{i:06}\": {{}}}}]}}")
        })
        .collect();
    let chain = format!(
        "{{\"materials\": {{\"version\": \"1.0.0\", {}}}}}",
        members.join(",")
    );
    let stack = stack(&[&[
        ("materials/common.json", LISTING),
        ("materials/a.material", &chain),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let common = &materials.common;
    assert_eq!(common.get("m019999").unwrap().variant_count(), LINKS);
    assert_eq!(common.get("m000000").unwrap().variant_count(), 1);
    assert_eq!(common.get("m010000").unwrap().variant_count(), 10_001);
    assert!(common.contains("m019999.v000000") && common.contains("m019999.v019999"));
    assert!(!common.contains("m010000.v010001"));
}

#[test]
fn a_cycle_of_twenty_thousand_materials_creates_none_and_reports_each() {
    let cycle = file(|i| format!("m{i:06}:m{:06}", (i + 1) % LINKS), "{}");
    let stack = stack(&[&[
        ("materials/common.json", LISTING),
        ("materials/a.material", &cycle),
    ]]);
    let materials = stack.materials();
    assert!(materials.common.is_empty());
    let found = all(materials);
    assert_eq!(found.len(), LINKS);
    assert!(
        found
            .iter()
            .all(|finding| finding.0 == "core/unresolved_reference")
    );
}
