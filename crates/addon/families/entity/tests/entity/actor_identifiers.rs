//! `families::entity::actor_identifiers`.
#![cfg(feature = "nbt")]

/// `families::entity::actor_identifiers`: the client's actor identifier list.
#[test]
fn actor_identifiers_nbt_shape() {
    use bedrock_addon_core::document::{CombineRule, Contribution, Document, RegistryBuilder};
    use bedrock_addon_core::load::LoadOptions;
    use bedrock_addon_family_entity::{Entity, actor_identifiers};
    use nbtx::{Value, ValueList};

    let entity = |text: &str| -> Document<Entity> {
        super::common::parse(&super::common::doc("1.26.50", text)).unwrap()
    };
    let mut builder: RegistryBuilder<Entity> = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        &LoadOptions::default(),
    );
    for text in [
        r#"{"description": {"identifier": "minecraft:npc", "is_spawnable": true, "is_summonable": true}, "components": {}}"#,
        r#"{"description": {"identifier": "demo:a", "runtime_identifier": "minecraft:skeleton", "is_experimental": true}, "components": {}}"#,
        r#"{"description": {"identifier": "demo:b"}, "components": {}}"#,
    ] {
        let document = entity(text);
        let id = document.description.identifier.clone();
        builder.push(Contribution::Stable(id, document));
    }
    let registry = builder.finish();
    let Value::Compound(root) = actor_identifiers(&registry) else {
        panic!("a compound");
    };
    let Some(Value::List(ValueList::Compound(list))) = root.get(&b"idlist"[..]) else {
        panic!("an idlist of compounds: {root:?}");
    };
    assert_eq!(list.len(), 3);
    let field = |i: usize, key: &str| list[i].get(key.as_bytes()).cloned().unwrap();
    assert_eq!(field(0, "id"), Value::from("minecraft:npc"));
    assert_eq!(field(0, "bid"), Value::from(""));
    assert_eq!(field(0, "rid"), Value::Int(0x33));
    assert_eq!(field(0, "hasspawnegg"), Value::Byte(1));
    assert_eq!(field(0, "summonable"), Value::Byte(1));
    assert_eq!(field(0, "experimental"), Value::Byte(0));
    assert_eq!(field(1, "bid"), Value::from("minecraft:skeleton"));
    assert_eq!(field(1, "rid"), Value::Int(1000));
    assert_eq!(field(1, "experimental"), Value::Byte(1));
    assert_eq!(field(2, "rid"), Value::Int(1001));
    assert_eq!(field(2, "hasspawnegg"), Value::Byte(0));
}
