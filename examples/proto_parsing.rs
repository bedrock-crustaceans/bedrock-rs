mod common;

use crate::common::logger::setup_logger;
use bedrockrs_proto::codec::{decode_packets, encode_packets};
use bedrockrs_proto::v662::packets::LoginPacket;
use bedrockrs_proto::V800;
use bedrockrs_proto::{ProtoVersion, ProtoVersionPackets};
use bedrockrs_proto_core::{Packets, ProtoCodec};

fn main() {
    setup_logger().unwrap();

    let login = V800::LoginPacket(LoginPacket {
        client_network_version: 800,
        connection_request: "{ \"chain\": [\"feels like something is missing here...\"] }"
            .to_string(),
    });

    let bytes = match encode_packets(&[login], None, None) {
        Ok(bytes) => bytes,
        Err(error) => {
            println!("Failed to encode packets: {:?}", error);
            return;
        }
    };

    let result = decode_packets::<V800>(bytes, None, None);

    println!("{:?}", result);
}
