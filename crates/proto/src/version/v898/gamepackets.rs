pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V898;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V898 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V898 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V898 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V898 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V898 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V898 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V898 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V898 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V898 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V898 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V898 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V898 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V898 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V898 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V898 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V898 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V898 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V898 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V898 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V898 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V898 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V898 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V898 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V898 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V898 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V898 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V898 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V898 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V898 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V898 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V898 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V898 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V898 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V898 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V898 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V898 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V898 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V898 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V898 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V898 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V898 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V898 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V898 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V898 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V898 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V898 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V898 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V898 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V898 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V898 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V898 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V898 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V898 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V898 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V898 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V898 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V898 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V898 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V898 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V898 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V898 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V898 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V898 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V898 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V898 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V898 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V898 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V898 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V898 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V898 as ProtoVersionPackets>::GameTestResultsPacket,
    GraphicsParameterOverride: <V898 as ProtoVersionPackets>::GraphicsParameterOverridePacket,
    GuiDataPickItem: <V898 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V898 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V898 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V898 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V898 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V898 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V898 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V898 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V898 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V898 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V898 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V898 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V898 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V898 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V898 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V898 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V898 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V898 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V898 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V898 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V898 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V898 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V898 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V898 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V898 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V898 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V898 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V898 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V898 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V898 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V898 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V898 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V898 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V898 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V898 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V898 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V898 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V898 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V898 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V898 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V898 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V898 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V898 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V898 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V898 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V898 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V898 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V898 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V898 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V898 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V898 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V898 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V898 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V898 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V898 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V898 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V898 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V898 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V898 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V898 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V898 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V898 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V898 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V898 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V898 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V898 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V898 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V898 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V898 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V898 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V898 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V898 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V898 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V898 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V898 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V898 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V898 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V898 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundPackSettingChange: <V898 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket,
    ServerPlayerPostMovePosition: <V898 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V898 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V898 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V898 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V898 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V898 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V898 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V898 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V898 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V898 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V898 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V898 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V898 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V898 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V898 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V898 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V898 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V898 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V898 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V898 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V898 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V898 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V898 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V898 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V898 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V898 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V898 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V898 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V898 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V898 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V898 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V898 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V898 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V898 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V898 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V898 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V898 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V898 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V898 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V898 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V898 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V898 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V898 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V898 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V898 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V898 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V898 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V898 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V898 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V898 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V898 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V898 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V898 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V898 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V898 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V898 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V898 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V898 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V898 as ProtoVersionPackets>::UpdateTradePacket,
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
