//! `texture_backwards_compatibility_mapping.json`: the game's own table of textures that moved.
//! The client reads it from its app package only, never from a pack.
//!
//! The file is read by the derived reader of [`MovedTexture`] under `Scalars::Lenient` (the
//! client's jsoncpp, as the texture files read): one map from a texture's path to where it
//! moved from, each entry's members typed.

#![cfg_attr(docsrs, feature(doc_cfg))]

use facet::Facet;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity, builders};
use bedrock_addon_core::document::{Content, Document, JsonContent, Placed, Rejected, unversioned};
use bedrock_addon_core::json::jsoncpp::lenient::read_lenient;
use bedrock_addon_core::json::kit::{Node, Profile, Read, UnknownAt};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading, Scalars};

/// The file's path at the root of the app package.
const PATH: &str = "texture_backwards_compatibility_mapping.json";

/// The backwards-compatibility texture mapping of the game's app package: for each texture
/// path, the older texture it is cut from.
///
/// A resource pack that ships the file is not read for it.
#[doc(alias = "texture_backwards_compatibility_mapping.json")]
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct LegacyPathMapping(
    /// The entries, keyed by the texture's path, in the file's order.
    #[facet(schema::unknown = "keep")]
    pub IndexMap<String, Entry>,
);

/// One entry of [`LegacyPathMapping`].
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum Entry {
    /// An object: the texture's place in the older one.
    Moved(MovedTexture),
    /// A value of another kind, kept as written.
    Kept(Value),
}

/// Where a texture of the mapping is cut from. Vanilla writes the members in this order.
///
/// Read as the texture files are read: each member coerced as jsoncpp's `asInt` and
/// `asString` coerce it, which the crate tells as an information; a member of another shape is
/// kept as written, and told; the members it does not read are kept, told as an information.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MovedTexture {
    /// `uv_size`: the size of the area in the older texture, in pixels.
    #[facet(schema::or_kept = "told")]
    pub uv_size: Option<[i32; 2]>,
    /// `uv`: the area's top left corner in the older texture, in pixels.
    #[facet(schema::or_kept = "told")]
    pub uv: Option<[i32; 2]>,
    /// `base_size`: the size of the older texture the corner and size are measured in.
    #[facet(schema::or_kept = "told")]
    pub base_size: Option<[i32; 2]>,
    /// `back_compat_texture`: the older texture's path, pack-relative and without extension.
    pub back_compat_texture: Option<String>,
}

impl FromJson for Entry {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        if node.value().is_object() {
            return MovedTexture::from_json(node, read).map(Self::Moved);
        }
        Some(Self::Kept(node.value().clone()))
    }
}

bedrock_addon_core::impl_schema_leaf!(Entry);
bedrock_addon_core::impl_no_molang!(Entry);

/// Reads the mapping from the file's JSON; the findings go to `d`. A member that reads nothing
/// resets, so the mapping always reads (a root that is no object is an empty one, told).
fn read_mapping(json: &Value, d: &mut Diagnostics) -> LegacyPathMapping {
    let reading = Reading::OWN
        .with_scalars(Scalars::Lenient)
        .with_null(NullHandling::Absent)
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut read = Read::new(reading, d)
        .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last))
        .unknown_severity(Severity::Info);
    schema::read_document(&Node::new(json, JsonPath::new()), &mut read).unwrap_or_default()
}

impl bedrock_addon_core::__private::Sealed for LegacyPathMapping {}

impl Content for LegacyPathMapping {
    const NAME: &'static str = "legacy texture path mapping";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads the file as JSON (comments allowed). Rejected when it is no JSON.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = unversioned::lone(path);
            let mut diagnostics = Diagnostics::default();
            match read_lenient(bytes) {
                Ok(read) => {
                    let mapping = read_mapping(&read.value, &mut diagnostics);
                    Ok(unversioned::document(mapping, origin, diagnostics))
                }
                Err(message) => {
                    diagnostics.push(
                        Diagnostic::of(builders::syntax(message.to_string())).in_file(&origin),
                    );
                    Err(unversioned::rejected(origin, diagnostics))
                }
            }
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(self, out, style)
    }
}

impl JsonContent for LegacyPathMapping {}

impl Placed for LegacyPathMapping {
    /// `texture_backwards_compatibility_mapping.json`, at the root.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Severity;
    use bedrock_addon_core::diagnostic::codes as core_codes;

    fn parse(bytes: &[u8]) -> Result<Document<LegacyPathMapping>, Rejected> {
        let path = PackPath::from_static(PATH);
        LegacyPathMapping::parse(&path, bytes, &LoadContext::detached())
    }

    /// Two entries of the game's file as the writer writes them: one of every member, and
    /// the one vanilla entry with `back_compat_texture` alone.
    const VANILLA: &str = r#"{
  "textures/ui/absorption_effect": {
    "uv_size": [
      18,
      18
    ],
    "uv": [
      36,
      220
    ],
    "base_size": [
      256,
      256
    ],
    "back_compat_texture": "textures/gui/gui"
  },
  "textures/ui/how_to_play_button_hover": {
    "back_compat_texture": "textures/ui/how_to_play_button_hovered"
  }
}
"#;

    fn path_of(finding: &Diagnostic) -> String {
        finding.location.json_path.to_string()
    }

    #[test]
    fn the_mapping_is_held_as_its_json_and_written_back() {
        let text = br#"// the game's table
{ "textures/old": { "back_compat_texture": "textures/new", "uv": [0, 0], "uv_size": [16, 16], "base_size": [64, 64] } }"#;
        let first = parse(text).unwrap().into_value();
        assert!(first.0.contains_key("textures/old"));
        let written = first.to_json().unwrap();
        let second = parse(written.as_bytes()).unwrap().into_value();
        assert_eq!(first, second);
        assert_eq!(second.to_json().unwrap(), written);
    }

    #[test]
    fn each_moved_texture_reads_its_members() {
        let document = parse(VANILLA.as_bytes()).unwrap();
        assert!(document.diagnostics().is_empty());
        let mapping = document.into_value();
        let expected = LegacyPathMapping(IndexMap::from([
            (
                "textures/ui/absorption_effect".to_owned(),
                Entry::Moved(MovedTexture {
                    uv_size: Some([18, 18]),
                    uv: Some([36, 220]),
                    base_size: Some([256, 256]),
                    back_compat_texture: Some("textures/gui/gui".to_owned()),
                }),
            ),
            (
                "textures/ui/how_to_play_button_hover".to_owned(),
                Entry::Moved(MovedTexture {
                    back_compat_texture: Some("textures/ui/how_to_play_button_hovered".to_owned()),
                    ..MovedTexture::default()
                }),
            ),
        ]));
        assert_eq!(mapping, expected);
    }

    #[test]
    fn the_vanilla_layout_writes_back_unchanged() {
        let mapping = parse(VANILLA.as_bytes()).unwrap().into_value();
        assert_eq!(mapping.to_json().unwrap(), VANILLA);
    }

    #[test]
    fn an_entry_that_is_no_object_is_kept_as_written() {
        let document = parse(br#"{"textures/a": 7, "textures/b": [1, "x"]}"#).unwrap();
        assert!(document.diagnostics().is_empty());
        let mapping = document.into_value();
        assert_eq!(mapping.0["textures/a"], Entry::Kept(Value::from(7)));
        assert_eq!(
            mapping.to_json().unwrap(),
            "{\n  \"textures/a\": 7,\n  \"textures/b\": [\n    1,\n    \"x\"\n  ]\n}\n"
        );
    }

    #[test]
    fn a_member_the_game_coerces_is_told_as_an_information() {
        let document =
            parse(br#"{"textures/a": {"uv": [0.5, 1], "back_compat_texture": 3}}"#).unwrap();
        let told: Vec<(Severity, String)> = document
            .diagnostics()
            .iter()
            .map(|finding| (finding.severity, path_of(finding)))
            .collect();
        assert_eq!(
            told,
            [
                (Severity::Info, "textures~1a/uv/0".to_owned()),
                (Severity::Info, "textures~1a/back_compat_texture".to_owned()),
            ]
        );
        let Entry::Moved(moved) = &document.value().0["textures/a"] else {
            panic!("an object is a moved texture");
        };
        assert_eq!(moved.uv, Some([0, 1]));
        assert_eq!(moved.back_compat_texture.as_deref(), Some(""));
    }

    #[test]
    fn a_member_of_another_shape_is_kept_as_written_and_told() {
        let document = parse(br#"{"textures/a": {"uv": "x", "base_size": [1, 2, 3]}}"#).unwrap();
        let told: Vec<String> = document.diagnostics().iter().map(path_of).collect();
        assert_eq!(told, ["textures~1a/uv", "textures~1a/base_size"]);
        let Entry::Moved(moved) = &document.value().0["textures/a"] else {
            panic!("an object is a moved texture");
        };
        assert_eq!((moved.uv, moved.base_size), (None, None));
        let written = document.to_json().unwrap();
        assert!(written.contains("\"uv\": \"x\""), "{written}");
    }

    #[test]
    fn an_unknown_member_is_kept_and_told_as_an_information() {
        let document =
            parse(br#"{"textures/a": {"back_compat_texture": "b", "note": 1}}"#).unwrap();
        let told: Vec<_> = document.diagnostics().iter().collect();
        assert!(
            matches!(told[..], [finding] if finding.code.is(&core_codes::UNRECOGNIZED_FIELD)
                && finding.severity == Severity::Info
                && path_of(finding) == "textures~1a/note"),
            "{told:?}"
        );
        let written = document.to_json().unwrap();
        assert!(written.contains("\"note\": 1"), "{written}");
    }

    #[test]
    fn a_root_that_is_no_object_reads_as_an_empty_mapping() {
        let document = parse(b"[1]").unwrap();
        let told: Vec<String> = document.diagnostics().iter().map(path_of).collect();
        assert_eq!(told, [""]);
        assert!(document.value().0.is_empty());
    }

    #[test]
    fn the_mapping_is_placed_at_the_root() {
        let mapping = parse(b"{}").unwrap().into_value();
        assert_eq!(
            mapping.default_path().as_str(),
            "texture_backwards_compatibility_mapping.json"
        );
    }

    #[test]
    fn a_file_that_is_no_json_is_rejected() {
        let rejected = parse(b"{").unwrap_err();
        let finding = rejected.diagnostics.iter().next().unwrap();
        assert!(finding.code.is(&core_codes::SYNTAX));
        assert_eq!(finding.severity, Severity::Error);
    }
}
