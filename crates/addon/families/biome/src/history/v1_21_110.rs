//! Format 1.21.110: the first format vanilla loads, read from the whole document
//! (`format_version` and `minecraft:biome`).

use super::v1_21_120;
use crate::Description;
use crate::codes;
use crate::component::BiomeComponent;
use crate::read::{Root, read_as};
use crate::write::object;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, Disposition, Kind,
};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

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
}

impl Versioned for Biome {
    const VERSION: FormatVersion = known::V1_21_110;
}

impl VersionStruct for Biome {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut found = Diagnostics::default();
        let parsed = read_as::<Root>(document, Self::VERSION, cx, &mut found).map(|root| Self {
            format_version: root.format_version,
            description: root.biome.description,
            components: root.biome.components.0,
        });
        // Any failure of the strict load is reported as vanilla's "Invalid Biome file", after
        // the finding that says where. An unknown member, which the slice's mask grades an
        // error, is not counted: the line follows the other failures alone (pinned by
        // `chain::a_prerelease_of_1_21_120_is_read_from_the_raw_text_by_the_legacy_struct`).
        if found
            .iter()
            .any(|finding| finding.rejects() && !is_unknown_member(finding))
        {
            found.push(
                Diagnostic::of(&codes::INVALID_BIOME_FILE)
                    .detail(DiagnosticDetail::Syntax(
                        codes::INVALID_BIOME_FILE.template().to_owned(),
                    ))
                    .at(cx.at.clone()),
            );
        }
        d.extend_with_unknown(found);
        parsed
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut root = facet_value::VObject::new();
        root.insert("format_version", Value::from(self.format_version.as_str()));
        root.insert(
            "minecraft:biome",
            object(&self.description, &self.components)?,
        );
        // The root's unknown members are the document's, written from its table.
        Ok(root.into())
    }
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
        }
    }
}

/// Whether `finding` tells of an unknown member or component.
fn is_unknown_member(finding: &Diagnostic) -> bool {
    // The kind holds the unrecognized field and component definitions and no others.
    matches!(finding.kind(), Kind::UnknownMember(Disposition::Kept))
}
