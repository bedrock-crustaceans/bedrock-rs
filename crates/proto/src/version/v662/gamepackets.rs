use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V662;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V662 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V662 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V662 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V662 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V662 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V662 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V662 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V662 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V662 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V662 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V662 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V662 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V662 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V662 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V662 as ProtoVersionPackets>::AvailableCommandsPacket,
    BiomeDefinitionList: <V662 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V662 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V662 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V662 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V662 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V662 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V662 as ProtoVersionPackets>::CameraPacket,
    CameraInstruction: <V662 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V662 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V662 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V662 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V662 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientCacheBlobStatus: <V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V662 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V662 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V662 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V662 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V662 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V662 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V662 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V662 as ProtoVersionPackets>::CompletedUsingItemPacket,
    CompressedBiomeDefinitionList: <V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    ContainerClose: <V662 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V662 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerSetData: <V662 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V662 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V662 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V662 as ProtoVersionPackets>::CreativeContentPacket,
    DeathInfo: <V662 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V662 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V662 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V662 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V662 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V662 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V662 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V662 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V662 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V662 as ProtoVersionPackets>::FeatureRegistryPacket,
    FilterText: <V662 as ProtoVersionPackets>::FilterTextPacket,
    GameRulesChanged: <V662 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V662 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V662 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V662 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V662 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V662 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V662 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V662 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V662 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V662 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V662 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V662 as ProtoVersionPackets>::ItemStackResponsePacket,
    LabTable: <V662 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V662 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V662 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V662 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V662 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V662 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V662 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelSoundEventV1: <V662 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelSoundEventV2: <V662 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    Login: <V662 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V662 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V662 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V662 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V662 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V662 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V662 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V662 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V662 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V662 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V662 as ProtoVersionPackets>::MovePlayerPacket,
    MultiplayerSettings: <V662 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V662 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V662 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V662 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V662 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V662 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V662 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PassengerJump: <V662 as ProtoVersionPackets>::PassengerJumpPacket,
    PhotoTransfer: <V662 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V662 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V662 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V662 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V662 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V662 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V662 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V662 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerInput: <V662 as ProtoVersionPackets>::PlayerInputPacket,
    PlayerList: <V662 as ProtoVersionPackets>::PlayerListPacket,
    PlayerSkin: <V662 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PositionTrackingDbClientRequest: <V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V662 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V662 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V662 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V662 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V662 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V662 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V662 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V662 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V662 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V662 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V662 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerPlayerPostMovePosition: <V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V662 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V662 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V662 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V662 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V662 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V662 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V662 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V662 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V662 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V662 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V662 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V662 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V662 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V662 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V662 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V662 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V662 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V662 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V662 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V662 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V662 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V662 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V662 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V662 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V662 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V662 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V662 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V662 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V662 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V662 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V662 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V662 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V662 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V662 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V662 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V662 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V662 as ProtoVersionPackets>::TextPacket,
    TickSync: <V662 as ProtoVersionPackets>::TickSyncPacket,
    TickingAreaLoadStatus: <V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V662 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V662 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V662 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V662 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V662 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V662 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V662 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateEquip: <V662 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V662 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V662 as ProtoVersionPackets>::UpdateTradePacket,
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
