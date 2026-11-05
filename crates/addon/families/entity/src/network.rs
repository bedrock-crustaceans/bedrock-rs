//! The client network NBT of the entity registry: the `AvailableActorIdentifiers`
//! idlist.

use bedrock_addon_core::document::Registry;
use bedrock_addon_core::vanilla::ActorType;
use nbtx::{Compound, Value, ValueList};

use super::Entity;

/// The first runtime id given to an entity that is not a vanilla type.
// The game numbers custom entities itself at run time; the crate's own scheme starts above every
// legacy type id, which keeps them apart from the vanilla ones.
const FIRST_CUSTOM_RID: i32 = 1000;

/// The registry's entities as the client's actor identifier list:
/// `{ "idlist": [ { id, bid, rid, hasspawnegg, summonable, experimental } ] }`, in registry
/// order.
///
/// `id` is the identifier and `bid` the runtime identifier (`""` without one); `rid` is the
/// legacy numeric type id of a vanilla identifier and a number from 1000 upward, in registry
/// order, for any other; `hasspawnegg`, `summonable` and `experimental` are bytes (0 or 1) of
/// `is_spawnable`, `is_summonable` and the description's `is_experimental`.
#[doc(alias = "AvailableActorIdentifiersPacket")]
pub fn actor_identifiers(registry: &Registry<Entity>) -> Value {
    let mut next_custom = FIRST_CUSTOM_RID;
    let mut list = Vec::new();
    for (id, document) in registry {
        let description = &document.description;
        let rid = if let Some(actor) = ActorType::from_identifier(id.as_str()) {
            i32::try_from(actor.id() & 0xff).unwrap_or_default()
        } else {
            next_custom += 1;
            next_custom - 1
        };
        // Held as an unknown member of the description, which the document keeps.
        let description_at =
            bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity", "description"]);
        let experimental = document
            .unknown()
            .get(&description_at, "is_experimental")
            .and_then(bedrock_addon_core::json::Value::as_bool)
            .unwrap_or(false);
        let byte = |b: bool| Value::Byte(i8::from(b));
        let mut entry = Compound::new();
        entry.insert("id".into(), Value::from(id.as_str()));
        entry.insert(
            "bid".into(),
            Value::from(
                description
                    .runtime_identifier
                    .as_ref()
                    .map_or("", |runtime| runtime.as_str()),
            ),
        );
        entry.insert("rid".into(), Value::Int(rid));
        entry.insert("hasspawnegg".into(), byte(document.is_spawnable()));
        entry.insert("summonable".into(), byte(document.is_summonable()));
        entry.insert("experimental".into(), byte(experimental));
        list.push(entry);
    }
    let mut root = Compound::new();
    root.insert("idlist".into(), Value::List(ValueList::Compound(list)));
    Value::Compound(root)
}
