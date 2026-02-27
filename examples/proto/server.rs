use bedrockrs::proto::connection::Connection;
use bedrockrs::proto::listener::Listener;
use bedrockrs_proto::compression::Compression;
use bedrockrs_proto::v800;
use bedrockrs_proto::v800::enums::{PacketCompressionAlgorithm, PlayStatus};
use bedrockrs_proto::v800::packets::{
    NetworkSettingsPacket, PlayStatusPacket, ResourcePackStackPacket, ResourcePacksInfoPacket,
};
use bedrockrs_proto::v800::types::{BaseGameVersion, Experiments};
use bedrockrs_proto::v800::GamePackets;
use bedrockrs_proto::v800::ProtoHelperV800;
use tokio::time::Instant;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let mut listener = Listener::new_raknet(
        "Bedrock in Rust".to_string(),
        "bedrockrs".to_string(),
        v800::info::GAME_VERSION.to_string(),
        v800::info::PROTOCOL_VERSION,
        100,
        10,
        "127.0.0.1:19132".parse().unwrap(),
        false,
    )
    .await
    .unwrap();

    listener.start().await.unwrap();

    loop {
        let conn = listener.accept().await.unwrap();

        tokio::spawn(async move {
            handle_login(conn).await;
        });
    }
}

async fn handle_login(mut conn: Connection) {
    let time_start = Instant::now();

    // NetworkSettingsRequest
    conn.recv::<ProtoHelperV800>().await.unwrap();
    println!("NetworkSettingsRequest");

    let compression = Compression::None;

    // NetworkSettings
    conn.send::<ProtoHelperV800>(&[GamePackets::NetworkSettings(NetworkSettingsPacket {
        compression_threshold: 1,
        compression_algorithm: PacketCompressionAlgorithm::None,
        client_throttle_enabled: false,
        client_throttle_threshold: 0,
        client_throttle_scalar: 0.0,
    })])
    .await
    .unwrap();
    println!("NetworkSettings");

    conn.compression = Some(compression);

    // Login
    conn.recv::<ProtoHelperV800>().await.unwrap();
    println!("Login");

    conn.send::<ProtoHelperV800>(&[
        GamePackets::PlayStatus(PlayStatusPacket {
            status: PlayStatus::LoginSuccess,
        }),
        GamePackets::ResourcePacksInfo(ResourcePacksInfoPacket {
            resource_pack_required: false,
            has_addon_packs: false,
            has_scripts: false,
            world_template_uuid: Uuid::nil(),
            resource_packs: vec![],
            world_template_version: "".to_string(),
        }),
        GamePackets::ResourcePackStack(ResourcePackStackPacket {
            texture_pack_required: false,
            addon_list: vec![],
            base_game_version: BaseGameVersion(String::from("1.0")),
            experiments: Experiments {
                experiments: vec![],
                ever_toggled: false,
            },
            texture_pack_list: vec![],
            include_editor_packs: false,
        }),
    ])
    .await
    .unwrap();
    println!("PlayStatus (LoginSuccess)");
    println!("ResourcePacksInfo");
    println!("ResourcePackStack");

    println!("{:#?}", conn.recv::<ProtoHelperV800>().await.unwrap());
    println!("ClientCacheStatus");
    println!("{:#?}", conn.recv::<ProtoHelperV800>().await.unwrap());
    println!("ResourcePackClientResponse");

    // conn.send::<ProtoHelperV729>(&[GamePackets::DisconnectPlayer(DisconnectPlayerPacket {
    //     reason: DisconnectReason::Unknown,
    //     message: Some(String::from("IDK")),
    // })])
    // .await
    // .unwrap();

    // let packet1 = StartGamePacket {
    //     target_actor_id: ActorUniqueID(609),
    //     target_runtime_id: ActorRuntimeID(402),
    //     gamemode: Gamemode::Creative,
    //     position: Vec3 {
    //         x: 4.0,
    //         y: 6.0,
    //         z: 7.0,
    //     },
    //     rotation: Vec2 { x: 270.0, y: 90.0 },
    //     settings: LevelSettings {
    //         seed: 777777777777,
    //         spawn_settings: SpawnSettings {
    //             biome_type: SpawnBiomeType::Default,
    //             user_defined_biome_name: String::from("RandomBiome"),
    //             dimension: Dimension::Overworld,
    //         },
    //         generator_type: GeneratorType::Overworld,
    //         gamemode: Gamemode::Creative,
    //         hardcore: false,
    //         difficulty: Difficulty::Peaceful,
    //         default_spawn_block: BlockPos {
    //             x: 100,
    //             y: 200,
    //             z: 300,
    //         },
    //         achievements_disabled: true,
    //         editor_world_type: EditorWorldType::NotEditor,
    //         created_in_editor: false,
    //         exported_from_editor: false,
    //         day_cycle_stop_time: 2000,
    //         education_edition_offer: 0,
    //         education_features: false,
    //         education_product_id: String::from(""),
    //         rain_level: 300.0,
    //         lightning_level: 400.0,
    //         platform_locked_content: false,
    //         multiplayer_intended: true,
    //         lan_broadcasting_intended: true,
    //         broadcasting_settings_xbox_live: 2,
    //         broadcasting_settings_platform: 2,
    //         commands: true,
    //         texture_pack_required: false,
    //         gamerules: vec![],
    //         experiments: Experiments {
    //             experiments: vec![],
    //             ever_toggled: false,
    //         },
    //         bonus_chest: false,
    //         start_with_map: false,
    //         player_permission: 3,
    //         server_chunk_tick_radius: 4,
    //         locked_behavior_packs: false,
    //         locked_resource_packs: false,
    //         from_locked_template: false,
    //         msa_gamertags_only: false,
    //         from_template: false,
    //         is_template_locked_settings: false,
    //         only_spawn_v1_villagers: false,
    //         persona_disabled: false,
    //         custom_skins_disabled: false,
    //         emote_chat_muted: false,
    //         base_game_version: BaseGameVersion(String::from("1.21.0")),
    //         limited_world_width: 16,
    //         limited_world_depth: 16,
    //         new_nether: true,
    //         edu_shared_uri_resource: EduSharedResourceUri {
    //             button_name: String::from(""),
    //             link_uri: String::from(""),
    //         },
    //         force_experimental_gameplay: Some(true),
    //         chat_restriction_level: ChatRestrictionLevel::None,
    //         disable_player_interactions: false,
    //         server_id: String::from(""),
    //         world_id: String::from(""),
    //         scenario_id: String::from(""),
    //     },
    //     level_id: String::from("UmFuZG9tIFdvcmxk"),
    //     level_name: String::from("Random World"),
    //     template_content_identity: String::from(""),
    //     trial: false,
    //     movement_settings: PlayerMovementSettings {
    //         authority_mode: PlayerMovementMode::Server,
    //         rewind_history_size: 3200,
    //         server_auth_block_breaking: false,
    //     },
    //     current_level_time: 9000,
    //     enchantment_seed: 99000,
    //     blocks: vec![],
    //     items: vec![],
    //     multiplayer_correlation_id: String::from("c5d3d2cc-27fd-4221-9de6-d22c4d423d53"),
    //     item_stack_net_manager: false,
    //     server_version: String::from("1.19.2"),
    //     player_property_data: nbtx::Value::Compound(HashMap::new()),
    //     block_state_checksum: 0,
    //     world_template_id: Uuid::nil(),
    //     clientside_world_generation: false,
    //     block_id_hashes: true,
    //     network_permission: NetworkPermissions {
    //         server_auth_sound: false,
    //     },
    // };
    //
    // let buf = vec![0xc2, 0x9, 0x92, 0x3, 0x2, 0x0, 0x0, 0x80, 0x40, 0x0, 0x0, 0xc0, 0x40, 0x0, 0x0, 0xe0, 0x40, 0x0, 0x0, 0x87, 0x43, 0x0, 0x0, 0xb4, 0x42, 0x71, 0xc, 0x2b, 0x17, 0xb5, 0x0, 0x0, 0x0, 0x0, 0x0, 0xb, 0x52, 0x61, 0x6e, 0x64, 0x6f, 0x6d, 0x42, 0x69, 0x6f, 0x6d, 0x65, 0x0, 0x2, 0x2, 0x0, 0x0, 0xc8, 0x1, 0xc8, 0x1, 0xd8, 0x4, 0x1, 0x0, 0x0, 0x0, 0xa0, 0x1f, 0x0, 0x0, 0x0, 0x0, 0x0, 0x96, 0x43, 0x0, 0x0, 0xc8, 0x43, 0x0, 0x1, 0x1, 0x4, 0x4, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x6, 0x4, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x6, 0x31, 0x2e, 0x32, 0x31, 0x2e, 0x30, 0x10, 0x0, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 0x55, 0x6d, 0x46, 0x75, 0x5a, 0x47, 0x39, 0x74, 0x49, 0x46, 0x64, 0x76, 0x63, 0x6d, 0x78, 0x6b, 0xc, 0x52, 0x61, 0x6e, 0x64, 0x6f, 0x6d, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x0, 0x0, 0x2, 0x80, 0x32, 0x0, 0x28, 0x23, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xf0, 0x8a, 0xc, 0x0, 0x0, 0x24, 0x63, 0x35, 0x64, 0x33, 0x64, 0x32, 0x63, 0x63, 0x2d, 0x32, 0x37, 0x66, 0x64, 0x2d, 0x34, 0x32, 0x32, 0x31, 0x2d, 0x39, 0x64, 0x65, 0x36, 0x2d, 0x64, 0x32, 0x32, 0x63, 0x34, 0x64, 0x34, 0x32, 0x33, 0x64, 0x35, 0x33, 0x0, 0x6, 0x31, 0x2e, 0x31, 0x39, 0x2e, 0x32, 0xa, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0];
    // println!("{buf:?}");
    // let mut buf = vec![];
    // packet1.proto_serialize(&mut buf).unwrap();
    // println!("{buf:?}");
    //
    // conn.send(&[GamePackets::StartGame(packet1)]).await.unwrap();
    // println!("StartGame");

    // conn.send(&[GamePackets::PlayStatus(PlayStatusPacket {
    //     status: PlayStatusType::PlayerSpawn,
    // })])
    //     .await.unwrap();
    // println!("PlayStatusPacket (PlayerSpawn)");

    let time_end = Instant::now();

    println!("{:?}", time_end.duration_since(time_start));

    loop {
        let res = conn.recv::<ProtoHelperV800>().await;

        if let Ok(packet) = res {
            println!("{:?}", packet);
        } else {
            break;
        }
    }
}
