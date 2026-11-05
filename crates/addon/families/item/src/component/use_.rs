//! Using the item: `compostable`, `food`, `fuel`, `use_animation`, `use_modifiers`.

use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::ItemDescriptor;
use facet::Facet;

use super::value::{ValueComponent, value_component};
use super::visit_item_descriptor;
use bedrock_addon_core::schema::{self, Schema};

/// `minecraft:compostable`: a composter takes the item.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Compostable {
    /// The chance in percent, 1–100, that it adds a layer (JSON `composting_chance`, required).
    #[facet(schema::min = 1, schema::max = 100)]
    pub composting_chance: i32,
}

/// `minecraft:food`: the item can be eaten.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Food {
    /// The hunger restored (JSON `nutrition`).
    pub nutrition: Option<i32>,
    /// The saturation factor (JSON `saturation_modifier`).
    pub saturation_modifier: Option<f32>,
    /// It can be eaten when not hungry (JSON `can_always_eat`).
    pub can_always_eat: Option<bool>,
    /// The item left after eating (JSON `using_converts_to`).
    pub using_converts_to: Option<ItemDescriptor>,
}

impl MolangFields for Food {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Some(item) = &mut self.using_converts_to {
            visitor.enter("using_converts_to".into());
            visit_item_descriptor(item, visitor);
            visitor.leave();
        }
    }
}

/// `minecraft:fuel`: the item burns in a furnace (JSON `{"duration": seconds}` or a bare
/// number; at least 0.05).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Fuel {
    /// The burn time in seconds (JSON `duration`).
    #[facet(default, schema::min = 0.05)]
    pub duration: f32,
}

impl Fuel {
    /// A burn time of `duration` seconds.
    pub fn new(duration: f32) -> Self {
        Self { duration }
    }
}

impl ValueComponent for Fuel {
    type Scalar = f32;
    fn from_scalar(value: f32) -> Self {
        Self::new(value)
    }
}

value_component! {
    /// `minecraft:use_animation`: the animation played while using the item.
    UseAnimation(UseAnimationKind);
}

/// The animation of [`UseAnimation`] (vanilla `UseAnimation`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum UseAnimationKind {
    /// None (JSON `none`).
    #[default]
    #[facet(rename = "none")]
    None,
    /// Eating (JSON `eat`).
    #[facet(rename = "eat")]
    Eat,
    /// Drinking (JSON `drink`).
    #[facet(rename = "drink")]
    Drink,
    /// Blocking (JSON `block`).
    #[facet(rename = "block")]
    Block,
    /// Drawing a bow (JSON `bow`).
    #[facet(rename = "bow")]
    Bow,
    /// A camera (JSON `camera`).
    #[facet(rename = "camera")]
    Camera,
    /// A spear (JSON `spear`).
    #[facet(rename = "spear")]
    Spear,
    /// A crossbow (JSON `crossbow`).
    #[facet(rename = "crossbow")]
    Crossbow,
    /// A spyglass (JSON `spyglass`).
    #[facet(rename = "spyglass")]
    Spyglass,
    /// A brush (JSON `brush`).
    #[facet(rename = "brush")]
    Brush,
}

/// `minecraft:use_modifiers`: how using the item behaves.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct UseModifiers {
    /// How long a use lasts, in seconds, at least 0 (JSON `use_duration`).
    #[facet(default)]
    pub use_duration: f32,
    /// The movement speed factor while using it, 0–1 (JSON `movement_modifier`).
    pub movement_modifier: Option<f32>,
    /// Using it emits vibrations (JSON `emit_vibrations`; default true).
    #[facet(default = true)]
    pub emit_vibrations: bool,
    /// The sound when a use starts (JSON `start_sound`).
    pub start_sound: Option<SoundEventName>,
    /// When a held use starts again (JSON `start_using`, from 1.26.30; default `if_first`,
    /// and `always` for a document older than 1.26.30).
    #[facet(default)]
    pub start_using: StartUsing,
}

impl UseModifiers {
    /// A use of `use_duration` seconds that emits vibrations and starts `if_first`.
    pub fn new(use_duration: f32) -> Self {
        Self {
            use_duration,
            movement_modifier: None,
            emit_vibrations: true,
            start_sound: None,
            start_using: StartUsing::IfFirst,
        }
    }
}

impl Default for UseModifiers {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// When a held use of an item starts again ([`UseModifiers::start_using`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum StartUsing {
    /// Only on the first press (JSON `if_first`).
    #[default]
    #[facet(rename = "if_first")]
    IfFirst,
    /// On every press (JSON `always`).
    #[facet(rename = "always")]
    Always,
}
