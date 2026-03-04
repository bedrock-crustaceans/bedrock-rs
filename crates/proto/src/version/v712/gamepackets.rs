use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V712;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V712 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V712 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V712 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V712 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V712 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V712 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V712 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V712 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V712 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V712 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V712 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V712 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V712 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V712 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V712 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V712 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievementPacket: <V712 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V712 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V712 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V712 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V712 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V712 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V712 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V712 as ProtoVersionPackets>::CameraPacket,
    CameraInstruction: <V712 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V712 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V712 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V712 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V712 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V712 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientCacheBlobStatus: <V712 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V712 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V712 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V712 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientBoundCloseForm: <V712 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientboundDebugRenderer: <V712 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V712 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V712 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V712 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V712 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V712 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V712 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V712 as ProtoVersionPackets>::CompletedUsingItemPacket,
    CompressedBiomeDefinitionList: <V712 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    ContainerClose: <V712 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V712 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerSetData: <V712 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V712 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V712 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V712 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V712 as ProtoVersionPackets>::CreativeContentPacket,
    CurrentStructureFeature: <V712 as ProtoVersionPackets>::CurrentStructureFeaturePacket,
    DeathInfo: <V712 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V712 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V712 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V712 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V712 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V712 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V712 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V712 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V712 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V712 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V712 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V712 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V712 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V712 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V712 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V712 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V712 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V712 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V712 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V712 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V712 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V712 as ProtoVersionPackets>::ItemStackResponsePacket,
    JigsawStructureData: <V712 as ProtoVersionPackets>::JigsawStructureDataPacket,
    LabTable: <V712 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V712 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V712 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V712 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V712 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V712 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V712 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V712 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelSoundEventV1: <V712 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelSoundEventV2: <V712 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    Login: <V712 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V712 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V712 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V712 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V712 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V712 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V712 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V712 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V712 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V712 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V712 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V712 as ProtoVersionPackets>::MovePlayerPacket,
    MultiplayerSettings: <V712 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V712 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V712 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V712 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V712 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V712 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V712 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V712 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V712 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PassengerJump: <V712 as ProtoVersionPackets>::PassengerJumpPacket,
    PhotoTransfer: <V712 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V712 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V712 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V712 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V712 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V712 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V712 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V712 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V712 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerInput: <V712 as ProtoVersionPackets>::PlayerInputPacket,
    PlayerList: <V712 as ProtoVersionPackets>::PlayerListPacket,
    PlayerSkin: <V712 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V712 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V712 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PositionTrackingDbClientRequest: <V712 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V712 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V712 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V712 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V712 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V712 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V712 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V712 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V712 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V712 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V712 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V712 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V712 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V712 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V712 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V712 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V712 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V712 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V712 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerBoundDiagnostics: <V712 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket,
    ServerBoundLoadingScreen: <V712 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePosition: <V712 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V712 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V712 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V712 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V712 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V712 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V712 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V712 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V712 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V712 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V712 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V712 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V712 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V712 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V712 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V712 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V712 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V712 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V712 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V712 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V712 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V712 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V712 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V712 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V712 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V712 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V712 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V712 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V712 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V712 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V712 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V712 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V712 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V712 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V712 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V712 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V712 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V712 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V712 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V712 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V712 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V712 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V712 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V712 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V712 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V712 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V712 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V712 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V712 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V712 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V712 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V712 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V712 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateEquip: <V712 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V712 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V712 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V712 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V712 as ProtoVersionPackets>::UpdateTradePacket,
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
