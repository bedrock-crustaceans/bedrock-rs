use bedrock::auth::ClientData;
use bedrock::network::login::{ClientIdentity, NetworkSettings};
use bedrock::network::tokio::{Client, Connection};
use bedrock::protocol::v662::enums::PlayStatus;
use bedrock::protocol::v662::packets::{
    ClientToServerHandshakePacket, LoginPacket, RequestNetworkSettingsPacket,
};
use bedrock::protocol::{ProtoVersion, V2225};
use std::net::SocketAddr;
use std::time::Duration;

type Protocol = V2225;

const PROTOCOL: i32 = Protocol::PROTOCOL_VERSION.cast_signed();

async fn recv_one(conn: &mut Connection<Protocol>) -> Result<Protocol, Box<dyn std::error::Error>> {
    let batch = conn.recv().await?;
    batch
        .into_iter()
        .next()
        .ok_or_else(|| "connection closed".into())
}

async fn login(
    conn: &mut Connection<Protocol>,
    identity: &ClientIdentity,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.send(&[Protocol::RequestNetworkSettingsPacket(Box::new(
        RequestNetworkSettingsPacket {
            client_network_version: PROTOCOL,
        },
    ))])
    .await?;

    let Protocol::NetworkSettingsPacket(settings) = recv_one(conn).await? else {
        return Err("expected network settings".into());
    };
    let settings = NetworkSettings::from_wire(
        settings.compression_algorithm as u16,
        settings.compression_threshold,
    )?;
    conn.enable_compression(settings.compression);

    conn.send(&[Protocol::LoginPacket(Box::new(LoginPacket {
        client_network_version: PROTOCOL,
        connection_request: identity.request().to_vec(),
    }))])
    .await?;

    loop {
        match recv_one(conn).await? {
            Protocol::ServerToClientHandshakePacket(handshake) => {
                conn.enable_encryption(identity.finish_encryption(&handshake.handshake_web_token)?);
                conn.send(&[Protocol::ClientToServerHandshakePacket(Box::new(
                    ClientToServerHandshakePacket {},
                ))])
                .await?;
            }
            Protocol::PlayStatusPacket(packet) => match packet.status {
                PlayStatus::LoginSuccess => return Ok(()),
                status => return Err(format!("login rejected: {status:?}").into()),
            },
            Protocol::DisconnectPacket(packet) => {
                return Err(format!("disconnected: {:?}", packet.message).into());
            }
            other => return Err(format!("unexpected packet during login: {other:?}").into()),
        }
    }
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:19132".to_string())
        .parse()
        .expect("address must look like 127.0.0.1:19132");

    let client_data = ClientData::offline(Protocol::GAME_VERSION, &addr.to_string(), "bedrock-rs");
    let identity =
        ClientIdentity::offline(ClientIdentity::generate_key(), "bedrock-rs", &client_data)
            .expect("the offline identity signs");

    let mut conn = match Client::new()
        .timeout(Duration::from_secs(30))
        .connect::<Protocol>(addr)
        .await
    {
        Ok(conn) => conn,
        Err(error) => {
            println!("Connecting failed: {error}");
            return;
        }
    };
    if let Err(error) = login(&mut conn, &identity).await {
        println!("Login failed: {error}");
        conn.close().await;
        return;
    }
    println!("Logged in to {addr}");

    match conn.recv().await {
        Ok(packets) => println!("{packets:#?}"),
        Err(error) => println!("Connection ended: {error}"),
    }
    conn.close().await;
}
