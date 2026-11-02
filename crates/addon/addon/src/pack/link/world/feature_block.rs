//! The blocks a feature names: every block member of a feature, and the `places_block` of a
//! `single_block_feature` and of a `multi_block_feature`.

use super::super::{
    At, Findings, LinkCx, Member, States, Target, block_descriptor, block_specifier, invalid,
    kinds, unresolved,
};
use crate::diagnostic::{JsonPath, JsonPathSegment, Severity, VanillaText};
use crate::document::Document;
use crate::families::feature::{Feature, Reference};
use crate::id::BlockId;
use crate::shared::{BlockDescriptor, StateValue};

/// vanilla's text for a `places_block` of a single-block feature that names no block.
const PLACES_BLOCK: &str = "Failed to resolve block for 'places_block' property.";

/// vanilla's text for a multi-block feature placing a block that is no multi-block.
const NOT_A_MULTI_BLOCK: &str = "Must place a multi-block in a 'minecraft:multi_block_feature'";

/// vanilla's text for a multi-block feature placing another part than the starting one.
const NOT_THE_STARTING_PART: &str = "If trying to place a multi-block in a 'minecraft:multi_block_feature' only the starting part may be used in field 'places_block'";

/// The state that says which part of a multi-block a block is; the starting part is 0.
const PART: &str = "minecraft:multi_block_part";

/// The blocks among `references`, the references of the feature `document`.
///
/// A member that names a block neither the stack nor the game has is an `UnresolvedReference`
/// information at the member; of a block of the stack, a state it does not have is an
/// `UnresolvedReference` information and a value the state cannot take an `InvalidReference`
/// information, at the state. A descriptor of tags is the game's to evaluate.
pub(super) fn feature_blocks(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    document: &Document<Feature>,
    references: &[(JsonPath, Reference<'_>)],
) {
    let feature = document.value();
    for (path, reference) in references {
        // `minecraft:<type>/places_block/…`.
        let places_block = path.segments().nth(1) == Some(JsonPathSegment::Key("places_block"));
        let member = Member {
            origin: document.origin(),
            path,
            // The game resolves a descriptor in silence: the crate reports an information.
            severity: Severity::Info,
            validator: None,
            // A state is the crate's own check: an information.
            states: States::Lint,
        };
        let told = match (feature, reference) {
            (Feature::SingleBlock(_), _) if places_block => {
                single_block(cx, out, member, reference)
            }
            (Feature::MultiBlock(_), Reference::Descriptor(descriptor)) if places_block => {
                multi_block(cx, out, member, descriptor)
            }
            _ => false,
        };
        if told {
            continue;
        }
        match reference {
            Reference::Descriptor(descriptor) => block_descriptor(cx, out, descriptor, member),
            Reference::Specifier(specifier) => block_specifier(cx, out, specifier, member),
            Reference::Feature(_) => {}
        }
    }
}

/// The path of the member that names the block of a descriptor at `member`: the descriptor
/// itself when it is a string, its `name` when it is an object.
fn name_at(member: Member<'_>, object: bool) -> JsonPath {
    let mut path = member.path.clone();
    if object {
        path.push("name");
    }
    path
}

/// A block `single_block_feature.places_block` names (the block itself, a weighted entry's
/// `block`, or the descriptor of a document older than 1.21.40) that neither the stack nor the
/// game has is an `UnresolvedReference` kept error with the text of vanilla's setter. Whether it
/// told the member.
fn single_block(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    member: Member<'_>,
    reference: &Reference<'_>,
) -> bool {
    let (name, object) = match reference {
        Reference::Specifier(specifier) => (&specifier.name, specifier.object),
        Reference::Descriptor(BlockDescriptor::Block { name, object, .. }) => (name, *object),
        _ => return false,
    };
    let Target::Missing(hint) = cx.block(name) else {
        return false;
    };
    // The game logs this at the error level and returns without the block, the feature kept:
    // a kept error.
    let at = At::new(member.origin, name_at(member, object), Severity::Error)
        .with_text(VanillaText::Fixed(PLACES_BLOCK));
    out.push(unresolved(at, kinds::BLOCK, name.as_str(), hint).into_kept());
    true
}

/// `multi_block_feature.places_block` must be the starting part of a block with the
/// `minecraft:multi_block` trait. Of a block of the stack, one without the trait is an
/// `InvalidReference` error at its name, and a `minecraft:multi_block_part` other than 0 an
/// `InvalidReference` error at the state, each with vanilla's text. The traits of the game's
/// own blocks are not known here. Whether it told the member.
fn multi_block(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    member: Member<'_>,
    descriptor: &BlockDescriptor,
) -> bool {
    let BlockDescriptor::Block {
        name,
        states,
        object,
        ..
    } = descriptor
    else {
        return false;
    };
    let Some(block) = placed_block(cx, name) else {
        return false;
    };
    if block.description.traits.multi_block.is_none() {
        let at = At::new(member.origin, name_at(member, *object), Severity::Error)
            .with_text(VanillaText::Fixed(NOT_A_MULTI_BLOCK));
        out.push(invalid(at, kinds::BLOCK, name.as_str()));
        return true;
    }
    match states.get(PART) {
        None | Some(StateValue::Int(0)) => false,
        Some(_) => {
            let mut path = member.path.clone();
            path.push("states");
            path.push(PART);
            let at = At::new(member.origin, path, Severity::Error)
                .with_text(VanillaText::Fixed(NOT_THE_STARTING_PART));
            out.push(invalid(at, kinds::BLOCK_STATE, PART));
            true
        }
    }
}

/// The block of the stack a descriptor names, when a pack defines it.
fn placed_block<'a>(cx: &LinkCx<'a>, name: &BlockId) -> Option<&'a crate::families::block::Block> {
    cx.block(name).origin()?;
    Some(cx.behavior().blocks().get(name)?.value())
}
