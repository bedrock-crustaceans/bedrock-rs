use crate::compression::Compression;
use bedrock_protocol_core::Packets;

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkSettings {
    pub compression: Compression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginStatus {
    LoginSuccess,
    LoginFailedClientOld,
    LoginFailedServerOld,
    PlayerSpawn,
    LoginFailedInvalidTenant,
    LoginFailedEditionMismatchEduToVanilla,
    LoginFailedEditionMismatchVanillaToEdu,
    LoginFailedServerFullSubClient,
    LoginFailedEditorMismatchEditorToVanilla,
    LoginFailedEditorMismatchVanillaToEditor,
}

impl LoginStatus {
    pub fn is_failure(&self) -> bool {
        match self {
            Self::LoginSuccess | Self::PlayerSpawn => false,
            Self::LoginFailedClientOld
            | Self::LoginFailedServerOld
            | Self::LoginFailedInvalidTenant
            | Self::LoginFailedEditionMismatchEduToVanilla
            | Self::LoginFailedEditionMismatchVanillaToEdu
            | Self::LoginFailedServerFullSubClient
            | Self::LoginFailedEditorMismatchEditorToVanilla
            | Self::LoginFailedEditorMismatchVanillaToEditor => true,
        }
    }
}

impl std::fmt::Display for LoginStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LoginSuccess => "login success",
            Self::LoginFailedClientOld => "client is out of date",
            Self::LoginFailedServerOld => "server is out of date",
            Self::PlayerSpawn => "player spawn",
            Self::LoginFailedInvalidTenant => "invalid tenant",
            Self::LoginFailedEditionMismatchEduToVanilla => "education client on a vanilla server",
            Self::LoginFailedEditionMismatchVanillaToEdu => "vanilla client on an education server",
            Self::LoginFailedServerFullSubClient => "server is full for the sub client",
            Self::LoginFailedEditorMismatchEditorToVanilla => "editor client on a vanilla server",
            Self::LoginFailedEditorMismatchVanillaToEditor => "vanilla client on an editor server",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisconnectInfo {
    pub message: Option<String>,
}

impl std::fmt::Display for DisconnectInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or("no message"))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoginEvent<V> {
    RequestNetworkSettings { protocol: i32 },
    NetworkSettings(NetworkSettings),
    Login { protocol: i32, request: Vec<u8> },
    ServerHandshake(String),
    ClientHandshake,
    PlayStatus(LoginStatus),
    Disconnect(DisconnectInfo),
    Other(V),
}

pub trait LoginPackets: Packets + Clone + Send + 'static {
    const PROTOCOL_VERSION: i32;
    const RAKNET_VERSION: u8;

    fn request_network_settings(protocol: i32) -> Self;
    fn network_settings(settings: &NetworkSettings) -> Self;
    fn login(protocol: i32, request: Vec<u8>) -> Self;
    fn server_handshake(jwt: String) -> Self;
    fn client_handshake() -> Self;
    fn play_status(status: LoginStatus) -> Self;
    fn disconnect(info: &DisconnectInfo) -> Self;
    fn into_event(self) -> LoginEvent<Self>;
}

macro_rules! impl_login_packets {
    ($version:ident, $reason:path) => {
        impl LoginPackets for bedrock_protocol::$version {
            const PROTOCOL_VERSION: i32 =
                <bedrock_protocol::$version as bedrock_protocol::ProtoVersion>::PROTOCOL_VERSION
                    as i32;
            const RAKNET_VERSION: u8 =
                <bedrock_protocol::$version as bedrock_protocol::ProtoVersion>::RAKNET_VERSION;

            fn request_network_settings(protocol: i32) -> Self {
                Self::RequestNetworkSettingsPacket(Box::new(
                    bedrock_protocol::v662::packets::RequestNetworkSettingsPacket {
                        client_network_version: protocol,
                    },
                ))
            }

            fn network_settings(settings: &NetworkSettings) -> Self {
                use bedrock_protocol::v662::enums::PacketCompressionAlgorithm as Algorithm;
                let compression_algorithm = match settings.compression {
                    Compression::Zlib { .. } => Algorithm::ZLib,
                    Compression::Snappy { .. } => Algorithm::Snappy,
                    Compression::None => Algorithm::None,
                };
                Self::NetworkSettingsPacket(Box::new(
                    bedrock_protocol::v662::packets::NetworkSettingsPacket {
                        compression_threshold: settings.compression.threshold(),
                        compression_algorithm,
                        client_throttle_enabled: false,
                        client_throttle_threshold: 0,
                        client_throttle_scalar: 0.0,
                    },
                ))
            }

            fn login(protocol: i32, request: Vec<u8>) -> Self {
                Self::LoginPacket(Box::new(bedrock_protocol::v662::packets::LoginPacket {
                    client_network_version: protocol,
                    connection_request: request,
                }))
            }

            fn server_handshake(jwt: String) -> Self {
                Self::ServerToClientHandshakePacket(Box::new(
                    bedrock_protocol::v662::packets::ServerToClientHandshakePacket {
                        handshake_web_token: jwt,
                    },
                ))
            }

            fn client_handshake() -> Self {
                Self::ClientToServerHandshakePacket(Box::new(
                    bedrock_protocol::v662::packets::ClientToServerHandshakePacket {},
                ))
            }

            fn play_status(status: LoginStatus) -> Self {
                use bedrock_protocol::v662::enums::PlayStatus;
                let status = match status {
                    LoginStatus::LoginSuccess => PlayStatus::LoginSuccess,
                    LoginStatus::LoginFailedClientOld => PlayStatus::LoginFailedClientOld,
                    LoginStatus::LoginFailedServerOld => PlayStatus::LoginFailedServerOld,
                    LoginStatus::PlayerSpawn => PlayStatus::PlayerSpawn,
                    LoginStatus::LoginFailedInvalidTenant => PlayStatus::LoginFailedInvalidTenant,
                    LoginStatus::LoginFailedEditionMismatchEduToVanilla => {
                        PlayStatus::LoginFailedEditionMismatchEduToVanilla
                    }
                    LoginStatus::LoginFailedEditionMismatchVanillaToEdu => {
                        PlayStatus::LoginFailedEditionMismatchVanillaToEdu
                    }
                    LoginStatus::LoginFailedServerFullSubClient => {
                        PlayStatus::LoginFailedServerFullSubClient
                    }
                    LoginStatus::LoginFailedEditorMismatchEditorToVanilla => {
                        PlayStatus::LoginFailedEditorMismatchEditorToVanilla
                    }
                    LoginStatus::LoginFailedEditorMismatchVanillaToEditor => {
                        PlayStatus::LoginFailedEditorMismatchVanillaToEditor
                    }
                };
                Self::PlayStatusPacket(Box::new(
                    bedrock_protocol::v662::packets::PlayStatusPacket { status },
                ))
            }

            fn disconnect(info: &DisconnectInfo) -> Self {
                use bedrock_protocol::v712::packets::{DisconnectMessage, DisconnectPacket};
                Self::DisconnectPacket(Box::new(DisconnectPacket {
                    reason: <$reason>::Unknown,
                    message: info.message.as_ref().map(|message| DisconnectMessage {
                        kick_message: message.clone(),
                        filtered_message: message.clone(),
                    }),
                }))
            }

            fn into_event(self) -> LoginEvent<Self> {
                use bedrock_protocol::v662::enums::{PacketCompressionAlgorithm, PlayStatus};
                match self {
                    Self::RequestNetworkSettingsPacket(packet) => {
                        LoginEvent::RequestNetworkSettings {
                            protocol: packet.client_network_version,
                        }
                    }
                    Self::NetworkSettingsPacket(packet) => {
                        let threshold = packet.compression_threshold;
                        let compression = match packet.compression_algorithm {
                            PacketCompressionAlgorithm::ZLib => Compression::Zlib {
                                threshold,
                                compression_level: 6,
                            },
                            PacketCompressionAlgorithm::Snappy => Compression::Snappy { threshold },
                            PacketCompressionAlgorithm::None => Compression::None,
                        };
                        LoginEvent::NetworkSettings(NetworkSettings { compression })
                    }
                    Self::LoginPacket(packet) => LoginEvent::Login {
                        protocol: packet.client_network_version,
                        request: packet.connection_request,
                    },
                    Self::ServerToClientHandshakePacket(packet) => {
                        LoginEvent::ServerHandshake(packet.handshake_web_token)
                    }
                    Self::ClientToServerHandshakePacket(_) => LoginEvent::ClientHandshake,
                    Self::PlayStatusPacket(packet) => LoginEvent::PlayStatus(match packet.status {
                        PlayStatus::LoginSuccess => LoginStatus::LoginSuccess,
                        PlayStatus::LoginFailedClientOld => LoginStatus::LoginFailedClientOld,
                        PlayStatus::LoginFailedServerOld => LoginStatus::LoginFailedServerOld,
                        PlayStatus::PlayerSpawn => LoginStatus::PlayerSpawn,
                        PlayStatus::LoginFailedInvalidTenant => {
                            LoginStatus::LoginFailedInvalidTenant
                        }
                        PlayStatus::LoginFailedEditionMismatchEduToVanilla => {
                            LoginStatus::LoginFailedEditionMismatchEduToVanilla
                        }
                        PlayStatus::LoginFailedEditionMismatchVanillaToEdu => {
                            LoginStatus::LoginFailedEditionMismatchVanillaToEdu
                        }
                        PlayStatus::LoginFailedServerFullSubClient => {
                            LoginStatus::LoginFailedServerFullSubClient
                        }
                        PlayStatus::LoginFailedEditorMismatchEditorToVanilla => {
                            LoginStatus::LoginFailedEditorMismatchEditorToVanilla
                        }
                        PlayStatus::LoginFailedEditorMismatchVanillaToEditor => {
                            LoginStatus::LoginFailedEditorMismatchVanillaToEditor
                        }
                    }),
                    Self::DisconnectPacket(packet) => LoginEvent::Disconnect(DisconnectInfo {
                        message: packet.message.map(|message| message.kick_message),
                    }),
                    other => LoginEvent::Other(other),
                }
            }
        }
    };
}

impl_login_packets!(V2225, bedrock_protocol::v2225::enums::ConnectionFailReason);

#[cfg(feature = "v2193")]
impl_login_packets!(V2193, bedrock_protocol::v2193::enums::ConnectionFailReason);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode_packets;
    use bedrock_protocol::V2225;

    fn erased(event: LoginEvent<V2225>) -> LoginEvent<()> {
        match event {
            LoginEvent::RequestNetworkSettings { protocol } => {
                LoginEvent::RequestNetworkSettings { protocol }
            }
            LoginEvent::NetworkSettings(settings) => LoginEvent::NetworkSettings(settings),
            LoginEvent::Login { protocol, request } => LoginEvent::Login { protocol, request },
            LoginEvent::ServerHandshake(jwt) => LoginEvent::ServerHandshake(jwt),
            LoginEvent::ClientHandshake => LoginEvent::ClientHandshake,
            LoginEvent::PlayStatus(status) => LoginEvent::PlayStatus(status),
            LoginEvent::Disconnect(info) => LoginEvent::Disconnect(info),
            LoginEvent::Other(_) => LoginEvent::Other(()),
        }
    }

    #[test]
    fn v2225_login_events_round_trip() {
        let settings = NetworkSettings {
            compression: Compression::Snappy { threshold: 256 },
        };
        let zlib_settings = NetworkSettings {
            compression: Compression::Zlib {
                threshold: 1,
                compression_level: 6,
            },
        };
        let cases = [
            (
                V2225::request_network_settings(2225),
                LoginEvent::RequestNetworkSettings { protocol: 2225 },
            ),
            (
                V2225::network_settings(&settings),
                LoginEvent::NetworkSettings(settings),
            ),
            (
                V2225::network_settings(&zlib_settings),
                LoginEvent::NetworkSettings(zlib_settings),
            ),
            (
                V2225::network_settings(&NetworkSettings {
                    compression: Compression::None,
                }),
                LoginEvent::NetworkSettings(NetworkSettings {
                    compression: Compression::None,
                }),
            ),
            (
                V2225::login(2225, vec![1, 2, 3]),
                LoginEvent::Login {
                    protocol: 2225,
                    request: vec![1, 2, 3],
                },
            ),
            (
                V2225::server_handshake("a.b.c".to_string()),
                LoginEvent::ServerHandshake("a.b.c".to_string()),
            ),
            (V2225::client_handshake(), LoginEvent::ClientHandshake),
            (
                V2225::play_status(LoginStatus::LoginFailedEditorMismatchVanillaToEditor),
                LoginEvent::PlayStatus(LoginStatus::LoginFailedEditorMismatchVanillaToEditor),
            ),
            (
                V2225::play_status(LoginStatus::PlayerSpawn),
                LoginEvent::PlayStatus(LoginStatus::PlayerSpawn),
            ),
            (
                V2225::disconnect(&DisconnectInfo {
                    message: Some("bye".to_string()),
                }),
                LoginEvent::Disconnect(DisconnectInfo {
                    message: Some("bye".to_string()),
                }),
            ),
            (
                V2225::disconnect(&DisconnectInfo { message: None }),
                LoginEvent::Disconnect(DisconnectInfo { message: None }),
            ),
        ];

        for (packet, expected) in cases {
            assert_eq!(erased(packet.into_event()), expected);
        }
    }

    #[test]
    fn v2225_exposes_protocol_and_raknet_versions() {
        let packet = V2225::PlayStatusPacket(Box::new(
            bedrock_protocol::v662::packets::PlayStatusPacket {
                status: bedrock_protocol::v662::enums::PlayStatus::LoginSuccess,
            },
        ));
        assert_eq!(V2225::PROTOCOL_VERSION, 2225);
        assert_eq!(V2225::RAKNET_VERSION, 11);
        assert!(matches!(
            erased(packet.into_event()),
            LoginEvent::PlayStatus(LoginStatus::LoginSuccess)
        ));
    }

    #[test]
    fn v2225_network_settings_encodes_like_a_capture() {
        let batch = [
            0x0c, 0x8f, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let zlib = Compression::Zlib {
            threshold: 1,
            compression_level: 6,
        };
        let packet = V2225::network_settings(&NetworkSettings {
            compression: zlib.clone(),
        });

        let wire =
            encode_packets::<V2225>(std::slice::from_ref(&packet), Some(&zlib), None).unwrap();
        assert_eq!(wire[0], 0x00);
        assert_eq!(zlib.decompress(wire, 1024).unwrap(), batch);

        let uncompressed = Compression::Zlib {
            threshold: 64,
            compression_level: 6,
        };
        let wire = encode_packets::<V2225>(&[packet], Some(&uncompressed), None).unwrap();
        assert_eq!(wire[0], 0xff);
        assert_eq!(wire[1..], batch);
    }
}
