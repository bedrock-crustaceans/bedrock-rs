use bedrockrs_network::connection::Connection;
use bedrockrs_network::listener::Listener;
use bedrockrs_proto::compression::Compression;
use bedrockrs_proto::v662::enums::{
    ChatRestrictionLevel, Difficulty, EditorWorldType, EducationEditionOffer, GamePublishSetting,
    GameType, GeneratorType, PacketCompressionAlgorithm, PlayStatus, PlayerPermissionLevel,
    SpawnBiomeType,
};
use bedrockrs_proto::v662::packets::{NetworkSettingsPacket, PlayStatusPacket};
use bedrockrs_proto::v662::types::{
    ActorRuntimeID, ActorUniqueID, BaseGameVersion, EduSharedUriResource, Experiments,
    NetworkBlockPosition, NetworkPermissions, SpawnSettings,
};
use bedrockrs_proto::v818::packets::ResourcePacksInfoPacket;
use bedrockrs_proto::v818::types::SyncedPlayerMovementSettings;
use bedrockrs_proto::v898::packets::ResourcePackStackPacket;
use bedrockrs_proto::v924::packets::{StartGamePacket, VoxelShapesPacket};
use bedrockrs_proto::v924::types::{GameRuleLegacyData, LevelSettings};
use bedrockrs_proto::{ProtoVersion, Unknown, V924};
use bedrockrs_shared::world::dimension::Dimension;
use std::collections::HashMap;
use tokio::time::Instant;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let mut listener = Listener::new_raknet(
        "127.0.0.1:19132".parse().unwrap(),
        "Bedrock in Rust".to_string(),
        "bedrockrs".to_string(),
        V924::GAME_VERSION.to_string(),
        V924::PROTOCOL_VERSION,
        V924::RAKNET_VERSION,
        100,
        10,
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

async fn handle_login(mut unknown_conn: Connection<Unknown>) {
    let time_start = Instant::now();

    // RequestNetworkSettings
    let packets = unknown_conn.recv().await.unwrap();
    let mut conn = match packets.first() {
        Some(Unknown::RequestNetworkSettingsPacket(request))
            if request.client_network_version == 924 =>
        {
            unknown_conn.into_ver::<V924>()
        }
        _ => return,
    };

    println!("RequestNetworkSettings");

    let compression = Compression::None;

    // NetworkSettings
    conn.send(&[V924::NetworkSettingsPacket(NetworkSettingsPacket {
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
    conn.recv().await.unwrap();
    println!("Login");

    conn.send(&[
        V924::PlayStatusPacket(PlayStatusPacket {
            status: PlayStatus::LoginSuccess,
        }),
        V924::ResourcePacksInfoPacket(ResourcePacksInfoPacket {
            resource_pack_required: false,
            has_addon_packs: false,
            has_scripts: false,
            force_disable_vibrant_visuals: false,
            world_template_uuid: Uuid::nil(),
            resource_packs: vec![],
            world_template_version: "".to_string(),
        }),
        V924::ResourcePackStackPacket(ResourcePackStackPacket {
            texture_pack_required: false,
            addon_list: vec![],
            base_game_version: BaseGameVersion(String::from("1.0")),
            experiments: Experiments {
                experiments: vec![],
                ever_toggled: false,
            },
            include_editor_packs: false,
        }),
    ])
    .await
    .unwrap();
    println!("PlayStatus (LoginSuccess)");
    println!("ResourcePacksInfo");
    println!("ResourcePackStack");

    println!("{:#?}", conn.recv().await.unwrap());
    println!("ClientCacheStatus");
    println!("{:#?}", conn.recv().await.unwrap());
    println!("ResourcePackClientResponse");

    conn.send(&[V924::VoxelShapesPacket(VoxelShapesPacket {
        shapes: vec![],
        names: vec![],
    })])
    .await
    .unwrap();
    println!("VoxelShapes");

    let packet1 = StartGamePacket {
        target_actor_id: ActorUniqueID(609),
        target_runtime_id: ActorRuntimeID(402),
        actor_game_type: GameType::Creative,
        position: Default::default(),
        rotation: Default::default(),
        settings: LevelSettings {
            seed: 777777777777,
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
            day_cycle_stop_time: 2000,
            education_edition_offer: EducationEditionOffer::None,
            education_features_enabled: false,
            education_product_id: String::from(""),
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
            player_permissions: PlayerPermissionLevel::Custom,
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
            base_game_version: BaseGameVersion(V924::GAME_VERSION.to_string()),
            limited_world_width: 16,
            limited_world_depth: 16,
            nether_type: true,
            edu_shared_uri_resource: EduSharedUriResource {
                button_name: String::from(""),
                link_uri: String::from(""),
            },
            override_force_experimental_gameplay: Some(true),
            chat_restriction_level: ChatRestrictionLevel::None,
            disable_player_interactions: false,
        },
        level_id: String::from("UmFuZG9tIFdvcmxk"),
        level_name: String::from("Random World"),
        template_content_identity: String::from(""),
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
        server_version: V924::GAME_VERSION.to_string(),
        player_property_data: HashMap::new(),
        server_block_type_registry_checksum: 0,
        world_template_id: Uuid::nil(),
        server_enabled_client_side_generation: false,
        block_network_ids_are_hashes: false,
        network_permissions: NetworkPermissions {
            server_auth_sound_enabled: false,
        },
        server_join_information: None,
        server_id: "".to_string(),
        world_id: "".to_string(),
        scenario_id: "".to_string(),
        owner_id: "".to_string(),
    };

    conn.send(&[V924::StartGamePacket(packet1)]).await.unwrap();
    println!("StartGame");

    conn.send(&[V924::PlayStatusPacket(PlayStatusPacket {
        status: PlayStatus::PlayerSpawn,
    })])
    .await
    .unwrap();
    println!("PlayStatusPacket (PlayerSpawn)");

    println!("Finished request in {:?}", time_start.elapsed());

    loop {
        let res = conn.recv().await;

        if let Ok(packet) = res {
            println!("Found packet: {:?}", packet);
        } else {
            break;
        }
    }
}
