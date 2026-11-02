//! Constructors, the defaults of members a document may leave out, and `validate`.

use super::history::{PoolFamily, ProcessorListFamily, StructureFamily, StructureSetFamily};
use super::load::{JsonFamily, parse_file};
use super::{
    Axis, BlockPredicate, DimensionPadding, GenerationStep, HeightProvider, HeightmapProjection,
    JigsawStructure, LiquidSettings, MaxDistanceFromCenter, PoolAlias, PoolDescription,
    PoolElement, PositionPredicate, ProcessorList, ProcessorListDescription, ProcessorRule,
    Projection, RandomSpreadPlacement, SpreadType, StructureDescription, StructureSet,
    StructureSetDescription, TemplatePool, TemplatePoolElement, TerrainAdaptation,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Content, Extra, Placed};
use bedrock_addon_core::id::{JigsawStructureId, ProcessorListId, StructureSetId, TemplatePoolId};
use bedrock_addon_core::json::StrictInt;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackRef, PackSide};
use bedrock_addon_core::shared::{BlockSpecifier, Filter};

/// The default horizontal distance from the start (`max_distance_from_center`).
const DEFAULT_HORIZONTAL: i32 = 80;

/// The template pool `fallback` defaults to.
const EMPTY_POOL: TemplatePoolId = TemplatePoolId::from_static("minecraft:empty");

/// Runs the family's checks on `value` as if it were loaded from a behaviour pack with `cx`:
/// the document is written and read back, so the parse constraints and the registration checks
/// both run. The value is not changed. A value that cannot be written has no document to
/// check: that is one error finding.
fn validate_detached<F: JsonFamily>(value: &F::Latest, cx: &LoadContext) -> Diagnostics
where
    F::Latest: Placed,
{
    let path = value.default_path();
    match value.to_json() {
        Ok(text) => match parse_file::<F>(
            PackRef::app_data(),
            PackSide::Behavior,
            &path,
            text.as_bytes(),
            cx,
        )
        .1
        {
            Ok(document) => document.diagnostics().clone(),
            Err(rejected) => rejected.diagnostics,
        },
        Err(error) => {
            let mut findings = Diagnostics::default();
            findings.push(Diagnostic::new(
                DiagnosticCode::MalformedInput {
                    message: error.to_string(),
                    text: None,
                },
                Severity::Error,
                Phase::Validate,
                Location::new(None, None, JsonPath::new(), None),
            ));
            findings
        }
    }
}

impl JigsawStructure {
    /// A structure with the required members and every other member left out (its default).
    pub fn new(
        identifier: JigsawStructureId,
        step: GenerationStep,
        start_pool: TemplatePoolId,
        max_depth: u8,
        start_height: HeightProvider,
    ) -> JigsawStructure {
        JigsawStructure {
            description: StructureDescription {
                identifier,
                extra: Extra::default(),
            },
            biome_filters: None,
            step,
            terrain_adaptation: None,
            start_pool,
            start_jigsaw_name: None,
            on_placed: None,
            max_depth: StrictInt(max_depth),
            start_height,
            heightmap_projection: None,
            dimension_padding: None,
            max_distance_from_center: None,
            pool_aliases: None,
            liquid_settings: None,
            extra: Extra::default(),
        }
    }

    /// The biome filter; absent is the empty group, which admits every biome.
    pub fn biome_filters(&self) -> Filter {
        self.biome_filters.clone().unwrap_or_default()
    }

    /// The terrain adaptation; `none` when absent.
    pub fn terrain_adaptation(&self) -> TerrainAdaptation {
        self.terrain_adaptation.unwrap_or_default()
    }

    /// The start jigsaw name; `""` when absent.
    pub fn start_jigsaw_name(&self) -> &str {
        self.start_jigsaw_name
            .as_ref()
            .map_or("", |name| name.0.as_str())
    }

    /// The `on_placed` callback name; `""` when absent (an empty name, never
    /// `minecraft:no_op`, which is a callback the game registers, not a default).
    pub fn on_placed(&self) -> &str {
        self.on_placed.as_ref().map_or("", |name| name.0.as_str())
    }

    /// The heightmap projection; `none` when absent.
    pub fn heightmap_projection(&self) -> HeightmapProjection {
        self.heightmap_projection.unwrap_or_default()
    }

    /// The padding at the top and the bottom of the world, `(top, bottom)`; `(0, 0)` when
    /// absent.
    pub fn dimension_padding(&self) -> (u32, u32) {
        match &self.dimension_padding {
            None => (0, 0),
            Some(DimensionPadding::Both(n)) => (*n, *n),
            Some(DimensionPadding::Range { top, bottom, .. }) => {
                (top.map_or(0, |n| n.0), bottom.map_or(0, |n| n.0))
            }
        }
    }

    /// The maximum distance from the start, `(horizontal, vertical)`; `(80, i32::MAX)` when
    /// absent, `vertical` `i32::MAX` (no limit) when not written.
    pub fn max_distance_from_center(&self) -> (i32, i32) {
        match &self.max_distance_from_center {
            None => (DEFAULT_HORIZONTAL, i32::MAX),
            Some(MaxDistanceFromCenter::Horizontal(h)) => (*h, i32::MAX),
            Some(MaxDistanceFromCenter::Object {
                horizontal,
                vertical,
                ..
            }) => (horizontal.0, vertical.map_or(i32::MAX, |v| v.0)),
        }
    }

    /// The pool aliases; none when absent.
    pub fn pool_aliases(&self) -> &[PoolAlias] {
        self.pool_aliases.as_deref().unwrap_or_default()
    }

    /// The liquid settings; `apply_waterlogging` when absent.
    pub fn liquid_settings(&self) -> LiquidSettings {
        self.liquid_settings.unwrap_or_default()
    }

    /// Runs the family's parse constraints and registration checks as if the structure were
    /// loaded from a behaviour pack with `cx`. The structure is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached::<StructureFamily>(self, cx)
    }
}

impl TemplatePool {
    /// A pool without elements, falling back to `minecraft:empty`.
    pub fn new(identifier: TemplatePoolId) -> TemplatePool {
        TemplatePool {
            description: PoolDescription {
                identifier,
                extra: Extra::default(),
            },
            elements: Vec::new(),
            fallback: None,
            extra: Extra::default(),
        }
    }

    /// The fallback pool; `minecraft:empty` when absent.
    pub fn fallback(&self) -> TemplatePoolId {
        self.fallback.clone().unwrap_or(EMPTY_POOL)
    }

    /// Runs the family's parse constraints and registration checks as if the pool were loaded
    /// from a behaviour pack with `cx`. The pool is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached::<PoolFamily>(self, cx)
    }
}

impl TemplatePoolElement {
    /// An entry placing `element` with the default weight.
    pub fn new(element: PoolElement) -> TemplatePoolElement {
        TemplatePoolElement {
            element,
            weight: None,
            extra: Extra::default(),
        }
    }

    /// The entry's weight; 1 when absent.
    pub fn weight(&self) -> u8 {
        self.weight.map_or(1, |w| w.0)
    }
}

impl PoolElement {
    /// The processor list of a single element; `None` for no processors (absent or `""`) and
    /// for the other element types.
    pub fn processors(&self) -> Option<&ProcessorListId> {
        match self {
            PoolElement::Single {
                processors: Some(list),
                ..
            } if !list.as_str().is_empty() => Some(list),
            _ => None,
        }
    }

    /// The projection of a single element (`rigid` when absent); `None` for the other element
    /// types.
    pub fn projection(&self) -> Option<Projection> {
        match self {
            PoolElement::Single { projection, .. } => Some(projection.unwrap_or_default()),
            _ => None,
        }
    }
}

impl StructureSet {
    /// A set without structures.
    pub fn new(identifier: StructureSetId, placement: RandomSpreadPlacement) -> StructureSet {
        StructureSet {
            description: StructureSetDescription {
                identifier,
                extra: Extra::default(),
            },
            placement,
            structures: Vec::new(),
            extra: Extra::default(),
        }
    }

    /// Runs the family's parse constraints and registration checks as if the set were loaded
    /// from a behaviour pack with `cx`. The set is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached::<StructureSetFamily>(self, cx)
    }
}

impl RandomSpreadPlacement {
    /// A random-spread placement with the default spread type.
    pub fn new(salt: i32, separation: u16, spacing: u16) -> RandomSpreadPlacement {
        RandomSpreadPlacement {
            placement_type: super::PlacementType::RandomSpread,
            salt: StrictInt(salt),
            separation: StrictInt(separation),
            spacing: StrictInt(spacing),
            spread_type: None,
            extra: Extra::default(),
        }
    }

    /// The spread type; `linear` when absent.
    pub fn spread_type(&self) -> SpreadType {
        self.spread_type.unwrap_or_default()
    }
}

impl ProcessorList {
    /// A list without processors.
    pub fn new(identifier: ProcessorListId) -> ProcessorList {
        ProcessorList {
            description: ProcessorListDescription {
                identifier,
                extra: Extra::default(),
            },
            processors: Vec::new(),
            extra: Extra::default(),
        }
    }

    /// Runs the family's parse constraints and registration checks as if the list were loaded
    /// from a behaviour pack with `cx`. The list is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached::<ProcessorListFamily>(self, cx)
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
            extra: Extra::default(),
        }
    }

    /// The input predicate; `always_true` when absent.
    pub fn input_predicate(&self) -> BlockPredicate {
        self.input_predicate
            .clone()
            .unwrap_or(BlockPredicate::AlwaysTrue {
                extra: Extra::default(),
            })
    }

    /// The location predicate; `always_true` when absent.
    pub fn location_predicate(&self) -> BlockPredicate {
        self.location_predicate
            .clone()
            .unwrap_or(BlockPredicate::AlwaysTrue {
                extra: Extra::default(),
            })
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

impl StructureDescription {
    /// A description with this identifier.
    pub fn new(identifier: JigsawStructureId) -> StructureDescription {
        StructureDescription {
            identifier,
            extra: Extra::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::id::BlockId;
    use bedrock_addon_core::json::StrictFloat;

    use crate::Processor;

    /// A list whose rule carries a probability no JSON text can spell.
    fn list_with_a_nan_probability() -> ProcessorList {
        let mut list = ProcessorList::new(ProcessorListId::from_static("ex:unwritable"));
        let mut rule =
            ProcessorRule::new(BlockSpecifier::new(BlockId::from_static("minecraft:stone")));
        rule.input_predicate = Some(BlockPredicate::RandomBlockMatch {
            block: BlockId::from_static("minecraft:dirt"),
            probability: StrictFloat(f32::NAN),
            extra: Extra::default(),
        });
        list.processors.push(Processor::Rule {
            rules: vec![rule],
            extra: Extra::default(),
        });
        list
    }

    #[test]
    fn validating_a_value_that_cannot_be_written_reports_a_finding() {
        let findings = list_with_a_nan_probability().validate(&LoadContext::detached());
        assert!(
            findings.iter().any(|finding| {
                finding.severity == Severity::Error
                    && matches!(finding.code, DiagnosticCode::MalformedInput { .. })
            }),
            "an unwritable value validated clean: {findings:?}"
        );
    }
}
