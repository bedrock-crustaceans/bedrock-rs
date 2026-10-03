use bedrock_macros::ProtoCodec;
use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::trace::{FieldSpan, trace};

#[derive(ProtoCodec, Debug, PartialEq)]
struct Inner {
    a: u8,
    #[endianness(le)]
    b: u16,
}

#[derive(ProtoCodec, Debug, PartialEq)]
struct Outer {
    name: String,
    inner: Inner,
    #[vec_repr(u8)]
    list: Vec<u8>,
}

fn span(
    name: &'static str,
    index: Option<usize>,
    depth: usize,
    start: usize,
    end: usize,
) -> FieldSpan {
    FieldSpan {
        name,
        index,
        depth,
        start,
        end,
    }
}

#[test]
fn derived_fields_record_their_byte_ranges() {
    let bytes = [2, b'h', b'i', 7, 0x34, 0x12, 2, 9, 8];

    let (value, spans) = trace(bytes.as_slice(), Outer::deserialize);

    assert_eq!(
        value.unwrap(),
        Outer {
            name: "hi".into(),
            inner: Inner { a: 7, b: 0x1234 },
            list: vec![9, 8],
        }
    );
    assert_eq!(
        spans,
        vec![
            span("name", None, 0, 0, 3),
            span("inner", None, 0, 3, 6),
            span("a", None, 1, 3, 4),
            span("b", None, 1, 4, 6),
            span("list", None, 0, 6, 9),
            span("list", Some(0), 1, 7, 8),
            span("list", Some(1), 1, 8, 9),
        ]
    );
}

#[test]
fn failed_decode_keeps_spans_up_to_the_error() {
    let bytes = [2, b'h', b'i', 7];

    let (value, spans) = trace(bytes.as_slice(), Outer::deserialize);

    assert!(value.is_err());
    assert_eq!(
        spans,
        vec![
            span("name", None, 0, 0, 3),
            span("inner", None, 0, 3, 4),
            span("a", None, 1, 3, 4),
            span("b", None, 1, 4, 4),
        ]
    );
}
