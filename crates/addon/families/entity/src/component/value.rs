//! Value types several entity components share.

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::json::{
    JsonType, ScalarShape, StrictError, StrictFloat, StrictString, Value,
};
use bedrock_addon_core::molang::{
    MolangFieldSpec, MolangVersion, MolangVersionSource, MolangVisitor,
};
use bedrock_addon_core::shared::{BlockDescriptor, ItemDescriptor};
use facet::Facet;
use facet_value::{VArray, VObject};

use super::parse::Check;

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

/// An equipment slot as the game names them. JSON: `slot.weapon.mainhand`,
/// `slot.weapon.offhand`, `slot.armor.head`, `slot.armor.chest`, `slot.armor.legs`,
/// `slot.armor.feet` or `slot.armor.body`.
// The names follow the game's equipment slot schema.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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
#[facet(proxy = ScalarShape)]
#[repr(u8)]
pub enum EffectSeconds {
    /// A number of seconds.
    Seconds(StrictFloat<f32>),
    /// `infinite`.
    Infinite,
}

impl Default for EffectSeconds {
    fn default() -> Self {
        Self::Seconds(StrictFloat(0.0))
    }
}

impl TryFrom<ScalarShape> for EffectSeconds {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        match json {
            ScalarShape::String(name) if name == INFINITE => Ok(Self::Infinite),
            ScalarShape::String(_) => Err(StrictError::WrongType {
                expected: "number or \"infinite\"",
                found: JsonType::String,
            }),
            other => StrictFloat::try_from(other).map(Self::Seconds),
        }
    }
}

impl TryFrom<&EffectSeconds> for ScalarShape {
    type Error = StrictError;

    fn try_from(value: &EffectSeconds) -> Result<Self, StrictError> {
        match value {
            EffectSeconds::Seconds(seconds) => ScalarShape::try_from(seconds),
            EffectSeconds::Infinite => Ok(ScalarShape::String(INFINITE.to_owned())),
        }
    }
}

/// The text of [`EffectSeconds::Infinite`].
const INFINITE: &str = "infinite";

/// A duration in seconds or a text (a number of seconds or a text, with no
/// constant bound): `healable.items[].effects[].duration`, whose help says it may be `"infinite"`; the
/// game keeps the text as written. JSON: a number or a string.
// A float or a string, the default being the float 0.0; "infinite" is kept as a string.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(proxy = ScalarShape)]
#[repr(u8)]
pub enum SecondsOrText {
    /// A number of seconds.
    Seconds(StrictFloat<f32>),
    /// A text, as written.
    Text(StrictString),
}

impl Default for SecondsOrText {
    fn default() -> Self {
        Self::Seconds(StrictFloat(0.0))
    }
}

impl TryFrom<ScalarShape> for SecondsOrText {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        match json {
            ScalarShape::String(text) => Ok(Self::Text(StrictString(text))),
            other => StrictFloat::try_from(other).map(Self::Seconds),
        }
    }
}

impl TryFrom<&SecondsOrText> for ScalarShape {
    type Error = StrictError;

    fn try_from(value: &SecondsOrText) -> Result<Self, StrictError> {
        match value {
            SecondsOrText::Seconds(seconds) => ScalarShape::try_from(seconds),
            SecondsOrText::Text(text) => Ok(ScalarShape::from(text)),
        }
    }
}

/// A member that is a single value where the typed read takes a list, made the list's one
/// element, with the move recorded: what is told of the element is told of the member the
/// file has. Returns whether it changed anything.
pub(crate) fn one_element(
    json: &mut VObject,
    key: &str,
    at: &JsonPath,
    cx: &mut Check<'_>,
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
pub(crate) fn tell_partial_vector(json: &VObject, key: &str, at: &JsonPath, cx: &mut Check<'_>) {
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
