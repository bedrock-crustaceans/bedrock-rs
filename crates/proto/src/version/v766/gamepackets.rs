use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V766;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    CameraAimAssistPresets: <V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    SetMovementAuthority: <V766 as ProtoVersionPackets>::SetMovementAuthorityPacket,
    MovementEffect: <V766 as ProtoVersionPackets>::MovementEffectPacket,
    ContainerRegistryCleanup: <V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    CameraAimAssist: <V766 as ProtoVersionPackets>::CameraAimAssistPacket,
    ServerBoundLoadingScreen: <V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerBoundDiagnostics: <V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    JigsawStructureData: <V766 as ProtoVersionPackets>::JigsawStructureDataPacket,
    CurrentStructureFeature: <V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    Login: <V766 as ProtoVersionPackets>::LoginPacket,
    PlayStatus: <V766 as ProtoVersionPackets>::PlayStatusPacket,
    ServerToClientHandshake: <V766 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    ClientToServerHandshake: <V766 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    Disconnect: <V766 as ProtoVersionPackets>::DisconnectPacket,
    ResourcePacksInfo: <V766 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    ResourcePackStack: <V766 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePackClientResponse: <V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    Text: <V766 as ProtoVersionPackets>::TextPacket,
    SetTime: <V766 as ProtoVersionPackets>::SetTimePacket,
    StartGame: <V766 as ProtoVersionPackets>::StartGamePacket,
    AddPlayer: <V766 as ProtoVersionPackets>::AddPlayerPacket,
    AddActor: <V766 as ProtoVersionPackets>::AddActorPacket,
    RemoveActor: <V766 as ProtoVersionPackets>::RemoveActorPacket,
    AddItemActor: <V766 as ProtoVersionPackets>::AddItemActorPacket,
    ClientBoundCloseForm: <V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ServerPlayerPostMovePosition: <V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    TakeItemActor: <V766 as ProtoVersionPackets>::TakeItemActorPacket,
    MoveActorAbsolute: <V766 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MovePlayer: <V766 as ProtoVersionPackets>::MovePlayerPacket,
    PassengerJump: <V766 as ProtoVersionPackets>::PassengerJumpPacket,
    UpdateBlock: <V766 as ProtoVersionPackets>::UpdateBlockPacket,
    AddPainting: <V766 as ProtoVersionPackets>::AddPaintingPacket,
    AwardAchievement: <V766 as ProtoVersionPackets>::AwardAchievementPacket,
    LevelSoundEventV1: <V766 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelEvent: <V766 as ProtoVersionPackets>::LevelEventPacket,
    BlockEvent: <V766 as ProtoVersionPackets>::BlockEventPacket,
    ActorEvent: <V766 as ProtoVersionPackets>::ActorEventPacket,
    MobEffect: <V766 as ProtoVersionPackets>::MobEffectPacket,
    UpdateAttributes: <V766 as ProtoVersionPackets>::UpdateAttributesPacket,
    InventoryTransaction: <V766 as ProtoVersionPackets>::InventoryTransactionPacket,
    MobEquipment: <V766 as ProtoVersionPackets>::MobEquipmentPacket,
    MobArmorEquipment: <V766 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    Interact: <V766 as ProtoVersionPackets>::InteractPacket,
    BlockPickRequest: <V766 as ProtoVersionPackets>::BlockPickRequestPacket,
    ActorPickRequest: <V766 as ProtoVersionPackets>::ActorPickRequestPacket,
    PlayerAction: <V766 as ProtoVersionPackets>::PlayerActionPacket,
    HurtArmor: <V766 as ProtoVersionPackets>::HurtArmorPacket,
    SetActorData: <V766 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorMotion: <V766 as ProtoVersionPackets>::SetActorMotionPacket,
    SetActorLink: <V766 as ProtoVersionPackets>::SetActorLinkPacket,
    SetHealth: <V766 as ProtoVersionPackets>::SetHealthPacket,
    SetSpawnPosition: <V766 as ProtoVersionPackets>::SetSpawnPositionPacket,
    Animate: <V766 as ProtoVersionPackets>::AnimatePacket,
    Respawn: <V766 as ProtoVersionPackets>::RespawnPacket,
    ContainerOpen: <V766 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerClose: <V766 as ProtoVersionPackets>::ContainerClosePacket,
    PlayerHotbar: <V766 as ProtoVersionPackets>::PlayerHotbarPacket,
    InventoryContent: <V766 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V766 as ProtoVersionPackets>::InventorySlotPacket,
    ContainerSetData: <V766 as ProtoVersionPackets>::ContainerSetDataPacket,
    CraftingData: <V766 as ProtoVersionPackets>::CraftingDataPacket,
    GuiDataPickItem: <V766 as ProtoVersionPackets>::GuiDataPickItemPacket,
    BlockActorData: <V766 as ProtoVersionPackets>::BlockActorDataPacket,
    PlayerInput: <V766 as ProtoVersionPackets>::PlayerInputPacket,
    LevelChunk: <V766 as ProtoVersionPackets>::LevelChunkPacket,
    SetCommandsEnabled: <V766 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDifficulty: <V766 as ProtoVersionPackets>::SetDifficultyPacket,
    ChangeDimension: <V766 as ProtoVersionPackets>::ChangeDimensionPacket,
    SetPlayerGameType: <V766 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    PlayerList: <V766 as ProtoVersionPackets>::PlayerListPacket,
    SimpleEvent: <V766 as ProtoVersionPackets>::SimpleEventPacket,
    LegacyTelemetryEvent: <V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    SpawnExperienceOrb: <V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    ClientboundMapItemData: <V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    MapInfoRequest: <V766 as ProtoVersionPackets>::MapInfoRequestPacket,
    RequestChunkRadius: <V766 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    ChunkRadiusUpdated: <V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    GameRulesChanged: <V766 as ProtoVersionPackets>::GameRulesChangedPacket,
    Camera: <V766 as ProtoVersionPackets>::CameraPacket,
    BossEvent: <V766 as ProtoVersionPackets>::BossEventPacket,
    ShowCredits: <V766 as ProtoVersionPackets>::ShowCreditsPacket,
    AvailableCommands: <V766 as ProtoVersionPackets>::AvailableCommandsPacket,
    CommandRequest: <V766 as ProtoVersionPackets>::CommandRequestPacket,
    CommandBlockUpdate: <V766 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V766 as ProtoVersionPackets>::CommandOutputPacket,
    UpdateTrade: <V766 as ProtoVersionPackets>::UpdateTradePacket,
    UpdateEquip: <V766 as ProtoVersionPackets>::UpdateEquipPacket,
    ResourcePackDataInfo: <V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackChunkData: <V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    TransferPlayer: <V766 as ProtoVersionPackets>::TransferPlayerPacket,
    PlaySound: <V766 as ProtoVersionPackets>::PlaySoundPacket,
    StopSound: <V766 as ProtoVersionPackets>::StopSoundPacket,
    SetTitle: <V766 as ProtoVersionPackets>::SetTitlePacket,
    AddBehaviourTree: <V766 as ProtoVersionPackets>::AddBehaviourTreePacket,
    StructureBlockUpdate: <V766 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    ShowStoreOffer: <V766 as ProtoVersionPackets>::ShowStoreOfferPacket,
    PurchaseReceipt: <V766 as ProtoVersionPackets>::PurchaseReceiptPacket,
    PlayerSkin: <V766 as ProtoVersionPackets>::PlayerSkinPacket,
    SubClientLogin: <V766 as ProtoVersionPackets>::SubClientLoginPacket,
    AutomationClientConnect: <V766 as ProtoVersionPackets>::AutomationClientConnectPacket,
    SetLastHurtBy: <V766 as ProtoVersionPackets>::SetLastHurtByPacket,
    BookEdit: <V766 as ProtoVersionPackets>::BookEditPacket,
    NpcRequest: <V766 as ProtoVersionPackets>::NpcRequestPacket,
    PhotoTransfer: <V766 as ProtoVersionPackets>::PhotoTransferPacket,
    ModalFormRequest: <V766 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V766 as ProtoVersionPackets>::ModalFormResponsePacket,
    ServerSettingsRequest: <V766 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V766 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ShowProfile: <V766 as ProtoVersionPackets>::ShowProfilePacket,
    SetDefaultGameType: <V766 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    RemoveObjective: <V766 as ProtoVersionPackets>::RemoveObjectivePacket,
    SetDisplayObjective: <V766 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetScore: <V766 as ProtoVersionPackets>::SetScorePacket,
    LabTable: <V766 as ProtoVersionPackets>::LabTablePacket,
    UpdateBlockSynced: <V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    MoveActorDelta: <V766 as ProtoVersionPackets>::MoveActorDeltaPacket,
    SetScoreboardIdentity: <V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetLocalPlayerAsInitialized: <V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    UpdateSoftEnum: <V766 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    NetworkStackLatency: <V766 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    SpawnParticleEffect: <V766 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    AvailableActorIdentifiers: <V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    LevelSoundEventV2: <V766 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    NetworkChunkPublisherUpdate: <V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    BiomeDefinitionList: <V766 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    LevelSoundEvent: <V766 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelEventGeneric: <V766 as ProtoVersionPackets>::LevelEventGenericPacket,
    LecternUpdate: <V766 as ProtoVersionPackets>::LecternUpdatePacket,
    ClientCacheStatus: <V766 as ProtoVersionPackets>::ClientCacheStatusPacket,
    OnScreenTextureAnimation: <V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    MapCreateLockedCopy: <V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    StructureDataRequest: <V766 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V766 as ProtoVersionPackets>::StructureDataResponsePacket,
    ClientCacheBlobStatus: <V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    EducationSettings: <V766 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V766 as ProtoVersionPackets>::EmotePacket,
    MultiplayerSettings: <V766 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    SettingsCommand: <V766 as ProtoVersionPackets>::SettingsCommandPacket,
    AnvilDamage: <V766 as ProtoVersionPackets>::AnvilDamagePacket,
    CompletedUsingItem: <V766 as ProtoVersionPackets>::CompletedUsingItemPacket,
    NetworkSettings: <V766 as ProtoVersionPackets>::NetworkSettingsPacket,
    PlayerAuthInput: <V766 as ProtoVersionPackets>::PlayerAuthInputPacket,
    CreativeContent: <V766 as ProtoVersionPackets>::CreativeContentPacket,
    PlayerEnchantOptions: <V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    ItemStackRequest: <V766 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V766 as ProtoVersionPackets>::ItemStackResponsePacket,
    PlayerArmorDamage: <V766 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    CodeBuilder: <V766 as ProtoVersionPackets>::CodeBuilderPacket,
    UpdatePlayerGameType: <V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    EmoteList: <V766 as ProtoVersionPackets>::EmoteListPacket,
    PositionTrackingDbServerBroadcast: <V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PositionTrackingDbClientRequest: <V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    DebugInfo: <V766 as ProtoVersionPackets>::DebugInfoPacket,
    PacketViolationWarning: <V766 as ProtoVersionPackets>::PacketViolationWarningPacket,
    MotionPredictionHints: <V766 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    AnimateEntity: <V766 as ProtoVersionPackets>::AnimateEntityPacket,
    CameraShake: <V766 as ProtoVersionPackets>::CameraShakePacket,
    PlayerFog: <V766 as ProtoVersionPackets>::PlayerFogPacket,
    CorrectPlayerMovePrediction: <V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    ItemComponent: <V766 as ProtoVersionPackets>::ItemComponentPacket,
    ClientboundDebugRenderer: <V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    SyncActorProperty: <V766 as ProtoVersionPackets>::SyncActorPropertyPacket,
    AddVolumeEntity: <V766 as ProtoVersionPackets>::AddVolumeEntityPacket,
    RemoveVolumeEntity: <V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    SimulationType: <V766 as ProtoVersionPackets>::SimulationTypePacket,
    NpcDialogue: <V766 as ProtoVersionPackets>::NpcDialoguePacket,
    EduUriResource: <V766 as ProtoVersionPackets>::EduUriResourcePacket,
    CreatePhoto: <V766 as ProtoVersionPackets>::CreatePhotoPacket,
    UpdateSubChunkBlocks: <V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    SubChunk: <V766 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V766 as ProtoVersionPackets>::SubChunkRequestPacket,
    PlayerStartItemCooldown: <V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    ScriptMessage: <V766 as ProtoVersionPackets>::ScriptMessagePacket,
    CodeBuilderSource: <V766 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    TickingAreaLoadStatus: <V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    DimensionData: <V766 as ProtoVersionPackets>::DimensionDataPacket,
    AgentActionEvent: <V766 as ProtoVersionPackets>::AgentActionEventPacket,
    ChangeMobProperty: <V766 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    LessonProgress: <V766 as ProtoVersionPackets>::LessonProgressPacket,
    RequestAbility: <V766 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestPermissions: <V766 as ProtoVersionPackets>::RequestPermissionsPacket,
    ToastRequest: <V766 as ProtoVersionPackets>::ToastRequestPacket,
    UpdateAbilities: <V766 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    DeathInfo: <V766 as ProtoVersionPackets>::DeathInfoPacket,
    EditorNetwork: <V766 as ProtoVersionPackets>::EditorNetworkPacket,
    FeatureRegistry: <V766 as ProtoVersionPackets>::FeatureRegistryPacket,
    ServerStats: <V766 as ProtoVersionPackets>::ServerStatsPacket,
    RequestNetworkSettings: <V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    GameTestRequest: <V766 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V766 as ProtoVersionPackets>::GameTestResultsPacket,
    UpdateClientInputLocks: <V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    CameraPresets: <V766 as ProtoVersionPackets>::CameraPresetsPacket,
    UnlockedRecipes: <V766 as ProtoVersionPackets>::UnlockedRecipesPacket,
    CameraInstruction: <V766 as ProtoVersionPackets>::CameraInstructionPacket,
    CompressedBiomeDefinitionList: <V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    TrimData: <V766 as ProtoVersionPackets>::TrimDataPacket,
    OpenSign: <V766 as ProtoVersionPackets>::OpenSignPacket,
    AgentAnimation: <V766 as ProtoVersionPackets>::AgentAnimationPacket,
    RefreshEntitlements: <V766 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    PlayerToggleCrafterSlotRequest: <V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    SetPlayerInventoryOptions: <V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetHud: <V766 as ProtoVersionPackets>::SetHudPacket
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
