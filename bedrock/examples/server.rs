use bedrock::auth::Login;
use bedrock::core::world::dimension::Dimension;
use bedrock::network::compression::Compression;
use bedrock::network::login::{LoginOptions, NetworkSettings, begin_encryption, check_protocol};
use bedrock::network::tokio::Connection;
use bedrock::network::tokio::Listener;
use bedrock::network::tokio::error::ConnectionError;
use bedrock::protocol::v662::enums::PacketCompressionAlgorithm;
use bedrock::protocol::v662::enums::{
    ChatRestrictionLevel, Difficulty, EditorWorldType, GamePublishSetting, GameType, GeneratorType,
    PlayStatus, SpawnBiomeType,
};
use bedrock::protocol::v662::packets::{
    NetworkSettingsPacket, PlayStatusPacket, ServerToClientHandshakePacket,
};
use bedrock::protocol::v662::types::{
    ActorRuntimeID, ActorUniqueID, BaseGameVersion, EduSharedUriResource, Experiments,
    NetworkPermissions, SpawnSettings,
};
use bedrock::protocol::v818::types::SyncedPlayerMovementSettings;
use bedrock::protocol::v898::packets::ResourcePackStackPacket;
use bedrock::protocol::v944::packets::VoxelShapesPacket;
use bedrock::protocol::v944::types::NetworkBlockPosition;
use bedrock::protocol::v2168::enums::EducationEditionOffer;
use bedrock::protocol::v2168::packets::ResourcePacksInfoPacket;
use bedrock::protocol::{ProtoVersion, Unknown, V2225};
use bedrock_protocol::v2168::packets::StartGamePacket;
use bedrock_protocol::v2225::types::{
    EditorLevelMigrationVersion, GameRuleLegacyData, LevelSettings,
};
use std::collections::HashMap;
use tokio::time::Instant;
use uuid::Uuid;

type Protocol = V2225;

const PROTOCOL: i32 = Protocol::PROTOCOL_VERSION.cast_signed();

async fn recv_one(conn: &mut Connection<Protocol>) -> Result<Protocol, ConnectionError> {
    let batch = conn.recv().await?;
    batch
        .into_iter()
        .next()
        .ok_or(ConnectionError::ConnectionClosed)
}

fn play_status(status: PlayStatus) -> Protocol {
    Protocol::PlayStatusPacket(Box::new(PlayStatusPacket { status }))
}

async fn login(
    conn: &mut Connection<Protocol>,
) -> Result<Login, Box<dyn std::error::Error + Send + Sync>> {
    let settings = NetworkSettings::default();
    let options = LoginOptions::default().require_authentication(false);

    let Protocol::RequestNetworkSettingsPacket(request) = recv_one(conn).await? else {
        return Err("expected a network settings request".into());
    };
    if let Err(error) = check_protocol(request.client_network_version, PROTOCOL) {
        let status = if request.client_network_version < PROTOCOL {
            PlayStatus::LoginFailedClientOld
        } else {
            PlayStatus::LoginFailedServerOld
        };
        conn.send(&[play_status(status)]).await?;
        return Err(error.into());
    }

    let algorithm = match settings.compression {
        Compression::Zlib { .. } => PacketCompressionAlgorithm::ZLib,
        Compression::Snappy { .. } => PacketCompressionAlgorithm::Snappy,
        Compression::None => PacketCompressionAlgorithm::None,
    };
    conn.send(&[Protocol::NetworkSettingsPacket(Box::new(
        NetworkSettingsPacket {
            compression_threshold: settings.threshold(),
            compression_algorithm: algorithm,
            client_throttle_enabled: false,
            client_throttle_threshold: 0,
            client_throttle_scalar: 0.0,
        },
    ))])
    .await?;
    conn.enable_compression(settings.compression);

    let Protocol::LoginPacket(packet) = recv_one(conn).await? else {
        return Err("expected a login".into());
    };
    check_protocol(packet.client_network_version, PROTOCOL)?;

    let login = options.verify(&packet.connection_request)?;
    let (token, encryption) = begin_encryption(&login)?;
    conn.send(&[Protocol::ServerToClientHandshakePacket(Box::new(
        ServerToClientHandshakePacket {
            handshake_web_token: token,
        },
    ))])
    .await?;
    conn.enable_encryption(encryption);

    let Protocol::ClientToServerHandshakePacket(_) = recv_one(conn).await? else {
        return Err("expected the client handshake".into());
    };
    conn.send(&[play_status(PlayStatus::LoginSuccess)]).await?;
    Ok(login)
}

#[tokio::main]
async fn main() {
    let mut listener = Listener::builder()
        .raknet("127.0.0.1:19132".parse().unwrap())
        .name("Bedrock in Rust")
        .sub_name("bedrock-rs")
        .protocol::<Protocol>()
        .max_players(100)
        .player_count(10)
        .bind()
        .await
        .unwrap();

    loop {
        let conn = listener.accept().await.unwrap();

        tokio::spawn(async move {
            handle_login(conn).await;
        });
    }
}

fn level_settings() -> LevelSettings<Protocol> {
    LevelSettings {
        seed: 777_777_777_777,
        spawn_settings: SpawnSettings {
            spawn_type: SpawnBiomeType::Default,
            user_defined_biome_name: String::from("RandomBiome"),
            dimension: i32::from(Dimension::Overworld),
        },
        generator_type: GeneratorType::Overworld,
        game_type: GameType::Creative,
        is_hardcore_enabled: false,
        game_difficulty: Difficulty::Peaceful,
        default_spawn_block_position: NetworkBlockPosition {
            x: 100,
            y: 200,
            z: 300,
        },
        achievements_disabled: true,
        editor_world_type: EditorWorldType::NonEditor,
        is_created_in_editor: false,
        is_exported_from_editor: false,
        editor_level_migration_version: EditorLevelMigrationVersion::Legacy,
        day_cycle_stop_time: 2000,
        education_edition_offer: EducationEditionOffer::None,
        education_features_enabled: false,
        education_product_id: String::new(),
        rain_level: 300.0,
        lightning_level: 400.0,
        has_confirmed_platform_locked_content: false,
        multiplayer_enabled: true,
        lan_broadcasting_enabled: true,
        xbox_live_broadcast_setting: GamePublishSetting::FriendsOnly,
        platform_broadcast_setting: GamePublishSetting::FriendsOnly,
        commands_enabled: true,
        texture_packs_required: false,
        rule_data: GameRuleLegacyData { rules_list: vec![] },
        experiments: Experiments {
            experiments: vec![],
            ever_toggled: false,
        },
        bonus_chest_enabled: false,
        starting_map_enabled: false,
        player_permissions: 3,
        server_chunk_tick_range: 4,
        locked_behaviour_pack: false,
        locked_resource_pack: false,
        from_locked_template: false,
        use_msa_gamer_tags: false,
        from_template: false,
        has_locked_template_settings: false,
        only_spawn_v1_villagers: false,
        persona_disabled: false,
        custom_skins_disabled: false,
        emote_chat_muted: false,
        base_game_version: BaseGameVersion(Protocol::GAME_VERSION.to_string()),
        limited_world_width: 16,
        limited_world_depth: 16,
        nether_type: true,
        edu_shared_uri_resource: EduSharedUriResource {
            button_name: String::new(),
            link_uri: String::new(),
        },
        override_force_experimental_gameplay: Some(true),
        chat_restriction_level: ChatRestrictionLevel::None,
        disable_player_interactions: false,
        server_editor_connection_policy: 0,
        allow_anonymous_block_drops_in_editor_worlds: false,
    }
}

fn start_game_packet() -> StartGamePacket<Protocol> {
    StartGamePacket {
        target_actor_id: ActorUniqueID(609),
        target_runtime_id: ActorRuntimeID(402),
        actor_game_type: GameType::Creative,
        position: Default::default(),
        rotation: Default::default(),
        settings: level_settings(),
        level_id: String::from("UmFuZG9tIFdvcmxk"),
        level_name: String::from("Random World"),
        template_content_identity: String::new(),
        is_trial: false,
        movement_settings: SyncedPlayerMovementSettings {
            rewind_history_size: 3200,
            server_authoritative_block_breaking: false,
        },
        current_level_time: 9000,
        enchantment_seed: 99000,
        block_properties: vec![],
        multiplayer_correlation_id: String::from("c5d3d2cc-27fd-4221-9de6-d22c4d423d53"),
        enable_item_stack_net_manager: false,
        server_version: Protocol::GAME_VERSION.to_string(),
        player_property_data: HashMap::new(),
        server_block_type_registry_checksum: 0,
        world_template_id: Uuid::nil(),
        server_enabled_client_side_generation: false,
        block_network_ids_are_hashes: false,
        network_permissions: NetworkPermissions {
            server_auth_sound_enabled: false,
        },
        server_join_information: None,
        server_id: String::new(),
        world_id: String::new(),
        scenario_id: String::new(),
        owner_id: String::new(),
    }
}

async fn handle_login(unknown_conn: Connection<Unknown>) {
    let time_start = Instant::now();

    let mut conn = unknown_conn.into_ver::<Protocol>();
    let login = match login(&mut conn).await {
        Ok(login) => login,
        Err(error) => {
            println!("Login failed: {error}");
            conn.close().await;
            return;
        }
    };
    println!("Login: {}", login.authentication.identity().display_name);

    conn.send(&[
        Protocol::ResourcePacksInfoPacket(Box::new(ResourcePacksInfoPacket {
            resource_pack_required: false,
            has_addon_packs: false,
            has_scripts: false,
            force_disable_vibrant_visuals: false,
            world_template_uuid: Uuid::nil(),
            resource_packs: vec![],
            world_template_version: String::new(),
        })),
        Protocol::ResourcePackStackPacket(Box::new(ResourcePackStackPacket {
            texture_pack_required: false,
            addon_list: vec![],
            base_game_version: BaseGameVersion(String::from("1.0")),
            experiments: Experiments {
                experiments: vec![],
                ever_toggled: false,
            },
            include_editor_packs: false,
        })),
    ])
    .await
    .unwrap();
    println!("ResourcePacksInfo");
    println!("ResourcePackStack");

    println!("{:#?}", conn.recv().await.unwrap());
    println!("ClientCacheStatus");
    println!("{:#?}", conn.recv().await.unwrap());
    println!("ResourcePackClientResponse");

    conn.send(&[Protocol::VoxelShapesPacket(Box::new(VoxelShapesPacket {
        shapes: vec![],
        names: vec![],
        custom_shape_count: 0,
    }))])
    .await
    .unwrap();
    println!("VoxelShapes");

    conn.send(&[Protocol::StartGamePacket(Box::new(start_game_packet()))])
        .await
        .unwrap();
    println!("StartGame");

    conn.send(&[Protocol::PlayStatusPacket(Box::new(PlayStatusPacket {
        status: PlayStatus::PlayerSpawn,
    }))])
    .await
    .unwrap();
    println!("PlayStatusPacket (PlayerSpawn)");

    println!("Finished request in {:?}", time_start.elapsed());

    loop {
        let res = conn.recv().await;

        if let Ok(packet) = res {
            println!("Found packet: {packet:?}");
        } else {
            break;
        }
    }
}
