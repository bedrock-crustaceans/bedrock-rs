use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V748;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    SetMovementAuthority: <V748 as ProtoVersionPackets>::SetMovementAuthorityPacket,
    MovementEffect: <V748 as ProtoVersionPackets>::MovementEffectPacket,
    ContainerRegistryCleanup: <V748 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    CameraAimAssist: <V748 as ProtoVersionPackets>::CameraAimAssistPacket,
    ServerBoundLoadingScreen: <V748 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundDiagnostics: <V748 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    JigsawStructureData: <V748 as ProtoVersionPackets>::JigsawStructureDataPacket,
    CurrentStructureFeature: <V748 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    Login: <V748 as ProtoVersionPackets>::LoginPacket,
    PlayStatus: <V748 as ProtoVersionPackets>::PlayStatusPacket,
    ServerToClientHandshake: <V748 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    ClientToServerHandshake: <V748 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    Disconnect: <V748 as ProtoVersionPackets>::DisconnectPacket,
    ResourcePacksInfo: <V748 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    ResourcePackStack: <V748 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePackClientResponse: <V748 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    Text: <V748 as ProtoVersionPackets>::TextPacket,
    SetTime: <V748 as ProtoVersionPackets>::SetTimePacket,
    StartGame: <V748 as ProtoVersionPackets>::StartGamePacket,
    AddPlayer: <V748 as ProtoVersionPackets>::AddPlayerPacket,
    AddActor: <V748 as ProtoVersionPackets>::AddActorPacket,
    RemoveActor: <V748 as ProtoVersionPackets>::RemoveActorPacket,
    AddItemActor: <V748 as ProtoVersionPackets>::AddItemActorPacket,
    ClientBoundCloseForm: <V748 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ServerPlayerPostMovePosition: <V748 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    TakeItemActor: <V748 as ProtoVersionPackets>::TakeItemActorPacket,
    MoveActorAbsolute: <V748 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MovePlayer: <V748 as ProtoVersionPackets>::MovePlayerPacket,
    PassengerJump: <V748 as ProtoVersionPackets>::PassengerJumpPacket,
    UpdateBlock: <V748 as ProtoVersionPackets>::UpdateBlockPacket,
    AddPainting: <V748 as ProtoVersionPackets>::AddPaintingPacket,
    AwardAchievement: <V748 as ProtoVersionPackets>::AwardAchievementPacket,
    LevelSoundEventV1: <V748 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelEvent: <V748 as ProtoVersionPackets>::LevelEventPacket,
    BlockEvent: <V748 as ProtoVersionPackets>::BlockEventPacket,
    ActorEvent: <V748 as ProtoVersionPackets>::ActorEventPacket,
    MobEffect: <V748 as ProtoVersionPackets>::MobEffectPacket,
    UpdateAttributes: <V748 as ProtoVersionPackets>::UpdateAttributesPacket,
    InventoryTransaction: <V748 as ProtoVersionPackets>::InventoryTransactionPacket,
    MobEquipment: <V748 as ProtoVersionPackets>::MobEquipmentPacket,
    MobArmorEquipment: <V748 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    Interact: <V748 as ProtoVersionPackets>::InteractPacket,
    BlockPickRequest: <V748 as ProtoVersionPackets>::BlockPickRequestPacket,
    ActorPickRequest: <V748 as ProtoVersionPackets>::ActorPickRequestPacket,
    PlayerAction: <V748 as ProtoVersionPackets>::PlayerActionPacket,
    HurtArmor: <V748 as ProtoVersionPackets>::HurtArmorPacket,
    SetActorData: <V748 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorMotion: <V748 as ProtoVersionPackets>::SetActorMotionPacket,
    SetActorLink: <V748 as ProtoVersionPackets>::SetActorLinkPacket,
    SetHealth: <V748 as ProtoVersionPackets>::SetHealthPacket,
    SetSpawnPosition: <V748 as ProtoVersionPackets>::SetSpawnPositionPacket,
    Animate: <V748 as ProtoVersionPackets>::AnimatePacket,
    Respawn: <V748 as ProtoVersionPackets>::RespawnPacket,
    ContainerOpen: <V748 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerClose: <V748 as ProtoVersionPackets>::ContainerClosePacket,
    PlayerHotbar: <V748 as ProtoVersionPackets>::PlayerHotbarPacket,
    InventoryContent: <V748 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V748 as ProtoVersionPackets>::InventorySlotPacket,
    ContainerSetData: <V748 as ProtoVersionPackets>::ContainerSetDataPacket,
    CraftingData: <V748 as ProtoVersionPackets>::CraftingDataPacket,
    GuiDataPickItem: <V748 as ProtoVersionPackets>::GuiDataPickItemPacket,
    BlockActorData: <V748 as ProtoVersionPackets>::BlockActorDataPacket,
    PlayerInput: <V748 as ProtoVersionPackets>::PlayerInputPacket,
    LevelChunk: <V748 as ProtoVersionPackets>::LevelChunkPacket,
    SetCommandsEnabled: <V748 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDifficulty: <V748 as ProtoVersionPackets>::SetDifficultyPacket,
    ChangeDimension: <V748 as ProtoVersionPackets>::ChangeDimensionPacket,
    SetPlayerGameType: <V748 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    PlayerList: <V748 as ProtoVersionPackets>::PlayerListPacket,
    SimpleEvent: <V748 as ProtoVersionPackets>::SimpleEventPacket,
    LegacyTelemetryEvent: <V748 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    SpawnExperienceOrb: <V748 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    ClientboundMapItemData: <V748 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    MapInfoRequest: <V748 as ProtoVersionPackets>::MapInfoRequestPacket,
    RequestChunkRadius: <V748 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    ChunkRadiusUpdated: <V748 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    GameRulesChanged: <V748 as ProtoVersionPackets>::GameRulesChangedPacket,
    Camera: <V748 as ProtoVersionPackets>::CameraPacket,
    BossEvent: <V748 as ProtoVersionPackets>::BossEventPacket,
    ShowCredits: <V748 as ProtoVersionPackets>::ShowCreditsPacket,
    AvailableCommands: <V748 as ProtoVersionPackets>::AvailableCommandsPacket,
    CommandRequest: <V748 as ProtoVersionPackets>::CommandRequestPacket,
    CommandBlockUpdate: <V748 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V748 as ProtoVersionPackets>::CommandOutputPacket,
    UpdateTrade: <V748 as ProtoVersionPackets>::UpdateTradePacket,
    UpdateEquip: <V748 as ProtoVersionPackets>::UpdateEquipPacket,
    ResourcePackDataInfo: <V748 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackChunkData: <V748 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V748 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    TransferPlayer: <V748 as ProtoVersionPackets>::TransferPlayerPacket,
    PlaySound: <V748 as ProtoVersionPackets>::PlaySoundPacket,
    StopSound: <V748 as ProtoVersionPackets>::StopSoundPacket,
    SetTitle: <V748 as ProtoVersionPackets>::SetTitlePacket,
    AddBehaviourTree: <V748 as ProtoVersionPackets>::AddBehaviourTreePacket,
    StructureBlockUpdate: <V748 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    ShowStoreOffer: <V748 as ProtoVersionPackets>::ShowStoreOfferPacket,
    PurchaseReceipt: <V748 as ProtoVersionPackets>::PurchaseReceiptPacket,
    PlayerSkin: <V748 as ProtoVersionPackets>::PlayerSkinPacket,
    SubClientLogin: <V748 as ProtoVersionPackets>::SubClientLoginPacket,
    AutomationClientConnect: <V748 as ProtoVersionPackets>::AutomationClientConnectPacket,
    SetLastHurtBy: <V748 as ProtoVersionPackets>::SetLastHurtByPacket,
    BookEdit: <V748 as ProtoVersionPackets>::BookEditPacket,
    NpcRequest: <V748 as ProtoVersionPackets>::NpcRequestPacket,
    PhotoTransfer: <V748 as ProtoVersionPackets>::PhotoTransferPacket,
    ModalFormRequest: <V748 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V748 as ProtoVersionPackets>::ModalFormResponsePacket,
    ServerSettingsRequest: <V748 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V748 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ShowProfile: <V748 as ProtoVersionPackets>::ShowProfilePacket,
    SetDefaultGameType: <V748 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    RemoveObjective: <V748 as ProtoVersionPackets>::RemoveObjectivePacket,
    SetDisplayObjective: <V748 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetScore: <V748 as ProtoVersionPackets>::SetScorePacket,
    LabTable: <V748 as ProtoVersionPackets>::LabTablePacket,
    UpdateBlockSynced: <V748 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    MoveActorDelta: <V748 as ProtoVersionPackets>::MoveActorDeltaPacket,
    SetScoreboardIdentity: <V748 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetLocalPlayerAsInitialized: <V748 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    UpdateSoftEnum: <V748 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    NetworkStackLatency: <V748 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    SpawnParticleEffect: <V748 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    AvailableActorIdentifiers: <V748 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    LevelSoundEventV2: <V748 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    NetworkChunkPublisherUpdate: <V748 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    BiomeDefinitionList: <V748 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    LevelSoundEvent: <V748 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelEventGeneric: <V748 as ProtoVersionPackets>::LevelEventGenericPacket,
    LecternUpdate: <V748 as ProtoVersionPackets>::LecternUpdatePacket,
    ClientCacheStatus: <V748 as ProtoVersionPackets>::ClientCacheStatusPacket,
    OnScreenTextureAnimation: <V748 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    MapCreateLockedCopy: <V748 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    StructureDataRequest: <V748 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V748 as ProtoVersionPackets>::StructureDataResponsePacket,
    ClientCacheBlobStatus: <V748 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V748 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    EducationSettings: <V748 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V748 as ProtoVersionPackets>::EmotePacket,
    MultiplayerSettings: <V748 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    SettingsCommand: <V748 as ProtoVersionPackets>::SettingsCommandPacket,
    AnvilDamage: <V748 as ProtoVersionPackets>::AnvilDamagePacket,
    CompletedUsingItem: <V748 as ProtoVersionPackets>::CompletedUsingItemPacket,
    NetworkSettings: <V748 as ProtoVersionPackets>::NetworkSettingsPacket,
    PlayerAuthInput: <V748 as ProtoVersionPackets>::PlayerAuthInputPacket,
    CreativeContent: <V748 as ProtoVersionPackets>::CreativeContentPacket,
    PlayerEnchantOptions: <V748 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    ItemStackRequest: <V748 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V748 as ProtoVersionPackets>::ItemStackResponsePacket,
    PlayerArmorDamage: <V748 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    CodeBuilder: <V748 as ProtoVersionPackets>::CodeBuilderPacket,
    UpdatePlayerGameType: <V748 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    EmoteList: <V748 as ProtoVersionPackets>::EmoteListPacket,
    PositionTrackingDbServerBroadcast: <V748 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PositionTrackingDbClientRequest: <V748 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    DebugInfo: <V748 as ProtoVersionPackets>::DebugInfoPacket,
    PacketViolationWarning: <V748 as ProtoVersionPackets>::PacketViolationWarningPacket,
    MotionPredictionHints: <V748 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    AnimateEntity: <V748 as ProtoVersionPackets>::AnimateEntityPacket,
    CameraShake: <V748 as ProtoVersionPackets>::CameraShakePacket,
    PlayerFog: <V748 as ProtoVersionPackets>::PlayerFogPacket,
    CorrectPlayerMovePrediction: <V748 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    ItemComponent: <V748 as ProtoVersionPackets>::ItemComponentPacket,
    ClientboundDebugRenderer: <V748 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    SyncActorProperty: <V748 as ProtoVersionPackets>::SyncActorPropertyPacket,
    AddVolumeEntity: <V748 as ProtoVersionPackets>::AddVolumeEntityPacket,
    RemoveVolumeEntity: <V748 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    SimulationType: <V748 as ProtoVersionPackets>::SimulationTypePacket,
    NpcDialogue: <V748 as ProtoVersionPackets>::NpcDialoguePacket,
    EduUriResource: <V748 as ProtoVersionPackets>::EduUriResourcePacket,
    CreatePhoto: <V748 as ProtoVersionPackets>::CreatePhotoPacket,
    UpdateSubChunkBlocks: <V748 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    SubChunk: <V748 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V748 as ProtoVersionPackets>::SubChunkRequestPacket,
    PlayerStartItemCooldown: <V748 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    ScriptMessage: <V748 as ProtoVersionPackets>::ScriptMessagePacket,
    CodeBuilderSource: <V748 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    TickingAreaLoadStatus: <V748 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    DimensionData: <V748 as ProtoVersionPackets>::DimensionDataPacket,
    AgentActionEvent: <V748 as ProtoVersionPackets>::AgentActionEventPacket,
    ChangeMobProperty: <V748 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    LessonProgress: <V748 as ProtoVersionPackets>::LessonProgressPacket,
    RequestAbility: <V748 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestPermissions: <V748 as ProtoVersionPackets>::RequestPermissionsPacket,
    ToastRequest: <V748 as ProtoVersionPackets>::ToastRequestPacket,
    UpdateAbilities: <V748 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V748 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    DeathInfo: <V748 as ProtoVersionPackets>::DeathInfoPacket,
    EditorNetwork: <V748 as ProtoVersionPackets>::EditorNetworkPacket,
    FeatureRegistry: <V748 as ProtoVersionPackets>::FeatureRegistryPacket,
    ServerStats: <V748 as ProtoVersionPackets>::ServerStatsPacket,
    RequestNetworkSettings: <V748 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    GameTestRequest: <V748 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V748 as ProtoVersionPackets>::GameTestResultsPacket,
    UpdateClientInputLocks: <V748 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    CameraPresets: <V748 as ProtoVersionPackets>::CameraPresetsPacket,
    UnlockedRecipes: <V748 as ProtoVersionPackets>::UnlockedRecipesPacket,
    CameraInstruction: <V748 as ProtoVersionPackets>::CameraInstructionPacket,
    CompressedBiomeDefinitionList: <V748 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    TrimData: <V748 as ProtoVersionPackets>::TrimDataPacket,
    OpenSign: <V748 as ProtoVersionPackets>::OpenSignPacket,
    AgentAnimation: <V748 as ProtoVersionPackets>::AgentAnimationPacket,
    RefreshEntitlements: <V748 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    PlayerToggleCrafterSlotRequest: <V748 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    SetPlayerInventoryOptions: <V748 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetHud: <V748 as ProtoVersionPackets>::SetHudPacket
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
