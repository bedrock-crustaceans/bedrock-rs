pub use crate::version::proto_version::ProtoVersionPackets;
pub use crate::version::proto_version::V800;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V800 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V800 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V800 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V800 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V800 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V800 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V800 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V800 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V800 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V800 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V800 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V800 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V800 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V800 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V800 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V800 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievement: <V800 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V800 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V800 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V800 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V800 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V800 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V800 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V800 as ProtoVersionPackets>::CameraPacket,
    CameraAimAssist: <V800 as ProtoVersionPackets>::CameraAimAssistPacket,
    CameraAimAssistInstruction: <V800 as ProtoVersionPackets>::CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: <V800 as ProtoVersionPackets>::CameraAimAssistPresetsPacket,
    CameraInstruction: <V800 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V800 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V800 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V800 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V800 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V800 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientBoundCloseForm: <V800 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientBoundControlSchemeSet: <V800 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket,
    ClientCacheBlobStatus: <V800 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V800 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V800 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V800 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V800 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V800 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V800 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V800 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V800 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V800 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V800 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V800 as ProtoVersionPackets>::CompletedUsingItemPacket,
    ContainerClose: <V800 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V800 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerRegistryCleanup: <V800 as ProtoVersionPackets>::ContainerRegistryCleanupPacket,
    ContainerSetData: <V800 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V800 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V800 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V800 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V800 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V800 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V800 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V800 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V800 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V800 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V800 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V800 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V800 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V800 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V800 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V800 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V800 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V800 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V800 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V800 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V800 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V800 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V800 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V800 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V800 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V800 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V800 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V800 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V800 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V800 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V800 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V800 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V800 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V800 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V800 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V800 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V800 as ProtoVersionPackets>::LevelSoundEventPacket,
    Login: <V800 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V800 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V800 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V800 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V800 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V800 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V800 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V800 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V800 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V800 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V800 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V800 as ProtoVersionPackets>::MovePlayerPacket,
    MovementEffect: <V800 as ProtoVersionPackets>::MovementEffectPacket,
    MovementPredictionSync: <V800 as ProtoVersionPackets>::MovementPredictionSyncPacket,
    MultiplayerSettings: <V800 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V800 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V800 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V800 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V800 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V800 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V800 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V800 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V800 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PhotoTransfer: <V800 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V800 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V800 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V800 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V800 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V800 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V800 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V800 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V800 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerList: <V800 as ProtoVersionPackets>::PlayerListPacket,
    PlayerLocation: <V800 as ProtoVersionPackets>::PlayerLocationPacket,
    PlayerSkin: <V800 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V800 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V800 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverrides: <V800 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket,
    PlayerVideoCapture: <V800 as ProtoVersionPackets>::PlayerVideoCapturePacket,
    PositionTrackingDbClientRequest: <V800 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V800 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V800 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V800 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V800 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V800 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V800 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V800 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V800 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V800 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V800 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V800 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V800 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V800 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V800 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V800 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V800 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V800 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V800 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V800 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V800 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePosition: <V800 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V800 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V800 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V800 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V800 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V800 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V800 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V800 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V800 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V800 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V800 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V800 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V800 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V800 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V800 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V800 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetMovementAuthority: <V800 as ProtoVersionPackets>::SetMovementAuthorityPacket,
    SetPlayerGameType: <V800 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V800 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V800 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V800 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V800 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V800 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V800 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V800 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V800 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V800 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V800 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V800 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V800 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V800 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V800 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V800 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V800 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V800 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V800 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V800 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V800 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V800 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V800 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V800 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V800 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V800 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V800 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V800 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V800 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V800 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V800 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V800 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V800 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V800 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V800 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V800 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V800 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateClientOptions: <V800 as ProtoVersionPackets>::UpdateClientOptionsPacket,
    UpdateEquip: <V800 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V800 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V800 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V800 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V800 as ProtoVersionPackets>::UpdateTradePacket,
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
