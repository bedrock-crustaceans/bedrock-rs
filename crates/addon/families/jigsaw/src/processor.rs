//! `minecraft:processor_list`: a processor list.

use super::validate::validate_detached;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::id::{BlockId, LootTablePath, ProcessorListId};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_shared_shapes::BlockSpecifier;

/// A processor list: rules that modify or replace blocks of a structure template while it is
/// placed (`minecraft:processor_list`, in `worldgen/processors/`).
///
/// JSON: `{"format_version": "1.21.20", "minecraft:processor_list": {"description",
/// "processors"}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[doc(alias = "minecraft:processor_list")]
#[doc(alias = "JigsawStructureProcessorList")]
#[facet(schema::unknown = "keep")]
pub struct ProcessorList {
    /// The list's identifier (JSON `description`).
    pub description: ProcessorListDescription,
    /// The processors, applied in order; may be empty (JSON `processors`).
    pub processors: Vec<Processor>,
}

/// `minecraft:processor_list/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ProcessorListDescription {
    /// The list's identifier, `namespace:name` (JSON `identifier`).
    pub identifier: ProcessorListId,
}

/// One processor, by its `processor_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "processor_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum Processor {
    /// Block rules (JSON `minecraft:rule`).
    #[facet(rename = "minecraft:rule")]
    #[facet(schema::unknown = "keep")]
    Rule {
        /// The rules, each evaluated on every block (JSON `rules`).
        rules: Vec<ProcessorRule>,
    },
    /// Blocks of a tag the template never replaces (JSON `minecraft:protected_blocks`).
    #[facet(rename = "minecraft:protected_blocks")]
    #[facet(schema::unknown = "keep")]
    ProtectedBlocks {
        /// The block tag (JSON `value`).
        value: String,
    },
    /// Blocks of the template that are not placed (JSON `minecraft:block_ignore`).
    #[facet(rename = "minecraft:block_ignore")]
    #[facet(schema::unknown = "keep")]
    BlockIgnore {
        /// The blocks (JSON `blocks`).
        blocks: Vec<BlockId>,
    },
    /// Another processor applied a limited number of times (JSON `minecraft:capped`).
    #[facet(rename = "minecraft:capped")]
    #[facet(schema::unknown = "keep")]
    Capped {
        /// The processor, never itself `capped` (JSON `delegate`).
        delegate: CappedDelegate,
        /// How many times it may apply (JSON `limit`).
        limit: IntProvider,
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
    #[facet(schema::unknown = "keep")]
    Rule {
        /// The rules (JSON `rules`).
        rules: Vec<ProcessorRule>,
    },
    /// Protected blocks (JSON `minecraft:protected_blocks`).
    #[facet(rename = "minecraft:protected_blocks")]
    #[facet(schema::unknown = "keep")]
    ProtectedBlocks {
        /// The block tag (JSON `value`).
        value: String,
    },
    /// Ignored blocks (JSON `minecraft:block_ignore`).
    #[facet(rename = "minecraft:block_ignore")]
    #[facet(schema::unknown = "keep")]
    BlockIgnore {
        /// The blocks (JSON `blocks`).
        blocks: Vec<BlockId>,
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
    #[facet(schema::unknown = "keep")]
    Constant {
        /// `constant` (JSON `type`).
        #[facet(rename = "type")]
        provider_type: IntProviderType,
        /// The number (JSON `value`).
        value: i32,
    },
    /// Uniformly between the two ends (JSON `{"type": "uniform", "min_inclusive",
    /// "max_inclusive"}`); vanilla does not reject `min_inclusive` above `max_inclusive`.
    #[facet(schema::unknown = "keep")]
    Uniform {
        /// `uniform` (JSON `type`).
        #[facet(rename = "type")]
        provider_type: IntProviderType,
        /// The lowest number (JSON `min_inclusive`).
        min_inclusive: i32,
        /// The highest number (JSON `max_inclusive`).
        max_inclusive: i32,
    },
}

/// The `type` of an object [`IntProvider`]; it must name the form the members make.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ProcessorRule {
    /// The test on the template's block (JSON `input_predicate`, default `always_true`).
    pub input_predicate: Option<BlockPredicate>,
    /// The test on the world's block at the position (JSON `location_predicate`, default
    /// `always_true`).
    pub location_predicate: Option<BlockPredicate>,
    /// The test on the position (JSON `position_predicate`; absent is no test).
    pub position_predicate: Option<PositionPredicate>,
    /// What the placed block's block entity receives (JSON `block_entity_modifier`; absent is
    /// nothing).
    pub block_entity_modifier: Option<BlockEntityModifier>,
    /// The block placed instead (JSON `output_state`).
    pub output_state: BlockSpecifier,
}

/// A test on a block, by its `predicate_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "predicate_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum BlockPredicate {
    /// Always passes (JSON `minecraft:always_true`).
    #[facet(rename = "minecraft:always_true")]
    #[facet(schema::unknown = "keep")]
    AlwaysTrue {},
    /// The block is `block` (JSON `minecraft:block_match`).
    #[facet(rename = "minecraft:block_match")]
    #[facet(schema::unknown = "keep")]
    BlockMatch {
        /// The block (JSON `block`).
        block: BlockId,
    },
    /// The block is `block`, with a chance (JSON `minecraft:random_block_match`).
    #[facet(rename = "minecraft:random_block_match")]
    #[facet(schema::unknown = "keep")]
    RandomBlockMatch {
        /// The block (JSON `block`).
        block: BlockId,
        /// The chance, 0 to 1 (JSON `probability`).
        probability: f32,
    },
    /// The block has a tag (JSON `minecraft:tag_match`).
    #[facet(rename = "minecraft:tag_match")]
    #[facet(schema::unknown = "keep")]
    TagMatch {
        /// The block tag (JSON `tag`).
        tag: String,
    },
    /// The block and its states match (JSON `minecraft:blockstate_match`).
    #[facet(rename = "minecraft:blockstate_match")]
    #[facet(schema::unknown = "keep")]
    BlockstateMatch {
        /// The block and states (JSON `block_state`).
        block_state: BlockSpecifier,
    },
    /// The block and its states match, with a chance (JSON
    /// `minecraft:random_blockstate_match`).
    #[facet(rename = "minecraft:random_blockstate_match")]
    #[facet(schema::unknown = "keep")]
    RandomBlockstateMatch {
        /// The block and states (JSON `block_state`).
        block_state: BlockSpecifier,
        /// The chance, 0 to 1 (JSON `probability`).
        probability: f32,
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
    #[facet(schema::unknown = "keep")]
    AlwaysTrue {},
    /// Passes with a chance that changes linearly with the distance along an axis (JSON
    /// `minecraft:axis_aligned_linear_pos`). Every member defaults to 0, `axis` to `y`.
    #[facet(rename = "minecraft:axis_aligned_linear_pos")]
    #[facet(schema::unknown = "keep")]
    AxisAlignedLinearPos {
        /// The chance at `min_dist`, in `[0, 1)` (JSON `min_chance`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        min_chance: Option<f32>,
        /// The chance at `max_dist`, in `[0, 1)` (JSON `max_chance`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        max_chance: Option<f32>,
        /// The distance where the chance is `min_chance`, 0 or more (JSON `min_dist`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        min_dist: Option<i32>,
        /// The distance where the chance is `max_chance`, 0 or more (JSON `max_dist`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        max_dist: Option<i32>,
        /// The axis (JSON `axis`, default `y`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        axis: Option<Axis>,
    },
}

/// An axis (`axis`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
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
    #[facet(schema::unknown = "keep")]
    Passthrough {},
    /// A loot table is added (JSON `minecraft:append_loot`).
    #[facet(rename = "minecraft:append_loot")]
    #[facet(schema::unknown = "keep")]
    AppendLoot {
        /// The loot table, at most 256 bytes, not empty (JSON `loot_table`).
        loot_table: LootTablePath,
    },
}

impl ProcessorList {
    /// A list without processors.
    pub fn new(identifier: ProcessorListId) -> ProcessorList {
        ProcessorList {
            description: ProcessorListDescription { identifier },
            processors: Vec::new(),
        }
    }

    /// Runs the family's parse constraints and registration checks as if the list were loaded
    /// from a behaviour pack with `cx`. The list is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached(self, cx)
    }
}

impl ProcessorRule {
    /// A rule placing `output_state` for every block, with every predicate left out.
    pub fn new(output_state: BlockSpecifier) -> ProcessorRule {
        ProcessorRule {
            input_predicate: None,
            location_predicate: None,
            position_predicate: None,
            block_entity_modifier: None,
            output_state,
        }
    }

    /// The input predicate; `always_true` when absent.
    pub fn input_predicate(&self) -> BlockPredicate {
        self.input_predicate
            .clone()
            .unwrap_or(BlockPredicate::AlwaysTrue {})
    }

    /// The location predicate; `always_true` when absent.
    pub fn location_predicate(&self) -> BlockPredicate {
        self.location_predicate
            .clone()
            .unwrap_or(BlockPredicate::AlwaysTrue {})
    }
}

impl PositionPredicate {
    /// The axis of an `axis_aligned_linear_pos` predicate (`y` when absent); `None` for
    /// `always_true`.
    pub fn axis(&self) -> Option<Axis> {
        match self {
            PositionPredicate::AxisAlignedLinearPos { axis, .. } => Some(axis.unwrap_or_default()),
            PositionPredicate::AlwaysTrue { .. } => None,
        }
    }
}

// The data enums walk nothing: their bounds and members are checked by hand (`read.rs`).
bedrock_addon_core::impl_schema_leaf!(
    Processor,
    CappedDelegate,
    IntProvider,
    BlockPredicate,
    PositionPredicate,
    BlockEntityModifier
);
bedrock_addon_core::impl_no_molang!(
    Processor,
    CappedDelegate,
    IntProvider,
    BlockPredicate,
    PositionPredicate,
    BlockEntityModifier
);

#[cfg(test)]
mod validate_tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Severity;
    use bedrock_addon_core::id::BlockId;

    use crate::Processor;

    /// A list whose rule carries a probability no JSON text can spell.
    fn list_with_a_nan_probability() -> ProcessorList {
        let mut list = ProcessorList::new(ProcessorListId::from_static("ex:unwritable"));
        let mut rule =
            ProcessorRule::new(BlockSpecifier::new(BlockId::from_static("minecraft:stone")));
        rule.input_predicate = Some(BlockPredicate::RandomBlockMatch {
            block: BlockId::from_static("minecraft:dirt"),
            probability: f32::NAN,
        });
        list.processors.push(Processor::Rule { rules: vec![rule] });
        list
    }

    #[test]
    fn validating_a_value_that_cannot_be_written_reports_a_finding() {
        let findings = list_with_a_nan_probability().validate(&LoadContext::detached());
        assert!(
            findings.iter().any(|finding| {
                finding.severity == Severity::Error && finding.code.name() == "MalformedInput"
            }),
            "an unwritable value validated clean: {findings:?}"
        );
    }
}
