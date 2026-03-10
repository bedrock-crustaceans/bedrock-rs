pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V819;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V819 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V819 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V819 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V819 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V819 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V819 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V819 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V819 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V819 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V819 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V819 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V819 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V819 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V819 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V819 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V819 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V819 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V819 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V819 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V819 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V819 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V819 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V819 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V819 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V819 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V819 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V819 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V819 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V819 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V819 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V819 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V819 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V819 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V819 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V819 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V819 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V819 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V819 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V819 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V819 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V819 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V819 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V819 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V819 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V819 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V819 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V819 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V819 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V819 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V819 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V819 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V819 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V819 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V819 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V819 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V819 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V819 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V819 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V819 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V819 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V819 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V819 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V819 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V819 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V819 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V819 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V819 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V819 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V819 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V819 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V819 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V819 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V819 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V819 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V819 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V819 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V819 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V819 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V819 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V819 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V819 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V819 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V819 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V819 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V819 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V819 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V819 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V819 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V819 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V819 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V819 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V819 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V819 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V819 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V819 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V819 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V819 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V819 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V819 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V819 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V819 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V819 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V819 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V819 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V819 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V819 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V819 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V819 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V819 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V819 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V819 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V819 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V819 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V819 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V819 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V819 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V819 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V819 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V819 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V819 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V819 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V819 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V819 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V819 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V819 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V819 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V819 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V819 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V819 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V819 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V819 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V819 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V819 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V819 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V819 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V819 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V819 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V819 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V819 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V819 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V819 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V819 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V819 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V819 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V819 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V819 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V819 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V819 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePosition: <V819 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V819 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V819 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V819 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V819 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V819 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V819 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V819 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V819 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V819 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V819 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V819 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V819 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V819 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V819 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V819 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V819 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V819 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V819 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V819 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V819 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V819 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V819 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V819 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V819 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V819 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V819 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V819 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V819 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V819 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V819 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V819 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V819 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V819 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V819 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V819 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V819 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V819 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V819 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V819 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V819 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V819 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V819 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V819 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V819 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V819 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V819 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V819 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V819 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V819 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V819 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V819 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V819 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V819 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V819 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V819 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V819 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V819 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V819 as ProtoVersionPackets>::UpdateTradePacket,
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
