#![cfg(feature = "packet-meta")]

use bedrock_network_core::{Packet, PacketDirection};
use bedrock_network_protocol::V2259;
use bedrock_network_protocol::v662::packets::{LoginPacket, PlayStatusPacket, TextPacket};

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
        PlayStatusPacket::<V2259>::META.direction,
        Some(PacketDirection::ServerToClient)
    );
}

#[test]
fn packet_both_sides_send_has_no_direction() {
    assert_eq!(TextPacket::<V2259>::META.direction, None);
}
