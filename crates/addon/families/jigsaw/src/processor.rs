//! `minecraft:processor_list`: a processor list.

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{BlockId, LootTablePath, ProcessorListId};
use bedrock_addon_core::json::{StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::shared::BlockSpecifier;

/// A processor list: rules that modify or replace blocks of a structure template while it is
/// placed (`minecraft:processor_list`, in `worldgen/processors/`).
///
/// JSON: `{"format_version": "1.21.20", "minecraft:processor_list": {"description",
/// "processors"}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:processor_list")]
#[doc(alias = "JigsawStructureProcessorList")]
pub struct ProcessorList {
    /// The list's identifier (JSON `description`).
    pub description: ProcessorListDescription,
    /// The processors, applied in order; may be empty (JSON `processors`).
    pub processors: Vec<Processor>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:processor_list/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct ProcessorListDescription {
    /// The list's identifier, `namespace:name` (JSON `identifier`).
    pub identifier: ProcessorListId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One processor, by its `processor_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "processor_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum Processor {
    /// Block rules (JSON `minecraft:rule`).
    #[facet(rename = "minecraft:rule")]
    Rule {
        /// The rules, each evaluated on every block (JSON `rules`).
        rules: Vec<ProcessorRule>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Blocks of a tag the template never replaces (JSON `minecraft:protected_blocks`).
    #[facet(rename = "minecraft:protected_blocks")]
    ProtectedBlocks {
        /// The block tag (JSON `value`).
        value: StrictString,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Blocks of the template that are not placed (JSON `minecraft:block_ignore`).
    #[facet(rename = "minecraft:block_ignore")]
    BlockIgnore {
        /// The blocks (JSON `blocks`).
        blocks: Vec<BlockId>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Another processor applied a limited number of times (JSON `minecraft:capped`).
    #[facet(rename = "minecraft:capped")]
    Capped {
        /// The processor, never itself `capped` (JSON `delegate`).
        delegate: CappedDelegate,
        /// How many times it may apply (JSON `limit`).
        limit: IntProvider,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// The processor a `minecraft:capped` processor limits: any processor type but `capped`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "processor_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum CappedDelegate {
    /// Block rules (JSON `minecraft:rule`).
    #[facet(rename = "minecraft:rule")]
    Rule {
        /// The rules (JSON `rules`).
        rules: Vec<ProcessorRule>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Protected blocks (JSON `minecraft:protected_blocks`).
    #[facet(rename = "minecraft:protected_blocks")]
    ProtectedBlocks {
        /// The block tag (JSON `value`).
        value: StrictString,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Ignored blocks (JSON `minecraft:block_ignore`).
    #[facet(rename = "minecraft:block_ignore")]
    BlockIgnore {
        /// The blocks (JSON `blocks`).
        blocks: Vec<BlockId>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// An integer drawn per use (`limit`): a number, `{"type": "constant", "value"}` or
/// `{"type": "uniform", "min_inclusive", "max_inclusive"}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum IntProvider {
    /// A number (JSON a number).
    Value(i32),
    /// Always `value` (JSON `{"type": "constant", "value"}`).
    Constant {
        /// `constant` (JSON `type`).
        #[facet(rename = "type")]
        provider_type: IntProviderType,
        /// The number (JSON `value`).
        value: StrictInt<i32>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Uniformly between the two ends (JSON `{"type": "uniform", "min_inclusive",
    /// "max_inclusive"}`); vanilla does not reject `min_inclusive` above `max_inclusive`.
    Uniform {
        /// `uniform` (JSON `type`).
        #[facet(rename = "type")]
        provider_type: IntProviderType,
        /// The lowest number (JSON `min_inclusive`).
        min_inclusive: StrictInt<i32>,
        /// The highest number (JSON `max_inclusive`).
        max_inclusive: StrictInt<i32>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// The `type` of an object [`IntProvider`]; it must name the form the members make.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum IntProviderType {
    /// JSON `constant`.
    Constant,
    /// JSON `uniform`.
    Uniform,
}

/// A block rule (`rules[]`): every block of the template for which all predicates hold becomes
/// `output_state`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct ProcessorRule {
    /// The test on the template's block (JSON `input_predicate`, default `always_true`).
    #[facet(default)]
    pub input_predicate: Option<BlockPredicate>,
    /// The test on the world's block at the position (JSON `location_predicate`, default
    /// `always_true`).
    #[facet(default)]
    pub location_predicate: Option<BlockPredicate>,
    /// The test on the position (JSON `position_predicate`; absent is no test).
    #[facet(default)]
    pub position_predicate: Option<PositionPredicate>,
    /// What the placed block's block entity receives (JSON `block_entity_modifier`; absent is
    /// nothing).
    #[facet(default)]
    pub block_entity_modifier: Option<BlockEntityModifier>,
    /// The block placed instead (JSON `output_state`).
    pub output_state: BlockSpecifier,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A test on a block, by its `predicate_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "predicate_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum BlockPredicate {
    /// Always passes (JSON `minecraft:always_true`).
    #[facet(rename = "minecraft:always_true")]
    AlwaysTrue {
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// The block is `block` (JSON `minecraft:block_match`).
    #[facet(rename = "minecraft:block_match")]
    BlockMatch {
        /// The block (JSON `block`).
        block: BlockId,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// The block is `block`, with a chance (JSON `minecraft:random_block_match`).
    #[facet(rename = "minecraft:random_block_match")]
    RandomBlockMatch {
        /// The block (JSON `block`).
        block: BlockId,
        /// The chance, 0 to 1 (JSON `probability`).
        probability: StrictFloat<f32>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// The block has a tag (JSON `minecraft:tag_match`).
    #[facet(rename = "minecraft:tag_match")]
    TagMatch {
        /// The block tag (JSON `tag`).
        tag: StrictString,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// The block and its states match (JSON `minecraft:blockstate_match`).
    #[facet(rename = "minecraft:blockstate_match")]
    BlockstateMatch {
        /// The block and states (JSON `block_state`).
        block_state: BlockSpecifier,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// The block and its states match, with a chance (JSON
    /// `minecraft:random_blockstate_match`).
    #[facet(rename = "minecraft:random_blockstate_match")]
    RandomBlockstateMatch {
        /// The block and states (JSON `block_state`).
        block_state: BlockSpecifier,
        /// The chance, 0 to 1 (JSON `probability`).
        probability: StrictFloat<f32>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// A test on a block's position, by its `predicate_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "predicate_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum PositionPredicate {
    /// Always passes (JSON `minecraft:always_true`).
    #[facet(rename = "minecraft:always_true")]
    AlwaysTrue {
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Passes with a chance that changes linearly with the distance along an axis (JSON
    /// `minecraft:axis_aligned_linear_pos`). Every member defaults to 0, `axis` to `y`.
    #[facet(rename = "minecraft:axis_aligned_linear_pos")]
    AxisAlignedLinearPos {
        /// The chance at `min_dist`, in `[0, 1)` (JSON `min_chance`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        min_chance: Option<StrictFloat<f32>>,
        /// The chance at `max_dist`, in `[0, 1)` (JSON `max_chance`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        max_chance: Option<StrictFloat<f32>>,
        /// The distance where the chance is `min_chance`, 0 or more (JSON `min_dist`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        min_dist: Option<StrictInt<i32>>,
        /// The distance where the chance is `max_chance`, 0 or more (JSON `max_dist`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        max_dist: Option<StrictInt<i32>>,
        /// The axis (JSON `axis`, default `y`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        axis: Option<Axis>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// An axis (`axis`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
pub enum Axis {
    /// JSON `x`.
    X,
    /// JSON `y`, the default.
    #[default]
    Y,
    /// JSON `z`.
    Z,
}

/// What a placed block's block entity receives, by its `type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "type")]
#[repr(u8)]
#[non_exhaustive]
pub enum BlockEntityModifier {
    /// Nothing changes (JSON `minecraft:passthrough`).
    #[facet(rename = "minecraft:passthrough")]
    Passthrough {
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A loot table is added (JSON `minecraft:append_loot`).
    #[facet(rename = "minecraft:append_loot")]
    AppendLoot {
        /// The loot table, at most 256 bytes, not empty (JSON `loot_table`).
        loot_table: LootTablePath,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}
