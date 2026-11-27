//! The component table: every key of has one row.

use bedrock_addon_core::versioning::ComponentTable;
use bedrock_addon_family_entity::component::EntityComponent;

fn table() -> &'static ComponentTable<bedrock_addon_family_entity::Entity> {
    EntityComponent::TABLE
}

#[test]
fn the_table_is_sorted_and_holds_every_key() {
    let rows = table().0;
    assert!(table().is_sorted(), "rows are sorted by name, each once");
    // 272 typed rows (components, goals, the move_to_position goal and the Brain-typed keys,
    // including two of the schema's dynamic keys); 143 dynamic rows (140 schema keys + 3
    // hidden Brain sensors) + 8 deprecated legacy keys. 1.26.60 typed 31 of
    // the dynamic rows, and the four 1.26.50 Brain keys are new typed
    // rows: 306 rows that parse a type, 121 kept as JSON.
    assert_eq!(rows.len(), 272 + 143 + 8 + 4);
    assert!(rows.iter().all(|row| row.name.starts_with("minecraft:")));
}

#[test]
fn metadata_rows_carry_the_d45_values() {
    let get = |name: &str| table().get(name).expect(name);
    let v = |s: &str| {
        let mut parts = s.split('.').map(|n| n.parse::<u16>().unwrap());
        let mut next = || parts.next().unwrap();
        bedrock_addon_core::version::FormatVersion::new(next(), next(), next())
    };
    assert_eq!(
        get("minecraft:block_climber").supersedes,
        Some("minecraft:scaffolding_climber")
    );
    assert_eq!(get("minecraft:block_climber").released, Some(v("1.17.10")));
    assert_eq!(get("minecraft:apply_knockback_rules").released, None);
    assert!(get("minecraft:apply_knockback_rules").requires.is_empty());
    assert_eq!(
        get("minecraft:behavior.enderman_take_block").deprecated_since,
        Some(v("1.21.100"))
    );
    assert_eq!(get("minecraft:bounciness").requires.len(), 1);
    // `owner_launch_immunity_ticks` is no member.
    assert_eq!(get("minecraft:projectile").fields.len(), 3);
    assert_eq!(get("minecraft:health").released, Some(v("1.8.0")));
}
