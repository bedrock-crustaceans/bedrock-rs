//! Value types several entity components share.

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{
    MolangFieldSpec, MolangVersion, MolangVersionSource, MolangVisitor,
};
use bedrock_addon_core::schema::Schema;
use bedrock_addon_core::shared::{BlockDescriptor, ItemDescriptor};
use facet::Facet;
use facet_value::{VArray, VObject};

use super::parse::ComponentCheck;

/// A block or item descriptor's `tags` in a typed component: read at the fixed Molang version
/// 13, only the tag queries resolve. The game logs a rejected expression and loads
/// the entity with the descriptor kept: a kept error.
// The game's tag-expression parser logs a level-3 line for a rejected expression and
// still returns the descriptor, so the entity loads: a kept error.
pub(crate) const DESCRIPTOR_TAGS: MolangFieldSpec = MolangFieldSpec::KEPT
    .version(MolangVersionSource::Constant(MolangVersion::V13))
    .tags();

/// Hands an item descriptor's `tags` expression to the Molang walk.
pub(crate) fn visit_item_tags(item: &mut ItemDescriptor, visitor: &mut MolangVisitor<'_>) {
    if let ItemDescriptor::Tags { tags, .. } = item {
        visitor.field("tags", &DESCRIPTOR_TAGS, tags);
    }
}

/// Hands a block descriptor's `tags` expression to the Molang walk.
pub(crate) fn visit_block_tags(block: &mut BlockDescriptor, visitor: &mut MolangVisitor<'_>) {
    if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } = block {
        visitor.field("tags", &DESCRIPTOR_TAGS, tags);
    }
}

/// A value holding block or item descriptors whose `tags` the walk reads under
/// [`DESCRIPTOR_TAGS`] (not the walk's default spec, which core's descriptor walks take).
pub(crate) trait DescriptorTags {
    /// Hands each descriptor's `tags` to the walk, at the visitor's current step.
    fn visit_tags(&mut self, visitor: &mut MolangVisitor<'_>);
}

impl DescriptorTags for ItemDescriptor {
    fn visit_tags(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_item_tags(self, visitor);
    }
}

impl DescriptorTags for BlockDescriptor {
    fn visit_tags(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_block_tags(self, visitor);
    }
}

impl<T: DescriptorTags> DescriptorTags for Option<T> {
    fn visit_tags(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Some(value) = self {
            value.visit_tags(visitor);
        }
    }
}

impl<T: DescriptorTags> DescriptorTags for Vec<T> {
    fn visit_tags(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (i, value) in self.iter_mut().enumerate() {
            visitor.enter(i.into());
            value.visit_tags(visitor);
            visitor.leave();
        }
    }
}

/// The `schema::molang_with` walk of a field holding descriptors: their `tags` under
/// [`DESCRIPTOR_TAGS`].
pub(crate) fn descriptor_tags<T: DescriptorTags>(value: &mut T, visitor: &mut MolangVisitor<'_>) {
    value.visit_tags(visitor);
}

/// An equipment slot as the game names them. JSON: `slot.weapon.mainhand`,
/// `slot.weapon.offhand`, `slot.armor.head`, `slot.armor.chest`, `slot.armor.legs`,
/// `slot.armor.feet` or `slot.armor.body`.
// The names follow the game's equipment slot schema.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum EquipmentSlot {
    /// The main hand (JSON `slot.weapon.mainhand`).
    #[default]
    #[facet(rename = "slot.weapon.mainhand")]
    WeaponMainhand,
    /// The off hand (JSON `slot.weapon.offhand`).
    #[facet(rename = "slot.weapon.offhand")]
    WeaponOffhand,
    /// The head armour slot (JSON `slot.armor.head`).
    #[facet(rename = "slot.armor.head")]
    ArmorHead,
    /// The chest armour slot (JSON `slot.armor.chest`).
    #[facet(rename = "slot.armor.chest")]
    ArmorChest,
    /// The leg armour slot (JSON `slot.armor.legs`).
    #[facet(rename = "slot.armor.legs")]
    ArmorLegs,
    /// The feet armour slot (JSON `slot.armor.feet`).
    #[facet(rename = "slot.armor.feet")]
    ArmorFeet,
    /// The body armour slot of a mob (JSON `slot.armor.body`).
    #[facet(rename = "slot.armor.body")]
    ArmorBody,
}

/// A duration in seconds that may be `infinite` instead (a number of seconds or the name
/// `infinite`): `attack.effect_duration`,
/// `spell_effects.add_effects[].duration`. JSON: a number or the string `"infinite"`; another
/// string or any other JSON type is refused.
// For `attack` and `spell_effects` the default is the float alternative 0.0.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum EffectSeconds {
    /// A number of seconds.
    Seconds(f32),
    /// `infinite`.
    #[facet(rename = "infinite")]
    Infinite,
}

impl Default for EffectSeconds {
    fn default() -> Self {
        Self::Seconds(0.0)
    }
}

/// A duration in seconds or a text (a number of seconds or a text, with no
/// constant bound): `healable.items[].effects[].duration`, whose help says it may be `"infinite"`; the
/// game keeps the text as written. JSON: a number or a string.
// A float or a string, the default being the float 0.0; "infinite" is kept as a string.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum SecondsOrText {
    /// A number of seconds.
    Seconds(f32),
    /// A text, as written.
    Text(String),
}

impl Default for SecondsOrText {
    fn default() -> Self {
        Self::Seconds(0.0)
    }
}

/// A member that is a single value where the typed read takes a list, made the list's one
/// element, with the move recorded: what is told of the element is told of the member the
/// file has. Returns whether it changed anything.
pub(crate) fn one_element(
    json: &mut VObject,
    key: &str,
    at: &JsonPath,
    cx: &mut ComponentCheck<'_>,
) -> bool {
    let Some(member) = json.get_mut(key) else {
        return false;
    };
    if member.is_array() || member.is_null() {
        return false;
    }
    let single = std::mem::replace(member, Value::NULL);
    let mut list = VArray::new();
    list.push(single);
    *member = list.into();
    let at_member = at.child(key);
    cx.moved(at_member.clone(), at_member.index(0));
    true
}

/// Tells, with an information `ValueRewritten`, a vector member written as an object that
/// leaves a coordinate out: `goal::Vec3` reads the coordinate as 0 and the write-back writes
/// all three, so the place carries a finding. Returns nothing; changes
/// nothing.
pub(crate) fn tell_partial_vector(
    json: &VObject,
    key: &str,
    at: &JsonPath,
    cx: &mut ComponentCheck<'_>,
) {
    if let Some(object) = json.get(key).and_then(Value::as_object)
        && ["x", "y", "z"]
            .iter()
            .any(|axis| !object.contains_key(axis))
    {
        cx.push(
            DiagnosticCode::ValueRewritten { text: None },
            Severity::Info,
            &at.child(key),
        );
    }
}

bedrock_addon_core::impl_schema_leaf!(EffectSeconds);
bedrock_addon_core::impl_no_molang!(EffectSeconds);
bedrock_addon_core::impl_schema_leaf!(SecondsOrText);
bedrock_addon_core::impl_no_molang!(SecondsOrText);
