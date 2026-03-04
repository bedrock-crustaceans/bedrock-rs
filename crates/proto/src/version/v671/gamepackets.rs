use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V671;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V671 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V671 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V671 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V671 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V671 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V671 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V671 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V671 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V671 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V671 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V671 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V671 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V671 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V671 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V671 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V671 as ProtoVersionPackets>::AvailableCommandsPacket,
    BiomeDefinitionList: <V671 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V671 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V671 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V671 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V671 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V671 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V671 as ProtoVersionPackets>::CameraPacket,
    CameraInstruction: <V671 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V671 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V671 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V671 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V671 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V671 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientCacheBlobStatus: <V671 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V671 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V671 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V671 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientboundDebugRenderer: <V671 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V671 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V671 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V671 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V671 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V671 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V671 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V671 as ProtoVersionPackets>::CompletedUsingItemPacket,
    CompressedBiomeDefinitionList: <V671 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    ContainerClose: <V671 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V671 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerSetData: <V671 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V671 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V671 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V671 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V671 as ProtoVersionPackets>::CreativeContentPacket,
    DeathInfo: <V671 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V671 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V671 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V671 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V671 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V671 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V671 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V671 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V671 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V671 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V671 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V671 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V671 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V671 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V671 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V671 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V671 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V671 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V671 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V671 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V671 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V671 as ProtoVersionPackets>::ItemStackResponsePacket,
    LabTable: <V671 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V671 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V671 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V671 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V671 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V671 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V671 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V671 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelSoundEventV1: <V671 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelSoundEventV2: <V671 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    Login: <V671 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V671 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V671 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V671 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V671 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V671 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V671 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V671 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V671 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V671 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V671 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V671 as ProtoVersionPackets>::MovePlayerPacket,
    MultiplayerSettings: <V671 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V671 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V671 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V671 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V671 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V671 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V671 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V671 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V671 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PassengerJump: <V671 as ProtoVersionPackets>::PassengerJumpPacket,
    PhotoTransfer: <V671 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V671 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V671 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V671 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V671 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V671 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V671 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V671 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V671 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerInput: <V671 as ProtoVersionPackets>::PlayerInputPacket,
    PlayerList: <V671 as ProtoVersionPackets>::PlayerListPacket,
    PlayerSkin: <V671 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V671 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V671 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PositionTrackingDbClientRequest: <V671 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V671 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V671 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V671 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V671 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V671 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V671 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V671 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V671 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V671 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V671 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V671 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V671 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V671 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V671 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V671 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V671 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V671 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V671 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerPlayerPostMovePosition: <V671 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V671 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V671 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V671 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V671 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V671 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V671 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V671 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V671 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V671 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V671 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V671 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V671 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V671 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V671 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V671 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V671 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V671 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V671 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V671 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V671 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V671 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V671 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V671 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V671 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V671 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V671 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V671 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V671 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V671 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V671 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V671 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V671 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V671 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V671 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V671 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V671 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V671 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V671 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V671 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V671 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V671 as ProtoVersionPackets>::TextPacket,
    TickSync: <V671 as ProtoVersionPackets>::TickSyncPacket,
    TickingAreaLoadStatus: <V671 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V671 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V671 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V671 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V671 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V671 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V671 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V671 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V671 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V671 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V671 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateEquip: <V671 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V671 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V671 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V671 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V671 as ProtoVersionPackets>::UpdateTradePacket,
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
