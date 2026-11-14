#![cfg(feature = "packet-meta")]

use bedrock_protocol::V2225;
use bedrock_protocol::v662::packets::{LoginPacket, PlayStatusPacket, TextPacket};
use bedrock_protocol_core::{Packet, PacketDirection};

#[test]
fn client_only_packet_is_client_to_server() {
    assert_eq!(
        LoginPacket::META.direction,
        Some(PacketDirection::ClientToServer)
    );
}

#[test]
fn server_only_packet_is_server_to_client() {
    assert_eq!(
        PlayStatusPacket::<V2225>::META.direction,
        Some(PacketDirection::ServerToClient)
    );
}

#[test]
fn packet_both_sides_send_has_no_direction() {
    assert_eq!(TextPacket::<V2225>::META.direction, None);
}
