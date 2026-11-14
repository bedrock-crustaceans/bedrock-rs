use bedrock_macros::ProtoCodec;
use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::{Cursor, ErrorKind};

#[derive(ProtoCodec, Debug)]
struct WideItems {
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    #[endianness(le)]
    items: Vec<[u64; 64]>,
}

#[test]
fn derived_vec_with_huge_length_prefix_and_no_items_is_an_eof_error() {
    let result = WideItems::deserialize(&mut Cursor::new([0xff, 0xff, 0xff, 0xff]));
    assert!(
        matches!(result, Err(ProtoCodecError::IOError(ref e)) if e.kind() == ErrorKind::UnexpectedEof),
        "expected EOF, got {result:?}"
    );
}
