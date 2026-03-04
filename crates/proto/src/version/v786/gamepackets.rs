use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V786;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V786 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V786 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V786 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V786 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V786 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V786 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V786 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V786 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V786 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V786 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V786 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V786 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V786 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V786 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V786 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V786 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V786 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V786 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V786 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V786 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V786 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V786 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V786 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V786 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V786 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V786 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V786 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V786 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V786 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V786 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V786 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V786 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V786 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V786 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientCacheBlobStatus: <V786 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V786 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V786 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V786 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V786 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V786 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V786 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V786 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V786 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V786 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V786 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V786 as ProtoVersionPackets>::CompletedUsingItemPacket,
    CompressedBiomeDefinitionList: <V786 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    ContainerClose: <V786 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V786 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V786 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V786 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V786 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V786 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V786 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V786 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V786 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V786 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V786 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V786 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V786 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V786 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V786 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V786 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V786 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V786 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V786 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V786 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V786 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V786 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V786 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V786 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V786 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V786 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V786 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V786 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V786 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V786 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V786 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V786 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V786 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V786 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V786 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V786 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V786 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V786 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V786 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V786 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V786 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V786 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V786 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V786 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V786 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V786 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V786 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V786 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V786 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V786 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V786 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V786 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V786 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V786 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V786 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V786 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V786 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V786 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V786 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V786 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V786 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V786 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V786 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PassengerJump: <V786 as ProtoVersionPackets>::PassengerJumpPacket,
    PhotoTransfer: <V786 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V786 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V786 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V786 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V786 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V786 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V786 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V786 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V786 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerInput: <V786 as ProtoVersionPackets>::PlayerInputPacket,
    PlayerList: <V786 as ProtoVersionPackets>::PlayerListPacket,
    PlayerSkin: <V786 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V786 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V786 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V786 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V786 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V786 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V786 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V786 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V786 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V786 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V786 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V786 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V786 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V786 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V786 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V786 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V786 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V786 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V786 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V786 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V786 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V786 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V786 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V786 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V786 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V786 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePosition: <V786 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V786 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V786 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V786 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V786 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V786 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V786 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V786 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V786 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V786 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V786 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V786 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V786 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V786 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V786 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V786 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetMovementAuthority: <V786 as ProtoVersionPackets>::SetMovementAuthorityPacket,
    SetPlayerGameType: <V786 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V786 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V786 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V786 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V786 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V786 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V786 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V786 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V786 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V786 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V786 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V786 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V786 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V786 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V786 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V786 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V786 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V786 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V786 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V786 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V786 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V786 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V786 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V786 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V786 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V786 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V786 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V786 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V786 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V786 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V786 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V786 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V786 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V786 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V786 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V786 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V786 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V786 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V786 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V786 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V786 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V786 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V786 as ProtoVersionPackets>::UpdateTradePacket,
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
