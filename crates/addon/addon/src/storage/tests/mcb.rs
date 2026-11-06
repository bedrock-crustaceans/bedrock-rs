//! `\x7FMCB` documents through the pack layer: the text the stack hands a loader, the source
//! it marks, and the family readers on the decoded text. The header, the writer and the body
//! decoders are tested in `bedrock_addon_storage`.

#[cfg(feature = "item")]
mod item;
#[cfg(all(
    feature = "voxel_shape",
    feature = "camera",
    feature = "item",
    feature = "particle",
    feature = "trade"
))]
mod vanilla;

use bytes::Bytes;

use super::fixture;
use crate::storage::mcb::to_json_text;

/// An MCB document: the header for `key` at `version`, then `body`.
fn document(key: &str, version: (u16, u16, u32), body: &[u8]) -> Vec<u8> {
    let mut bytes = b"\x7fMCB".to_vec();
    bytes.extend(version.0.to_le_bytes());
    bytes.extend(version.1.to_le_bytes());
    bytes.extend(version.2.to_le_bytes());
    bytes.push(u8::try_from(key.len()).unwrap());
    bytes.extend(key.as_bytes());
    bytes.extend(body);
    bytes
}

/// The pieces a body is built from (the part of `bedrock_addon_storage`'s test builder the
/// pack layer's tests use).
#[derive(Default)]
struct Body(Vec<u8>);

impl Body {
    /// Raw bytes: a flag, a count, a pinned field.
    fn raw(mut self, bytes: &[u8]) -> Body {
        self.0.extend(bytes);
        self
    }

    /// A string with its one- or two-byte length.
    fn string(mut self, text: &str) -> Body {
        let len = text.len();
        if len < 0x80 {
            self.0.push(u8::try_from(len).unwrap());
        } else {
            self.0.push(u8::try_from(len & 0x7F).unwrap() | 0x80);
            self.0.push(u8::try_from(len >> 7).unwrap());
        }
        self.0.extend(text.as_bytes());
        self
    }

    fn f32(mut self, value: f32) -> Body {
        self.0.extend(value.to_le_bytes());
        self
    }

    /// Several floats.
    fn f32s(self, values: &[f32]) -> Body {
        values.iter().fold(self, |body, value| body.f32(*value))
    }
}

mod pack_layer {
    use super::*;
    use crate::pack::stack::mcb_text;
    use crate::storage::PackPath;

    fn shape() -> Vec<u8> {
        let body = Body::default()
            .string("demo:step")
            .raw(&[1])
            .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0]);
        document("minecraft:voxel_shape", (1, 21, 110), &body.0)
    }

    #[test]
    fn a_decoded_document_goes_on_as_json_text_and_a_json_file_unchanged() {
        let path = PackPath::from_static("shapes/step.json");
        let text = mcb_text(&path, Bytes::from(shape())).unwrap();
        assert_eq!(text, to_json_text(&shape()).unwrap().as_bytes());
        let json = Bytes::from_static(b"{\"format_version\": \"1.21.110\"}");
        assert_eq!(mcb_text(&path, json.clone()).unwrap(), json);
    }

    #[test]
    fn a_body_that_is_not_decoded_is_an_unsupported_file_format() {
        let path = PackPath::from_static("shapes/step.json");
        let mut cut = shape();
        cut.pop();
        let finding = mcb_text(&path, Bytes::from(cut)).unwrap_err();
        assert!(matches!(
            finding.code,
            crate::diagnostic::DiagnosticCode::UnsupportedFileFormat
        ));
    }

    /// The family's loader reads the text like a JSON file's.
    #[cfg(feature = "voxel_shape")]
    #[test]
    fn the_familys_loader_reads_the_decoded_text() {
        use crate::document::Content;
        use crate::families::voxel_shape::VoxelShape;

        let path = PackPath::from_static("shapes/step.json");
        let text = mcb_text(&path, Bytes::from(shape())).unwrap();
        let shape = VoxelShape::parse(&path, &text, &crate::load::LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{:#?}", rejected.diagnostics));
        assert!(shape.diagnostics().is_empty(), "{:#?}", shape.diagnostics());
        assert_eq!(shape.shape.boxes.len(), 1);
        assert_eq!(shape.description.identifier.as_str(), "demo:step");
    }
}

/// The stack tells the loader what the bytes were. The game's `items/trial_key.json` is a
/// binary document (`\x7FMCB`, baked at 1.26.30): it is read as text, marked `Mcb`, and the
/// context it is read with says so; a JSON document is `Json` and keeps the pack's context.
/// (`red_shrub.json`, baked at 1.26.50, decodes too: `mcb::item::baked_after_1_26_30`.)
mod document_source {
    use bedrock_addon_core::load::{DocumentSource, LoadContext};
    use bytes::Bytes;

    use super::fixture;
    use crate::pack::stack::{context_for, mcb_source};
    use crate::storage::PackPath;

    #[test]
    fn an_mcb_document_is_marked_and_a_json_one_is_not() {
        let path = PackPath::from_static("items/trial_key.json");
        let (text, source) =
            mcb_source(&path, Bytes::from(fixture("mcb/trial_key.json"))).expect("decoded");
        assert_eq!(source, DocumentSource::Mcb);
        assert!(
            std::str::from_utf8(&text)
                .unwrap()
                .contains("minecraft:item")
        );
        let pack = LoadContext::detached();
        assert_eq!(context_for(&pack, source).source, DocumentSource::Mcb);
        assert_eq!(pack.source, DocumentSource::Json);

        let json = Bytes::from_static(br#"{"format_version": "1.26.50", "minecraft:item": {}}"#);
        let (same, source) = mcb_source(&path, json.clone()).unwrap();
        assert_eq!((same, source), (json, DocumentSource::Json));
        assert!(matches!(
            context_for(&pack, source),
            std::borrow::Cow::Borrowed(_)
        ));
    }
}
