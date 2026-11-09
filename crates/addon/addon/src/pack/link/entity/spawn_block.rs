//! The blocks the block filters of spawn rules and of spawn groups name.
use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_shared_shapes::BlockDescriptor;

use super::spawn::{self, Conditions};
use crate::families::spawn_rules::OneOrMany;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for conditions in spawn::each_condition(cx) {
        filters(cx, out, &conditions);
    }
}

/// The three block filters of one condition.
fn filters(cx: &LinkCx<'_>, out: &mut Findings, conditions: &Conditions<'_>) {
    let Conditions {
        origin,
        ref at,
        condition,
        ..
    } = *conditions;
    let above = condition
        .spawns_above_block_filter
        .as_ref()
        .and_then(|filter| filter.blocks.as_ref());
    let lists: [(&[&str], Option<&OneOrMany<BlockDescriptor>>); 3] = [
        (
            &["minecraft:spawns_on_block_filter"],
            condition.spawns_on_block_filter.as_ref(),
        ),
        (
            &["minecraft:spawns_on_block_prevented_filter"],
            condition.spawns_on_block_prevented_filter.as_ref(),
        ),
        (&["minecraft:spawns_above_block_filter", "blocks"], above),
    ];
    for (keys, list) in lists {
        let Some(list) = list else {
            continue;
        };
        let mut list_at = at.clone();
        for key in keys {
            list_at.push(key);
        }
        for (index, descriptor) in list.as_slice().iter().enumerate() {
            let mut path = list_at.clone();
            // One descriptor outside an array is at the member itself.
            if matches!(list, OneOrMany::Many(_)) {
                path.push_index(index);
            }
            // As the file has it: the reader leaves out a descriptor it refuses.
            let path = conditions.place(&path);
            let member = Member {
                origin,
                path: &path,
                // The linked-asset check's grading: no path of the resolver reads another.
                severity: Severity::Error,
                validator: Some(1),
                // A state is the crate's own check: an information.
                states: States::Lint,
            };
            block_descriptor(cx, out, descriptor, member);
        }
    }
}
