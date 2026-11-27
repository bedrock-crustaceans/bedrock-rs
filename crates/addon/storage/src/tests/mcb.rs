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
use crate::brarchive::Brarchive;
use crate::magic::{FileType, detect};
use crate::mcb::{McbHeader, McbValue, to_json_text, write_json};
use crate::{McbFault, StorageError};

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
        bedrock_addon_core::json::from_str_jsonc(text).unwrap()
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
    fn an_f32_is_written_in_its_own_shortest_form() {
        assert_eq!(text(&McbValue::F32(0.1)), "0.1");
        for x in [16_777_217.0_f32, 1e-45, f32::MAX, f32::MIN_POSITIVE, 0.3] {
            let written = text(&McbValue::F32(x));
            assert_eq!(written.parse::<f32>().unwrap(), x, "{written} is not {x:?}");
            assert_eq!(written, format!("{x:?}"), "not the f32's own shortest text");
        }
    }

    #[test]
    fn nested_values_keep_their_order_and_escapes_in_compact_form() {
        let value = McbValue::Object(vec![
            (
                "b".into(),
                McbValue::Array(vec![McbValue::Int(-1), McbValue::UInt(2)]),
            ),
            ("a".into(), McbValue::String("\u{8}\u{c}\r\u{1f}".into())),
        ]);
        assert_eq!(text(&value), r#"{"b":[-1,2],"a":"\b\f\r\u001f"}"#);
    }

    #[test]
    fn a_del_character_is_escaped() {
        assert_eq!(text(&McbValue::String("a\u{7f}b".into())), r#""a\u007fb""#);
    }

    #[test]
    fn a_value_too_deep_is_refused() {
        let nested =
            |depth: usize| (0..depth).fold(McbValue::Null, |inner, _| McbValue::Array(vec![inner]));
        assert!(write_json(&nested(127)).is_ok());
        assert!(write_json(&nested(128)).is_ok());
        assert!(matches!(
            write_json(&nested(129)),
            Err(StorageError::Mcb {
                fault: McbFault::TooDeep
            })
        ));
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
