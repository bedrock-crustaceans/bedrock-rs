pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V924;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V924 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V924 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V924 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V924 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V924 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V924 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V924 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V924 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V924 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V924 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V924 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V924 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V924 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V924 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V924 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V924 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V924 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V924 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V924 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V924 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V924 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V924 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V924 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V924 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V924 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistActorPriority: <V924 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket,
    CameraAimAssistInstruction: <V924 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V924 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V924 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V924 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V924 as ProtoVersionPackets>::CameraShakePacket,
    CameraSpline: <V924 as ProtoVersionPackets>::CameraSplinePacket,
    ChangeDimension: <V924 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V924 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V924 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V924 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V924 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientBoundDataDrivenUICloseAllScreens: <V924 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseAllScreensPacket,
    ClientBoundDataDrivenUIReload: <V924 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket,
    ClientBoundDataDrivenUIShowScreen: <V924 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket,
    ClientBoundDataStore: <V924 as ProtoVersionPackets>::ClientBoundDataStorePacket,
    ClientBoundTextureShift: <V924 as ProtoVersionPackets>::ClientBoundTextureShiftPacket,
    ClientCacheBlobStatus: <V924 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V924 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V924 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V924 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V924 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V924 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V924 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V924 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V924 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V924 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V924 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V924 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V924 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V924 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V924 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V924 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V924 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V924 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V924 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V924 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V924 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V924 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V924 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V924 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V924 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V924 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V924 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V924 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V924 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V924 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V924 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V924 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V924 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V924 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V924 as ProtoVersionPackets>::GameTestResultsPacket,
    GraphicsParameterOverride: <V924 as ProtoVersionPackets>::GraphicsParameterOverridePacket,
    GuiDataPickItem: <V924 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V924 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V924 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V924 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V924 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V924 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V924 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V924 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V924 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V924 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V924 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V924 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V924 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V924 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V924 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V924 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V924 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V924 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V924 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V924 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V924 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V924 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V924 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V924 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V924 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V924 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V924 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V924 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V924 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V924 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V924 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V924 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V924 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V924 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V924 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V924 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V924 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V924 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V924 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V924 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V924 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V924 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V924 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V924 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V924 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V924 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V924 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V924 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V924 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V924 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V924 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V924 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V924 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V924 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V924 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V924 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V924 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V924 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V924 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V924 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V924 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V924 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V924 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V924 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V924 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V924 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V924 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V924 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V924 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V924 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V924 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V924 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V924 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V924 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V924 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V924 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDataStore: <V924 as ProtoVersionPackets>::ServerBoundDataStorePacket,
    ServerBoundDiagnostics: <V924 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V924 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundPackSettingChange: <V924 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket,
    ServerPlayerPostMovePosition: <V924 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V924 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V924 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V924 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V924 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V924 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V924 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V924 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V924 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V924 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V924 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V924 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V924 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V924 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V924 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V924 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V924 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V924 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V924 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V924 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V924 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V924 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V924 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V924 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V924 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V924 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V924 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V924 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V924 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V924 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V924 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V924 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V924 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V924 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V924 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V924 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V924 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V924 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V924 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V924 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V924 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V924 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V924 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V924 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V924 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V924 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V924 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V924 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V924 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V924 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V924 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V924 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V924 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V924 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V924 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V924 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V924 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V924 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V924 as ProtoVersionPackets>::UpdateTradePacket,
    VoxelShapes: <V924 as ProtoVersionPackets>::VoxelShapesPacket,
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
