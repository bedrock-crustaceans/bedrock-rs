#![allow(dead_code)]

use bedrock_protocol::ProtoVersionPackets;
use bedrock_protocol_core::ProtoCodec;
use std::io::Cursor;

type Packet<V> = <V as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket;

const ONE: [u8; 4] = [0, 0, 0x80, 0x3f];

fn round_trip<P: ProtoCodec>(bytes: &[u8]) {
    let mut cursor = Cursor::new(bytes);
    let packet = P::deserialize(&mut cursor).expect("packet decodes");
    assert_eq!(
        usize::try_from(cursor.position()).unwrap(),
        bytes.len(),
        "packet left bytes unread"
    );
    let mut encoded = Vec::new();
    packet.serialize(&mut encoded).unwrap();
    assert_eq!(encoded, bytes, "encoding differs from the input");
}

fn settings_update(weight: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1, 1, b'a', 0, 0, 0, 0, 0];
    bytes.extend(weight);
    bytes.extend([1, 0]);
    bytes
}

fn tagged_weight() -> Vec<u8> {
    let mut weight = vec![0];
    weight.extend(ONE);
    weight
}

fn float_attribute_with_only_a_min() -> Vec<u8> {
    let mut attribute = vec![1];
    attribute.extend(ONE);
    attribute.extend([3, b'a', b'd', b'd', 1, 0, 0, 0, 0, 0]);
    attribute
}

fn environment_update(trailer: &[u8]) -> Vec<u8> {
    let mut bytes = vec![2, 1, b'a', 0, 1, 1, b'b', 0];
    bytes.extend(float_attribute_with_only_a_min());
    bytes.extend([0; 10]);
    bytes.extend(trailer);
    bytes
}

#[cfg(feature = "v944")]
#[test]
fn v944_weight_is_a_tagged_f32_and_min_max_are_optional() {
    round_trip::<Packet<bedrock_protocol::V944>>(&settings_update(&tagged_weight()));
    round_trip::<Packet<bedrock_protocol::V944>>(&environment_update(&[]));
}

#[cfg(feature = "v975")]
#[test]
fn v975_weight_is_a_bare_f32() {
    round_trip::<Packet<bedrock_protocol::V975>>(&settings_update(&ONE));
    round_trip::<Packet<bedrock_protocol::V975>>(&environment_update(&[]));
}

#[cfg(feature = "v1001")]
#[test]
fn v1001_weight_is_a_bare_f32_and_min_max_are_optional() {
    round_trip::<Packet<bedrock_protocol::V1001>>(&settings_update(&ONE));
    round_trip::<Packet<bedrock_protocol::V1001>>(&environment_update(&[0, 0, 0, 0, 0]));
}

#[cfg(feature = "v2193")]
#[test]
fn v2193_environment_attribute_ends_with_a_noise_alignment() {
    round_trip::<Packet<bedrock_protocol::V2193>>(&settings_update(&ONE));
    round_trip::<Packet<bedrock_protocol::V2193>>(&environment_update(&[0, 0, 0, 0, 0, 0, 5]));
}

#[cfg(feature = "v2225")]
#[test]
fn v2225_weight_is_a_bare_f32_and_min_max_are_optional() {
    round_trip::<Packet<bedrock_protocol::V2225>>(&settings_update(&ONE));
    let mut bytes = vec![2, 1, b'a', 0, 1, 1, b'b', 0];
    bytes.extend(float_attribute_with_only_a_min());
    round_trip::<Packet<bedrock_protocol::V2225>>(&bytes);
}
