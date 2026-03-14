use crate::server::Server;
use bedrockrs_network::listener::Listener;
use bedrockrs_proto::{ProtoVersion, V924};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct ServerBuilder {
    name: String,
    sub_name: String,
    listeners_info: Vec<SocketAddr>,
    max_player: u32,
}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        Self::default()
    }

    pub fn name(mut self, name: &str) -> ServerBuilder {
        self.name = name.to_owned();
        self
    }

    pub fn sub_name(mut self, sub_name: &str) -> ServerBuilder {
        self.sub_name = sub_name.to_owned();
        self
    }

    pub fn listener(mut self, addr: SocketAddr) -> ServerBuilder {
        self.listeners_info.push(addr);
        self
    }

    pub async fn build(mut self) -> Server {
        if self.listeners_info.is_empty() {
            self.listeners_info.push(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0)),
                19132,
            ));
        }

        let mut listeners = Vec::with_capacity(self.listeners_info.len());

        for addr in self.listeners_info {
            listeners.push(
                Listener::new_raknet(
                    addr,
                    self.name.clone(),
                    self.sub_name.clone(),
                    V924::GAME_VERSION.to_string(),
                    V924::PROTOCOL_VERSION,
                    V924::RAKNET_VERSION,
                    self.max_player,
                    0,
                    false,
                )
                .await
                .unwrap(),
            )
        }

        Server {
            listeners,
            name: self.name,
            sub_name: self.sub_name,
            world: Default::default(),
        }
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self {
            name: "bedrock-server".to_string(),
            sub_name: "bedrock-rs".to_string(),
            listeners_info: vec![],
            max_player: 100,
        }
    }
}
