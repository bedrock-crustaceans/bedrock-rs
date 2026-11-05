//! Writing [`Dimension`]: the newest struct's `VersionStruct` (its parse reads the whole
//! document with `read`), and the whole document it writes.

use super::read::{RootLatest, clouds_constraint, height_constraints, parse_root};
use super::{Dimension, V1_26_50};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

impl Versioned for Dimension {
    const VERSION: FormatVersion = V1_26_50;
}

impl VersionStruct for Dimension {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let rule = cx.load.null_rule;
        let root: RootLatest =
            parse_root(text, cx, rule, |root: &mut RootLatest| &mut root.extra, d)?;
        let inner = root.dimension;
        let components = cx.at.with(&["minecraft:dimension", "components"]);
        let height = inner
            .components
            .dimension_height
            .as_ref()
            .is_none_or(|height| {
                height_constraints(height, &components.child("minecraft:dimension_height"), d)
            });
        let clouds = inner.components.clouds.as_ref().is_none_or(|clouds| {
            clouds_constraint(clouds, &components.child("minecraft:clouds"), d)
        });
        if !(height && clouds) {
            return None;
        }
        Some(Dimension {
            description: inner.description,
            components: inner.components,
            source_format_version: cx.original_version,
            extra: inner.extra,
            root_extra: root.extra,
        })
    }

    /// The whole document: `format_version` (the document's own when it is 1.26.50 or newer,
    /// else 1.26.50), the payload and the other root members. Not core's `write_envelope`: the
    /// family reads the raw text at every version (`EnvelopeCheck::Never`), so the struct writes
    /// its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        let mut payload = facet_value::VObject::new();
        payload.insert("description", write_payload(&self.description)?);
        let mut components = write_payload(&self.components)?;
        if let Some(object) = components.as_object_mut()
            && self
                .components
                .dimension_height
                .as_ref()
                .is_some_and(|height| !height.is_writable())
        {
            object.remove("minecraft:dimension_height");
        }
        payload.insert("components", components);
        for (key, value) in self.extra.iter() {
            payload.insert(key, value.clone());
        }
        let mut root = facet_value::VObject::new();
        root.insert(
            "format_version",
            self.source_format_version.max(V1_26_50).to_string(),
        );
        root.insert("minecraft:dimension", payload);
        for (key, value) in self.root_extra.iter() {
            root.insert(key, value.clone());
        }
        Ok(root.into())
    }
}
