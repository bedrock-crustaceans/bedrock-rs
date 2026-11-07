//! The blocks an entity's components name.
use bedrock_addon_core::diagnostic::{JsonPath, JsonPathSegment, Kind, Severity, sentence};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_shared_shapes::{BlockDescriptor, ItemDescriptor};

use super::super::{
    At, Findings, LinkCx, Member, States, Target, block_descriptor, kinds, unresolved,
};
use super::table::{BLOCK_NAMES, BLOCKS, BLOCKS_AS_ITEMS};
use super::walk::{Entities, sections};
use crate::families::entity::goal::EatBlock;
use crate::families::entity::goal::convert::eat_block_pairs;

/// The key of the goal.
const EAT_BLOCK: &str = "minecraft:behavior.eat_block";

/// The component whose typed reader checks a descriptor's states itself.
const INSIDE_BLOCK_NOTIFIER: &str = "minecraft:inside_block_notifier";

/// The game's line for a block descriptor whose block nothing defines, logged at the error
/// level when the descriptor is first used, after which the game goes on with the descriptor
/// unresolved: a kept error.
const UNKNOWN_BLOCK: &str = "Unknown block during Deferred BlockDescriptor resolution: {}\n";

/// `inside_block_notifier`'s line for a state the block does not have, logged at the error
/// level, after which the entry is left out and the entity loads: a kept error.
const STATE_NOT_OF_BLOCK: &str = "Block of type '{}' does not have the state '{}'. This block will not be added to InsideBlockNotifierComponent's watch list due to an invalid state.";

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    eat_block(cx, out, entities);
    // One walk for both tables: the findings of a document are in its components' order.
    let rows = [BLOCKS, BLOCK_NAMES].concat();
    entities.each(&rows, &mut |component, at, value| {
        if BLOCK_NAMES.iter().any(|(key, _)| *key == component.key) {
            // A name of a list of names; what is no string there, the load has refused. The
            // game binds the members of `home` and of the `raid_garden` goal as names, not as
            // deferred descriptors, and logs no line for a name it lacks: the game writes no
            // content-log line for this, so the crate reports it as an information.
            let Some(name) = value.as_string() else {
                return;
            };
            let descriptor = BlockDescriptor::block(BlockId::parse(name.as_str()));
            let member = Member {
                origin: component.origin,
                path: &at,
                severity: Severity::Info,
                validator: None,
                states: States::Lint,
            };
            block_descriptor(cx, out, &descriptor, member);
            return;
        }
        let descriptor = BlockDescriptor::from(value.clone());
        if let BlockDescriptor::Block { name, object, .. } = &descriptor
            && let Target::Missing(hint) = cx.block(name)
        {
            let mut path = at.clone();
            if *object {
                path.push("name");
            }
            let text = sentence(UNKNOWN_BLOCK, &[name.as_str().to_owned()]);
            let place = At::new(component.origin, path, Severity::Error).with_text(text);
            out.push(unresolved(place, kinds::BLOCK, name.as_str(), hint).into_kept());
            return;
        }
        let states = if component.key == INSIDE_BLOCK_NOTIFIER {
            States::Kept(STATE_NOT_OF_BLOCK)
        } else {
            // A state is the crate's own check: an information.
            States::Lint
        };
        let member = Member {
            origin: component.origin,
            path: &at,
            // A name nothing defines was told above; no other path reads this.
            severity: Severity::Error,
            validator: None,
            states,
        };
        block_descriptor(cx, out, &descriptor, member);
    });
    entities.each(BLOCKS_AS_ITEMS, &mut |component, at, value| {
        block_as_item(cx, out, component.origin, at, value);
    });
}

/// Each pair of `behavior.eat_block.eat_and_replace_block_pairs` names two blocks. The
/// goal's own check finds the pairs (one finding per pair, at its first unknown name, with
/// vanilla's text; an error where the goal was read strictly, a warning where an older
/// document was converted); the finding is told at the member that holds the name.
fn eat_block(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    let is_block = |name: &str| !cx.block(&BlockId::parse(name)).is_missing();
    for definition in &entities.definitions {
        for (section, map) in sections(definition.entity) {
            let Some(goal) = map.get::<EatBlock>() else {
                continue;
            };
            // The check drops the pairs it reports; the pass changes nothing, so it runs on
            // a copy.
            let mut checked = goal.clone();
            for finding in eat_block_pairs(&mut checked, is_block) {
                // By kind: the check raises only `entity/eat_block_invalid_block`; the game's
                // sentence, if any, goes on to the link's finding.
                if finding.code.kind() != Kind::UnresolvedReference {
                    continue;
                }
                let text = finding.code.vanilla();
                let Some(JsonPathSegment::Index(index)) =
                    finding.location.json_path.segments().last()
                else {
                    continue;
                };
                let pair = goal
                    .eat_and_replace_block_pairs
                    .as_ref()
                    .and_then(|pairs| pairs.get(index));
                let Some(pair) = pair else {
                    continue;
                };
                let names = [
                    ("eat_block", &pair.eat_block),
                    ("replace_block", &pair.replace_block),
                ];
                let unknown = names.into_iter().find_map(|(key, name)| {
                    let name = name.as_ref()?;
                    let id = BlockId::parse(name.as_str());
                    match cx.block(&id) {
                        Target::Missing(hint) => Some((key, name.as_str(), hint)),
                        _ => None,
                    }
                });
                let Some((key, name, hint)) = unknown else {
                    continue;
                };
                let mut path = section.clone();
                path.push(EAT_BLOCK);
                path.append(&finding.location.json_path);
                path.push(key);
                // The pair is counted among the pairs held: told where the file has it.
                let path = definition.places.place_of(&path);
                let mut at = At::new(definition.origin, path, finding.severity);
                at.text = text;
                out.push(unresolved(at, kinds::BLOCK, name, hint));
            }
        }
    }
}

/// A block a goal names through an item descriptor: the name must be a block or an item.
/// Without the `item` feature an item cannot be told from a missing name, and nothing is
/// told. A name that is neither is logged at the lowest level only (an information-level
/// line): the crate reports an information.
#[cfg_attr(
    not(feature = "item"),
    allow(unused_variables, clippy::needless_pass_by_value)
)]
fn block_as_item(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &bedrock_addon_core::document::Origin,
    at: JsonPath,
    value: &bedrock_addon_core::json::Value,
) {
    #[cfg(feature = "item")]
    {
        use bedrock_addon_shared_shapes::ItemDescriptorForm;
        let mut at = at;
        let ItemDescriptor::Item { id, form, .. } = ItemDescriptor::from(value.clone()) else {
            return;
        };
        let block = BlockId::parse(id.as_str());
        let Target::Missing(hint) = cx.block(&block) else {
            return;
        };
        if !cx.item(&id).is_missing() {
            return;
        }
        match form {
            ItemDescriptorForm::Item => at.push("item"),
            ItemDescriptorForm::Name => at.push("name"),
            _ => {}
        }
        let at = At::new(origin, at, Severity::Info);
        out.push(unresolved(at, kinds::BLOCK, id.as_str(), hint));
    }
    #[cfg(not(feature = "item"))]
    let _ = ItemDescriptor::from(value.clone());
}
