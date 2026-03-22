use core::net::SocketAddr;

use rak_rs::Motd;
use rak_rs::mcpe::motd::Gamemode;
use rand::random;

use crate::connection::Connection;
use crate::error::{ListenerError, RakNetError, TransportLayerError};
use crate::transport::TransportLayerListener;
use bedrockrs_proto::Unknown;
use bedrockrs_proto::info::MINECRAFT_EDITION_MOTD;

pub struct Listener {
    listener: TransportLayerListener,
    name: String,
    sub_name: String,
    player_max: u32,
    player_count: u32,
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
        rak_version: u8,
        player_max: u32,
        player_count: u32,
        nintendo_limited: bool,
    ) -> Result<Self, ListenerError> {
        let mut rak_listener = rak_rs::Listener::bind(socket_addr).await.map_err(|err| {
            ListenerError::TransportListenerError(TransportLayerError::RakNetError(
                RakNetError::ServerError(err),
            ))
        })?;

        // generate a random guid
        let guid: u64 = random::<u64>();

        // Set the RakNet version
        rak_listener.versions = Box::leak(Box::new([rak_version])); // Currently rak_rs gave an array to us, but obviously we won't use multi rak versions

        // Set up the motd
        rak_listener.motd = Motd {
            edition: String::from(MINECRAFT_EDITION_MOTD),
            version: display_version,
            name: name.clone(),
            sub_name: sub_name.clone(),
            player_max,
            player_count,
            protocol: protocol as u16,
            server_guid: guid,
            gamemode: Gamemode::Survival,
            port: Some(socket_addr.clone().port().to_string()),
            ipv6_port: Some(socket_addr.clone().port().to_string()),
            nintendo_limited: Some(nintendo_limited),
        };

        Ok(Self {
            listener: TransportLayerListener::RakNet(rak_listener),
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

    pub async fn accept(&mut self) -> Result<Connection<Unknown>, ListenerError> {
        let rak_conn = self.listener.accept().await?;

        Ok(Connection::from_transport_conn(rak_conn))
    }
}
