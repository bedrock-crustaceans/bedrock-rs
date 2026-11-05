//! What the link pass does with the shared descriptor types: one resolver per type, called by
//! every rule whose member holds one, with the severity of the rule's row and the JSON path of
//! the descriptor in its file.

#![allow(
    dead_code,
    reason = "the frame of the rule packages: each calls the resolvers its rows need"
)]

#[cfg(any(feature = "block", feature = "item"))]
use bedrock_addon_core::diagnostic::Severity;
#[cfg(any(feature = "block", feature = "item"))]
use bedrock_addon_core::document::Origin;
#[cfg(feature = "block")]
use bedrock_addon_core::id::BlockId;
#[cfg(feature = "block")]
use bedrock_addon_shared_shapes::{BlockDescriptor, BlockSpecifier, StateValue};
#[cfg(feature = "item")]
use bedrock_addon_shared_shapes::{ItemDescriptor, ItemDescriptorForm};
#[cfg(feature = "block")]
use indexmap::IndexMap;

#[cfg(any(feature = "block", feature = "item"))]
use super::{At, Findings, LinkCx, Target, kinds, unresolved};
#[cfg(feature = "block")]
use bedrock_addon_core::diagnostic::VanillaText;

/// How a rule's row grades the states of a block descriptor that names a block of the stack:
/// a state the block does not have, and a value the state cannot take.
///
/// The game resolves a deferred descriptor's states without writing a content-log line for a
/// state the block lacks or a value it cannot take, so the check is the crate's own and an
/// information, unless the row's reader has a line of its own.
#[cfg(any(feature = "block", feature = "item"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum States {
    /// The crate's own check: an information.
    Lint,
    /// A row whose reader fails the block on a state it does not have, as a name nothing
    /// defines fails it: both findings errors.
    Error,
    /// A row whose reader logs a state the block does not have at the content log's error
    /// level with this text (`{}` the block, then the state) and drops the entry, keeping the
    /// document: a kept error. A value the state cannot take has no line: an
    /// information.
    Kept(&'static str),
}

/// Where a descriptor is and how bad a miss in it is: the document that holds it, its own JSON
/// path as the file has it, the severity of the calling rule's row and the grading of its
/// states.
#[cfg(any(feature = "block", feature = "item"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Member<'r> {
    /// The document that holds the descriptor.
    pub(crate) origin: &'r Origin,
    /// The descriptor's path in the file.
    pub(crate) path: &'r bedrock_addon_core::diagnostic::JsonPath,
    /// The severity of the rule's row for a name nothing defines (where the game's validator
    /// does not hold the reference; see [`Self::validator`]).
    pub(crate) severity: Severity,
    /// Where the game's linked-asset check holds the reference, the
    /// segment its breadcrumb starts at: a miss then carries its line, the breadcrumb of the
    /// descriptor itself. `None` where the game registers no reference.
    pub(crate) validator: Option<usize>,
    /// How the row grades the states of a block of the stack (an item descriptor has none).
    pub(crate) states: States,
}

#[cfg(any(feature = "block", feature = "item"))]
impl<'r> Member<'r> {
    /// The finding's place: the descriptor itself, or its member `keys` below it.
    fn at(&self, keys: &[&str]) -> At<'r> {
        let mut path = self.path.clone();
        for key in keys {
            path.push(key);
        }
        At::new(self.origin, path, self.severity)
    }

    /// [`Self::at`] with the validator's line for `name`, when the game holds the reference.
    fn missing(&self, keys: &[&str], name: &str) -> At<'r> {
        let mut at = self.at(keys);
        match self.validator {
            Some(from) => {
                // The game's linked-asset check writes this line at the error level and drops
                // nothing, so the finding is an error.
                at.severity = Severity::Error;
                at.with_text(super::validator_text(
                    super::Validator::Missing,
                    self.path,
                    from,
                    name,
                ))
            }
            None => at,
        }
    }
}

/// An item descriptor. One item (`"ns:name"`, `{"item": …}`, `{"name": …}`): the item must be
/// one of the stack's, a block's or the game's, else `UnresolvedReference` with the row's
/// severity at the member that names it. `{"item_tag": …}`: a lint, an `UnresolvedReference`
/// information when no item of the stack and none of the game's carries the tag. A tag
/// expression and a shape the load refused resolve to nothing here.
#[cfg(feature = "item")]
pub(crate) fn item_descriptor(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    descriptor: &ItemDescriptor,
    member: Member<'_>,
) {
    match descriptor {
        ItemDescriptor::Item { id, form, .. } => {
            if let Target::Missing(hint) = cx.item(id) {
                let keys: &[&str] = match form {
                    ItemDescriptorForm::Item => &["item"],
                    ItemDescriptorForm::Name => &["name"],
                    _ => &[],
                };
                let at = member.missing(keys, id.as_str());
                out.push(unresolved(at, kinds::ITEM, id.as_str(), hint));
            }
        }
        ItemDescriptor::ItemTag { item_tag, .. }
            if cx.lints() && cx.item_tag(item_tag).is_missing() =>
        {
            let mut at = member.at(&["item_tag"]);
            at.severity = Severity::Info;
            out.push(unresolved(at, kinds::ITEM_TAG, item_tag, None));
        }
        // A tag expression is the game's to evaluate; an invalid shape was told at load.
        _ => {}
    }
}

/// A block descriptor. A block (`"ns:name"`, `{"name": …, "states"?: …}`): the block must be
/// one of the stack's or the game's, else `UnresolvedReference` with the row's severity at
/// the member that names it; of a block the stack defines, each state must be one of the
/// block's (`UnresolvedReference`) and its value one the state can take (`InvalidReference`),
/// both at `states/<name>`, graded by the row's [`States`]. A tag expression, a descriptor
/// with both a name and tags (the game uses the tags) and a shape the load refused resolve to
/// nothing here.
#[cfg(feature = "block")]
pub(crate) fn block_descriptor(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    descriptor: &BlockDescriptor,
    member: Member<'_>,
) {
    if let BlockDescriptor::Block {
        name,
        states,
        object,
        ..
    } = descriptor
    {
        block(cx, out, name, states, *object, member);
    }
}

/// A block specifier (`"ns:name"` or `{"name": …, "states"?: …}`), by the rules of the block
/// arm of [`block_descriptor`].
#[cfg(feature = "block")]
pub(crate) fn block_specifier(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    specifier: &BlockSpecifier,
    member: Member<'_>,
) {
    block(
        cx,
        out,
        &specifier.name,
        &specifier.states,
        specifier.object,
        member,
    );
}

/// The block `name` with the required `states`, written as an object or as a string.
#[cfg(feature = "block")]
fn block(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    name: &BlockId,
    states: &IndexMap<String, StateValue>,
    object: bool,
    member: Member<'_>,
) {
    match cx.block(name) {
        Target::Missing(hint) => {
            let keys: &[&str] = if object { &["name"] } else { &[] };
            out.push(unresolved(
                member.missing(keys, name.as_str()),
                kinds::BLOCK,
                name.as_str(),
                hint,
            ));
        }
        Target::Defined(_) if !states.is_empty() => {
            // The states of the game's own blocks are not known here: only a block of the
            // stack is held to its states.
            let Some(own) = cx.block_states(name) else {
                return;
            };
            for (state, value) in states {
                let mut at = member.at(&["states", state]);
                at.severity = match member.states {
                    States::Lint | States::Kept(_) => Severity::Info,
                    States::Error => Severity::Error,
                };
                match own.get(state) {
                    None => {
                        if let States::Kept(template) = member.states {
                            let text = VanillaText::Formatted {
                                template,
                                args: vec![name.as_str().to_owned(), state.clone()],
                            };
                            at.severity = Severity::Error;
                            let finding =
                                unresolved(at.with_text(text), kinds::BLOCK_STATE, state, None);
                            out.push(finding.into_kept());
                        } else {
                            out.push(unresolved(at, kinds::BLOCK_STATE, state, None));
                        }
                    }
                    Some(domain) => {
                        let domain = domain.values();
                        if !domain.is_empty() && !domain.contains(value) {
                            out.push(super::invalid(at, kinds::BLOCK_STATE, state));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
