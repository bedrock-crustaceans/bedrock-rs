//! A map keyed by an identifier is walked like a map keyed by text: its values' bounds are
//! checked at the identifier's text.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;
use indexmap::IndexMap;

#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
struct Light {
    #[facet(schema::min = 1)]
    level: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
struct Lights {
    lights: IndexMap<BlockId, Light>,
}

#[test]
fn the_values_of_a_map_keyed_by_identifier_are_checked_at_the_identifier() {
    let mut lights = Lights::default();
    lights
        .lights
        .insert(BlockId::parse("minecraft:stone"), Light { level: 0 });
    let mut found = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::OWN, JsonPath::new(), &mut found);
    lights.check(&mut cx);
    let told: Vec<String> = found
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert_eq!(told, ["lights/minecraft:stone/level"]);
}
