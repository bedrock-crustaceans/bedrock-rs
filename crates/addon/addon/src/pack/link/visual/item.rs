//! The item an attachable is drawn for (`description.item`) is an item: one a
//! behaviour pack defines, the item of a block of the stack, or one of the game's.
//!
//! The game compares the member with an item's full identifier byte for byte when it draws:
//! nothing is lower-cased and no `minecraft:` is added, so a name in another case or without
//! its namespace never matches. The finding names the item that was meant when there is one.

use std::collections::{HashMap, HashSet};

use bedrock_addon_core::diagnostic::{JsonPath, ReferenceHint, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::pack_identity::PackSide;

use super::super::{At, Findings, LinkCx, canonical, kinds, unresolved};
use crate::families::attachable::{ItemBinding, ItemBindingEntry};

/// The full identifiers of the stack's items, as their definitions write them.
#[derive(Debug)]
struct Known {
    exact: HashSet<String>,
    /// By the identifier in ASCII lower case: the first definition with it.
    folded: HashMap<String, String>,
}

impl Known {
    fn of(cx: &LinkCx<'_>) -> Self {
        let mut known = Self {
            exact: HashSet::new(),
            folded: HashMap::new(),
        };
        for (id, _) in cx.item_names().iter() {
            known.add(&canonical(id));
        }
        // Every block has an item of its name.
        #[cfg(feature = "block")]
        for (id, _) in cx.block_names().iter() {
            known.add(&canonical(id));
        }
        known
    }

    fn add(&mut self, name: &str) {
        if self.exact.insert(name.to_owned()) {
            self.folded
                .entry(name.to_ascii_lowercase())
                .or_insert_with(|| name.to_owned());
        }
    }
}

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let attachables = cx.resource().attachables();
    // Every binding the stack holds, in the order the game gathered them: those of the
    // documents a later one replaced are still tried.
    let mut bindings: Vec<&ItemBindingEntry> = attachables
        .iter()
        .flat_map(|(_, document)| document.value().bound_items())
        .collect();
    if bindings.is_empty() {
        return;
    }
    bindings.sort_by_key(|binding| binding.sequence);
    let known = Known::of(cx);
    // The game's own items: the host's answer, else the crate's list.
    let of_the_game = |name: &str| match cx.built_ins() {
        Some(host) => host.contains(kinds::ITEM, name),
        None => cx.item_names().is_vanilla(name),
    };
    for binding in bindings {
        let item = binding.item.as_str();
        if known.exact.contains(item) || of_the_game(item) {
            continue;
        }
        let lowered = item.to_ascii_lowercase();
        let hint = if let Some(meant) = known.folded.get(&lowered) {
            Some(ReferenceHint::CaseMismatch(meant.clone()))
        } else if lowered != item && of_the_game(&lowered) {
            Some(ReferenceHint::CaseMismatch(lowered))
        } else if !item.contains(':') && {
            let named = format!("minecraft:{item}");
            known.exact.contains(&named) || of_the_game(&named)
        } {
            Some(ReferenceHint::MissingNamespace)
        } else {
            None
        };
        // The document the binding came from, when it is still the definition; a binding
        // that outlived its document is told at the file that wrote it.
        let written = attachables.get(&binding.attachable).filter(|document| {
            let origin = document.origin();
            !binding.shadowed && origin.pack == binding.pack && origin.path == binding.file
        });
        let mut path = JsonPath::new();
        path.push("minecraft:attachable");
        path.push("description");
        path.push("item");
        // The object form: the member that binds, its key as written.
        let object = written.is_some_and(|document| {
            matches!(
                document.value().description.item,
                Some(ItemBinding::Conditional(_))
            )
        });
        if object {
            path.push(item);
        }
        let elsewhere;
        let origin = if let Some(document) = written {
            document.origin()
        } else {
            elsewhere = Origin::new(
                binding.pack.clone(),
                PackSide::Resource,
                binding.file.clone(),
                None,
                Vec::new(),
            );
            &elsewhere
        };
        // An attachable whose `item` names nothing matches no held item, and the game says
        // nothing: the crate reports an information.
        let at = At::new(origin, path, Severity::Info);
        out.push(unresolved(at, kinds::ITEM, item, hint));
    }
}
