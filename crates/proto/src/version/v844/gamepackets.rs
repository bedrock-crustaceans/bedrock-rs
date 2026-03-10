pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V844;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V844 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V844 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V844 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V844 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V844 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V844 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V844 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V844 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V844 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V844 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V844 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V844 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V844 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V844 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V844 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V844 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V844 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V844 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V844 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V844 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V844 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V844 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V844 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V844 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V844 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V844 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V844 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V844 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V844 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V844 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V844 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V844 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V844 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V844 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V844 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V844 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V844 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V844 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V844 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V844 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V844 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V844 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V844 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V844 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V844 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V844 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V844 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V844 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V844 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V844 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V844 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V844 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V844 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V844 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V844 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V844 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V844 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V844 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V844 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V844 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V844 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V844 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V844 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V844 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V844 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V844 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V844 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V844 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V844 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V844 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V844 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V844 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V844 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V844 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V844 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V844 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V844 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V844 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V844 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V844 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V844 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V844 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V844 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V844 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V844 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V844 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V844 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V844 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V844 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V844 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V844 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V844 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V844 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V844 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V844 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V844 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V844 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V844 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V844 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V844 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V844 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V844 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V844 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V844 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V844 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V844 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V844 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V844 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V844 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V844 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V844 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V844 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V844 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V844 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V844 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V844 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V844 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V844 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V844 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V844 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V844 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V844 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V844 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V844 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V844 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V844 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V844 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V844 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V844 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V844 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V844 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V844 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V844 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V844 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V844 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V844 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V844 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V844 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V844 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V844 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V844 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V844 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V844 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V844 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V844 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V844 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V844 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V844 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundPackSettingChange: <V844 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket,
    ServerPlayerPostMovePosition: <V844 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V844 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V844 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V844 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V844 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V844 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V844 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V844 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V844 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V844 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V844 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V844 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V844 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V844 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V844 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V844 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V844 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V844 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V844 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V844 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V844 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V844 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V844 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V844 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V844 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V844 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V844 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V844 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V844 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V844 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V844 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V844 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V844 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V844 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V844 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V844 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V844 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V844 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V844 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V844 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V844 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V844 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V844 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V844 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V844 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V844 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V844 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V844 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V844 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V844 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V844 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V844 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V844 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V844 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V844 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V844 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V844 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V844 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V844 as ProtoVersionPackets>::UpdateTradePacket,
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
