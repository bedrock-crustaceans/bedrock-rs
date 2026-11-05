//! Formats 1.18.0 to 1.26.50 (vanilla `v1_21_60::DimensionDocument`, read by both 1.26.20
//! slices): the height is `minecraft:dimension_bounds`.

use crate::{
    DimensionBounds, DimensionComponents, DimensionDescription, DimensionHeight, Generation,
};
use crate::{RootV1, constraints, parse_root, slice_null_rule};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned, write_payload,
};

/// The newest format's height component.
const HEIGHT: &str = "minecraft:dimension_height";
/// This format's height component.
const BOUNDS: &str = "minecraft:dimension_bounds";

/// A dimension document of `[1.18.0, 1.26.50)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Dimension {
    /// JSON `description`.
    pub description: DimensionDescription,
    /// JSON `components`.
    pub components: Components,
    /// The document's `format_version` (the floor when absent); written as `format_version`.
    pub source_format_version: FormatVersion,
}

/// `minecraft:dimension/components` of `[1.18.0, 1.26.50)`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Components {
    /// The lowest and highest y (JSON `minecraft:dimension_bounds`); vanilla reads an absent
    /// member as `{0, 0}`.
    #[facet(rename = "minecraft:dimension_bounds")]
    pub dimension_bounds: Option<DimensionBounds>,
    /// The generator (JSON `minecraft:generation`).
    #[facet(rename = "minecraft:generation")]
    pub generation: Generation,
}

impl Components {
    /// The bounds vanilla uses: the written ones, or `{0, 0}`.
    pub fn bounds(&self) -> DimensionBounds {
        self.dimension_bounds
            .clone()
            .unwrap_or_else(|| DimensionBounds::new(0, 0))
    }
}

impl Versioned for Dimension {
    const VERSION: FormatVersion = crate::FLOOR;
}

impl VersionStruct for Dimension {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let rule = slice_null_rule(cx);
        let root: RootV1 = parse_root(text, cx, rule, d)?;
        let mut inner = root.dimension;
        // `{-512, 512, multipleOf 16}` on both members, the same in every format that reads
        // the bounds.
        if let Some(bounds) = &mut inner.components.dimension_bounds {
            let at = cx.at.with(&[
                "minecraft:dimension",
                "components",
                "minecraft:dimension_bounds",
            ]);
            if !constraints(bounds, &at, d) {
                return None;
            }
        }
        Some(Dimension {
            description: inner.description,
            components: inner.components,
            source_format_version: cx.original_version,
        })
    }

    /// The whole document: `format_version` (the document's own), the payload and the other
    /// root members.
    fn write(&self) -> Result<Value, EncodeError> {
        let mut payload = facet_value::VObject::new();
        payload.insert("description", write_payload(&self.description)?);
        payload.insert("components", write_payload(&self.components)?);
        let mut root = facet_value::VObject::new();
        let version = self.source_format_version.max(Self::VERSION);
        root.insert("format_version", version.to_string());
        root.insert("minecraft:dimension", payload);
        // The unknown members of the payload and the root are the document's, written from
        // its table.
        Ok(root.into())
    }
}

/// The step to the newest format: the bounds always become a present height `{min, max − min}`
/// (absent bounds are `{0, 0}`),
/// and `minecraft:dimension_bounds` is dropped without a message; `minecraft:default_biome` and
/// `minecraft:clouds` stay absent.
impl Upgrade for Dimension {
    type Next = crate::Dimension;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let bounds = self.components.bounds();
        let dimension_height = bounds_to_height(&bounds);
        // A `minecraft:dimension_height` of the file is no member of this format (the game
        // reads the bounds here and does not know the name): the height is the step's, and
        // the file's member is not carried under the newest format's name, where it would
        // be read as the height. The crate notes it where it is, as an information.
        let components = JsonPath::of_keys(["minecraft:dimension", "components"]);
        // `minecraft:dimension_bounds` is not carried: what it held besides its members goes.
        cx.diagnostics
            .unknown_mut()
            .forget(&components.child("minecraft:dimension_bounds"));
        if cx
            .diagnostics
            .unknown_mut()
            .take(&components, HEIGHT)
            .is_some()
        {
            cx.diagnostics.push(dropped(HEIGHT));
        }
        // The one height the newest format cannot spell, `{0, 0}` (the bind constraint keeps
        // `height_range` in [16, 1024]; the nether and the end load with it), is written as
        // neither member (`Dimension::write`): the crate notes it, as an information, where the
        // file has the bounds.
        if self.components.dimension_bounds.is_some() && !dimension_height.is_writable() {
            cx.diagnostics.push(dropped(BOUNDS));
        }
        let dimension_height = Some(dimension_height);
        crate::Dimension {
            description: self.description,
            components: DimensionComponents {
                dimension_height,
                default_biome: None,
                clouds: None,
                generation: self.components.generation,
            },
            source_format_version: self.source_format_version,
        }
    }
}

/// The note that the component `name` of the file is not carried into the written text.
fn dropped(name: &str) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::DroppedField { text: None })
        .at(JsonPath::new().with(&["minecraft:dimension", "components", name]))
        .phase(Phase::Upgrade)
}

/// The bounds as a height: `min_y = (i16)min`, `height_range = (i16)(max − min)`, for every
/// pair, `{0, 0}` and an empty or negative range included (the validator judges them). Each is
/// the low sixteen bits, as the game stores them; the bind constraints keep a parsed file's
/// bounds in `[-512, 512]`, so nothing is cut there.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the game keeps the low sixteen bits of each"
)]
fn bounds_to_height(bounds: &DimensionBounds) -> DimensionHeight {
    let (min, max) = (bounds.min, bounds.max);
    DimensionHeight {
        min_y: min as i16,
        height_range: max.wrapping_sub(min) as i16,
    }
}
