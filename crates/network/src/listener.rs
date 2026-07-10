use core::net::SocketAddr;

use rand::random;

use crate::connection::Connection;
use crate::error::ListenerError;
use crate::info::MINECRAFT_EDITION_MOTD;
use crate::transport::TransportLayerListener;
use bedrock_protocol_core::Packets;

use crate::motd::BedrockMOTD;
use raknet_tokio::prelude::*;

#[allow(dead_code)]
pub struct Listener {
    listener: TransportLayerListener,
    name: String,
    sub_name: String,
    player_max: i32,
    player_count: i32,
    socket_addr: SocketAddr,
    guid: u64,
}

impl Listener {
    #[allow(clippy::too_many_arguments)]
    pub async fn new_raknet(
        socket_addr: SocketAddr,
        name: String,
        sub_name: String,
        display_version: String,
        protocol: u32,
        // TODO: expose in raknet crate
        rak_version: u8,
        player_max: i32,
        player_count: i32,
        nintendo_limited: bool,
    ) -> Result<Self, ListenerError> {
        // generate a random guid
        let guid: u64 = random::<u64>();

        let rak_server = RakServer::new(socket_addr, |conf| {
            conf.guid = guid;
            conf.protocols = Box::new([rak_version]);
            conf.message = BedrockMOTD {
                edition: MINECRAFT_EDITION_MOTD.to_owned(),
                version: display_version,
                name: name.clone(),
                sub_name: sub_name.clone(),
                player_max,
                player_count,
                protocol,
                guid,
                game_mode: "Survival".to_string(),
                port_v4: Some(socket_addr.port()),
                port_v6: Some(socket_addr.port()),
                nintendo_limited: Some(nintendo_limited),
            }
            .into()
        });

        Ok(Self {
            listener: TransportLayerListener::RakNet(rak_server),
            name,
            sub_name,
            player_max,
            player_count,
            socket_addr,
            guid,
        })
    }

    pub async fn start(&mut self) -> Result<(), ListenerError> {
        self.listener.start().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), ListenerError> {
        self.listener.stop().await?;
        Ok(())
    }

    pub async fn accept<V: Packets>(&mut self) -> Result<Connection<V>, ListenerError> {
        let rak_conn = self.listener.accept().await?;

        Ok(Connection::from_transport_conn(rak_conn))
    }
}
