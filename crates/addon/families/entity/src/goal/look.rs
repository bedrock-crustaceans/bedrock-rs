//! Looking goals: looking at entities and around.

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, GoalMeta, Schema};
use bedrock_addon_core::shared::{Filter, FloatRange, IntRange, ItemDescriptor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::Facet;
use facet_value::VObject;

use super::check::{Bound, Deprecated, Goal, GoalCheck, deprecated_text};
use super::common::ControlFlag;
use super::convert;
use super::molang::visit_items;

/// `minecraft:behavior.beg`: allows this mob to look at and follow the player that holds food they like.
// `behavior.beg`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.beg")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Beg {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `items`: list of items that this mob likes.
    pub items: Option<Vec<ItemDescriptor>>,
    /// `look_distance`: distance in blocks the mob will beg from. Default 8.
    pub look_distance: Option<f32>,
    /// `look_time`: the range of time in seconds this mob will stare at the player holding a food they like, begging for it. Default 2 to 4.
    pub look_time: Option<IntRange>,
}

impl Beg {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.unwrap_or(8.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (i32, i32) {
        self.look_time.map_or((2, 4), |range| range.resolve(2, 4))
    }
}

impl Goal for Beg {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[facet(schema::unknown = "keep")]
pub struct LookAtEntity {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    pub angle_of_view_horizontal: Option<i32>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    pub angle_of_view_vertical: Option<i32>,
    /// `filters`: filter to determine the conditions for this mob to look at the entity.
    pub filters: Option<Filter>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    pub look_distance: Option<f32>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    pub probability: Option<f32>,
}

impl LookAtEntity {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.unwrap_or(360)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.unwrap_or(360)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.unwrap_or(8.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.unwrap_or(0.02)
    }
}

impl Goal for LookAtEntity {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

bedrock_addon_core::impl_no_molang!(LookAtEntity);

/// `minecraft:behavior.look_at_player`: allows the mob to look at the player when the player is nearby.
// `behavior.look_at_player`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_player")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct LookAtPlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    pub angle_of_view_horizontal: Option<i32>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    pub angle_of_view_vertical: Option<i32>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    pub look_distance: Option<f32>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    pub probability: Option<f32>,
}

impl LookAtPlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.unwrap_or(360)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.unwrap_or(360)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.unwrap_or(8.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.unwrap_or(0.02)
    }
}

impl Goal for LookAtPlayer {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

bedrock_addon_core::impl_no_molang!(LookAtPlayer);

/// `minecraft:behavior.look_at_target`: allows the mob to look at the entity they are targeting.
// `behavior.look_at_target`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct LookAtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    pub angle_of_view_horizontal: Option<i32>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    pub angle_of_view_vertical: Option<i32>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    pub look_distance: Option<f32>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    pub probability: Option<f32>,
}

impl LookAtTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.unwrap_or(360)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.unwrap_or(360)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.unwrap_or(8.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.unwrap_or(0.02)
    }
}

impl Goal for LookAtTarget {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

bedrock_addon_core::impl_no_molang!(LookAtTarget);

/// `minecraft:behavior.look_at_trading_player`: allows the mob to look at the player they are trading with.
// `behavior.look_at_trading_player`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.look_at_trading_player")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct LookAtTradingPlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_of_view_horizontal`: the angle in degrees that the mob can see rotated on the Y-axis (left-right). Default 360; 0 to 360.
    pub angle_of_view_horizontal: Option<i32>,
    /// `angle_of_view_vertical`: the angle in degrees that the mob can see rotated on the X-axis (up-down). Default 360; 0 to 360.
    pub angle_of_view_vertical: Option<i32>,
    /// `look_distance`: the distance in blocks from which the entity will look at the nearest entity. Default 8; at least 0.
    pub look_distance: Option<f32>,
    /// `look_time`: time range to look at the nearest entity. Default 2 to 4.
    pub look_time: Option<FloatRange>,
    /// `probability`: the probability of looking at the target. A value of 1.00 is 100%. Default 0.02; 0 to 1.
    pub probability: Option<f32>,
}

impl LookAtTradingPlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `angle_of_view_horizontal`, or its default 360.
    pub fn angle_of_view_horizontal(&self) -> i32 {
        self.angle_of_view_horizontal.unwrap_or(360)
    }

    /// `angle_of_view_vertical`, or its default 360.
    pub fn angle_of_view_vertical(&self) -> i32 {
        self.angle_of_view_vertical.unwrap_or(360)
    }

    /// `look_distance`, or its default 8.
    pub fn look_distance(&self) -> f32 {
        self.look_distance.unwrap_or(8.0)
    }

    /// `look_time`, or its default 2 to 4.
    pub fn look_time(&self) -> (f32, f32) {
        self.look_time
            .map_or((2.0, 4.0), |range| range.resolve(2.0, 4.0))
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.unwrap_or(0.02)
    }
}

impl Goal for LookAtTradingPlayer {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

bedrock_addon_core::impl_no_molang!(LookAtTradingPlayer);

/// `minecraft:behavior.random_look_around`: allows the mob to randomly look around.
// `behavior.random_look_around`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_look_around")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct RandomLookAround {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `look_time`: the range of time in seconds the mob will stay looking in a random direction before looking elsewhere. Default 20 to 40.
    pub look_time: Option<IntRange>,
    /// `max_angle_of_view_horizontal`: the rightmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default 1.8e+02.
    pub max_angle_of_view_horizontal: Option<f32>,
    /// `min_angle_of_view_horizontal`: the leftmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default -1.8e+02.
    pub min_angle_of_view_horizontal: Option<f32>,
    /// `probability`: the probability of randomly looking around. Default 0.02.
    pub probability: Option<f32>,
}

impl RandomLookAround {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `look_time`, or its default 20 to 40.
    pub fn look_time(&self) -> (i32, i32) {
        self.look_time
            .map_or((20, 40), |range| range.resolve(20, 40))
    }

    /// `max_angle_of_view_horizontal`, or its default 1.8e+02.
    pub fn max_angle_of_view_horizontal(&self) -> f32 {
        self.max_angle_of_view_horizontal.unwrap_or(1.8e2)
    }

    /// `min_angle_of_view_horizontal`, or its default -1.8e+02.
    pub fn min_angle_of_view_horizontal(&self) -> f32 {
        self.min_angle_of_view_horizontal.unwrap_or(-1.8e2)
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.unwrap_or(0.02)
    }
}

impl Goal for RandomLookAround {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.range("look_time", &mut self.look_time);
    }
}

bedrock_addon_core::impl_no_molang!(RandomLookAround);

/// `minecraft:behavior.random_look_around_and_sit`: allows the mob to randomly sit and look around for a duration. Note: Must have a sitting animation set up to use this.
///
/// Deprecated members, reported and dropped: `look_time`.
// `behavior.random_look_around_and_sit`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_look_around_and_sit")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomLookAroundAndSit {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `continue_if_leashed`: if the goal should continue to be used as long as the mob is leashed. Default `false`.
    pub continue_if_leashed: Option<bool>,
    /// `continue_sitting_on_reload`: the mob will stay sitting on reload. Default `false`.
    pub continue_sitting_on_reload: Option<bool>,
    /// `max_angle_of_view_horizontal`: the rightmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default 1.8e+02.
    #[facet(schema::fallback = 1.8e2)]
    pub max_angle_of_view_horizontal: Option<f32>,
    /// `max_look_count`: the max amount of unique looks a mob will have while looking around. Default 2.
    #[facet(schema::fallback = 2)]
    pub max_look_count: Option<i32>,
    /// `max_look_time`: the max amount of time (in ticks) a mob will stay looking at a direction while looking around. Default 40.
    #[facet(schema::fallback = 40)]
    pub max_look_time: Option<i32>,
    /// `min_angle_of_view_horizontal`: the leftmost angle a mob can look at on the horizontal plane with respect to its initial facing direction. Default -1.8e+02.
    #[facet(schema::fallback = -1.8e2)]
    pub min_angle_of_view_horizontal: Option<f32>,
    /// `min_look_count`: the min amount of unique looks a mob will have while looking around. Default 1.
    #[facet(schema::fallback = 1)]
    pub min_look_count: Option<i32>,
    /// `min_look_time`: the min amount of time (in ticks) a mob will stay looking at a direction while looking around. Default 20.
    #[facet(schema::fallback = 20)]
    pub min_look_time: Option<i32>,
    /// `probability`: the probability of randomly looking around. Default 0.02.
    #[facet(schema::fallback = 0.02)]
    pub probability: Option<f32>,
    /// `random_look_around_cooldown`: the cooldown in seconds before the goal can be used again. Default 0.
    #[facet(schema::fallback = 0)]
    pub random_look_around_cooldown: Option<i32>,
}

impl RandomLookAroundAndSit {
    /// `continue_if_leashed`, or its default `false`.
    pub fn continue_if_leashed(&self) -> bool {
        self.continue_if_leashed.is_some_and(|v| v)
    }

    /// `continue_sitting_on_reload`, or its default `false`.
    pub fn continue_sitting_on_reload(&self) -> bool {
        self.continue_sitting_on_reload.is_some_and(|v| v)
    }
}

impl Goal for RandomLookAroundAndSit {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] =
        &[("look_time", deprecated_text!("look_time"), known::V1_26_30)];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.trade_interest`: allows the mob to look at a player that is holding a tradable item.
// `behavior.trade_interest`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.trade_interest")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct TradeInterest {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `carried_item_switch_time`: the max time in seconds that the trader will hold an item before attempting to switch for a different item that takes the same trade. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub carried_item_switch_time: Option<f32>,
    /// `cooldown`: the time in seconds before the trader can use this goal again. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub cooldown: Option<f32>,
    /// `interest_time`: the max time in seconds that the trader will be interested with showing its trade items. Default 45.
    #[facet(schema::fallback = 45.0)]
    pub interest_time: Option<f32>,
    /// `remove_item_time`: the max time in seconds that the trader will wait when you no longer have items to trade. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub remove_item_time: Option<f32>,
    /// `within_radius`: distance in blocks this mob can be interested by a player holding an item they like. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for TradeInterest {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

crate::hand_fields!(Beg {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "look_distance": Option<f32>,
});
crate::hand_fields!(LookAtEntity {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "angle_of_view_horizontal": Option<i32>,
    "angle_of_view_vertical": Option<i32>,
    "look_distance": Option<f32>,
    "probability": Option<f32>,
});
crate::hand_fields!(LookAtPlayer {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "angle_of_view_horizontal": Option<i32>,
    "angle_of_view_vertical": Option<i32>,
    "look_distance": Option<f32>,
    "probability": Option<f32>,
});
crate::hand_fields!(LookAtTarget {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "angle_of_view_horizontal": Option<i32>,
    "angle_of_view_vertical": Option<i32>,
    "look_distance": Option<f32>,
    "probability": Option<f32>,
});
crate::hand_fields!(LookAtTradingPlayer {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "angle_of_view_horizontal": Option<i32>,
    "angle_of_view_vertical": Option<i32>,
    "look_distance": Option<f32>,
    "probability": Option<f32>,
});
crate::hand_fields!(RandomLookAround {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "max_angle_of_view_horizontal": Option<f32>,
    "min_angle_of_view_horizontal": Option<f32>,
    "probability": Option<f32>,
});
