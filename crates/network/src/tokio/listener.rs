use core::net::SocketAddr;

use rand::random;

use super::connection::Connection;
use super::error::ListenerError;
use super::transport::TransportLayerListener;
use crate::info::MINECRAFT_EDITION_MOTD;
use bedrock_protocol::ProtoVersion;
use bedrock_protocol_core::Packets;

use crate::motd::BedrockMOTD;
use raknet_tokio::prelude::*;

pub struct Listener {
    listener: TransportLayerListener,
    motd: BedrockMOTD,
    max_connections: usize,
}

const DEFAULT_RAKNET_VERSION: u8 = 11;

pub struct ListenerBuilder {
    addr: SocketAddr,
    name: String,
    sub_name: String,
    display_version: String,
    protocol: u32,
    rak_version: u8,
    max_players: i32,
    player_count: i32,
    nintendo_limited: bool,
}

impl ListenerBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn sub_name(mut self, sub_name: impl Into<String>) -> Self {
        self.sub_name = sub_name.into();
        self
    }

    pub fn max_players(mut self, max_players: i32) -> Self {
        self.max_players = max_players;
        self
    }

    pub fn player_count(mut self, player_count: i32) -> Self {
        self.player_count = player_count;
        self
    }

    pub fn nintendo_limited(mut self, limited: bool) -> Self {
        self.nintendo_limited = limited;
        self
    }

    pub fn protocol<V: ProtoVersion>(mut self) -> Self {
        self.display_version = V::GAME_VERSION.to_owned();
        self.protocol = V::PROTOCOL_VERSION;
        self.rak_version = V::RAKNET_VERSION;
        self
    }

    fn build_motd(&self, guid: u64) -> BedrockMOTD {
        BedrockMOTD {
            edition: MINECRAFT_EDITION_MOTD.to_owned(),
            version: self.display_version.clone(),
            name: self.name.clone(),
            sub_name: self.sub_name.clone(),
            player_max: self.max_players,
            player_count: self.player_count,
            protocol: self.protocol,
            guid,
            game_mode: "Survival".to_string(),
            port_v4: Some(self.addr.port()),
            port_v6: Some(self.addr.port()),
            nintendo_limited: Some(self.nintendo_limited),
        }
    }

    fn into_listener(self) -> Listener {
        let guid = random::<u64>();
        let motd = self.build_motd(guid);
        let max_connections = usize::try_from(self.max_players).unwrap_or(0);
        let rak_version = self.rak_version;

        let rak_server = RakServer::new(self.addr, |conf| {
            conf.guid = guid;
            conf.max_connections = max_connections;
            conf.protocols = Box::new([rak_version]);
            conf.message = (&motd).into()
        });

        Listener {
            listener: TransportLayerListener::RakNet(rak_server),
            motd,
            max_connections,
        }
    }

    pub async fn bind(self) -> Result<Listener, ListenerError> {
        let mut listener = self.into_listener();
        listener.listener.start().await?;
        Ok(listener)
    }
}

impl Listener {
    pub fn builder(addr: SocketAddr) -> ListenerBuilder {
        ListenerBuilder {
            addr,
            name: String::new(),
            sub_name: String::new(),
            display_version: String::new(),
            protocol: 0,
            rak_version: DEFAULT_RAKNET_VERSION,
            max_players: 10,
            player_count: 0,
            nintendo_limited: false,
        }
    }

    pub fn motd(&self) -> &BedrockMOTD {
        &self.motd
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    pub fn set_max_connections(&mut self, n: usize) {
        self.max_connections = n;
        self.listener.set_max_connections(n);
    }

    pub fn update_motd(&mut self, f: impl FnOnce(&mut BedrockMOTD)) {
        f(&mut self.motd);
        self.listener.set_message((&self.motd).into());
    }

    pub async fn shutdown(mut self) -> Result<(), ListenerError> {
        self.listener.stop().await?;
        Ok(())
    }

    pub async fn accept<V: Packets>(&mut self) -> Result<Connection<V>, ListenerError> {
        let rak_conn = self.listener.accept().await?;

        Ok(Connection::from_transport_conn(rak_conn))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_protocol::V2225;

    fn unbound_builder() -> ListenerBuilder {
        Listener::builder("127.0.0.1:19132".parse().unwrap())
            .name("Before")
            .sub_name("Sub")
            .protocol::<V2225>()
            .max_players(20)
            .player_count(3)
            .nintendo_limited(true)
    }

    fn unbound_listener() -> Listener {
        unbound_builder().into_listener()
    }

    #[test]
    fn builder_sets_the_motd_fields() {
        let motd = unbound_builder().build_motd(7);

        assert_eq!(motd.name, "Before");
        assert_eq!(motd.sub_name, "Sub");
        assert_eq!(motd.version, V2225::GAME_VERSION);
        assert_eq!(motd.protocol, V2225::PROTOCOL_VERSION);
        assert_eq!((motd.player_max, motd.player_count), (20, 3));
        assert_eq!(motd.nintendo_limited, Some(true));
        assert_eq!(motd.port_v4, Some(19132));
        assert_eq!(motd.guid, 7);
    }

    #[::tokio::test]
    async fn update_motd_changes_the_advertised_bytes() {
        let mut listener = unbound_listener();
        let before: Box<[u8]> = listener.motd().into();

        listener.update_motd(|motd| motd.name = "After".into());

        let after: Box<[u8]> = listener.motd().into();
        assert_ne!(before, after);
        assert!(
            String::from_utf8(after.into_vec())
                .unwrap()
                .contains("After")
        );
    }

    #[::tokio::test]
    async fn listener_reports_configured_max_connections() {
        let mut listener = unbound_listener();
        assert_eq!(listener.max_connections(), 20);

        listener.set_max_connections(50);
        assert_eq!(listener.max_connections(), 50);
    }
}
