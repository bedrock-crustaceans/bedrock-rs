pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V859;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V859 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V859 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V859 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V859 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V859 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V859 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V859 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V859 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V859 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V859 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V859 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V859 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V859 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V859 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V859 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V859 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V859 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V859 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V859 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V859 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V859 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V859 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V859 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V859 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V859 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V859 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V859 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V859 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V859 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V859 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V859 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V859 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V859 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V859 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V859 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V859 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V859 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V859 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V859 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V859 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V859 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V859 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V859 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V859 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V859 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V859 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V859 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V859 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V859 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V859 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V859 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V859 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V859 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V859 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V859 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V859 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V859 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V859 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V859 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V859 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V859 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V859 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V859 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V859 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V859 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V859 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V859 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V859 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V859 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V859 as ProtoVersionPackets>::GameTestResultsPacket,
    GraphicsParameterOverride: <V859 as ProtoVersionPackets>::GraphicsParameterOverridePacket,
    GuiDataPickItem: <V859 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V859 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V859 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V859 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V859 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V859 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V859 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V859 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V859 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V859 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V859 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V859 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V859 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V859 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V859 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V859 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V859 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V859 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V859 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V859 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V859 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V859 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V859 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V859 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V859 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V859 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V859 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V859 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V859 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V859 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V859 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V859 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V859 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V859 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V859 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V859 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V859 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V859 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V859 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V859 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V859 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V859 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V859 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V859 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V859 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V859 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V859 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V859 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V859 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V859 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V859 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V859 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V859 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V859 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V859 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V859 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V859 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V859 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V859 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V859 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V859 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V859 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V859 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V859 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V859 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V859 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V859 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V859 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V859 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V859 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V859 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V859 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V859 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V859 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V859 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V859 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V859 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V859 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundPackSettingChange: <V859 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket,
    ServerPlayerPostMovePosition: <V859 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V859 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V859 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V859 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V859 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V859 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V859 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V859 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V859 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V859 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V859 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V859 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V859 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V859 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V859 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V859 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V859 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V859 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V859 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V859 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V859 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V859 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V859 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V859 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V859 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V859 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V859 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V859 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V859 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V859 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V859 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V859 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V859 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V859 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V859 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V859 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V859 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V859 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V859 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V859 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V859 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V859 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V859 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V859 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V859 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V859 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V859 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V859 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V859 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V859 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V859 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V859 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V859 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V859 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V859 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V859 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V859 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V859 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V859 as ProtoVersionPackets>::UpdateTradePacket,
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
