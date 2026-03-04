use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V729;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ContainerRegistryCleanup: <V729 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    CameraAimAssist: <V729 as ProtoVersionPackets>::CameraAimAssistPacket,
    ServerBoundLoadingScreen: <V729 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundDiagnostics: <V729 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    JigsawStructureData: <V729 as ProtoVersionPackets>::JigsawStructureDataPacket,
    CurrentStructureFeature: <V729 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    Login: <V729 as ProtoVersionPackets>::LoginPacket,
    PlayStatus: <V729 as ProtoVersionPackets>::PlayStatusPacket,
    ServerToClientHandshake: <V729 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    ClientToServerHandshake: <V729 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    Disconnect: <V729 as ProtoVersionPackets>::DisconnectPacket,
    ResourcePacksInfo: <V729 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    ResourcePackStack: <V729 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePackClientResponse: <V729 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    Text: <V729 as ProtoVersionPackets>::TextPacket,
    SetTime: <V729 as ProtoVersionPackets>::SetTimePacket,
    StartGame: <V729 as ProtoVersionPackets>::StartGamePacket,
    AddPlayer: <V729 as ProtoVersionPackets>::AddPlayerPacket,
    AddActor: <V729 as ProtoVersionPackets>::AddActorPacket,
    RemoveActor: <V729 as ProtoVersionPackets>::RemoveActorPacket,
    AddItemActor: <V729 as ProtoVersionPackets>::AddItemActorPacket,
    ClientBoundCloseForm: <V729 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ServerPlayerPostMovePosition: <V729 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    TakeItemActor: <V729 as ProtoVersionPackets>::TakeItemActorPacket,
    MoveActorAbsolute: <V729 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MovePlayer: <V729 as ProtoVersionPackets>::MovePlayerPacket,
    PassengerJump: <V729 as ProtoVersionPackets>::PassengerJumpPacket,
    UpdateBlock: <V729 as ProtoVersionPackets>::UpdateBlockPacket,
    AddPainting: <V729 as ProtoVersionPackets>::AddPaintingPacket,
    AwardAchievement: <V729 as ProtoVersionPackets>::AwardAchievementPacket,
    LevelSoundEventV1: <V729 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelEvent: <V729 as ProtoVersionPackets>::LevelEventPacket,
    BlockEvent: <V729 as ProtoVersionPackets>::BlockEventPacket,
    ActorEvent: <V729 as ProtoVersionPackets>::ActorEventPacket,
    MobEffect: <V729 as ProtoVersionPackets>::MobEffectPacket,
    UpdateAttributes: <V729 as ProtoVersionPackets>::UpdateAttributesPacket,
    InventoryTransaction: <V729 as ProtoVersionPackets>::InventoryTransactionPacket,
    MobEquipment: <V729 as ProtoVersionPackets>::MobEquipmentPacket,
    MobArmorEquipment: <V729 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    Interact: <V729 as ProtoVersionPackets>::InteractPacket,
    BlockPickRequest: <V729 as ProtoVersionPackets>::BlockPickRequestPacket,
    ActorPickRequest: <V729 as ProtoVersionPackets>::ActorPickRequestPacket,
    PlayerAction: <V729 as ProtoVersionPackets>::PlayerActionPacket,
    HurtArmor: <V729 as ProtoVersionPackets>::HurtArmorPacket,
    SetActorData: <V729 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorMotion: <V729 as ProtoVersionPackets>::SetActorMotionPacket,
    SetActorLink: <V729 as ProtoVersionPackets>::SetActorLinkPacket,
    SetHealth: <V729 as ProtoVersionPackets>::SetHealthPacket,
    SetSpawnPosition: <V729 as ProtoVersionPackets>::SetSpawnPositionPacket,
    Animate: <V729 as ProtoVersionPackets>::AnimatePacket,
    Respawn: <V729 as ProtoVersionPackets>::RespawnPacket,
    ContainerOpen: <V729 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerClose: <V729 as ProtoVersionPackets>::ContainerClosePacket,
    PlayerHotbar: <V729 as ProtoVersionPackets>::PlayerHotbarPacket,
    InventoryContent: <V729 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V729 as ProtoVersionPackets>::InventorySlotPacket,
    ContainerSetData: <V729 as ProtoVersionPackets>::ContainerSetDataPacket,
    CraftingData: <V729 as ProtoVersionPackets>::CraftingDataPacket,
    GuiDataPickItem: <V729 as ProtoVersionPackets>::GuiDataPickItemPacket,
    BlockActorData: <V729 as ProtoVersionPackets>::BlockActorDataPacket,
    PlayerInput: <V729 as ProtoVersionPackets>::PlayerInputPacket,
    LevelChunk: <V729 as ProtoVersionPackets>::LevelChunkPacket,
    SetCommandsEnabled: <V729 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDifficulty: <V729 as ProtoVersionPackets>::SetDifficultyPacket,
    ChangeDimension: <V729 as ProtoVersionPackets>::ChangeDimensionPacket,
    SetPlayerGameType: <V729 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    PlayerList: <V729 as ProtoVersionPackets>::PlayerListPacket,
    SimpleEvent: <V729 as ProtoVersionPackets>::SimpleEventPacket,
    LegacyTelemetryEvent: <V729 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    SpawnExperienceOrb: <V729 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    ClientboundMapItemData: <V729 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    MapInfoRequest: <V729 as ProtoVersionPackets>::MapInfoRequestPacket,
    RequestChunkRadius: <V729 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    ChunkRadiusUpdated: <V729 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    GameRulesChanged: <V729 as ProtoVersionPackets>::GameRulesChangedPacket,
    Camera: <V729 as ProtoVersionPackets>::CameraPacket,
    BossEvent: <V729 as ProtoVersionPackets>::BossEventPacket,
    ShowCredits: <V729 as ProtoVersionPackets>::ShowCreditsPacket,
    AvailableCommands: <V729 as ProtoVersionPackets>::AvailableCommandsPacket,
    CommandRequest: <V729 as ProtoVersionPackets>::CommandRequestPacket,
    CommandBlockUpdate: <V729 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V729 as ProtoVersionPackets>::CommandOutputPacket,
    UpdateTrade: <V729 as ProtoVersionPackets>::UpdateTradePacket,
    UpdateEquip: <V729 as ProtoVersionPackets>::UpdateEquipPacket,
    ResourcePackDataInfo: <V729 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackChunkData: <V729 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V729 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    TransferPlayer: <V729 as ProtoVersionPackets>::TransferPlayerPacket,
    PlaySound: <V729 as ProtoVersionPackets>::PlaySoundPacket,
    StopSound: <V729 as ProtoVersionPackets>::StopSoundPacket,
    SetTitle: <V729 as ProtoVersionPackets>::SetTitlePacket,
    AddBehaviourTree: <V729 as ProtoVersionPackets>::AddBehaviourTreePacket,
    StructureBlockUpdate: <V729 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    ShowStoreOffer: <V729 as ProtoVersionPackets>::ShowStoreOfferPacket,
    PurchaseReceipt: <V729 as ProtoVersionPackets>::PurchaseReceiptPacket,
    PlayerSkin: <V729 as ProtoVersionPackets>::PlayerSkinPacket,
    SubClientLogin: <V729 as ProtoVersionPackets>::SubClientLoginPacket,
    AutomationClientConnect: <V729 as ProtoVersionPackets>::AutomationClientConnectPacket,
    SetLastHurtBy: <V729 as ProtoVersionPackets>::SetLastHurtByPacket,
    BookEdit: <V729 as ProtoVersionPackets>::BookEditPacket,
    NpcRequest: <V729 as ProtoVersionPackets>::NpcRequestPacket,
    PhotoTransfer: <V729 as ProtoVersionPackets>::PhotoTransferPacket,
    ModalFormRequest: <V729 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V729 as ProtoVersionPackets>::ModalFormResponsePacket,
    ServerSettingsRequest: <V729 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V729 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ShowProfile: <V729 as ProtoVersionPackets>::ShowProfilePacket,
    SetDefaultGameType: <V729 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    RemoveObjective: <V729 as ProtoVersionPackets>::RemoveObjectivePacket,
    SetDisplayObjective: <V729 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetScore: <V729 as ProtoVersionPackets>::SetScorePacket,
    LabTable: <V729 as ProtoVersionPackets>::LabTablePacket,
    UpdateBlockSynced: <V729 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    MoveActorDelta: <V729 as ProtoVersionPackets>::MoveActorDeltaPacket,
    SetScoreboardIdentity: <V729 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetLocalPlayerAsInitialized: <V729 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    UpdateSoftEnum: <V729 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    NetworkStackLatency: <V729 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    SpawnParticleEffect: <V729 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    AvailableActorIdentifiers: <V729 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    LevelSoundEventV2: <V729 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    NetworkChunkPublisherUpdate: <V729 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    BiomeDefinitionList: <V729 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    LevelSoundEvent: <V729 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelEventGeneric: <V729 as ProtoVersionPackets>::LevelEventGenericPacket,
    LecternUpdate: <V729 as ProtoVersionPackets>::LecternUpdatePacket,
    ClientCacheStatus: <V729 as ProtoVersionPackets>::ClientCacheStatusPacket,
    OnScreenTextureAnimation: <V729 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    MapCreateLockedCopy: <V729 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    StructureDataRequest: <V729 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V729 as ProtoVersionPackets>::StructureDataResponsePacket,
    ClientCacheBlobStatus: <V729 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V729 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    EducationSettings: <V729 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V729 as ProtoVersionPackets>::EmotePacket,
    MultiplayerSettings: <V729 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    SettingsCommand: <V729 as ProtoVersionPackets>::SettingsCommandPacket,
    AnvilDamage: <V729 as ProtoVersionPackets>::AnvilDamagePacket,
    CompletedUsingItem: <V729 as ProtoVersionPackets>::CompletedUsingItemPacket,
    NetworkSettings: <V729 as ProtoVersionPackets>::NetworkSettingsPacket,
    PlayerAuthInput: <V729 as ProtoVersionPackets>::PlayerAuthInputPacket,
    CreativeContent: <V729 as ProtoVersionPackets>::CreativeContentPacket,
    PlayerEnchantOptions: <V729 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    ItemStackRequest: <V729 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V729 as ProtoVersionPackets>::ItemStackResponsePacket,
    PlayerArmorDamage: <V729 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    CodeBuilder: <V729 as ProtoVersionPackets>::CodeBuilderPacket,
    UpdatePlayerGameType: <V729 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    EmoteList: <V729 as ProtoVersionPackets>::EmoteListPacket,
    PositionTrackingDbServerBroadcast: <V729 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PositionTrackingDbClientRequest: <V729 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    DebugInfo: <V729 as ProtoVersionPackets>::DebugInfoPacket,
    PacketViolationWarning: <V729 as ProtoVersionPackets>::PacketViolationWarningPacket,
    MotionPredictionHints: <V729 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    AnimateEntity: <V729 as ProtoVersionPackets>::AnimateEntityPacket,
    CameraShake: <V729 as ProtoVersionPackets>::CameraShakePacket,
    PlayerFog: <V729 as ProtoVersionPackets>::PlayerFogPacket,
    CorrectPlayerMovePrediction: <V729 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    ItemComponent: <V729 as ProtoVersionPackets>::ItemComponentPacket,
    ClientboundDebugRenderer: <V729 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    SyncActorProperty: <V729 as ProtoVersionPackets>::SyncActorPropertyPacket,
    AddVolumeEntity: <V729 as ProtoVersionPackets>::AddVolumeEntityPacket,
    RemoveVolumeEntity: <V729 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    SimulationType: <V729 as ProtoVersionPackets>::SimulationTypePacket,
    NpcDialogue: <V729 as ProtoVersionPackets>::NpcDialoguePacket,
    EduUriResource: <V729 as ProtoVersionPackets>::EduUriResourcePacket,
    CreatePhoto: <V729 as ProtoVersionPackets>::CreatePhotoPacket,
    UpdateSubChunkBlocks: <V729 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    SubChunk: <V729 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V729 as ProtoVersionPackets>::SubChunkRequestPacket,
    PlayerStartItemCooldown: <V729 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    ScriptMessage: <V729 as ProtoVersionPackets>::ScriptMessagePacket,
    CodeBuilderSource: <V729 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    TickingAreaLoadStatus: <V729 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    DimensionData: <V729 as ProtoVersionPackets>::DimensionDataPacket,
    AgentActionEvent: <V729 as ProtoVersionPackets>::AgentActionEventPacket,
    ChangeMobProperty: <V729 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    LessonProgress: <V729 as ProtoVersionPackets>::LessonProgressPacket,
    RequestAbility: <V729 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestPermissions: <V729 as ProtoVersionPackets>::RequestPermissionsPacket,
    ToastRequest: <V729 as ProtoVersionPackets>::ToastRequestPacket,
    UpdateAbilities: <V729 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V729 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    DeathInfo: <V729 as ProtoVersionPackets>::DeathInfoPacket,
    EditorNetwork: <V729 as ProtoVersionPackets>::EditorNetworkPacket,
    FeatureRegistry: <V729 as ProtoVersionPackets>::FeatureRegistryPacket,
    ServerStats: <V729 as ProtoVersionPackets>::ServerStatsPacket,
    RequestNetworkSettings: <V729 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    GameTestRequest: <V729 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V729 as ProtoVersionPackets>::GameTestResultsPacket,
    UpdateClientInputLocks: <V729 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    CameraPresets: <V729 as ProtoVersionPackets>::CameraPresetsPacket,
    UnlockedRecipes: <V729 as ProtoVersionPackets>::UnlockedRecipesPacket,
    CameraInstruction: <V729 as ProtoVersionPackets>::CameraInstructionPacket,
    CompressedBiomeDefinitionList: <V729 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    TrimData: <V729 as ProtoVersionPackets>::TrimDataPacket,
    OpenSign: <V729 as ProtoVersionPackets>::OpenSignPacket,
    AgentAnimation: <V729 as ProtoVersionPackets>::AgentAnimationPacket,
    RefreshEntitlements: <V729 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    PlayerToggleCrafterSlotRequest: <V729 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    SetPlayerInventoryOptions: <V729 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetHud: <V729 as ProtoVersionPackets>::SetHudPacket
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
