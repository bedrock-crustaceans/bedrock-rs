use crate::version::proto_version::ProtoVersionPackets;
use crate::version::proto_version::V686;
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    ActorEvent: <V686 as ProtoVersionPackets>::ActorEventPacket,
    ActorPickRequest: <V686 as ProtoVersionPackets>::ActorPickRequestPacket,
    AddActor: <V686 as ProtoVersionPackets>::AddActorPacket,
    AddBehaviourTree: <V686 as ProtoVersionPackets>::AddBehaviourTreePacket,
    AddItemActor: <V686 as ProtoVersionPackets>::AddItemActorPacket,
    AddPainting: <V686 as ProtoVersionPackets>::AddPaintingPacket,
    AddPlayer: <V686 as ProtoVersionPackets>::AddPlayerPacket,
    AddVolumeEntity: <V686 as ProtoVersionPackets>::AddVolumeEntityPacket,
    AgentActionEvent: <V686 as ProtoVersionPackets>::AgentActionEventPacket,
    AgentAnimation: <V686 as ProtoVersionPackets>::AgentAnimationPacket,
    Animate: <V686 as ProtoVersionPackets>::AnimatePacket,
    AnimateEntity: <V686 as ProtoVersionPackets>::AnimateEntityPacket,
    AnvilDamage: <V686 as ProtoVersionPackets>::AnvilDamagePacket,
    AutomationClientConnect: <V686 as ProtoVersionPackets>::AutomationClientConnectPacket,
    AvailableActorIdentifiers: <V686 as ProtoVersionPackets>::AvailableActorIdentifiersPacket,
    AvailableCommands: <V686 as ProtoVersionPackets>::AvailableCommandsPacket,
    AwardAchievementPacket: <V686 as ProtoVersionPackets>::AwardAchievementPacket,
    BiomeDefinitionList: <V686 as ProtoVersionPackets>::BiomeDefinitionListPacket,
    BlockActorData: <V686 as ProtoVersionPackets>::BlockActorDataPacket,
    BlockEvent: <V686 as ProtoVersionPackets>::BlockEventPacket,
    BlockPickRequest: <V686 as ProtoVersionPackets>::BlockPickRequestPacket,
    BookEdit: <V686 as ProtoVersionPackets>::BookEditPacket,
    BossEvent: <V686 as ProtoVersionPackets>::BossEventPacket,
    Camera: <V686 as ProtoVersionPackets>::CameraPacket,
    CameraInstruction: <V686 as ProtoVersionPackets>::CameraInstructionPacket,
    CameraPresets: <V686 as ProtoVersionPackets>::CameraPresetsPacket,
    CameraShake: <V686 as ProtoVersionPackets>::CameraShakePacket,
    ChangeDimension: <V686 as ProtoVersionPackets>::ChangeDimensionPacket,
    ChangeMobProperty: <V686 as ProtoVersionPackets>::ChangeMobPropertyPacket,
    ChunkRadiusUpdated: <V686 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket,
    ClientCacheBlobStatus: <V686 as ProtoVersionPackets>::ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: <V686 as ProtoVersionPackets>::ClientCacheMissResponsePacket,
    ClientCacheStatus: <V686 as ProtoVersionPackets>::ClientCacheStatusPacket,
    ClientToServerHandshake: <V686 as ProtoVersionPackets>::ClientToServerHandshakePacket,
    ClientBoundCloseForm: <V686 as ProtoVersionPackets>::ClientBoundCloseFormPacket,
    ClientboundDebugRenderer: <V686 as ProtoVersionPackets>::ClientBoundDebugRendererPacket,
    ClientboundMapItemData: <V686 as ProtoVersionPackets>::ClientBoundMapItemDataPacket,
    CodeBuilder: <V686 as ProtoVersionPackets>::CodeBuilderPacket,
    CodeBuilderSource: <V686 as ProtoVersionPackets>::CodeBuilderSourcePacket,
    CommandBlockUpdate: <V686 as ProtoVersionPackets>::CommandBlockUpdatePacket,
    CommandOutput: <V686 as ProtoVersionPackets>::CommandOutputPacket,
    CommandRequest: <V686 as ProtoVersionPackets>::CommandRequestPacket,
    CompletedUsingItem: <V686 as ProtoVersionPackets>::CompletedUsingItemPacket,
    CompressedBiomeDefinitionList: <V686 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket,
    ContainerClose: <V686 as ProtoVersionPackets>::ContainerClosePacket,
    ContainerOpen: <V686 as ProtoVersionPackets>::ContainerOpenPacket,
    ContainerSetData: <V686 as ProtoVersionPackets>::ContainerSetDataPacket,
    CorrectPlayerMovePrediction: <V686 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket,
    CraftingData: <V686 as ProtoVersionPackets>::CraftingDataPacket,
    CreatePhoto: <V686 as ProtoVersionPackets>::CreatePhotoPacket,
    CreativeContent: <V686 as ProtoVersionPackets>::CreativeContentPacket,
    DeathInfo: <V686 as ProtoVersionPackets>::DeathInfoPacket,
    DebugInfo: <V686 as ProtoVersionPackets>::DebugInfoPacket,
    DimensionData: <V686 as ProtoVersionPackets>::DimensionDataPacket,
    Disconnect: <V686 as ProtoVersionPackets>::DisconnectPacket,
    EditorNetwork: <V686 as ProtoVersionPackets>::EditorNetworkPacket,
    EduUriResource: <V686 as ProtoVersionPackets>::EduUriResourcePacket,
    EducationSettings: <V686 as ProtoVersionPackets>::EducationSettingsPacket,
    Emote: <V686 as ProtoVersionPackets>::EmotePacket,
    EmoteList: <V686 as ProtoVersionPackets>::EmoteListPacket,
    FeatureRegistry: <V686 as ProtoVersionPackets>::FeatureRegistryPacket,
    GameRulesChanged: <V686 as ProtoVersionPackets>::GameRulesChangedPacket,
    GameTestRequest: <V686 as ProtoVersionPackets>::GameTestRequestPacket,
    GameTestResults: <V686 as ProtoVersionPackets>::GameTestResultsPacket,
    GuiDataPickItem: <V686 as ProtoVersionPackets>::GuiDataPickItemPacket,
    HurtArmor: <V686 as ProtoVersionPackets>::HurtArmorPacket,
    Interact: <V686 as ProtoVersionPackets>::InteractPacket,
    InventoryContent: <V686 as ProtoVersionPackets>::InventoryContentPacket,
    InventorySlot: <V686 as ProtoVersionPackets>::InventorySlotPacket,
    InventoryTransaction: <V686 as ProtoVersionPackets>::InventoryTransactionPacket,
    ItemComponent: <V686 as ProtoVersionPackets>::ItemComponentPacket,
    ItemStackRequest: <V686 as ProtoVersionPackets>::ItemStackRequestPacket,
    ItemStackResponse: <V686 as ProtoVersionPackets>::ItemStackResponsePacket,
    LabTable: <V686 as ProtoVersionPackets>::LabTablePacket,
    LecternUpdate: <V686 as ProtoVersionPackets>::LecternUpdatePacket,
    LegacyTelemetryEvent: <V686 as ProtoVersionPackets>::LegacyTelemetryEventPacket,
    LessonProgress: <V686 as ProtoVersionPackets>::LessonProgressPacket,
    LevelChunk: <V686 as ProtoVersionPackets>::LevelChunkPacket,
    LevelEvent: <V686 as ProtoVersionPackets>::LevelEventPacket,
    LevelEventGeneric: <V686 as ProtoVersionPackets>::LevelEventGenericPacket,
    LevelSoundEvent: <V686 as ProtoVersionPackets>::LevelSoundEventPacket,
    LevelSoundEventV1: <V686 as ProtoVersionPackets>::LevelSoundEventV1Packet,
    LevelSoundEventV2: <V686 as ProtoVersionPackets>::LevelSoundEventV2Packet,
    Login: <V686 as ProtoVersionPackets>::LoginPacket,
    MapCreateLockedCopy: <V686 as ProtoVersionPackets>::MapCreateLockedCopyPacket,
    MapInfoRequest: <V686 as ProtoVersionPackets>::MapInfoRequestPacket,
    MobArmorEquipment: <V686 as ProtoVersionPackets>::MobArmorEquipmentPacket,
    MobEffect: <V686 as ProtoVersionPackets>::MobEffectPacket,
    MobEquipment: <V686 as ProtoVersionPackets>::MobEquipmentPacket,
    ModalFormRequest: <V686 as ProtoVersionPackets>::ModalFormRequestPacket,
    ModalFormResponse: <V686 as ProtoVersionPackets>::ModalFormResponsePacket,
    MotionPredictionHints: <V686 as ProtoVersionPackets>::MotionPredictionHintsPacket,
    MoveActorAbsolute: <V686 as ProtoVersionPackets>::MoveActorAbsolutePacket,
    MoveActorDelta: <V686 as ProtoVersionPackets>::MoveActorDeltaPacket,
    MovePlayer: <V686 as ProtoVersionPackets>::MovePlayerPacket,
    MultiplayerSettings: <V686 as ProtoVersionPackets>::MultiplayerSettingsPacket,
    NetworkChunkPublisherUpdate: <V686 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket,
    NetworkSettings: <V686 as ProtoVersionPackets>::NetworkSettingsPacket,
    NetworkStackLatency: <V686 as ProtoVersionPackets>::NetworkStackLatencyPacket,
    NpcDialogue: <V686 as ProtoVersionPackets>::NpcDialoguePacket,
    NpcRequest: <V686 as ProtoVersionPackets>::NpcRequestPacket,
    OnScreenTextureAnimation: <V686 as ProtoVersionPackets>::OnScreenTextureAnimationPacket,
    OpenSign: <V686 as ProtoVersionPackets>::OpenSignPacket,
    PacketViolationWarning: <V686 as ProtoVersionPackets>::PacketViolationWarningPacket,
    PassengerJump: <V686 as ProtoVersionPackets>::PassengerJumpPacket,
    PhotoTransfer: <V686 as ProtoVersionPackets>::PhotoTransferPacket,
    PlaySound: <V686 as ProtoVersionPackets>::PlaySoundPacket,
    PlayStatus: <V686 as ProtoVersionPackets>::PlayStatusPacket,
    PlayerAction: <V686 as ProtoVersionPackets>::PlayerActionPacket,
    PlayerArmorDamage: <V686 as ProtoVersionPackets>::PlayerArmorDamagePacket,
    PlayerAuthInput: <V686 as ProtoVersionPackets>::PlayerAuthInputPacket,
    PlayerEnchantOptions: <V686 as ProtoVersionPackets>::PlayerEnchantOptionsPacket,
    PlayerFog: <V686 as ProtoVersionPackets>::PlayerFogPacket,
    PlayerHotbar: <V686 as ProtoVersionPackets>::PlayerHotbarPacket,
    PlayerInput: <V686 as ProtoVersionPackets>::PlayerInputPacket,
    PlayerList: <V686 as ProtoVersionPackets>::PlayerListPacket,
    PlayerSkin: <V686 as ProtoVersionPackets>::PlayerSkinPacket,
    PlayerStartItemCooldown: <V686 as ProtoVersionPackets>::PlayerStartItemCooldownPacket,
    PlayerToggleCrafterSlotRequest: <V686 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket,
    PositionTrackingDbClientRequest: <V686 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket,
    PositionTrackingDbServerBroadcast: <V686 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket,
    PurchaseReceipt: <V686 as ProtoVersionPackets>::PurchaseReceiptPacket,
    RefreshEntitlements: <V686 as ProtoVersionPackets>::RefreshEntitlementsPacket,
    RemoveActor: <V686 as ProtoVersionPackets>::RemoveActorPacket,
    RemoveObjective: <V686 as ProtoVersionPackets>::RemoveObjectivePacket,
    RemoveVolumeEntity: <V686 as ProtoVersionPackets>::RemoveVolumeEntityPacket,
    RequestAbility: <V686 as ProtoVersionPackets>::RequestAbilityPacket,
    RequestChunkRadius: <V686 as ProtoVersionPackets>::RequestChunkRadiusPacket,
    RequestNetworkSettings: <V686 as ProtoVersionPackets>::RequestNetworkSettingsPacket,
    RequestPermissions: <V686 as ProtoVersionPackets>::RequestPermissionsPacket,
    ResourcePackChunkData: <V686 as ProtoVersionPackets>::ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: <V686 as ProtoVersionPackets>::ResourcePackChunkRequestPacket,
    ResourcePackClientResponse: <V686 as ProtoVersionPackets>::ResourcePackClientResponsePacket,
    ResourcePackDataInfo: <V686 as ProtoVersionPackets>::ResourcePackDataInfoPacket,
    ResourcePackStack: <V686 as ProtoVersionPackets>::ResourcePackStackPacket,
    ResourcePacksInfo: <V686 as ProtoVersionPackets>::ResourcePacksInfoPacket,
    Respawn: <V686 as ProtoVersionPackets>::RespawnPacket,
    ScriptMessage: <V686 as ProtoVersionPackets>::ScriptMessagePacket,
    ServerPlayerPostMovePosition: <V686 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket,
    ServerSettingsRequest: <V686 as ProtoVersionPackets>::ServerSettingsRequestPacket,
    ServerSettingsResponse: <V686 as ProtoVersionPackets>::ServerSettingsResponsePacket,
    ServerStats: <V686 as ProtoVersionPackets>::ServerStatsPacket,
    ServerToClientHandshake: <V686 as ProtoVersionPackets>::ServerToClientHandshakePacket,
    SetActorData: <V686 as ProtoVersionPackets>::SetActorDataPacket,
    SetActorLink: <V686 as ProtoVersionPackets>::SetActorLinkPacket,
    SetActorMotion: <V686 as ProtoVersionPackets>::SetActorMotionPacket,
    SetCommandsEnabled: <V686 as ProtoVersionPackets>::SetCommandsEnabledPacket,
    SetDefaultGameType: <V686 as ProtoVersionPackets>::SetDefaultGameTypePacket,
    SetDifficulty: <V686 as ProtoVersionPackets>::SetDifficultyPacket,
    SetDisplayObjective: <V686 as ProtoVersionPackets>::SetDisplayObjectivePacket,
    SetHealth: <V686 as ProtoVersionPackets>::SetHealthPacket,
    SetHud: <V686 as ProtoVersionPackets>::SetHudPacket,
    SetLastHurtBy: <V686 as ProtoVersionPackets>::SetLastHurtByPacket,
    SetLocalPlayerAsInitialized: <V686 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket,
    SetPlayerGameType: <V686 as ProtoVersionPackets>::SetPlayerGameTypePacket,
    SetPlayerInventoryOptions: <V686 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket,
    SetScore: <V686 as ProtoVersionPackets>::SetScorePacket,
    SetScoreboardIdentity: <V686 as ProtoVersionPackets>::SetScoreboardIdentityPacket,
    SetSpawnPosition: <V686 as ProtoVersionPackets>::SetSpawnPositionPacket,
    SetTime: <V686 as ProtoVersionPackets>::SetTimePacket,
    SetTitle: <V686 as ProtoVersionPackets>::SetTitlePacket,
    SettingsCommand: <V686 as ProtoVersionPackets>::SettingsCommandPacket,
    ShowCredits: <V686 as ProtoVersionPackets>::ShowCreditsPacket,
    ShowProfile: <V686 as ProtoVersionPackets>::ShowProfilePacket,
    ShowStoreOffer: <V686 as ProtoVersionPackets>::ShowStoreOfferPacket,
    SimpleEvent: <V686 as ProtoVersionPackets>::SimpleEventPacket,
    SimulationType: <V686 as ProtoVersionPackets>::SimulationTypePacket,
    SpawnExperienceOrb: <V686 as ProtoVersionPackets>::SpawnExperienceOrbPacket,
    SpawnParticleEffect: <V686 as ProtoVersionPackets>::SpawnParticleEffectPacket,
    StartGame: <V686 as ProtoVersionPackets>::StartGamePacket,
    StopSound: <V686 as ProtoVersionPackets>::StopSoundPacket,
    StructureBlockUpdate: <V686 as ProtoVersionPackets>::StructureBlockUpdatePacket,
    StructureDataRequest: <V686 as ProtoVersionPackets>::StructureDataRequestPacket,
    StructureDataResponse: <V686 as ProtoVersionPackets>::StructureDataResponsePacket,
    SubChunk: <V686 as ProtoVersionPackets>::SubChunkPacket,
    SubChunkRequest: <V686 as ProtoVersionPackets>::SubChunkRequestPacket,
    SubClientLogin: <V686 as ProtoVersionPackets>::SubClientLoginPacket,
    SyncActorProperty: <V686 as ProtoVersionPackets>::SyncActorPropertyPacket,
    TakeItemActor: <V686 as ProtoVersionPackets>::TakeItemActorPacket,
    Text: <V686 as ProtoVersionPackets>::TextPacket,
    TickingAreaLoadStatus: <V686 as ProtoVersionPackets>::TickingAreaLoadStatusPacket,
    ToastRequest: <V686 as ProtoVersionPackets>::ToastRequestPacket,
    TransferPlayer: <V686 as ProtoVersionPackets>::TransferPlayerPacket,
    TrimData: <V686 as ProtoVersionPackets>::TrimDataPacket,
    UnlockedRecipes: <V686 as ProtoVersionPackets>::UnlockedRecipesPacket,
    UpdateAbilities: <V686 as ProtoVersionPackets>::UpdateAbilitiesPacket,
    UpdateAdventureSettings: <V686 as ProtoVersionPackets>::UpdateAdventureSettingsPacket,
    UpdateAttributes: <V686 as ProtoVersionPackets>::UpdateAttributesPacket,
    UpdateBlock: <V686 as ProtoVersionPackets>::UpdateBlockPacket,
    UpdateBlockSynced: <V686 as ProtoVersionPackets>::UpdateBlockSyncedPacket,
    UpdateClientInputLocks: <V686 as ProtoVersionPackets>::UpdateClientInputLocksPacket,
    UpdateEquip: <V686 as ProtoVersionPackets>::UpdateEquipPacket,
    UpdatePlayerGameType: <V686 as ProtoVersionPackets>::UpdatePlayerGameTypePacket,
    UpdateSoftEnum: <V686 as ProtoVersionPackets>::UpdateSoftEnumPacket,
    UpdateSubChunkBlocks: <V686 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket,
    UpdateTrade: <V686 as ProtoVersionPackets>::UpdateTradePacket,
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
