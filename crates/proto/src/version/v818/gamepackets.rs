pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V818;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V818 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V818 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V818 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V818 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V818 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V818 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V818 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V818 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V818 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V818 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V818 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V818 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V818 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V818 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V818 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V818 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V818 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V818 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V818 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V818 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V818 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V818 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V818 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V818 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V818 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V818 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V818 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V818 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V818 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V818 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V818 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V818 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V818 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V818 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V818 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V818 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V818 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V818 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V818 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V818 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V818 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V818 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V818 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V818 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V818 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V818 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V818 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V818 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V818 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V818 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V818 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V818 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V818 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V818 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V818 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V818 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V818 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V818 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V818 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V818 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V818 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V818 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V818 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V818 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V818 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V818 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V818 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V818 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V818 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V818 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V818 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V818 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V818 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V818 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V818 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V818 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V818 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V818 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V818 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V818 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V818 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V818 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V818 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V818 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V818 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V818 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V818 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V818 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V818 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V818 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V818 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V818 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V818 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V818 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V818 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V818 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V818 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V818 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V818 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V818 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V818 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V818 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V818 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V818 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V818 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V818 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V818 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V818 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V818 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V818 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V818 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V818 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V818 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V818 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V818 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V818 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V818 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V818 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V818 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V818 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V818 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V818 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V818 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V818 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V818 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V818 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V818 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V818 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V818 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V818 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V818 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V818 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V818 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V818 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V818 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V818 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V818 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V818 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V818 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V818 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V818 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V818 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V818 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V818 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V818 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V818 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V818 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V818 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePosition: <V818 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V818 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V818 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V818 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V818 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V818 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V818 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V818 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V818 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V818 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V818 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V818 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V818 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V818 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V818 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V818 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V818 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V818 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V818 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V818 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V818 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V818 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V818 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V818 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V818 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V818 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V818 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V818 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V818 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V818 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V818 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V818 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V818 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V818 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V818 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V818 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V818 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V818 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V818 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V818 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V818 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V818 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V818 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V818 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V818 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V818 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V818 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V818 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V818 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V818 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V818 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V818 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V818 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V818 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V818 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V818 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V818 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V818 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V818 as ProtoVersionPackets>::UpdateTradePacket,
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
