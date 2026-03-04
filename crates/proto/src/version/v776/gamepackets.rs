use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V776;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    MovementPredictionSync: <V776 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    CameraAimAssistInstruction: <V776 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V776 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    SetMovementAuthority: <V776 as ProtoVersionPackets>::SetMovementAuthorityPacket,
    MovementEffect: <V776 as ProtoVersionPackets>::MovementEffectPacket,
    ContainerRegistryCleanup: <V776 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    CameraAimAssist: <V776 as ProtoVersionPackets>::CameraAimAssistPacket,
    ServerBoundLoadingScreen: <V776 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundDiagnostics: <V776 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    JigsawStructureData: <V776 as ProtoVersionPackets>::JigsawStructureDataPacket,
    CurrentStructureFeature: <V776 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    Login: <V776 as ProtoVersionPackets>::LoginPacket,
    PlayStatus: <V776 as ProtoVersionPackets>::PlayStatusPacket,
    ServerToClientHandshake: <V776 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    ClientToServerHandshake: <V776 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    Disconnect: <V776 as ProtoVersionPackets>::DisconnectPacket,
    ResourcePacksInfo: <V776 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    ResourcePackStack: <V776 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePackClientResponse: <V776 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    Text: <V776 as ProtoVersionPackets>::TextPacket,
    SetTime: <V776 as ProtoVersionPackets>::SetTimePacket,
    StartGame: <V776 as ProtoVersionPackets>::StartGamePacket,
    AddPlayer: <V776 as ProtoVersionPackets>::AddPlayerPacket,
    AddActor: <V776 as ProtoVersionPackets>::AddActorPacket,
    RemoveActor: <V776 as ProtoVersionPackets>::RemoveActorPacket,
    AddItemActor: <V776 as ProtoVersionPackets>::AddItemActorPacket,
    ClientBoundCloseForm: <V776 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ServerPlayerPostMovePosition: <V776 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    TakeItemActor: <V776 as ProtoVersionPackets>::TakeItemActorPacket,
    MoveActorAbsolute: <V776 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MovePlayer: <V776 as ProtoVersionPackets>::MovePlayerPacket,
    PassengerJump: <V776 as ProtoVersionPackets>::PassengerJumpPacket,
    UpdateBlock: <V776 as ProtoVersionPackets>::UpdateBlockPacket,
    AddPainting: <V776 as ProtoVersionPackets>::AddPaintingPacket,
    AwardAchievement: <V776 as ProtoVersionPackets>::AwardAchievementPacket,
    LevelSoundEventV1: <V776 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelEvent: <V776 as ProtoVersionPackets>::LevelEventPacket,
    BlockEvent: <V776 as ProtoVersionPackets>::BlockEventPacket,
    ActorEvent: <V776 as ProtoVersionPackets>::ActorEventPacket,
    MobEffect: <V776 as ProtoVersionPackets>::MobEffectPacket,
    UpdateAttributes: <V776 as ProtoVersionPackets>::UpdateAttributesPacket,
    InventoryTransaction: <V776 as ProtoVersionPackets>::InventoryTransactionPacket,
    MobEquipment: <V776 as ProtoVersionPackets>::MobEquipmentPacket,
    MobArmorEquipment: <V776 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    Interact: <V776 as ProtoVersionPackets>::InteractPacket,
    BlockPickRequest: <V776 as ProtoVersionPackets>::BlockPickRequestPacket,
    ActorPickRequest: <V776 as ProtoVersionPackets>::ActorPickRequestPacket,
    PlayerAction: <V776 as ProtoVersionPackets>::PlayerActionPacket,
    HurtArmor: <V776 as ProtoVersionPackets>::HurtArmorPacket,
    SetActorData: <V776 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorMotion: <V776 as ProtoVersionPackets>::SetActorMotionPacket,
    SetActorLink: <V776 as ProtoVersionPackets>::SetActorLinkPacket,
    SetHealth: <V776 as ProtoVersionPackets>::SetHealthPacket,
    SetSpawnPosition: <V776 as ProtoVersionPackets>::SetSpawnPositionPacket,
    Animate: <V776 as ProtoVersionPackets>::AnimatePacket,
    Respawn: <V776 as ProtoVersionPackets>::RespawnPacket,
    ContainerOpen: <V776 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerClose: <V776 as ProtoVersionPackets>::ContainerClosePacket,
    PlayerHotbar: <V776 as ProtoVersionPackets>::PlayerHotbarPacket,
    InventoryContent: <V776 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V776 as ProtoVersionPackets>::InventorySlotPacket,
    ContainerSetData: <V776 as ProtoVersionPackets>::ContainerSetDataPacket,
    CraftingData: <V776 as ProtoVersionPackets>::CraftingDataPacket,
    GuiDataPickItem: <V776 as ProtoVersionPackets>::GuiDataPickItemPacket,
    BlockActorData: <V776 as ProtoVersionPackets>::BlockActorDataPacket,
    PlayerInput: <V776 as ProtoVersionPackets>::PlayerInputPacket,
    LevelChunk: <V776 as ProtoVersionPackets>::LevelChunkPacket,
    SetCommandsEnabled: <V776 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDifficulty: <V776 as ProtoVersionPackets>::SetDifficultyPacket,
    ChangeDimension: <V776 as ProtoVersionPackets>::ChangeDimensionPacket,
    SetPlayerGameType: <V776 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    PlayerList: <V776 as ProtoVersionPackets>::PlayerListPacket,
    SimpleEvent: <V776 as ProtoVersionPackets>::SimpleEventPacket,
    LegacyTelemetryEvent: <V776 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    SpawnExperienceOrb: <V776 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    ClientboundMapItemData: <V776 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    MapInfoRequest: <V776 as ProtoVersionPackets>::MapInfoRequestPacket,
    RequestChunkRadius: <V776 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    ChunkRadiusUpdated: <V776 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    GameRulesChanged: <V776 as ProtoVersionPackets>::GameRulesChangedPacket,
    Camera: <V776 as ProtoVersionPackets>::CameraPacket,
    BossEvent: <V776 as ProtoVersionPackets>::BossEventPacket,
    ShowCredits: <V776 as ProtoVersionPackets>::ShowCreditsPacket,
    AvailableCommands: <V776 as ProtoVersionPackets>::AvailableCommandsPacket,
    CommandRequest: <V776 as ProtoVersionPackets>::CommandRequestPacket,
    CommandBlockUpdate: <V776 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V776 as ProtoVersionPackets>::CommandOutputPacket,
    UpdateTrade: <V776 as ProtoVersionPackets>::UpdateTradePacket,
    UpdateEquip: <V776 as ProtoVersionPackets>::UpdateEquipPacket,
    ResourcePackDataInfo: <V776 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackChunkData: <V776 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V776 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    TransferPlayer: <V776 as ProtoVersionPackets>::TransferPlayerPacket,
    PlaySound: <V776 as ProtoVersionPackets>::PlaySoundPacket,
    StopSound: <V776 as ProtoVersionPackets>::StopSoundPacket,
    SetTitle: <V776 as ProtoVersionPackets>::SetTitlePacket,
    AddBehaviourTree: <V776 as ProtoVersionPackets>::AddBehaviourTreePacket,
    StructureBlockUpdate: <V776 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    ShowStoreOffer: <V776 as ProtoVersionPackets>::ShowStoreOfferPacket,
    PurchaseReceipt: <V776 as ProtoVersionPackets>::PurchaseReceiptPacket,
    PlayerSkin: <V776 as ProtoVersionPackets>::PlayerSkinPacket,
    SubClientLogin: <V776 as ProtoVersionPackets>::SubClientLoginPacket,
    AutomationClientConnect: <V776 as ProtoVersionPackets>::AutomationClientConnectPacket,
    SetLastHurtBy: <V776 as ProtoVersionPackets>::SetLastHurtByPacket,
    BookEdit: <V776 as ProtoVersionPackets>::BookEditPacket,
    NpcRequest: <V776 as ProtoVersionPackets>::NpcRequestPacket,
    PhotoTransfer: <V776 as ProtoVersionPackets>::PhotoTransferPacket,
    ModalFormRequest: <V776 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V776 as ProtoVersionPackets>::ModalFormResponsePacket,
    ServerSettingsRequest: <V776 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V776 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ShowProfile: <V776 as ProtoVersionPackets>::ShowProfilePacket,
    SetDefaultGameType: <V776 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    RemoveObjective: <V776 as ProtoVersionPackets>::RemoveObjectivePacket,
    SetDisplayObjective: <V776 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetScore: <V776 as ProtoVersionPackets>::SetScorePacket,
    LabTable: <V776 as ProtoVersionPackets>::LabTablePacket,
    UpdateBlockSynced: <V776 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    MoveActorDelta: <V776 as ProtoVersionPackets>::MoveActorDeltaPacket,
    SetScoreboardIdentity: <V776 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetLocalPlayerAsInitialized: <V776 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    UpdateSoftEnum: <V776 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    NetworkStackLatency: <V776 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    SpawnParticleEffect: <V776 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    AvailableActorIdentifiers: <V776 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    LevelSoundEventV2: <V776 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    NetworkChunkPublisherUpdate: <V776 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    BiomeDefinitionList: <V776 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    LevelSoundEvent: <V776 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelEventGeneric: <V776 as ProtoVersionPackets>::LevelEventGenericPacket,
    LecternUpdate: <V776 as ProtoVersionPackets>::LecternUpdatePacket,
    ClientCacheStatus: <V776 as ProtoVersionPackets>::ClientCacheStatusPacket,
    OnScreenTextureAnimation: <V776 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    MapCreateLockedCopy: <V776 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    StructureDataRequest: <V776 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V776 as ProtoVersionPackets>::StructureDataResponsePacket,
    ClientCacheBlobStatus: <V776 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V776 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    EducationSettings: <V776 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V776 as ProtoVersionPackets>::EmotePacket,
    MultiplayerSettings: <V776 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    SettingsCommand: <V776 as ProtoVersionPackets>::SettingsCommandPacket,
    AnvilDamage: <V776 as ProtoVersionPackets>::AnvilDamagePacket,
    CompletedUsingItem: <V776 as ProtoVersionPackets>::CompletedUsingItemPacket,
    NetworkSettings: <V776 as ProtoVersionPackets>::NetworkSettingsPacket,
    PlayerAuthInput: <V776 as ProtoVersionPackets>::PlayerAuthInputPacket,
    CreativeContent: <V776 as ProtoVersionPackets>::CreativeContentPacket,
    PlayerEnchantOptions: <V776 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    ItemStackRequest: <V776 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V776 as ProtoVersionPackets>::ItemStackResponsePacket,
    PlayerArmorDamage: <V776 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    CodeBuilder: <V776 as ProtoVersionPackets>::CodeBuilderPacket,
    UpdatePlayerGameType: <V776 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    EmoteList: <V776 as ProtoVersionPackets>::EmoteListPacket,
    PositionTrackingDbServerBroadcast: <V776 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PositionTrackingDbClientRequest: <V776 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    DebugInfo: <V776 as ProtoVersionPackets>::DebugInfoPacket,
    PacketViolationWarning: <V776 as ProtoVersionPackets>::PacketViolationWarningPacket,
    MotionPredictionHints: <V776 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    AnimateEntity: <V776 as ProtoVersionPackets>::AnimateEntityPacket,
    CameraShake: <V776 as ProtoVersionPackets>::CameraShakePacket,
    PlayerFog: <V776 as ProtoVersionPackets>::PlayerFogPacket,
    CorrectPlayerMovePrediction: <V776 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    ItemComponent: <V776 as ProtoVersionPackets>::ItemComponentPacket,
    ClientboundDebugRenderer: <V776 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    SyncActorProperty: <V776 as ProtoVersionPackets>::SyncActorPropertyPacket,
    AddVolumeEntity: <V776 as ProtoVersionPackets>::AddVolumeEntityPacket,
    RemoveVolumeEntity: <V776 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    SimulationType: <V776 as ProtoVersionPackets>::SimulationTypePacket,
    NpcDialogue: <V776 as ProtoVersionPackets>::NpcDialoguePacket,
    EduUriResource: <V776 as ProtoVersionPackets>::EduUriResourcePacket,
    CreatePhoto: <V776 as ProtoVersionPackets>::CreatePhotoPacket,
    UpdateSubChunkBlocks: <V776 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    SubChunk: <V776 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V776 as ProtoVersionPackets>::SubChunkRequestPacket,
    PlayerStartItemCooldown: <V776 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    ScriptMessage: <V776 as ProtoVersionPackets>::ScriptMessagePacket,
    CodeBuilderSource: <V776 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    TickingAreaLoadStatus: <V776 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    DimensionData: <V776 as ProtoVersionPackets>::DimensionDataPacket,
    AgentActionEvent: <V776 as ProtoVersionPackets>::AgentActionEventPacket,
    ChangeMobProperty: <V776 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    LessonProgress: <V776 as ProtoVersionPackets>::LessonProgressPacket,
    RequestAbility: <V776 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestPermissions: <V776 as ProtoVersionPackets>::RequestPermissionsPacket,
    ToastRequest: <V776 as ProtoVersionPackets>::ToastRequestPacket,
    UpdateAbilities: <V776 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V776 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    DeathInfo: <V776 as ProtoVersionPackets>::DeathInfoPacket,
    EditorNetwork: <V776 as ProtoVersionPackets>::EditorNetworkPacket,
    FeatureRegistry: <V776 as ProtoVersionPackets>::FeatureRegistryPacket,
    ServerStats: <V776 as ProtoVersionPackets>::ServerStatsPacket,
    RequestNetworkSettings: <V776 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    GameTestRequest: <V776 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V776 as ProtoVersionPackets>::GameTestResultsPacket,
    UpdateClientInputLocks: <V776 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    CameraPresets: <V776 as ProtoVersionPackets>::CameraPresetsPacket,
    UnlockedRecipes: <V776 as ProtoVersionPackets>::UnlockedRecipesPacket,
    CameraInstruction: <V776 as ProtoVersionPackets>::CameraInstructionPacket,
    CompressedBiomeDefinitionList: <V776 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    TrimData: <V776 as ProtoVersionPackets>::TrimDataPacket,
    OpenSign: <V776 as ProtoVersionPackets>::OpenSignPacket,
    AgentAnimation: <V776 as ProtoVersionPackets>::AgentAnimationPacket,
    RefreshEntitlements: <V776 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    PlayerToggleCrafterSlotRequest: <V776 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    SetPlayerInventoryOptions: <V776 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetHud: <V776 as ProtoVersionPackets>::SetHudPacket
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
