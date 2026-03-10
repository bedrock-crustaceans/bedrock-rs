pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V827;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V827 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V827 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V827 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V827 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V827 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V827 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V827 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V827 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V827 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V827 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V827 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V827 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V827 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V827 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V827 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V827 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V827 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V827 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V827 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V827 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V827 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V827 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V827 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V827 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V827 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V827 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V827 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V827 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V827 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V827 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V827 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V827 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V827 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V827 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V827 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V827 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V827 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V827 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V827 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V827 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V827 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V827 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V827 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V827 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V827 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V827 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V827 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V827 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V827 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V827 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V827 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V827 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V827 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V827 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V827 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V827 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V827 as ProtoVersionPackets>::DeathInfoPacket,
    DebugDrawer: <V827 as ProtoVersionPackets>::DebugDrawerPacket,
    DebugInfo: <V827 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V827 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V827 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V827 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V827 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V827 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V827 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V827 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V827 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V827 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V827 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V827 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V827 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V827 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V827 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V827 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V827 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V827 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V827 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V827 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V827 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V827 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V827 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V827 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V827 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V827 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V827 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V827 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V827 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V827 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V827 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V827 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V827 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V827 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V827 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V827 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V827 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V827 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V827 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V827 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V827 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V827 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V827 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V827 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V827 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V827 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V827 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V827 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V827 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V827 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V827 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V827 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V827 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V827 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V827 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V827 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V827 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V827 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V827 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V827 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V827 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V827 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V827 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V827 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V827 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V827 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V827 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V827 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V827 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V827 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V827 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V827 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V827 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V827 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V827 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V827 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V827 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V827 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V827 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V827 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V827 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V827 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V827 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V827 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V827 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V827 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V827 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V827 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V827 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V827 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePosition: <V827 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V827 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V827 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V827 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V827 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V827 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V827 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V827 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V827 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V827 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V827 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V827 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V827 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V827 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V827 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V827 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V827 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V827 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V827 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V827 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V827 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V827 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V827 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V827 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V827 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V827 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V827 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V827 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V827 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V827 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V827 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V827 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V827 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V827 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V827 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V827 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V827 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V827 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V827 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V827 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V827 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V827 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V827 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V827 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V827 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V827 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V827 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V827 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V827 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V827 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V827 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V827 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V827 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V827 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V827 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V827 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V827 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V827 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V827 as ProtoVersionPackets>::UpdateTradePacket,
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
