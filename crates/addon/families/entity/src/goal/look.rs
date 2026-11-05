//! Looking goals: looking at entities and around.

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{Filter, FloatRange, IntRange, ItemDescriptor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::Facet;
use facet_value::VObject;

use super::check::{Bound, Check, Deprecated, Goal, deprecated_text};
use super::common::ControlFlag;
use super::convert;
use super::molang::visit_items;

/// `minecraft:behavior.beg`: allows this mob to look at and follow the player that holds food they like.
// `behavior.beg`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.beg")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Beg {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `items`: list of items that this mob likes.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub items: Option<Vec<ItemDescriptor>>,
    /// `look_distance`: distance in blocks the mob will beg from. Default 8.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_distance: Option<StrictFloat<f32>>,
    /// `look_time`: the range of time in seconds this mob will stare at the player holding a food they like, begging for it. Default 2 to 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_time: Option<IntRange>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Beg {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.map_or(8.0, |v| v.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (i32, i32) {
        self.look_time.map_or((2, 4), |range| range.resolve(2, 4))
    }
}

impl Goal for Beg {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.items("items", self.items.as_deref());
        c.object_range("look_time", &mut self.look_time);
    }
}

impl MolangFields for Beg {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "items", &mut self.items);
    }
}

/// `minecraft:behavior.look_at_entity`: allows the mob to look at nearby entities.
// `behavior.look_at_entity`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_entity")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LookAtEntity {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_horizontal: Option<StrictInt<i32>>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_vertical: Option<StrictInt<i32>>,
    /// `filters`: filter to determine the conditions for this mob to look at the entity.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_distance: Option<StrictFloat<f32>>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LookAtEntity {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.map_or(360, |v| v.0)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.map_or(360, |v| v.0)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.map_or(8.0, |v| v.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }
}

impl Goal for LookAtEntity {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int(
            "angle_of_view_horizontal",
            &mut self.angle_of_view_horizontal,
            Bound::MinMax(0, 360),
        );
        c.int(
            "angle_of_view_vertical",
            &mut self.angle_of_view_vertical,
            Bound::MinMax(0, 360),
        );
        c.filter("filters", self.filters.as_ref());
        c.float("look_distance", &mut self.look_distance, Bound::Min(0.0));
        c.range("look_time", &mut self.look_time);
        c.float(
            "probability",
            &mut self.probability,
            Bound::MinMax(0.0, 1.0),
        );
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::look_time(json, d);
    }
}

impl MolangFields for LookAtEntity {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.look_at_player`: allows the mob to look at the player when the player is nearby.
// `behavior.look_at_player`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_player")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LookAtPlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_horizontal: Option<StrictInt<i32>>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_vertical: Option<StrictInt<i32>>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_distance: Option<StrictFloat<f32>>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LookAtPlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.map_or(360, |v| v.0)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.map_or(360, |v| v.0)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.map_or(8.0, |v| v.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }
}

impl Goal for LookAtPlayer {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int(
            "angle_of_view_horizontal",
            &mut self.angle_of_view_horizontal,
            Bound::MinMax(0, 360),
        );
        c.int(
            "angle_of_view_vertical",
            &mut self.angle_of_view_vertical,
            Bound::MinMax(0, 360),
        );
        c.float("look_distance", &mut self.look_distance, Bound::Min(0.0));
        c.range("look_time", &mut self.look_time);
        c.float(
            "probability",
            &mut self.probability,
            Bound::MinMax(0.0, 1.0),
        );
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::look_time(json, d);
    }
}

impl MolangFields for LookAtPlayer {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.look_at_target`: allows the mob to look at the entity they are targeting.
// `behavior.look_at_target`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LookAtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_horizontal: Option<StrictInt<i32>>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_vertical: Option<StrictInt<i32>>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_distance: Option<StrictFloat<f32>>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LookAtTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.map_or(360, |v| v.0)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.map_or(360, |v| v.0)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.map_or(8.0, |v| v.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }
}

impl Goal for LookAtTarget {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int(
            "angle_of_view_horizontal",
            &mut self.angle_of_view_horizontal,
            Bound::MinMax(0, 360),
        );
        c.int(
            "angle_of_view_vertical",
            &mut self.angle_of_view_vertical,
            Bound::MinMax(0, 360),
        );
        c.float("look_distance", &mut self.look_distance, Bound::Min(0.0));
        c.range("look_time", &mut self.look_time);
        c.float(
            "probability",
            &mut self.probability,
            Bound::MinMax(0.0, 1.0),
        );
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::look_time(json, d);
    }
}

impl MolangFields for LookAtTarget {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.look_at_trading_player`: allows the mob to look at the player they are trading with.
// `behavior.look_at_trading_player`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_trading_player")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LookAtTradingPlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_horizontal: Option<StrictInt<i32>>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub angle_of_view_vertical: Option<StrictInt<i32>>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_distance: Option<StrictFloat<f32>>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LookAtTradingPlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.map_or(360, |v| v.0)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.map_or(360, |v| v.0)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.map_or(8.0, |v| v.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }
}

impl Goal for LookAtTradingPlayer {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int(
            "angle_of_view_horizontal",
            &mut self.angle_of_view_horizontal,
            Bound::MinMax(0, 360),
        );
        c.int(
            "angle_of_view_vertical",
            &mut self.angle_of_view_vertical,
            Bound::MinMax(0, 360),
        );
        c.float("look_distance", &mut self.look_distance, Bound::Min(0.0));
        c.range("look_time", &mut self.look_time);
        c.float(
            "probability",
            &mut self.probability,
            Bound::MinMax(0.0, 1.0),
        );
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::look_time(json, d);
    }
}

impl MolangFields for LookAtTradingPlayer {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.random_look_around`: allows the mob to randomly look around.
// `behavior.random_look_around`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_look_around")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RandomLookAround {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `look_time`: the range of time in seconds the mob will stay looking in a random direction before looking elsewhere. Default 20 to 40.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_time: Option<IntRange>,
    /// `max_angle_of_view_horizontal`: the rightmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default 1.8e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_angle_of_view_horizontal: Option<StrictFloat<f32>>,
    /// `min_angle_of_view_horizontal`: the leftmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default -1.8e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_angle_of_view_horizontal: Option<StrictFloat<f32>>,
    /// `probability`: the probability of randomly looking around. Default 0.02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RandomLookAround {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `look_time`, or its default 20 to 40.
    pub fn look_time(&self) -> (i32, i32) {
        self.look_time
            .map_or((20, 40), |range| range.resolve(20, 40))
    }

    /// `max_angle_of_view_horizontal`, or its default 1.8e+02.
    pub fn max_angle_of_view_horizontal(&self) -> f32 {
        self.max_angle_of_view_horizontal.map_or(1.8e2, |v| v.0)
    }

    /// `min_angle_of_view_horizontal`, or its default -1.8e+02.
    pub fn min_angle_of_view_horizontal(&self) -> f32 {
        self.min_angle_of_view_horizontal.map_or(-1.8e2, |v| v.0)
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }
}

impl Goal for RandomLookAround {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("look_time", &mut self.look_time);
    }
}

impl MolangFields for RandomLookAround {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.random_look_around_and_sit`: allows the mob to randomly sit and look around for a duration. Note: Must have a sitting animation set up to use this.
///
/// Deprecated members, reported and dropped: `look_time`.
// `behavior.random_look_around_and_sit`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_look_around_and_sit")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RandomLookAroundAndSit {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `continue_if_leashed`: if the goal should continue to be used as long as the mob is leashed. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub continue_if_leashed: Option<StrictBool>,
    /// `continue_sitting_on_reload`: the mob will stay sitting on reload. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub continue_sitting_on_reload: Option<StrictBool>,
    /// `max_angle_of_view_horizontal`: the rightmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default 1.8e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_angle_of_view_horizontal: Option<StrictFloat<f32>>,
    /// `max_look_count`: the max amount of unique looks a mob will have while looking around. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_look_count: Option<StrictInt<i32>>,
    /// `max_look_time`: the max amount of time (in ticks) a mob will stay looking at a direction while looking around. Default 40.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_look_time: Option<StrictInt<i32>>,
    /// `min_angle_of_view_horizontal`: the leftmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default -1.8e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_angle_of_view_horizontal: Option<StrictFloat<f32>>,
    /// `min_look_count`: the min amount of unique looks a mob will have while looking around. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_look_count: Option<StrictInt<i32>>,
    /// `min_look_time`: the min amount of time (in ticks) a mob will stay looking at a direction while looking around. Default 20.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_look_time: Option<StrictInt<i32>>,
    /// `probability`: the probability of randomly looking around. Default 0.02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// `random_look_around_cooldown`: the cooldown in seconds before the goal can be used again. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_look_around_cooldown: Option<StrictInt<i32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RandomLookAroundAndSit {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `continue_if_leashed`, or its default `false`.
    pub fn continue_if_leashed(&self) -> bool {
        self.continue_if_leashed.is_some_and(|v| v.0)
    }

    /// `continue_sitting_on_reload`, or its default `false`.
    pub fn continue_sitting_on_reload(&self) -> bool {
        self.continue_sitting_on_reload.is_some_and(|v| v.0)
    }

    /// `max_angle_of_view_horizontal`, or its default 1.8e+02.
    pub fn max_angle_of_view_horizontal(&self) -> f32 {
        self.max_angle_of_view_horizontal.map_or(1.8e2, |v| v.0)
    }

    /// `max_look_count`, or its default 2.
    pub fn max_look_count(&self) -> i32 {
        self.max_look_count.map_or(2, |v| v.0)
    }

    /// `max_look_time`, or its default 40.
    pub fn max_look_time(&self) -> i32 {
        self.max_look_time.map_or(40, |v| v.0)
    }

    /// `min_angle_of_view_horizontal`, or its default -1.8e+02.
    pub fn min_angle_of_view_horizontal(&self) -> f32 {
        self.min_angle_of_view_horizontal.map_or(-1.8e2, |v| v.0)
    }

    /// `min_look_count`, or its default 1.
    pub fn min_look_count(&self) -> i32 {
        self.min_look_count.map_or(1, |v| v.0)
    }

    /// `min_look_time`, or its default 20.
    pub fn min_look_time(&self) -> i32 {
        self.min_look_time.map_or(20, |v| v.0)
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }

    /// `random_look_around_cooldown`, or its default 0.
    pub fn random_look_around_cooldown(&self) -> i32 {
        self.random_look_around_cooldown.map_or(0, |v| v.0)
    }
}

impl Goal for RandomLookAroundAndSit {
    const TYPED: FormatVersion = known::V1_26_30;

    const DEPRECATED: &'static [Deprecated] =
        &[("look_time", deprecated_text!("look_time"), known::V1_26_30)];

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for RandomLookAroundAndSit {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.trade_interest`: allows the mob to look at a player that is holding a tradable item.
// `behavior.trade_interest`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.trade_interest")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TradeInterest {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `carried_item_switch_time`: the max time in seconds that the trader will hold an item before attempting to switch for a different item that takes the same trade. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub carried_item_switch_time: Option<StrictFloat<f32>>,
    /// `cooldown`: the time in seconds before the trader can use this goal again. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictFloat<f32>>,
    /// `interest_time`: the max time in seconds that the trader will be interested with showing its trade items. Default 45.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub interest_time: Option<StrictFloat<f32>>,
    /// `remove_item_time`: the max time in seconds that the trader will wait when you no longer have items to trade. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub remove_item_time: Option<StrictFloat<f32>>,
    /// `within_radius`: distance in blocks this mob can be interested by a player holding an item they like. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub within_radius: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl TradeInterest {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `carried_item_switch_time`, or its default 2.
    pub fn carried_item_switch_time(&self) -> f32 {
        self.carried_item_switch_time.map_or(2.0, |v| v.0)
    }

    /// `cooldown`, or its default 2.
    pub fn cooldown(&self) -> f32 {
        self.cooldown.map_or(2.0, |v| v.0)
    }

    /// `interest_time`, or its default 45.
    pub fn interest_time(&self) -> f32 {
        self.interest_time.map_or(45.0, |v| v.0)
    }

    /// `remove_item_time`, or its default 1.
    pub fn remove_item_time(&self) -> f32 {
        self.remove_item_time.map_or(1.0, |v| v.0)
    }

    /// `within_radius`, or its default 0.
    pub fn within_radius(&self) -> f32 {
        self.within_radius.map_or(0.0, |v| v.0)
    }
}

impl Goal for TradeInterest {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for TradeInterest {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}
