//! `\x7FMCB` documents: the header, the JSON text writer that keeps the lexical number tag, the
//! body decoders on bodies built by hand from the known layouts, and every MCB entry of
//! the game's archives against its JSON twin.

mod camera;
mod catalog;
mod hostile;
mod item;
mod particle;
mod poi;
mod trade;
mod vanilla;
mod voxel;

use bytes::Bytes;
use facet_value::Value;

use super::fixture;
use crate::storage::brarchive::Brarchive;
use crate::storage::magic::{FileType, detect};
use crate::storage::mcb::{McbHeader, McbValue, to_json_text, write_json};
use crate::storage::{McbFault, StorageError};

fn mcb_archive() -> Brarchive {
    Brarchive::decode(Bytes::from(fixture("brarchive/mcb.brarchive"))).unwrap()
}

mod header {
    use super::*;

    #[test]
    fn an_archive_entry_is_told_apart_by_magic() {
        let archive = mcb_archive();
        assert_eq!(detect(&archive.get("ruby.json").unwrap()), FileType::Mcb);
        assert_eq!(
            detect(&archive.get("sapphire.json").unwrap()),
            FileType::Plain
        );
    }

    #[test]
    fn reads_the_format_version_and_payload_key() {
        let bytes = mcb_archive().get("ruby.json").unwrap();
        let header = McbHeader::parse(&bytes).unwrap();
        assert_eq!((header.major, header.minor, header.patch), (1, 26, 30));
        assert_eq!(header.payload_key, "minecraft:item");
        assert_eq!(header.format_version(), "1.26.30");
        assert_eq!(&bytes[header.body_offset..], b"\x0bdemo:ruby\x00");
    }

    #[test]
    fn a_cut_header_is_refused() {
        let bytes = mcb_archive().get("ruby.json").unwrap();
        for len in [0, 4, 11, 12, 20] {
            assert!(
                matches!(
                    McbHeader::parse(&bytes[..len]),
                    Err(StorageError::Mcb {
                        fault: McbFault::Header
                    })
                ),
                "{len}"
            );
        }
        assert!(matches!(
            McbHeader::parse(b"{\"format_version\":2}"),
            Err(StorageError::Mcb {
                fault: McbFault::Header
            })
        ));
    }

    #[test]
    fn a_body_that_is_not_a_known_layout_is_unsupported() {
        // the fixture's body is an identifier and one byte: an item is cut short there
        let bytes = mcb_archive().get("ruby.json").unwrap();
        assert!(matches!(
            to_json_text(&bytes),
            Err(StorageError::Mcb {
                fault: McbFault::UnsupportedBody
            })
        ));
    }
}

mod writer {
    use super::*;

    fn text(value: &McbValue) -> String {
        write_json(value).unwrap()
    }

    /// The text read back by facet-json, as the content parsers read it.
    fn reparse(text: &str) -> Value {
        crate::json::from_str_jsonc(text).unwrap()
    }

    #[test]
    fn a_float_always_has_a_fraction_or_an_exponent() {
        assert_eq!(text(&McbValue::F64(1.0)), "1.0");
        assert_eq!(text(&McbValue::F64(-0.0)), "-0.0");
        assert_eq!(text(&McbValue::F64(0.5)), "0.5");
        assert_eq!(text(&McbValue::F64(1e300)), "1e300");
        assert_eq!(text(&McbValue::F64(1.5e-7)), "1.5e-7");
        assert_eq!(text(&McbValue::F32(0.1)), "0.1");
        assert_eq!(text(&McbValue::F32(2.0)), "2.0");
    }

    #[test]
    fn an_integer_never_has_one() {
        assert_eq!(text(&McbValue::Int(1)), "1");
        assert_eq!(text(&McbValue::Int(-64)), "-64");
        assert_eq!(text(&McbValue::UInt(u64::MAX)), "18446744073709551615");
    }

    #[test]
    fn the_number_tag_survives_facet() {
        let value = reparse(&text(&McbValue::Object(vec![
            ("scale".into(), McbValue::F64(1.0)),
            ("count".into(), McbValue::Int(1)),
            ("zero".into(), McbValue::F64(-0.0)),
        ])));
        let object = value.as_object().unwrap();
        assert!(object.get("scale").unwrap().as_number().unwrap().is_float());
        assert!(
            object
                .get("count")
                .unwrap()
                .as_number()
                .unwrap()
                .is_integer()
        );
        assert!(object.get("zero").unwrap().as_number().unwrap().is_float());
    }

    #[test]
    fn a_non_finite_float_has_no_json_form() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                write_json(&McbValue::Array(vec![McbValue::F64(value)])),
                Err(StorageError::Mcb {
                    fault: McbFault::NonFiniteFloat
                })
            ));
        }
        assert!(matches!(
            write_json(&McbValue::F32(f32::NAN)),
            Err(StorageError::Mcb {
                fault: McbFault::NonFiniteFloat
            })
        ));
    }

    #[test]
    fn strings_are_escaped_and_order_is_kept() {
        let value = McbValue::Object(vec![
            ("z".into(), McbValue::String("a\"b\\c\n\t\u{1}é".into())),
            (
                "a".into(),
                McbValue::Array(vec![McbValue::Null, McbValue::Bool(true)]),
            ),
        ]);
        let written = text(&value);
        assert_eq!(written, r#"{"z":"a\"b\\c\n\t\u0001é","a":[null,true]}"#);
        let reparsed = reparse(&written);
        let object = reparsed.as_object().unwrap();
        let keys: Vec<&str> = object.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["z", "a"]);
        assert_eq!(
            object.get("z").unwrap().as_string().unwrap().as_str(),
            "a\"b\\c\n\t\u{1}é"
        );
    }
}

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

/// The pieces a body is built from.
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

    fn i32(mut self, value: i32) -> Body {
        self.0.extend(value.to_le_bytes());
        self
    }

    fn u32(mut self, value: u32) -> Body {
        self.0.extend(value.to_le_bytes());
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

    /// A component's key: the FNV-1a 32 hash of its name.
    fn component(self, name: &str) -> Body {
        let hash = name.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
        });
        self.u32(hash)
    }

    /// A particle expression that is a float.
    fn number(self, value: f32) -> Body {
        self.raw(&[1]).f32(value)
    }

    /// A particle expression that is Molang text, in the form without a version.
    fn molang(self, text: &str) -> Body {
        self.raw(&[0]).string(text)
    }

    /// A particle expression that is Molang text with its version.
    fn molang_at(mut self, text: &str, version: u16) -> Body {
        self = self.raw(&[0]).string(text);
        self.0.extend(version.to_le_bytes());
        self
    }
}

/// Requires `result` to be the unsupported-body fault.
#[track_caller]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a test hands over what it built"
)]
fn assert_unsupported(result: Result<String, StorageError>) {
    assert!(
        matches!(
            result,
            Err(StorageError::Mcb {
                fault: McbFault::UnsupportedBody
            })
        ),
        "{result:?}"
    );
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
