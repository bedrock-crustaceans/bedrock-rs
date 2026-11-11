use bedrock_protocol::ProtoVersionPackets;
use bedrock_protocol::V2225;
use bedrock_protocol::v662::enums::EasingType;
use bedrock_protocol::v924::types::{AttachInstruction, CameraInstruction, FieldOfViewInstruction};
use bedrock_protocol_core::ProtoCodec;
use std::io::Cursor;

type Packet = <V2225 as ProtoVersionPackets>::CameraInstructionPacket;

fn empty() -> CameraInstruction<V2225> {
    CameraInstruction {
        set: None,
        clear: None,
        fade: None,
        target: None,
        remove_target: None,
        field_of_view: None,
        spline: None,
        attach: None,
        detach: None,
    }
}

fn encode(instruction: CameraInstruction<V2225>) -> Vec<u8> {
    let mut bytes = Vec::new();
    Packet {
        camera_instruction: instruction,
    }
    .serialize(&mut bytes)
    .unwrap();
    bytes
}

#[test]
fn field_of_view_writes_its_easing_as_a_string() {
    let bytes = encode(CameraInstruction {
        field_of_view: Some(FieldOfViewInstruction {
            field_of_view: 70.0,
            ease_time: 2.0,
            ease_type: EasingType::InOutQuad,
            clear: false,
        }),
        ..empty()
    });

    let mut expected = vec![0, 0, 0, 0, 0, 1];
    expected.extend(70.0f32.to_le_bytes());
    expected.extend(2.0f32.to_le_bytes());
    expected.push(11);
    expected.extend(b"in_out_quad");
    expected.extend([0, 0, 0, 0]);
    assert_eq!(bytes, expected);
}

#[test]
fn remove_target_sits_between_target_and_field_of_view() {
    let bytes = encode(CameraInstruction {
        remove_target: Some(true),
        ..empty()
    });
    assert_eq!(bytes, [0, 0, 0, 0, 1, 1, 0, 0, 0, 0]);
}

#[test]
fn attach_and_detach_follow_the_field_of_view() {
    let bytes = encode(CameraInstruction {
        attach: Some(AttachInstruction { actor_unique_id: 9 }),
        detach: Some(true),
        ..empty()
    });

    let mut expected = vec![0, 0, 0, 0, 0, 0, 0, 1];
    expected.extend(9i64.to_le_bytes());
    expected.extend([1, 1]);
    assert_eq!(bytes, expected);
}

#[test]
fn the_packet_decodes_what_it_encodes() {
    let bytes = encode(CameraInstruction {
        remove_target: Some(true),
        ..empty()
    });
    let mut cursor = Cursor::new(bytes.as_slice());
    let packet = Packet::deserialize(&mut cursor).unwrap();
    assert_eq!(usize::try_from(cursor.position()).unwrap(), bytes.len());
    assert_eq!(packet.camera_instruction.remove_target, Some(true));
}
