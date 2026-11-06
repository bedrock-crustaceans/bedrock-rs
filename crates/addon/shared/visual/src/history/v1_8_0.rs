//! Format versions 1.8.0 up to 1.10.0.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::id::AnimationRef;
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::versioning::UpgradeCx;
use indexmap::IndexMap;

use crate::AnimateEntry;
use crate::cereal::not_valid_here;
use crate::rules::NAMED_SCRIPT;

/// `scripts` at format versions 1.8.0 up to 1.10.0: `pre_animation`, the scale scripts and
/// `parent_setup` only. `variables`, `initialize`, `animate` and the off-screen scripts do not
/// exist yet.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Scripts {
    /// `pre_animation`: each element a Molang value.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pre_animation: Option<Vec<Molang>>,
    /// `scale`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scale: Option<Molang>,
    /// `scaleX` (the same script as `scalex`).
    #[facet(default, rename = "scaleX", skip_serializing_if = Option::is_none)]
    pub scale_x: Option<Molang>,
    /// `scalex`; wins over `scaleX`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scalex: Option<Molang>,
    /// `scaleY` (the same script as `scaley`).
    #[facet(default, rename = "scaleY", skip_serializing_if = Option::is_none)]
    pub scale_y: Option<Molang>,
    /// `scaley`; wins over `scaleY`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scaley: Option<Molang>,
    /// `scaleZ` (the same script as `scalez`).
    #[facet(default, rename = "scaleZ", skip_serializing_if = Option::is_none)]
    pub scale_z: Option<Molang>,
    /// `scalez`; wins over `scaleZ`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scalez: Option<Molang>,
    /// `parent_setup`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub parent_setup: Option<Molang>,
}

/// The named scripts and the `pre_animation` elements, as the 1.8 schema types them (the walk
/// enters `scripts` first).
impl MolangFields for Scripts {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("pre_animation", &NAMED_SCRIPT, &mut self.pre_animation);
        visitor.field("scale", &NAMED_SCRIPT, &mut self.scale);
        visitor.field("scaleX", &NAMED_SCRIPT, &mut self.scale_x);
        visitor.field("scalex", &NAMED_SCRIPT, &mut self.scalex);
        visitor.field("scaleY", &NAMED_SCRIPT, &mut self.scale_y);
        visitor.field("scaley", &NAMED_SCRIPT, &mut self.scaley);
        visitor.field("scaleZ", &NAMED_SCRIPT, &mut self.scale_z);
        visitor.field("scalez", &NAMED_SCRIPT, &mut self.scalez);
        visitor.field("parent_setup", &NAMED_SCRIPT, &mut self.parent_setup);
    }
}

impl Scripts {
    /// The same members at format version 1.10.0, which adds `variables`, `initialize`,
    /// `animate` and the off-screen scripts (absent here).
    #[must_use]
    pub fn into_v1_10(self) -> super::v1_10_0::Scripts {
        super::v1_10_0::Scripts {
            variables: None,
            initialize: None,
            pre_animation: self.pre_animation,
            scale: self.scale,
            scale_x: self.scale_x,
            scalex: self.scalex,
            scale_y: self.scale_y,
            scaley: self.scaley,
            scale_z: self.scale_z,
            scalez: self.scalez,
            parent_setup: self.parent_setup,
            should_update_bones_and_effects_offscreen: None,
            should_update_effects_offscreen: None,
            animate: None,
        }
    }
}

/// What vanilla's 1.8 → 1.10 step (`upgrade_v1_8_to_v1_10`) does with a description's
/// `animation_controllers`: its converter visits `minecraft:client_entity` only, so the list
/// of a client entity moves and an attachable's stays. A game rule of the family, not a drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerList {
    /// A client entity's: when the description has `animations`, each element becomes the
    /// animation `controller__<name>` and an entry of `scripts.animate`, and the list is
    /// removed. Only the member whose key sorts first byte by byte is read from an element;
    /// the others are dropped with an information. Without `animations` nothing moves.
    Moved,
    /// An attachable's: nothing moves.
    Kept,
}

/// The 1.8 → 1.10 step of `animation_controllers` (`controllers`), into `animations` and
/// `scripts` (already the 1.10 shape) as `list` says; `at` is the description. A list that
/// stays is a member 1.10.0 does not have: "child '`animation_controllers`' not valid here.",
/// and the document is rejected.
pub fn animation_controllers(
    list: ControllerList,
    controllers: Option<Vec<IndexMap<String, String>>>,
    animations: &mut Option<IndexMap<String, AnimationRef>>,
    scripts: &mut Option<super::v1_10_0::Scripts>,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) {
    if list == ControllerList::Kept {
        if controllers.is_some() {
            refused(at, cx);
        }
        return;
    }
    match (animations, controllers) {
        (Some(animations), Some(controllers)) => {
            let mut animate = Vec::with_capacity(controllers.len());
            // The element and key each made name comes from.
            let mut made_at: HashMap<String, (usize, String)> = HashMap::new();
            for (index, element) in controllers.iter().enumerate() {
                // The game takes the first member of the element in its map order, which
                // is the smallest key by bytes.
                let Some((first, controller)) = element
                    .iter()
                    .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))
                else {
                    cx.diagnostics.push(empty_element(at, index));
                    continue;
                };
                for key in element.keys().filter(|key| *key != first) {
                    cx.diagnostics.push(
                        Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                            .at(controller_at(at, index, key))
                            .phase(Phase::Upgrade),
                    );
                }
                let name = format!("controller__{first}");
                let entry = animate.len();
                animate.push(AnimateEntry::Always(name.clone()));
                let made = AnimationRef::parse(controller.clone());
                // An animation of the file under the name is written over, as the
                // game's step writes over it; the crate notes it where it is (the game says
                // nothing). Where the animation is one an earlier element of this list
                // made, the file has it there, not under the made name.
                if animations
                    .get(&name)
                    .is_some_and(|own| own.as_str() != made.as_str())
                {
                    cx.diagnostics
                        .push(written_over(at, &name, made_at.get(&name)));
                }
                made_at.insert(name.clone(), (index, first.clone()));
                // The notes above are of members the file has and stay where they are.
                made_of(cx, at, controller_at(at, index, first), &name, entry);
                animations.insert(name, made);
            }
            if !animate.is_empty() {
                scripts
                    .get_or_insert_with(super::v1_10_0::Scripts::default)
                    .animate
                    .get_or_insert_with(Vec::new)
                    .extend(animate);
            }
        }
        (None, Some(_)) => refused(at, cx),
        (_, None) => {}
    }
}

/// The list stays: the `v1_10` schema that runs next does not declare the member.
fn refused(at: &JsonPath, cx: &mut UpgradeCx<'_>) {
    let found = not_valid_here("animation_controllers", at.with(&["animation_controllers"]))
        .phase(Phase::Upgrade);
    cx.diagnostics.push(found);
}

/// Records that the animation `name` and the entry `entry` of `scripts.animate` were made of
/// the controller at `from` (`at` is the description): what is told of them from here on is
/// told at the controller.
fn made_of(cx: &mut UpgradeCx<'_>, at: &JsonPath, from: JsonPath, name: &str, entry: usize) {
    let mut entry_at = at.with(&["scripts", "animate"]);
    entry_at.push_index(entry);
    cx.diagnostics
        .moved(from.clone(), at.with(&["animations", name]));
    cx.diagnostics.moved(from, entry_at);
}

/// The note of the animation `name` a controller writes over, where the file has it: the
/// element and key of `animation_controllers` an earlier controller made it from (`made`),
/// or `animations` (`at` is the description).
fn written_over(at: &JsonPath, name: &str, made: Option<&(usize, String)>) -> Diagnostic {
    let replaced = made.map_or_else(
        || at.with(&["animations", name]),
        |(earlier, key)| controller_at(at, *earlier, key),
    );
    Diagnostic::info(DiagnosticCode::DroppedField { text: None })
        .at(replaced)
        .phase(Phase::Upgrade)
}

/// The member `key` of the element `index` of `animation_controllers` (`at` is the
/// description).
/// The note of an `animation_controllers` element without a member: it moves nothing and
/// goes with the list (the crate's own note; the game is silent).
fn empty_element(at: &JsonPath, index: usize) -> Diagnostic {
    let mut element_at = at.with(&["animation_controllers"]);
    element_at.push_index(index);
    Diagnostic::info(DiagnosticCode::DroppedField { text: None })
        .at(element_at)
        .phase(Phase::Upgrade)
}

fn controller_at(at: &JsonPath, index: usize, key: &str) -> JsonPath {
    at.child("animation_controllers").index(index).child(key)
}
