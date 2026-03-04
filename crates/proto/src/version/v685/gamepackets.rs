use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V685;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V685 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V685 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V685 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V685 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V685 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V685 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V685 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V685 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V685 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V685 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V685 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V685 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V685 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V685 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V685 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievementPacket: <V685 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V685 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V685 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V685 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V685 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V685 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V685 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V685 as ProtoVersionPackets>::CameraPacket,
    CameraInstruction: <V685 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V685 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V685 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V685 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V685 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientCacheBlobStatus: <V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V685 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V685 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V685 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V685 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V685 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V685 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V685 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V685 as ProtoVersionPackets>::CompletedUsingItemPacket,
    CompressedBiomeDefinitionList: <V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    ContainerClose: <V685 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V685 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerSetData: <V685 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V685 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V685 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V685 as ProtoVersionPackets>::CreativeContentPacket,
    DeathInfo: <V685 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V685 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V685 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V685 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V685 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V685 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V685 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V685 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V685 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V685 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V685 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V685 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V685 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V685 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V685 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V685 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V685 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V685 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V685 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V685 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V685 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V685 as ProtoVersionPackets>::ItemStackResponsePacket,
    LabTable: <V685 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V685 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V685 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V685 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V685 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V685 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V685 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelSoundEventV1: <V685 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelSoundEventV2: <V685 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    Login: <V685 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V685 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V685 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V685 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V685 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V685 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V685 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V685 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V685 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V685 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V685 as ProtoVersionPackets>::MovePlayerPacket,
    MultiplayerSettings: <V685 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V685 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V685 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V685 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V685 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V685 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V685 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PassengerJump: <V685 as ProtoVersionPackets>::PassengerJumpPacket,
    PhotoTransfer: <V685 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V685 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V685 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V685 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V685 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V685 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V685 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V685 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerInput: <V685 as ProtoVersionPackets>::PlayerInputPacket,
    PlayerList: <V685 as ProtoVersionPackets>::PlayerListPacket,
    PlayerSkin: <V685 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PositionTrackingDbClientRequest: <V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V685 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V685 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V685 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V685 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V685 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V685 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V685 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V685 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V685 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V685 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V685 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerPlayerPostMovePosition: <V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V685 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V685 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V685 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V685 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V685 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V685 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V685 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V685 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V685 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V685 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V685 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V685 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V685 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V685 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V685 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V685 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V685 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V685 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V685 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V685 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V685 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V685 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V685 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V685 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V685 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V685 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V685 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V685 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V685 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V685 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V685 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V685 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V685 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V685 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V685 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V685 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V685 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V685 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V685 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V685 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V685 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V685 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V685 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V685 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateEquip: <V685 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V685 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V685 as ProtoVersionPackets>::UpdateTradePacket,
}

pub fn read_gamepacket_header(
    stream: &mut Cursor<&[u8]>,
) -> Result<(u32, u16, SubClientID, SubClientID), ProtoCodecError> {
    // Read the gamepacket length
    let length = stream.read_u32_varint()?;

    // Read the gamepacket header and parse it into an u16
    // Since the (var)int is only storing 14 bytes we can treat it as an u16
    // This is normally treated as u32 varint
    let gamepacket_header: u16 = stream.read_u16_varint()?;

    // Get the first 10 bits as the packet id
    // Can never be more than a 16-bit integer due to being 10-bits big
    // Gamepacket IDs through 200-299 are used for spin-offs, they are free to use for custom packets
    let gamepacket_id = gamepacket_header & 0b0000_0011_1111_1111;

    // Get the next 2 bits as the sub client sender id
    // Can never be more than an 8-bit integer due to being 2 bits big
    let subclient_sender_id =
        SubClientID::try_from(((gamepacket_header & 0b0000_1100_0000_0000) >> 10) as u8)?;
    // Get the next 2 bits as the sub client target id
    // Never more than an 8-bit integer due to being 2 bits big
    let subclient_target_id =
        SubClientID::try_from(((gamepacket_header & 0b0011_0000_0000_0000) >> 12) as u8)?;

    Ok((
        length,
        gamepacket_id,
        subclient_sender_id,
        subclient_target_id,
    ))
}

pub fn write_gamepacket_header(
    stream: &mut Vec<u8>,
    length: u32,
    gamepacket_id: u16,
    subclient_sender_id: SubClientID,
    subclient_target_id: SubClientID,
) -> Result<(), ProtoCodecError> {
    // Since the (var)int is only storing 14 bytes, we can treat it as an u16
    // This is normally treated as u32 varint
    let mut gamepacket_header: u16 = 0;

    // Set the first 10 bits as the packet id
    // Can never be more than a 16-bit integer due to being 10-bits big
    // Gamepacket IDs through 200-299 are used for spin-offs, they are free to use for custom packets
    gamepacket_header |= 0b0000_0011_1111_1111 & gamepacket_id;

    // Set the next 2 bits as the sub client sender id
    // Never more than an 8-bit integer due to being 2 bits big
    gamepacket_header |= (Into::<u16>::into(subclient_sender_id) >> 10) & 0b0000_1100_0000_0000;
    // Set the next 2 bits as the sub client target id
    // Never more than an 8-bit integer due to being 2 bits big
    gamepacket_header |= (Into::<u16>::into(subclient_target_id) >> 12) & 0b0011_0000_0000_0000;

    // Since the size of the header is also included in the batched packet size,
    // we need to write it to a temporary buffer
    let mut gamepacket_header_buf = Vec::new();

    // Write the gamepacket header into temporary buffer
    gamepacket_header_buf.write_u16_varint(gamepacket_header)?;

    // Write the gamepacket length and the header length
    stream.write_u32_varint(length + gamepacket_header_buf.len() as u32)?;

    // Write the final game packet header
    stream.write_all(gamepacket_header_buf.as_slice())?;

    Ok(())
}

pub const fn get_gamepacket_header_size_prediction() -> usize {
    // 2 = gamepacket header (varint u32, only 14 bites can be treated as an u16)
    // 4 = gamepacket length size (varint u32)
    2 + 4
}
