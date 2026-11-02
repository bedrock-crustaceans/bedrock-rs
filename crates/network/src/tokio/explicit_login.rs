use super::memory::{MemoryConnection, memory_connections};
use crate::codec::encode_packets;
use crate::compression::Compression;
use crate::login::{
    ClientIdentity, LoginOptions, NetworkSettings, begin_encryption, check_protocol,
};
use crate::test_helpers::{identity_key, self_signed_request_bytes};
use bedrock_auth::Login;
use bedrock_protocol::v662::enums::{PacketCompressionAlgorithm, PlayStatus};
use bedrock_protocol::v662::packets::{
    ClientToServerHandshakePacket, LoginPacket, NetworkSettingsPacket, PlayStatusPacket,
    RequestNetworkSettingsPacket, ServerToClientHandshakePacket,
};
use bedrock_protocol::{ProtoVersion, V2225};

const PROTOCOL: i32 = V2225::PROTOCOL_VERSION.cast_signed();

fn play_status(status: PlayStatus) -> V2225 {
    V2225::PlayStatusPacket(Box::new(PlayStatusPacket { status }))
}

fn compression_algorithm(settings: &NetworkSettings) -> PacketCompressionAlgorithm {
    match settings.compression {
        Compression::Zlib { .. } => PacketCompressionAlgorithm::ZLib,
        Compression::Snappy { .. } => PacketCompressionAlgorithm::Snappy,
        Compression::None => PacketCompressionAlgorithm::None,
    }
}

async fn serve(mut conn: MemoryConnection) -> Login {
    let settings = NetworkSettings::new(Compression::Zlib {
        threshold: 1,
        compression_level: 6,
    });
    let options = LoginOptions::default().require_authentication(false);

    let V2225::RequestNetworkSettingsPacket(request) = conn.recv().await.unwrap().remove(0) else {
        panic!("a client opens with a network settings request");
    };
    check_protocol(request.client_network_version, PROTOCOL).unwrap();
    conn.send(&[V2225::NetworkSettingsPacket(Box::new(
        NetworkSettingsPacket {
            compression_threshold: settings.threshold(),
            compression_algorithm: compression_algorithm(&settings),
            client_throttle_enabled: false,
            client_throttle_threshold: 0,
            client_throttle_scalar: 0.0,
        },
    ))])
    .await
    .unwrap();
    conn.enable_compression(settings.compression);

    let V2225::LoginPacket(packet) = conn.recv().await.unwrap().remove(0) else {
        panic!("the login follows the network settings");
    };
    check_protocol(packet.client_network_version, PROTOCOL).unwrap();

    let login = options.verify(&packet.connection_request).unwrap();
    let (token, encryption) = begin_encryption(&login).unwrap();
    conn.send(&[V2225::ServerToClientHandshakePacket(Box::new(
        ServerToClientHandshakePacket {
            handshake_web_token: token,
        },
    ))])
    .await
    .unwrap();
    conn.enable_encryption(encryption);

    let V2225::ClientToServerHandshakePacket(_) = conn.recv().await.unwrap().remove(0) else {
        panic!("the client answers the handshake");
    };
    conn.send(&[play_status(PlayStatus::LoginSuccess)])
        .await
        .unwrap();

    let burst = conn.recv().await.unwrap();
    assert_eq!(burst.len(), 200);
    conn.send(&vec![play_status(PlayStatus::PlayerSpawn); 200])
        .await
        .unwrap();
    assert!(conn.compression.is_some() && conn.encryption.is_some());
    login
}

async fn dial(mut conn: MemoryConnection, identity: ClientIdentity) -> Vec<u8> {
    conn.send(&[V2225::RequestNetworkSettingsPacket(Box::new(
        RequestNetworkSettingsPacket {
            client_network_version: PROTOCOL,
        },
    ))])
    .await
    .unwrap();

    let V2225::NetworkSettingsPacket(settings) = conn.recv().await.unwrap().remove(0) else {
        panic!("the server answers with network settings");
    };
    let settings = NetworkSettings::from_wire(
        settings.compression_algorithm as u16,
        settings.compression_threshold,
    )
    .unwrap();
    conn.enable_compression(settings.compression);
    conn.send(&[V2225::LoginPacket(Box::new(LoginPacket {
        client_network_version: PROTOCOL,
        connection_request: identity.request().to_vec(),
    }))])
    .await
    .unwrap();

    let V2225::ServerToClientHandshakePacket(handshake) = conn.recv().await.unwrap().remove(0)
    else {
        panic!("the server asks for encryption");
    };
    conn.enable_encryption(
        identity
            .finish_encryption(&handshake.handshake_web_token)
            .unwrap(),
    );
    conn.send(&[V2225::ClientToServerHandshakePacket(Box::new(
        ClientToServerHandshakePacket {},
    ))])
    .await
    .unwrap();

    let V2225::PlayStatusPacket(status) = conn.recv().await.unwrap().remove(0) else {
        panic!("the server confirms the login");
    };
    assert!(matches!(status.status, PlayStatus::LoginSuccess));

    conn.send(&vec![play_status(PlayStatus::PlayerSpawn); 200])
        .await
        .unwrap();
    conn.recv_raw().await.unwrap()
}

#[::tokio::test]
async fn client_and_server_log_in_explicitly_and_exchange_an_encrypted_compressed_batch() {
    let (server_conn, client_conn) = memory_connections();
    let identity = ClientIdentity::new(identity_key(), self_signed_request_bytes());

    let (login, on_the_wire) = ::tokio::join!(serve(server_conn), dial(client_conn, identity));

    let claimed = login.authentication.identity();
    assert_eq!(claimed.display_name, "Alex");
    assert_eq!(claimed.public_key().unwrap(), identity_key().public_key());
    let plain =
        encode_packets::<V2225>(&vec![play_status(PlayStatus::PlayerSpawn); 200], None, None)
            .unwrap();
    assert_ne!(on_the_wire, plain);
    assert!(on_the_wire.len() < plain.len());
}
