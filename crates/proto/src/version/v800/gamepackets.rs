use super::packets::{
    ActorEventPacket, ActorPickRequestPacket, AddActorPacket, AddBehaviourTreePacket,
    AddItemActorPacket, AddPaintingPacket, AddPlayerPacket, AddVolumeEntityPacket,
    AgentActionEventPacket, AgentAnimationPacket, AnimateEntityPacket, AnimatePacket,
    AnvilDamagePacket, AutomationClientConnectPacket, AvailableActorIdentifiersPacket,
    AvailableCommandsPacket, AwardAchievementPacket, BiomeDefinitionListPacket,
    BlockActorDataPacket, BlockEventPacket, BlockPickRequestPacket, BookEditPacket,
    BossEventPacket, CameraAimAssistInstructionPacket, CameraAimAssistPacket,
    CameraAimAssistPresetsPacket, CameraInstructionPacket, CameraPacket, CameraPresetsPacket,
    CameraShakePacket, ChangeDimensionPacket, ChangeMobPropertyPacket, ChunkRadiusUpdatedPacket,
    ClientBoundCloseFormPacket, ClientBoundControlSchemeSetPacket, ClientBoundDebugRendererPacket,
    ClientBoundMapItemDataPacket, ClientCacheBlobStatusPacket, ClientCacheMissResponsePacket,
    ClientCacheStatusPacket, ClientToServerHandshakePacket, CodeBuilderPacket,
    CodeBuilderSourcePacket, CommandBlockUpdatePacket, CommandOutputPacket, CommandRequestPacket,
    CompletedUsingItemPacket, CompressedBiomeDefinitionListPacket, ContainerClosePacket,
    ContainerOpenPacket, ContainerRegistryCleanupPacket, ContainerSetDataPacket,
    CorrectPlayerMovePredictionPacket, CraftingDataPacket, CreatePhotoPacket,
    CreativeContentPacket, CurrentStructureFeaturePacket, DeathInfoPacket, DebugInfoPacket,
    DimensionDataPacket, DisconnectPacket, EditorNetworkPacket, EduUriResourcePacket,
    EducationSettingsPacket, EmoteListPacket, EmotePacket, FeatureRegistryPacket,
    GameRulesChangedPacket, GameTestRequestPacket, GameTestResultsPacket, GuiDataPickItemPacket,
    HurtArmorPacket, InteractPacket, InventoryContentPacket, InventorySlotPacket,
    InventoryTransactionPacket, ItemComponentPacket, ItemStackRequestPacket,
    ItemStackResponsePacket, JigsawStructureDataPacket, LabTablePacket, LecternUpdatePacket,
    LegacyTelemetryEventPacket, LessonProgressPacket, LevelChunkPacket, LevelEventGenericPacket,
    LevelEventPacket, LevelSoundEventPacket, LoginPacket, MapCreateLockedCopyPacket,
    MapInfoRequestPacket, MobArmorEquipmentPacket, MobEffectPacket, MobEquipmentPacket,
    ModalFormRequestPacket, ModalFormResponsePacket, MotionPredictionHintsPacket,
    MoveActorAbsolutePacket, MoveActorDeltaPacket, MovePlayerPacket, MovementEffectPacket,
    MovementPredictionSyncPacket, MultiplayerSettingsPacket, NetworkChunkPublisherUpdatePacket,
    NetworkSettingsPacket, NetworkStackLatencyPacket, NpcDialoguePacket, NpcRequestPacket,
    OnScreenTextureAnimationPacket, OpenSignPacket, PacketViolationWarningPacket,
    PassengerJumpPacket, PhotoTransferPacket, PlaySoundPacket, PlayStatusPacket,
    PlayerActionPacket, PlayerArmorDamagePacket, PlayerAuthInputPacket, PlayerEnchantOptionsPacket,
    PlayerFogPacket, PlayerHotbarPacket, PlayerInputPacket, PlayerListPacket, PlayerLocationPacket,
    PlayerSkinPacket, PlayerStartItemCooldownPacket, PlayerToggleCrafterSlotRequestPacket,
    PlayerUpdateEntityOverridesPacket, PlayerVideoCapturePacket,
    PositionTrackingDBClientRequestPacket, PositionTrackingDBServerBroadcastPacket,
    PurchaseReceiptPacket, RefreshEntitlementsPacket, RemoveActorPacket, RemoveObjectivePacket,
    RemoveVolumeEntityPacket, RequestAbilityPacket, RequestChunkRadiusPacket,
    RequestNetworkSettingsPacket, RequestPermissionsPacket, ResourcePackChunkDataPacket,
    ResourcePackChunkRequestPacket, ResourcePackClientResponsePacket, ResourcePackDataInfoPacket,
    ResourcePackStackPacket, ResourcePacksInfoPacket, RespawnPacket, ScriptMessagePacket,
    ServerBoundDiagnosticsPacket, ServerBoundLoadingScreenPacket,
    ServerPlayerPostMovePositionPacket, ServerSettingsRequestPacket, ServerSettingsResponsePacket,
    ServerStatsPacket, ServerToClientHandshakePacket, SetActorDataPacket, SetActorLinkPacket,
    SetActorMotionPacket, SetCommandsEnabledPacket, SetDefaultGameTypePacket, SetDifficultyPacket,
    SetDisplayObjectivePacket, SetHealthPacket, SetHudPacket, SetLastHurtByPacket,
    SetLocalPlayerAsInitializedPacket, SetMovementAuthorityPacket, SetPlayerGameTypePacket,
    SetPlayerInventoryOptionsPacket, SetScorePacket, SetScoreboardIdentityPacket,
    SetSpawnPositionPacket, SetTimePacket, SetTitlePacket, SettingsCommandPacket,
    ShowCreditsPacket, ShowProfilePacket, ShowStoreOfferPacket, SimpleEventPacket,
    SimulationTypePacket, SpawnExperienceOrbPacket, SpawnParticleEffectPacket, StartGamePacket,
    StopSoundPacket, StructureBlockUpdatePacket, StructureDataRequestPacket,
    StructureDataResponsePacket, SubChunkPacket, SubChunkRequestPacket, SubClientLoginPacket,
    SyncActorPropertyPacket, TakeItemActorPacket, TextPacket, TickingAreaLoadStatusPacket,
    ToastRequestPacket, TransferPlayerPacket, TrimDataPacket, UnlockedRecipesPacket,
    UpdateAbilitiesPacket, UpdateAdventureSettingsPacket, UpdateAttributesPacket,
    UpdateBlockPacket, UpdateBlockSyncedPacket, UpdateClientInputLocksPacket,
    UpdateClientOptionsPacket, UpdateEquipPacket, UpdatePlayerGameTypePacket, UpdateSoftEnumPacket,
    UpdateSubChunkBlocksPacket, UpdateTradePacket,
};
use bedrockrs_macros::gamepackets;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::{Cursor, Write};
use varint_rs::{VarintReader, VarintWriter};

gamepackets! {
    UpdateClientOptions: UpdateClientOptionsPacket,
    PlayerVideoCapture: PlayerVideoCapturePacket,
    PlayerUpdateEntityOverrides: PlayerUpdateEntityOverridesPacket,
    MovementPredictionSync: MovementPredictionSyncPacket,
    CameraAimAssistInstruction: CameraAimAssistInstructionPacket,
    CameraAimAssistPresets: CameraAimAssistPresetsPacket,
    SetMovementAuthority: SetMovementAuthorityPacket,
    MovementEffect: MovementEffectPacket,
    ContainerRegistryCleanup: ContainerRegistryCleanupPacket,
    CameraAimAssist: CameraAimAssistPacket,
    ServerBoundLoadingScreen: ServerBoundLoadingScreenPacket,
    ServerBoundDiagnostics: ServerBoundDiagnosticsPacket,
    JigsawStructureData: JigsawStructureDataPacket,
    CurrentStructureFeature: CurrentStructureFeaturePacket,
    Login: LoginPacket,
    PlayStatus: PlayStatusPacket,
    ServerToClientHandshake: ServerToClientHandshakePacket,
    ClientToServerHandshake: ClientToServerHandshakePacket,
    Disconnect: DisconnectPacket,
    ResourcePacksInfo: ResourcePacksInfoPacket,
    ResourcePackStack: ResourcePackStackPacket,
    ResourcePackClientResponse: ResourcePackClientResponsePacket,
    Text: TextPacket,
    SetTime: SetTimePacket,
    StartGame: StartGamePacket,
    AddPlayer: AddPlayerPacket,
    AddActor: AddActorPacket,
    RemoveActor: RemoveActorPacket,
    AddItemActor: AddItemActorPacket,
    ClientBoundCloseForm: ClientBoundCloseFormPacket,
    ServerPlayerPostMovePosition: ServerPlayerPostMovePositionPacket,
    TakeItemActor: TakeItemActorPacket,
    MoveActorAbsolute: MoveActorAbsolutePacket,
    MovePlayer: MovePlayerPacket,
    PassengerJump: PassengerJumpPacket,
    UpdateBlock: UpdateBlockPacket,
    AddPainting: AddPaintingPacket,
    AwardAchievement: AwardAchievementPacket,
    LevelEvent: LevelEventPacket,
    BlockEvent: BlockEventPacket,
    ActorEvent: ActorEventPacket,
    MobEffect: MobEffectPacket,
    UpdateAttributes: UpdateAttributesPacket,
    InventoryTransaction: InventoryTransactionPacket,
    MobEquipment: MobEquipmentPacket,
    MobArmorEquipment: MobArmorEquipmentPacket,
    Interact: InteractPacket,
    BlockPickRequest: BlockPickRequestPacket,
    ActorPickRequest: ActorPickRequestPacket,
    PlayerAction: PlayerActionPacket,
    HurtArmor: HurtArmorPacket,
    SetActorData: SetActorDataPacket,
    SetActorMotion: SetActorMotionPacket,
    SetActorLink: SetActorLinkPacket,
    SetHealth: SetHealthPacket,
    SetSpawnPosition: SetSpawnPositionPacket,
    Animate: AnimatePacket,
    Respawn: RespawnPacket,
    ContainerOpen: ContainerOpenPacket,
    ContainerClose: ContainerClosePacket,
    PlayerHotbar: PlayerHotbarPacket,
    InventoryContent: InventoryContentPacket,
    InventorySlot: InventorySlotPacket,
    ContainerSetData: ContainerSetDataPacket,
    CraftingData: CraftingDataPacket,
    GuiDataPickItem: GuiDataPickItemPacket,
    BlockActorData: BlockActorDataPacket,
    PlayerInput: PlayerInputPacket,
    LevelChunk: LevelChunkPacket,
    SetCommandsEnabled: SetCommandsEnabledPacket,
    SetDifficulty: SetDifficultyPacket,
    ChangeDimension: ChangeDimensionPacket,
    SetPlayerGameType: SetPlayerGameTypePacket,
    PlayerList: PlayerListPacket,
    PlayerLocation: PlayerLocationPacket,
    SimpleEvent: SimpleEventPacket,
    LegacyTelemetryEvent: LegacyTelemetryEventPacket,
    SpawnExperienceOrb: SpawnExperienceOrbPacket,
    ClientboundMapItemData: ClientBoundMapItemDataPacket,
    MapInfoRequest: MapInfoRequestPacket,
    RequestChunkRadius: RequestChunkRadiusPacket,
    ChunkRadiusUpdated: ChunkRadiusUpdatedPacket,
    GameRulesChanged: GameRulesChangedPacket,
    Camera: CameraPacket,
    BossEvent: BossEventPacket,
    ShowCredits: ShowCreditsPacket,
    AvailableCommands: AvailableCommandsPacket,
    CommandRequest: CommandRequestPacket,
    CommandBlockUpdate: CommandBlockUpdatePacket,
    CommandOutput: CommandOutputPacket,
    UpdateTrade: UpdateTradePacket,
    UpdateEquip: UpdateEquipPacket,
    ResourcePackDataInfo: ResourcePackDataInfoPacket,
    ResourcePackChunkData: ResourcePackChunkDataPacket,
    ResourcePackChunkRequest: ResourcePackChunkRequestPacket,
    TransferPlayer: TransferPlayerPacket,
    PlaySound: PlaySoundPacket,
    StopSound: StopSoundPacket,
    SetTitle: SetTitlePacket,
    AddBehaviourTree: AddBehaviourTreePacket,
    StructureBlockUpdate: StructureBlockUpdatePacket,
    ShowStoreOffer: ShowStoreOfferPacket,
    PurchaseReceipt: PurchaseReceiptPacket,
    PlayerSkin: PlayerSkinPacket,
    SubClientLogin: SubClientLoginPacket,
    AutomationClientConnect: AutomationClientConnectPacket,
    SetLastHurtBy: SetLastHurtByPacket,
    BookEdit: BookEditPacket,
    NpcRequest: NpcRequestPacket,
    PhotoTransfer: PhotoTransferPacket,
    ModalFormRequest: ModalFormRequestPacket,
    ModalFormResponse: ModalFormResponsePacket,
    ServerSettingsRequest: ServerSettingsRequestPacket,
    ServerSettingsResponse: ServerSettingsResponsePacket,
    ShowProfile: ShowProfilePacket,
    SetDefaultGameType: SetDefaultGameTypePacket,
    RemoveObjective: RemoveObjectivePacket,
    SetDisplayObjective: SetDisplayObjectivePacket,
    SetScore: SetScorePacket,
    LabTable: LabTablePacket,
    UpdateBlockSynced: UpdateBlockSyncedPacket,
    MoveActorDelta: MoveActorDeltaPacket,
    SetScoreboardIdentity: SetScoreboardIdentityPacket,
    SetLocalPlayerAsInitialized: SetLocalPlayerAsInitializedPacket,
    UpdateSoftEnum: UpdateSoftEnumPacket,
    NetworkStackLatency: NetworkStackLatencyPacket,
    SpawnParticleEffect: SpawnParticleEffectPacket,
    AvailableActorIdentifiers: AvailableActorIdentifiersPacket,
    NetworkChunkPublisherUpdate: NetworkChunkPublisherUpdatePacket,
    BiomeDefinitionList: BiomeDefinitionListPacket,
    LevelSoundEvent: LevelSoundEventPacket,
    LevelEventGeneric: LevelEventGenericPacket,
    LecternUpdate: LecternUpdatePacket,
    ClientCacheStatus: ClientCacheStatusPacket,
    OnScreenTextureAnimation: OnScreenTextureAnimationPacket,
    MapCreateLockedCopy: MapCreateLockedCopyPacket,
    StructureDataRequest: StructureDataRequestPacket,
    StructureDataResponse: StructureDataResponsePacket,
    ClientCacheBlobStatus: ClientCacheBlobStatusPacket,
    ClientCacheMissResponse: ClientCacheMissResponsePacket,
    EducationSettings: EducationSettingsPacket,
    Emote: EmotePacket,
    MultiplayerSettings: MultiplayerSettingsPacket,
    SettingsCommand: SettingsCommandPacket,
    AnvilDamage: AnvilDamagePacket,
    CompletedUsingItem: CompletedUsingItemPacket,
    NetworkSettings: NetworkSettingsPacket,
    PlayerAuthInput: PlayerAuthInputPacket,
    CreativeContent: CreativeContentPacket,
    PlayerEnchantOptions: PlayerEnchantOptionsPacket,
    ItemStackRequest: ItemStackRequestPacket,
    ItemStackResponse: ItemStackResponsePacket,
    PlayerArmorDamage: PlayerArmorDamagePacket,
    CodeBuilder: CodeBuilderPacket,
    UpdatePlayerGameType: UpdatePlayerGameTypePacket,
    EmoteList: EmoteListPacket,
    PositionTrackingDbServerBroadcast: PositionTrackingDBServerBroadcastPacket,
    PositionTrackingDbClientRequest: PositionTrackingDBClientRequestPacket,
    DebugInfo: DebugInfoPacket,
    PacketViolationWarning: PacketViolationWarningPacket,
    MotionPredictionHints: MotionPredictionHintsPacket,
    AnimateEntity: AnimateEntityPacket,
    CameraShake: CameraShakePacket,
    PlayerFog: PlayerFogPacket,
    CorrectPlayerMovePrediction: CorrectPlayerMovePredictionPacket,
    ItemComponent: ItemComponentPacket,
    ClientboundDebugRenderer: ClientBoundDebugRendererPacket,
    SyncActorProperty: SyncActorPropertyPacket,
    AddVolumeEntity: AddVolumeEntityPacket,
    RemoveVolumeEntity: RemoveVolumeEntityPacket,
    SimulationType: SimulationTypePacket,
    NpcDialogue: NpcDialoguePacket,
    EduUriResource: EduUriResourcePacket,
    CreatePhoto: CreatePhotoPacket,
    UpdateSubChunkBlocks: UpdateSubChunkBlocksPacket,
    SubChunk: SubChunkPacket,
    SubChunkRequest: SubChunkRequestPacket,
    PlayerStartItemCooldown: PlayerStartItemCooldownPacket,
    ScriptMessage: ScriptMessagePacket,
    CodeBuilderSource: CodeBuilderSourcePacket,
    TickingAreaLoadStatus: TickingAreaLoadStatusPacket,
    DimensionData: DimensionDataPacket,
    AgentActionEvent: AgentActionEventPacket,
    ChangeMobProperty: ChangeMobPropertyPacket,
    LessonProgress: LessonProgressPacket,
    RequestAbility: RequestAbilityPacket,
    RequestPermissions: RequestPermissionsPacket,
    ToastRequest: ToastRequestPacket,
    UpdateAbilities: UpdateAbilitiesPacket,
    UpdateAdventureSettings: UpdateAdventureSettingsPacket,
    DeathInfo: DeathInfoPacket,
    EditorNetwork: EditorNetworkPacket,
    FeatureRegistry: FeatureRegistryPacket,
    ServerStats: ServerStatsPacket,
    RequestNetworkSettings: RequestNetworkSettingsPacket,
    GameTestRequest: GameTestRequestPacket,
    GameTestResults: GameTestResultsPacket,
    UpdateClientInputLocks: UpdateClientInputLocksPacket,
    CameraPresets: CameraPresetsPacket,
    UnlockedRecipes: UnlockedRecipesPacket,
    CameraInstruction: CameraInstructionPacket,
    CompressedBiomeDefinitionList: CompressedBiomeDefinitionListPacket,
    TrimData: TrimDataPacket,
    OpenSign: OpenSignPacket,
    AgentAnimation: AgentAnimationPacket,
    RefreshEntitlements: RefreshEntitlementsPacket,
    PlayerToggleCrafterSlotRequest: PlayerToggleCrafterSlotRequestPacket,
    SetPlayerInventoryOptions: SetPlayerInventoryOptionsPacket,
    SetHud: SetHudPacket,
    ClientBoundControlSchemeSet: ClientBoundControlSchemeSetPacket,
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
