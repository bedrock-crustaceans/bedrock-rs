//! Reading and writing the aim-assist documents: each root struct typed from the whole text
//! after the shape check.

use super::history::v1_21_50;
use super::{AimAssistCategories, AimAssistPreset};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_support::chain_file as file;
use bedrock_addon_support::chain_file::document;
use bedrock_addon_support::shape::{self, Member, Shape, opt, req};

/// A list of strings (`vector<Reference<…>>`, `vector<string>`).
const STRINGS: Shape = Shape::List(&Shape::Str);
/// A map of integers (the priority maps).
const PRIORITY_MAP: Shape = Shape::Map(&Shape::Float);

/// The preset at format 1.21.50, members in bind order.
const PRESET_V1_21_50: &[Member] = &[
    req("identifier", Shape::Str),
    opt("exclusion_list", STRINGS),
    opt("liquid_targeting_list", STRINGS),
    opt("item_settings", Shape::Map(&Shape::Str)),
    opt("default_item_settings", Shape::Str),
    opt("hand_settings", Shape::Str),
];

/// The exclusion settings of a preset at format 1.21.120.
const EXCLUSION_SETTINGS: &[Member] = &[
    opt("blocks", STRINGS),
    opt("entities", STRINGS),
    opt("block_tags", STRINGS),
    opt("entity_type_families", STRINGS),
];

/// The preset at format 1.21.120, members in bind order.
const PRESET: &[Member] = &[
    req("identifier", Shape::Str),
    opt("exclusion_settings", Shape::Object(EXCLUSION_SETTINGS)),
    opt("liquid_targeting_list", STRINGS),
    opt("item_settings", Shape::Map(&Shape::Str)),
    opt("default_item_settings", Shape::Str),
    opt("hand_settings", Shape::Str),
];

/// A preset file at 1.21.50: both members required.
const PRESET_FILE_V1_21_50: &[Member] = &[
    req("format_version", Shape::Str),
    req(
        "minecraft:aim_assist_preset",
        Shape::Object(PRESET_V1_21_50),
    ),
];

/// A preset file at 1.21.120.
const PRESET_FILE: &[Member] = &[
    req("format_version", Shape::Str),
    req("minecraft:aim_assist_preset", Shape::Object(PRESET)),
];

/// The priority maps of a category.
const PRIORITIES: &[Member] = &[
    opt("entities", PRIORITY_MAP),
    opt("blocks", PRIORITY_MAP),
    opt("block_tags", PRIORITY_MAP),
    opt("entity_type_families", PRIORITY_MAP),
    opt("entity_default", Shape::Float),
    opt("block_default", Shape::Float),
];

/// One category.
const CATEGORY: &[Member] = &[
    req("name", Shape::Str),
    req("priorities", Shape::Object(PRIORITIES)),
];

/// The categories list of a categories file.
const CATEGORIES: &[Member] = &[req("categories", Shape::List(&Shape::Object(CATEGORY)))];

/// A categories file.
const CATEGORIES_FILE: &[Member] = &[
    req("format_version", Shape::Str),
    req("minecraft:aim_assist_categories", Shape::Object(CATEGORIES)),
];

pub(crate) use super::checks::{category_namespaces, preset_namespaces};

impl VersionStruct for v1_21_50::AimAssistPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut preset: Self = file::typed(
            text,
            &Shape::Object(PRESET_FILE_V1_21_50),
            Some("minecraft:aim_assist_preset"),
            shape::NOT_IN_SCHEMA,
            cx,
            d,
        )?;
        preset.source_format_version = cx.original_version.into();
        Some(preset)
    }

    /// The whole document, at the version it was read at.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            &self
                .source_format_version
                .version()
                .max(Self::VERSION)
                .to_string(),
            "minecraft:aim_assist_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for AimAssistPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        file::typed(
            text,
            &Shape::Object(PRESET_FILE),
            Some("minecraft:aim_assist_preset"),
            shape::NOT_IN_SCHEMA,
            cx,
            d,
        )
    }

    /// The whole document, at format 1.21.120.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            "1.21.120",
            "minecraft:aim_assist_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for AimAssistCategories {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let file: Self = file::typed(
            text,
            &Shape::Object(CATEGORIES_FILE),
            Some("minecraft:aim_assist_categories"),
            shape::NOT_IN_SCHEMA,
            cx,
            d,
        )?;
        super::checks::priorities(&file, &cx.at, d).then_some(file)
    }

    /// The whole document, at format 1.21.50.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            "1.21.50",
            "minecraft:aim_assist_categories",
            write_payload(self)?,
        ))
    }
}
