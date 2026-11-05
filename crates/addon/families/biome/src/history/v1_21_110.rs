//! Format 1.21.110: the first format vanilla loads, read from the whole document
//! (`format_version` and `minecraft:biome`).

use facet::Facet;
use facet_json::RawJson;

use super::v1_21_120;
use crate::Description;
use crate::component::{BiomeComponent, child};
use crate::parse::payload;
use crate::texts::INVALID_BIOME_FILE;
use crate::write::object;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, VanillaText};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, StrictString, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{
    ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned, parse_payload,
};

/// A biome of format `[1.21.110, 1.21.120)`, read with its envelope: vanilla reads
/// `format_version` (a string) and `minecraft:biome` as members of one strict struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Biome {
    /// The document's `format_version`, as written (JSON `format_version`, a string).
    pub format_version: String,
    /// The biome's name (JSON `minecraft:biome/description`).
    pub description: Description,
    /// The components (JSON `minecraft:biome/components`).
    pub components: ComponentMap<BiomeComponent>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
    /// Unknown members of the document's root.
    pub root_extra: Extra,
}

/// The document's root.
#[derive(Facet)]
struct Root<'a> {
    format_version: StrictString,
    #[facet(rename = "minecraft:biome")]
    biome: RawJson<'a>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

impl Versioned for Biome {
    const VERSION: FormatVersion = known::V1_21_110;
}

impl VersionStruct for Biome {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut found = Diagnostics::default();
        let parsed = read(document, cx, &mut found);
        // Any failure of the strict load is reported as vanilla's "Invalid Biome file", after
        // the finding that says where.
        if found.has_errors() {
            found.push(
                Diagnostic::error(DiagnosticCode::MalformedInput {
                    message: INVALID_BIOME_FILE.to_owned(),
                    text: Some(VanillaText::Fixed(INVALID_BIOME_FILE)),
                })
                .at(cx.at.clone()),
            );
        }
        d.extend(found);
        parsed
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut root = facet_value::VObject::new();
        root.insert("format_version", Value::from(self.format_version.as_str()));
        root.insert(
            "minecraft:biome",
            object(&self.description, &self.components, &self.extra)?,
        );
        for (key, value) in self.root_extra.iter() {
            root.insert(key, value.clone());
        }
        Ok(root.into())
    }
}

/// The document, root first.
fn read(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Biome> {
    let root: Root<'_> = parse_payload(document, cx, d)?;
    let slice = document.nested(&root.biome)?;
    let inner = ParseCx {
        at: child(&cx.at, &["minecraft:biome"]),
        ..cx.clone()
    };
    let payload = payload(slice, Biome::VERSION, &inner, d)?;
    Some(Biome {
        format_version: root.format_version.0,
        description: payload.description,
        components: payload.components,
        extra: payload.extra,
        root_extra: root.extra,
    })
}

impl MolangFields for Biome {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::from("minecraft:biome"));
        visitor.nested("components", &mut self.components);
        visitor.leave();
    }
}

/// The step to 1.21.120: `description` and every component move unchanged; `format_version`
/// stays in the envelope.
impl Upgrade for Biome {
    type Next = v1_21_120::Biome;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_21_120::Biome {
        v1_21_120::Biome {
            description: self.description,
            components: self.components,
            extra: self.extra,
        }
    }
}
