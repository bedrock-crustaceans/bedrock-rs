#![allow(unused, clippy::too_many_lines)]
use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_version_traits::ProtoVersionEnums;
use bedrock_network_version_traits::ProtoVersionPackets;
use bedrock_network_version_traits::ProtoVersionTypes;
#[derive(Clone, std::fmt::Debug)]
pub enum V2259 {
    ActorEventPacket(Box<<Self as ProtoVersionPackets>::ActorEventPacket>),
    ActorPickRequestPacket(Box<<Self as ProtoVersionPackets>::ActorPickRequestPacket>),
    AddActorPacket(Box<<Self as ProtoVersionPackets>::AddActorPacket>),
    AddBehaviourTreePacket(Box<<Self as ProtoVersionPackets>::AddBehaviourTreePacket>),
    AddItemActorPacket(Box<<Self as ProtoVersionPackets>::AddItemActorPacket>),
    AddPaintingPacket(Box<<Self as ProtoVersionPackets>::AddPaintingPacket>),
    AddPlayerPacket(Box<<Self as ProtoVersionPackets>::AddPlayerPacket>),
    AgentActionEventPacket(Box<<Self as ProtoVersionPackets>::AgentActionEventPacket>),
    AgentAnimationPacket(Box<<Self as ProtoVersionPackets>::AgentAnimationPacket>),
    AnimateEntityPacket(Box<<Self as ProtoVersionPackets>::AnimateEntityPacket>),
    AnimatePacket(Box<<Self as ProtoVersionPackets>::AnimatePacket>),
    AnvilDamagePacket(Box<<Self as ProtoVersionPackets>::AnvilDamagePacket>),
    AutomationClientConnectPacket(
        Box<<Self as ProtoVersionPackets>::AutomationClientConnectPacket>,
    ),
    AvailableActorIdentifiersPacket(
        Box<<Self as ProtoVersionPackets>::AvailableActorIdentifiersPacket>,
    ),
    AvailableCommandsPacket(Box<<Self as ProtoVersionPackets>::AvailableCommandsPacket>),
    AwardAchievementPacket(Box<<Self as ProtoVersionPackets>::AwardAchievementPacket>),
    BiomeDefinitionListPacket(Box<<Self as ProtoVersionPackets>::BiomeDefinitionListPacket>),
    BlockActorDataPacket(Box<<Self as ProtoVersionPackets>::BlockActorDataPacket>),
    BlockEventPacket(Box<<Self as ProtoVersionPackets>::BlockEventPacket>),
    BlockPickRequestPacket(Box<<Self as ProtoVersionPackets>::BlockPickRequestPacket>),
    BookEditPacket(Box<<Self as ProtoVersionPackets>::BookEditPacket>),
    BossEventPacket(Box<<Self as ProtoVersionPackets>::BossEventPacket>),
    CameraAimAssistActorPriorityPacket(
        Box<<Self as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket>,
    ),
    CameraAimAssistInstructionPacket(
        Box<<Self as ProtoVersionPackets>::CameraAimAssistInstructionPacket>,
    ),
    CameraAimAssistPacket(Box<<Self as ProtoVersionPackets>::CameraAimAssistPacket>),
    CameraAimAssistPresetsPacket(Box<<Self as ProtoVersionPackets>::CameraAimAssistPresetsPacket>),
    CameraInstructionPacket(Box<<Self as ProtoVersionPackets>::CameraInstructionPacket>),
    CameraPacket(Box<<Self as ProtoVersionPackets>::CameraPacket>),
    CameraPresetsPacket(Box<<Self as ProtoVersionPackets>::CameraPresetsPacket>),
    CameraShakePacket(Box<<Self as ProtoVersionPackets>::CameraShakePacket>),
    CameraSplinePacket(Box<<Self as ProtoVersionPackets>::CameraSplinePacket>),
    ChangeDimensionPacket(Box<<Self as ProtoVersionPackets>::ChangeDimensionPacket>),
    ChangeMobPropertyPacket(Box<<Self as ProtoVersionPackets>::ChangeMobPropertyPacket>),
    ChatPacket(Box<<Self as ProtoVersionPackets>::ChatPacket>),
    ChunkRadiusUpdatedPacket(Box<<Self as ProtoVersionPackets>::ChunkRadiusUpdatedPacket>),
    ClientBoundAttributeLayerSyncPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket>,
    ),
    ClientBoundCloseFormPacket(Box<<Self as ProtoVersionPackets>::ClientBoundCloseFormPacket>),
    ClientBoundControlSchemeSetPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket>,
    ),
    ClientBoundDataDrivenUICloseScreenPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket>,
    ),
    ClientBoundDataDrivenUIReloadPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket>,
    ),
    ClientBoundDataDrivenUIShowScreenPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket>,
    ),
    ClientBoundDataStorePacket(Box<<Self as ProtoVersionPackets>::ClientBoundDataStorePacket>),
    ClientBoundDebugRendererPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundDebugRendererPacket>,
    ),
    ClientBoundMapItemDataPacket(Box<<Self as ProtoVersionPackets>::ClientBoundMapItemDataPacket>),
    ClientBoundMatchmakingStatePacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket>,
    ),
    ClientBoundMusicPlaybackFailurePacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket>,
    ),
    ClientBoundOnlineMusicPlaybackResponsePacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket>,
    ),
    ClientBoundPlayAudioContentPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket>,
    ),
    ClientBoundStonecutterSetRecipePacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket>,
    ),
    ClientBoundTextureShiftPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundTextureShiftPacket>,
    ),
    ClientBoundUpdateSoundDataPacket(
        Box<<Self as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket>,
    ),
    ClientCacheBlobStatusPacket(Box<<Self as ProtoVersionPackets>::ClientCacheBlobStatusPacket>),
    ClientCacheMissResponsePacket(
        Box<<Self as ProtoVersionPackets>::ClientCacheMissResponsePacket>,
    ),
    ClientCacheStatusPacket(Box<<Self as ProtoVersionPackets>::ClientCacheStatusPacket>),
    ClientToServerHandshakePacket(
        Box<<Self as ProtoVersionPackets>::ClientToServerHandshakePacket>,
    ),
    CodeBuilderPacket(Box<<Self as ProtoVersionPackets>::CodeBuilderPacket>),
    CodeBuilderSourcePacket(Box<<Self as ProtoVersionPackets>::CodeBuilderSourcePacket>),
    CommandBlockUpdatePacket(Box<<Self as ProtoVersionPackets>::CommandBlockUpdatePacket>),
    CommandOutputPacket(Box<<Self as ProtoVersionPackets>::CommandOutputPacket>),
    CommandRequestPacket(Box<<Self as ProtoVersionPackets>::CommandRequestPacket>),
    CompletedUsingItemPacket(Box<<Self as ProtoVersionPackets>::CompletedUsingItemPacket>),
    ContainerClosePacket(Box<<Self as ProtoVersionPackets>::ContainerClosePacket>),
    ContainerOpenPacket(Box<<Self as ProtoVersionPackets>::ContainerOpenPacket>),
    ContainerRegistryCleanupPacket(
        Box<<Self as ProtoVersionPackets>::ContainerRegistryCleanupPacket>,
    ),
    ContainerSetDataPacket(Box<<Self as ProtoVersionPackets>::ContainerSetDataPacket>),
    CorrectPlayerMovePredictionPacket(
        Box<<Self as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket>,
    ),
    CraftingDataPacket(Box<<Self as ProtoVersionPackets>::CraftingDataPacket>),
    CreatePhotoPacket(Box<<Self as ProtoVersionPackets>::CreatePhotoPacket>),
    CreativeContentPacket(Box<<Self as ProtoVersionPackets>::CreativeContentPacket>),
    CurrentStructureFeaturePacket(
        Box<<Self as ProtoVersionPackets>::CurrentStructureFeaturePacket>,
    ),
    DeathInfoPacket(Box<<Self as ProtoVersionPackets>::DeathInfoPacket>),
    DebugDrawerPacket(Box<<Self as ProtoVersionPackets>::DebugDrawerPacket>),
    DebugInfoPacket(Box<<Self as ProtoVersionPackets>::DebugInfoPacket>),
    DimensionDataPacket(Box<<Self as ProtoVersionPackets>::DimensionDataPacket>),
    DisconnectPacket(Box<<Self as ProtoVersionPackets>::DisconnectPacket>),
    EditorNetworkPacket(Box<<Self as ProtoVersionPackets>::EditorNetworkPacket>),
    EduUriResourcePacket(Box<<Self as ProtoVersionPackets>::EduUriResourcePacket>),
    EducationSettingsPacket(Box<<Self as ProtoVersionPackets>::EducationSettingsPacket>),
    EmoteListPacket(Box<<Self as ProtoVersionPackets>::EmoteListPacket>),
    EmotePacket(Box<<Self as ProtoVersionPackets>::EmotePacket>),
    FeatureRegistryPacket(Box<<Self as ProtoVersionPackets>::FeatureRegistryPacket>),
    GameRulesChangedPacket(Box<<Self as ProtoVersionPackets>::GameRulesChangedPacket>),
    GameTestRequestPacket(Box<<Self as ProtoVersionPackets>::GameTestRequestPacket>),
    GameTestResultsPacket(Box<<Self as ProtoVersionPackets>::GameTestResultsPacket>),
    GraphicsParameterOverridePacket(
        Box<<Self as ProtoVersionPackets>::GraphicsParameterOverridePacket>,
    ),
    GuiDataPickItemPacket(Box<<Self as ProtoVersionPackets>::GuiDataPickItemPacket>),
    HurtArmorPacket(Box<<Self as ProtoVersionPackets>::HurtArmorPacket>),
    InteractPacket(Box<<Self as ProtoVersionPackets>::InteractPacket>),
    InventoryContentPacket(Box<<Self as ProtoVersionPackets>::InventoryContentPacket>),
    InventorySlotPacket(Box<<Self as ProtoVersionPackets>::InventorySlotPacket>),
    InventoryTransactionPacket(Box<<Self as ProtoVersionPackets>::InventoryTransactionPacket>),
    ItemComponentPacket(Box<<Self as ProtoVersionPackets>::ItemComponentPacket>),
    ItemStackRequestPacket(Box<<Self as ProtoVersionPackets>::ItemStackRequestPacket>),
    ItemStackResponsePacket(Box<<Self as ProtoVersionPackets>::ItemStackResponsePacket>),
    JigsawStructureDataPacket(Box<<Self as ProtoVersionPackets>::JigsawStructureDataPacket>),
    LabTablePacket(Box<<Self as ProtoVersionPackets>::LabTablePacket>),
    LecternUpdatePacket(Box<<Self as ProtoVersionPackets>::LecternUpdatePacket>),
    LegacyTelemetryEventPacket(Box<<Self as ProtoVersionPackets>::LegacyTelemetryEventPacket>),
    LessonProgressPacket(Box<<Self as ProtoVersionPackets>::LessonProgressPacket>),
    LevelChunkPacket(Box<<Self as ProtoVersionPackets>::LevelChunkPacket>),
    LevelEventGenericPacket(Box<<Self as ProtoVersionPackets>::LevelEventGenericPacket>),
    LevelEventPacket(Box<<Self as ProtoVersionPackets>::LevelEventPacket>),
    LevelSoundEventPacket(Box<<Self as ProtoVersionPackets>::LevelSoundEventPacket>),
    LocatorBarPacket(Box<<Self as ProtoVersionPackets>::LocatorBarPacket>),
    LoginPacket(Box<<Self as ProtoVersionPackets>::LoginPacket>),
    MapCreateLockedCopyPacket(Box<<Self as ProtoVersionPackets>::MapCreateLockedCopyPacket>),
    MapInfoRequestPacket(Box<<Self as ProtoVersionPackets>::MapInfoRequestPacket>),
    MobArmorEquipmentPacket(Box<<Self as ProtoVersionPackets>::MobArmorEquipmentPacket>),
    MobEffectPacket(Box<<Self as ProtoVersionPackets>::MobEffectPacket>),
    MobEquipmentPacket(Box<<Self as ProtoVersionPackets>::MobEquipmentPacket>),
    ModalFormRequestPacket(Box<<Self as ProtoVersionPackets>::ModalFormRequestPacket>),
    ModalFormResponsePacket(Box<<Self as ProtoVersionPackets>::ModalFormResponsePacket>),
    MotionPredictionHintsPacket(Box<<Self as ProtoVersionPackets>::MotionPredictionHintsPacket>),
    MoveActorAbsolutePacket(Box<<Self as ProtoVersionPackets>::MoveActorAbsolutePacket>),
    MoveActorDeltaPacket(Box<<Self as ProtoVersionPackets>::MoveActorDeltaPacket>),
    MovePlayerPacket(Box<<Self as ProtoVersionPackets>::MovePlayerPacket>),
    MovementEffectPacket(Box<<Self as ProtoVersionPackets>::MovementEffectPacket>),
    MovementPredictionSyncPacket(Box<<Self as ProtoVersionPackets>::MovementPredictionSyncPacket>),
    MultiplayerSettingsPacket(Box<<Self as ProtoVersionPackets>::MultiplayerSettingsPacket>),
    NetworkChunkPublisherUpdatePacket(
        Box<<Self as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket>,
    ),
    NetworkSettingsPacket(Box<<Self as ProtoVersionPackets>::NetworkSettingsPacket>),
    NetworkStackLatencyPacket(Box<<Self as ProtoVersionPackets>::NetworkStackLatencyPacket>),
    NpcDialoguePacket(Box<<Self as ProtoVersionPackets>::NpcDialoguePacket>),
    NpcRequestPacket(Box<<Self as ProtoVersionPackets>::NpcRequestPacket>),
    OnScreenTextureAnimationPacket(
        Box<<Self as ProtoVersionPackets>::OnScreenTextureAnimationPacket>,
    ),
    OpenSignPacket(Box<<Self as ProtoVersionPackets>::OpenSignPacket>),
    PacketViolationWarningPacket(Box<<Self as ProtoVersionPackets>::PacketViolationWarningPacket>),
    PartyChangedPacket(Box<<Self as ProtoVersionPackets>::PartyChangedPacket>),
    PartyDestinationCookieResponsePacket(
        Box<<Self as ProtoVersionPackets>::PartyDestinationCookieResponsePacket>,
    ),
    PhotoTransferPacket(Box<<Self as ProtoVersionPackets>::PhotoTransferPacket>),
    PlaySoundPacket(Box<<Self as ProtoVersionPackets>::PlaySoundPacket>),
    PlayStatusPacket(Box<<Self as ProtoVersionPackets>::PlayStatusPacket>),
    PlayerActionPacket(Box<<Self as ProtoVersionPackets>::PlayerActionPacket>),
    PlayerArmorDamagePacket(Box<<Self as ProtoVersionPackets>::PlayerArmorDamagePacket>),
    PlayerAuthInputPacket(Box<<Self as ProtoVersionPackets>::PlayerAuthInputPacket>),
    PlayerEnchantOptionsPacket(Box<<Self as ProtoVersionPackets>::PlayerEnchantOptionsPacket>),
    PlayerFogPacket(Box<<Self as ProtoVersionPackets>::PlayerFogPacket>),
    PlayerHotbarPacket(Box<<Self as ProtoVersionPackets>::PlayerHotbarPacket>),
    PlayerListPacket(Box<<Self as ProtoVersionPackets>::PlayerListPacket>),
    PlayerLocationPacket(Box<<Self as ProtoVersionPackets>::PlayerLocationPacket>),
    PlayerSkinPacket(Box<<Self as ProtoVersionPackets>::PlayerSkinPacket>),
    PlayerStartItemCooldownPacket(
        Box<<Self as ProtoVersionPackets>::PlayerStartItemCooldownPacket>,
    ),
    PlayerToggleArmorVisibilityRequestPacket(
        Box<<Self as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket>,
    ),
    PlayerToggleCrafterSlotRequestPacket(
        Box<<Self as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket>,
    ),
    PlayerUpdateEntityOverridesPacket(
        Box<<Self as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket>,
    ),
    PlayerVideoCapturePacket(Box<<Self as ProtoVersionPackets>::PlayerVideoCapturePacket>),
    PositionTrackingDBClientRequestPacket(
        Box<<Self as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket>,
    ),
    PositionTrackingDBServerBroadcastPacket(
        Box<<Self as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket>,
    ),
    PurchaseReceiptPacket(Box<<Self as ProtoVersionPackets>::PurchaseReceiptPacket>),
    RecordStartedPacket(Box<<Self as ProtoVersionPackets>::RecordStartedPacket>),
    RefreshEntitlementsPacket(Box<<Self as ProtoVersionPackets>::RefreshEntitlementsPacket>),
    RemoveActorPacket(Box<<Self as ProtoVersionPackets>::RemoveActorPacket>),
    RemoveObjectivePacket(Box<<Self as ProtoVersionPackets>::RemoveObjectivePacket>),
    RequestAbilityPacket(Box<<Self as ProtoVersionPackets>::RequestAbilityPacket>),
    RequestChunkRadiusPacket(Box<<Self as ProtoVersionPackets>::RequestChunkRadiusPacket>),
    RequestNetworkSettingsPacket(Box<<Self as ProtoVersionPackets>::RequestNetworkSettingsPacket>),
    RequestPermissionsPacket(Box<<Self as ProtoVersionPackets>::RequestPermissionsPacket>),
    ResourcePackChunkDataPacket(Box<<Self as ProtoVersionPackets>::ResourcePackChunkDataPacket>),
    ResourcePackChunkRequestPacket(
        Box<<Self as ProtoVersionPackets>::ResourcePackChunkRequestPacket>,
    ),
    ResourcePackClientResponsePacket(
        Box<<Self as ProtoVersionPackets>::ResourcePackClientResponsePacket>,
    ),
    ResourcePackDataInfoPacket(Box<<Self as ProtoVersionPackets>::ResourcePackDataInfoPacket>),
    ResourcePackStackPacket(Box<<Self as ProtoVersionPackets>::ResourcePackStackPacket>),
    ResourcePacksInfoPacket(Box<<Self as ProtoVersionPackets>::ResourcePacksInfoPacket>),
    ResourcePacksReadyForValidationPacket(
        Box<<Self as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket>,
    ),
    RespawnPacket(Box<<Self as ProtoVersionPackets>::RespawnPacket>),
    ScriptMessagePacket(Box<<Self as ProtoVersionPackets>::ScriptMessagePacket>),
    SendPartyDestinationCookiePacket(
        Box<<Self as ProtoVersionPackets>::SendPartyDestinationCookiePacket>,
    ),
    ServerBoundCursorItemDragPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundCursorItemDragPacket>,
    ),
    ServerBoundDataDrivenClosedPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket>,
    ),
    ServerBoundDataStorePacket(Box<<Self as ProtoVersionPackets>::ServerBoundDataStorePacket>),
    ServerBoundDiagnosticsPacket(Box<<Self as ProtoVersionPackets>::ServerBoundDiagnosticsPacket>),
    ServerBoundLoadingScreenPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundLoadingScreenPacket>,
    ),
    ServerBoundMatchmakingCancelPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket>,
    ),
    ServerBoundPackSettingChangePacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundPackSettingChangePacket>,
    ),
    ServerBoundRegisterAudioContentPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket>,
    ),
    ServerBoundRequestOnlineMusicPlaybackPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket>,
    ),
    ServerBoundStonecutterSetRecipePacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket>,
    ),
    ServerPlayerPostMovePositionPacket(
        Box<<Self as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket>,
    ),
    ServerPresenceInfoPacket(Box<<Self as ProtoVersionPackets>::ServerPresenceInfoPacket>),
    ServerSettingsRequestPacket(Box<<Self as ProtoVersionPackets>::ServerSettingsRequestPacket>),
    ServerSettingsResponsePacket(Box<<Self as ProtoVersionPackets>::ServerSettingsResponsePacket>),
    ServerStatsPacket(Box<<Self as ProtoVersionPackets>::ServerStatsPacket>),
    ServerStoreInfoPacket(Box<<Self as ProtoVersionPackets>::ServerStoreInfoPacket>),
    ServerToClientHandshakePacket(
        Box<<Self as ProtoVersionPackets>::ServerToClientHandshakePacket>,
    ),
    SetActorDataPacket(Box<<Self as ProtoVersionPackets>::SetActorDataPacket>),
    SetActorLinkPacket(Box<<Self as ProtoVersionPackets>::SetActorLinkPacket>),
    SetActorMotionPacket(Box<<Self as ProtoVersionPackets>::SetActorMotionPacket>),
    SetCommandsEnabledPacket(Box<<Self as ProtoVersionPackets>::SetCommandsEnabledPacket>),
    SetDefaultGameTypePacket(Box<<Self as ProtoVersionPackets>::SetDefaultGameTypePacket>),
    SetDifficultyPacket(Box<<Self as ProtoVersionPackets>::SetDifficultyPacket>),
    SetDisplayObjectivePacket(Box<<Self as ProtoVersionPackets>::SetDisplayObjectivePacket>),
    SetHealthPacket(Box<<Self as ProtoVersionPackets>::SetHealthPacket>),
    SetHudPacket(Box<<Self as ProtoVersionPackets>::SetHudPacket>),
    SetLastHurtByPacket(Box<<Self as ProtoVersionPackets>::SetLastHurtByPacket>),
    SetLocalPlayerAsInitializedPacket(
        Box<<Self as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket>,
    ),
    SetPassengerOfBlockPacket(Box<<Self as ProtoVersionPackets>::SetPassengerOfBlockPacket>),
    SetPlayerFurnaceOptionsPacket(
        Box<<Self as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket>,
    ),
    SetPlayerGameTypePacket(Box<<Self as ProtoVersionPackets>::SetPlayerGameTypePacket>),
    SetPlayerInventoryOptionsPacket(
        Box<<Self as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket>,
    ),
    SetScorePacket(Box<<Self as ProtoVersionPackets>::SetScorePacket>),
    SetScoreboardIdentityPacket(Box<<Self as ProtoVersionPackets>::SetScoreboardIdentityPacket>),
    SetSpawnPositionPacket(Box<<Self as ProtoVersionPackets>::SetSpawnPositionPacket>),
    SetTimePacket(Box<<Self as ProtoVersionPackets>::SetTimePacket>),
    SetTitlePacket(Box<<Self as ProtoVersionPackets>::SetTitlePacket>),
    SettingsCommandPacket(Box<<Self as ProtoVersionPackets>::SettingsCommandPacket>),
    ShowCreditsPacket(Box<<Self as ProtoVersionPackets>::ShowCreditsPacket>),
    ShowProfilePacket(Box<<Self as ProtoVersionPackets>::ShowProfilePacket>),
    ShowStoreOfferPacket(Box<<Self as ProtoVersionPackets>::ShowStoreOfferPacket>),
    SimpleEventPacket(Box<<Self as ProtoVersionPackets>::SimpleEventPacket>),
    SimulationTypePacket(Box<<Self as ProtoVersionPackets>::SimulationTypePacket>),
    SpawnExperienceOrbPacket(Box<<Self as ProtoVersionPackets>::SpawnExperienceOrbPacket>),
    SpawnParticleEffectPacket(Box<<Self as ProtoVersionPackets>::SpawnParticleEffectPacket>),
    StartGamePacket(Box<<Self as ProtoVersionPackets>::StartGamePacket>),
    StopSoundPacket(Box<<Self as ProtoVersionPackets>::StopSoundPacket>),
    StructureBlockUpdatePacket(Box<<Self as ProtoVersionPackets>::StructureBlockUpdatePacket>),
    StructureDataRequestPacket(Box<<Self as ProtoVersionPackets>::StructureDataRequestPacket>),
    StructureDataResponsePacket(Box<<Self as ProtoVersionPackets>::StructureDataResponsePacket>),
    SubChunkPacket(Box<<Self as ProtoVersionPackets>::SubChunkPacket>),
    SubChunkRequestPacket(Box<<Self as ProtoVersionPackets>::SubChunkRequestPacket>),
    SubClientLoginPacket(Box<<Self as ProtoVersionPackets>::SubClientLoginPacket>),
    SyncActorPropertyPacket(Box<<Self as ProtoVersionPackets>::SyncActorPropertyPacket>),
    SyncWorldClocksPacket(Box<<Self as ProtoVersionPackets>::SyncWorldClocksPacket>),
    TakeItemActorPacket(Box<<Self as ProtoVersionPackets>::TakeItemActorPacket>),
    TextPacket(Box<<Self as ProtoVersionPackets>::TextPacket>),
    TickingAreaLoadStatusPacket(Box<<Self as ProtoVersionPackets>::TickingAreaLoadStatusPacket>),
    ToastRequestPacket(Box<<Self as ProtoVersionPackets>::ToastRequestPacket>),
    TransferPlayerPacket(Box<<Self as ProtoVersionPackets>::TransferPlayerPacket>),
    TrimDataPacket(Box<<Self as ProtoVersionPackets>::TrimDataPacket>),
    UnlockedRecipesPacket(Box<<Self as ProtoVersionPackets>::UnlockedRecipesPacket>),
    UpdateAbilitiesPacket(Box<<Self as ProtoVersionPackets>::UpdateAbilitiesPacket>),
    UpdateAdventureSettingsPacket(
        Box<<Self as ProtoVersionPackets>::UpdateAdventureSettingsPacket>,
    ),
    UpdateAttributesPacket(Box<<Self as ProtoVersionPackets>::UpdateAttributesPacket>),
    UpdateBlockPacket(Box<<Self as ProtoVersionPackets>::UpdateBlockPacket>),
    UpdateBlockSyncedPacket(Box<<Self as ProtoVersionPackets>::UpdateBlockSyncedPacket>),
    UpdateClientInputLocksPacket(Box<<Self as ProtoVersionPackets>::UpdateClientInputLocksPacket>),
    UpdateClientOptionsPacket(Box<<Self as ProtoVersionPackets>::UpdateClientOptionsPacket>),
    UpdateEquipPacket(Box<<Self as ProtoVersionPackets>::UpdateEquipPacket>),
    UpdatePlayerGameTypePacket(Box<<Self as ProtoVersionPackets>::UpdatePlayerGameTypePacket>),
    UpdateSoftEnumPacket(Box<<Self as ProtoVersionPackets>::UpdateSoftEnumPacket>),
    UpdateSubChunkBlocksPacket(Box<<Self as ProtoVersionPackets>::UpdateSubChunkBlocksPacket>),
    UpdateTradePacket(Box<<Self as ProtoVersionPackets>::UpdateTradePacket>),
    VirtualContainerClosePacket(Box<<Self as ProtoVersionPackets>::VirtualContainerClosePacket>),
    VirtualContainerOpenPacket(Box<<Self as ProtoVersionPackets>::VirtualContainerOpenPacket>),
    VoxelShapesPacket(Box<<Self as ProtoVersionPackets>::VoxelShapesPacket>),
    Unknown(Box<bedrock_network_core::UnknownPacket>),
}
impl bedrock_network_core::Packets for V2259 {
    #[inline]
    fn serialize<W: std::io::Write>(
        &self,
        header: &bedrock_network_core::PacketHeader,
        stream: &mut W,
    ) -> Result<(), bedrock_network_core::error::PacketCodecError> {
        <bedrock_network_core::PacketHeader as bedrock_network_core::ProtoCodec>::serialize(
            header, stream,
        )
        .map_err(bedrock_network_core::error::PacketCodecError::InvalidHeader)?;
        match self {
            V2259::ActorEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ActorPickRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorPickRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AddActorPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AddBehaviourTreePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddBehaviourTreePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AddItemActorPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddItemActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AddPaintingPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPaintingPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AddPlayerPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPlayerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AgentActionEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentActionEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AgentAnimationPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentAnimationPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AnimateEntityPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimateEntityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AnimatePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AnvilDamagePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnvilDamagePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AutomationClientConnectPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AutomationClientConnectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AvailableActorIdentifiersPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableActorIdentifiersPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AvailableCommandsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableCommandsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::AwardAchievementPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AwardAchievementPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::BiomeDefinitionListPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BiomeDefinitionListPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::BlockActorDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockActorDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::BlockEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::BlockPickRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockPickRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::BookEditPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BookEditPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::BossEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BossEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraAimAssistActorPriorityPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistActorPriorityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraAimAssistInstructionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistInstructionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraAimAssistPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraAimAssistPresetsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPresetsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraInstructionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraInstructionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraPresetsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPresetsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraShakePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraShakePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CameraSplinePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraSplinePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ChangeDimensionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeDimensionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ChangeMobPropertyPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeMobPropertyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ChatPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChatPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ChunkRadiusUpdatedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChunkRadiusUpdatedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundAttributeLayerSyncPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundAttributeLayerSyncPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundCloseFormPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundCloseFormPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundControlSchemeSetPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundControlSchemeSetPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundDataDrivenUICloseScreenPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUICloseScreenPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundDataDrivenUIReloadPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataDrivenUIReloadPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundDataDrivenUIShowScreenPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUIShowScreenPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundDataStorePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataStorePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundDebugRendererPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDebugRendererPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundMapItemDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundMapItemDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundMatchmakingStatePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundMatchmakingStatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundMusicPlaybackFailurePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundMusicPlaybackFailurePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundOnlineMusicPlaybackResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundOnlineMusicPlaybackResponsePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundPlayAudioContentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundPlayAudioContentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundStonecutterSetRecipePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundStonecutterSetRecipePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundTextureShiftPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundTextureShiftPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientBoundUpdateSoundDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundUpdateSoundDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientCacheBlobStatusPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheBlobStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientCacheMissResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheMissResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientCacheStatusPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ClientToServerHandshakePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientToServerHandshakePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CodeBuilderPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CodeBuilderSourcePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderSourcePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CommandBlockUpdatePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandBlockUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CommandOutputPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandOutputPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CommandRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CompletedUsingItemPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CompletedUsingItemPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ContainerClosePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerClosePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ContainerOpenPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerOpenPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ContainerRegistryCleanupPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerRegistryCleanupPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ContainerSetDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerSetDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CorrectPlayerMovePredictionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CraftingDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CraftingDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CreatePhotoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreatePhotoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CreativeContentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreativeContentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::CurrentStructureFeaturePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CurrentStructureFeaturePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::DeathInfoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DeathInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::DebugDrawerPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugDrawerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::DebugInfoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::DimensionDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DimensionDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::DisconnectPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DisconnectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::EditorNetworkPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EditorNetworkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::EduUriResourcePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EduUriResourcePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::EducationSettingsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EducationSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::EmoteListPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmoteListPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::EmotePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmotePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::FeatureRegistryPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(FeatureRegistryPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::GameRulesChangedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameRulesChangedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::GameTestRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::GameTestResultsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestResultsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::GraphicsParameterOverridePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GraphicsParameterOverridePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::GuiDataPickItemPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GuiDataPickItemPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::HurtArmorPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(HurtArmorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::InteractPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InteractPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::InventoryContentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryContentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::InventorySlotPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventorySlotPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::InventoryTransactionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryTransactionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ItemComponentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemComponentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ItemStackRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ItemStackResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::JigsawStructureDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(JigsawStructureDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LabTablePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LabTablePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LecternUpdatePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LecternUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LegacyTelemetryEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LegacyTelemetryEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LessonProgressPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LessonProgressPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LevelChunkPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelChunkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LevelEventGenericPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventGenericPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LevelEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LevelSoundEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelSoundEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LocatorBarPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LocatorBarPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::LoginPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LoginPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MapCreateLockedCopyPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapCreateLockedCopyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MapInfoRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapInfoRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MobArmorEquipmentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobArmorEquipmentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MobEffectPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEffectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MobEquipmentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEquipmentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ModalFormRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ModalFormResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MotionPredictionHintsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MotionPredictionHintsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MoveActorAbsolutePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorAbsolutePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MoveActorDeltaPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorDeltaPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MovePlayerPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovePlayerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MovementEffectPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementEffectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MovementPredictionSyncPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementPredictionSyncPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::MultiplayerSettingsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MultiplayerSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::NetworkChunkPublisherUpdatePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::NetworkSettingsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::NetworkStackLatencyPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkStackLatencyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::NpcDialoguePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcDialoguePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::NpcRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::OnScreenTextureAnimationPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OnScreenTextureAnimationPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::OpenSignPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OpenSignPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PacketViolationWarningPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PacketViolationWarningPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PartyChangedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PartyChangedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PartyDestinationCookieResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PartyDestinationCookieResponsePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PhotoTransferPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PhotoTransferPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlaySoundPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlaySoundPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayStatusPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerActionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerActionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerArmorDamagePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerArmorDamagePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerAuthInputPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerAuthInputPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerEnchantOptionsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerEnchantOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerFogPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerFogPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerHotbarPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerHotbarPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerListPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerListPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerLocationPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerLocationPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerSkinPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerSkinPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerStartItemCooldownPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerStartItemCooldownPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerToggleArmorVisibilityRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PlayerToggleArmorVisibilityRequestPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerToggleCrafterSlotRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PlayerToggleCrafterSlotRequestPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerUpdateEntityOverridesPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerUpdateEntityOverridesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PlayerVideoCapturePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerVideoCapturePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PositionTrackingDBClientRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBClientRequestPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PositionTrackingDBServerBroadcastPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBServerBroadcastPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::PurchaseReceiptPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PurchaseReceiptPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RecordStartedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RecordStartedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RefreshEntitlementsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RefreshEntitlementsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RemoveActorPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RemoveObjectivePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveObjectivePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RequestAbilityPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestAbilityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RequestChunkRadiusPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestChunkRadiusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RequestNetworkSettingsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestNetworkSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RequestPermissionsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestPermissionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePackChunkDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePackChunkRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePackClientResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackClientResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePackDataInfoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackDataInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePackStackPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackStackPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePacksInfoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePacksInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ResourcePacksReadyForValidationPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ResourcePacksReadyForValidationPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::RespawnPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RespawnPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ScriptMessagePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ScriptMessagePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SendPartyDestinationCookiePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SendPartyDestinationCookiePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundCursorItemDragPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundCursorItemDragPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundDataDrivenClosedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataDrivenClosedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundDataStorePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataStorePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundDiagnosticsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDiagnosticsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundLoadingScreenPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundLoadingScreenPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundMatchmakingCancelPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundMatchmakingCancelPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundPackSettingChangePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundPackSettingChangePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundRegisterAudioContentPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ServerBoundRegisterAudioContentPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundRequestOnlineMusicPlaybackPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ServerBoundRequestOnlineMusicPlaybackPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerBoundStonecutterSetRecipePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ServerBoundStonecutterSetRecipePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerPlayerPostMovePositionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerPresenceInfoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPresenceInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerSettingsRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerSettingsResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerStatsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStatsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerStoreInfoPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStoreInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ServerToClientHandshakePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerToClientHandshakePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetActorDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetActorLinkPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorLinkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetActorMotionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorMotionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetCommandsEnabledPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetCommandsEnabledPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetDefaultGameTypePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDefaultGameTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetDifficultyPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDifficultyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetDisplayObjectivePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDisplayObjectivePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetHealthPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHealthPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetHudPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHudPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetLastHurtByPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLastHurtByPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetLocalPlayerAsInitializedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetPassengerOfBlockPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPassengerOfBlockPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetPlayerFurnaceOptionsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerFurnaceOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetPlayerGameTypePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerGameTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetPlayerInventoryOptionsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetScorePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScorePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetScoreboardIdentityPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScoreboardIdentityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetSpawnPositionPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetSpawnPositionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetTimePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTimePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SetTitlePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTitlePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SettingsCommandPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SettingsCommandPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ShowCreditsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowCreditsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ShowProfilePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowProfilePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ShowStoreOfferPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowStoreOfferPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SimpleEventPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimpleEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SimulationTypePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimulationTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SpawnExperienceOrbPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnExperienceOrbPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SpawnParticleEffectPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnParticleEffectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::StartGamePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StartGamePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::StopSoundPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StopSoundPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::StructureBlockUpdatePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureBlockUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::StructureDataRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::StructureDataResponsePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SubChunkPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SubChunkRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SubClientLoginPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubClientLoginPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SyncActorPropertyPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncActorPropertyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::SyncWorldClocksPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncWorldClocksPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::TakeItemActorPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TakeItemActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::TextPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TextPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::TickingAreaLoadStatusPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TickingAreaLoadStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::ToastRequestPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ToastRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::TransferPlayerPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TransferPlayerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::TrimDataPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TrimDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UnlockedRecipesPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UnlockedRecipesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateAbilitiesPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAbilitiesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateAdventureSettingsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAdventureSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateAttributesPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAttributesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateBlockPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateBlockSyncedPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockSyncedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateClientInputLocksPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientInputLocksPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateClientOptionsPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateEquipPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateEquipPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdatePlayerGameTypePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdatePlayerGameTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateSoftEnumPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSoftEnumPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateSubChunkBlocksPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSubChunkBlocksPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::UpdateTradePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateTradePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::VirtualContainerClosePacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VirtualContainerClosePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::VirtualContainerOpenPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VirtualContainerOpenPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::VoxelShapesPacket(pk) => {
                match <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VoxelShapesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2259::Unknown(pk) => {
                stream
                    .write_all(pk.buf.as_ref())
                    .map_err(|e| bedrock_network_core::error::PacketCodecError::InvalidPacket {
                        packet_name: "Unknown",
                        packet_id: header.packet_id,
                        error: e.into(),
                    })?;
            }
        }
        Ok(())
    }
    #[inline]
    fn deserialize<R: std::io::Read>(
        stream: &mut R,
    ) -> Result<
        (Self, bedrock_network_core::PacketHeader),
        bedrock_network_core::error::PacketCodecError,
    > {
        let header =
            <bedrock_network_core::PacketHeader as bedrock_network_core::ProtoCodec>::deserialize(
                stream,
            )
            .map_err(bedrock_network_core::error::PacketCodecError::InvalidHeader)?;
        let packet = match header.packet_id {
            <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ActorEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ActorPickRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorPickRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AddActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AddBehaviourTreePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddBehaviourTreePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AddItemActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddItemActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AddPaintingPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPaintingPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AddPlayerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPlayerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AgentActionEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentActionEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AgentAnimationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentAnimationPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AnimateEntityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimateEntityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AnimatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AnvilDamagePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnvilDamagePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AutomationClientConnectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AutomationClientConnectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AvailableActorIdentifiersPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableActorIdentifiersPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AvailableCommandsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableCommandsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::AwardAchievementPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AwardAchievementPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::BiomeDefinitionListPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BiomeDefinitionListPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::BlockActorDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockActorDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::BlockEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::BlockPickRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockPickRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::BookEditPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BookEditPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::BossEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BossEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraAimAssistActorPriorityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistActorPriorityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraAimAssistInstructionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistInstructionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraAimAssistPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraAimAssistPresetsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPresetsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraInstructionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraInstructionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraPresetsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPresetsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraShakePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraShakePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CameraSplinePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraSplinePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ChangeDimensionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeDimensionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ChangeMobPropertyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeMobPropertyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ChatPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChatPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ChunkRadiusUpdatedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChunkRadiusUpdatedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundAttributeLayerSyncPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundAttributeLayerSyncPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundCloseFormPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundCloseFormPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundControlSchemeSetPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundControlSchemeSetPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2259::ClientBoundDataDrivenUICloseScreenPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUICloseScreenPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundDataDrivenUIReloadPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataDrivenUIReloadPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2259::ClientBoundDataDrivenUIShowScreenPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUIShowScreenPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundDataStorePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataStorePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundDebugRendererPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDebugRendererPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundMapItemDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundMapItemDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundMatchmakingStatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundMatchmakingStatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundMusicPlaybackFailurePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundMusicPlaybackFailurePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2259::ClientBoundOnlineMusicPlaybackResponsePacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundOnlineMusicPlaybackResponsePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundPlayAudioContentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundPlayAudioContentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundStonecutterSetRecipePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundStonecutterSetRecipePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundTextureShiftPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundTextureShiftPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientBoundUpdateSoundDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundUpdateSoundDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientCacheBlobStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheBlobStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientCacheMissResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheMissResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientCacheStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ClientToServerHandshakePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientToServerHandshakePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CodeBuilderPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CodeBuilderSourcePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderSourcePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CommandBlockUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandBlockUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CommandOutputPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandOutputPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CommandRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CompletedUsingItemPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CompletedUsingItemPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ContainerClosePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerClosePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ContainerOpenPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerOpenPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ContainerRegistryCleanupPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerRegistryCleanupPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ContainerSetDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerSetDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CorrectPlayerMovePredictionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CraftingDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CraftingDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CreatePhotoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreatePhotoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CreativeContentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreativeContentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::CurrentStructureFeaturePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CurrentStructureFeaturePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::DeathInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DeathInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::DebugDrawerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugDrawerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::DebugInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::DimensionDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DimensionDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::DisconnectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DisconnectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::EditorNetworkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EditorNetworkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::EduUriResourcePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EduUriResourcePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::EducationSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EducationSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::EmoteListPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmoteListPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::EmotePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmotePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::FeatureRegistryPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(FeatureRegistryPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::GameRulesChangedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameRulesChangedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::GameTestRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::GameTestResultsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestResultsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::GraphicsParameterOverridePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GraphicsParameterOverridePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::GuiDataPickItemPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GuiDataPickItemPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::HurtArmorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(HurtArmorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::InteractPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InteractPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::InventoryContentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryContentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::InventorySlotPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventorySlotPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::InventoryTransactionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryTransactionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ItemComponentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemComponentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ItemStackRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ItemStackResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::JigsawStructureDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(JigsawStructureDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LabTablePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LabTablePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LecternUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LecternUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LegacyTelemetryEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LegacyTelemetryEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LessonProgressPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LessonProgressPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LevelChunkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelChunkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LevelEventGenericPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventGenericPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LevelEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LevelSoundEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelSoundEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LocatorBarPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LocatorBarPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::LoginPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LoginPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MapCreateLockedCopyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapCreateLockedCopyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MapInfoRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapInfoRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MobArmorEquipmentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobArmorEquipmentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MobEffectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEffectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MobEquipmentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEquipmentPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ModalFormRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ModalFormResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MotionPredictionHintsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MotionPredictionHintsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MoveActorAbsolutePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorAbsolutePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MoveActorDeltaPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorDeltaPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MovePlayerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovePlayerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MovementEffectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementEffectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MovementPredictionSyncPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementPredictionSyncPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::MultiplayerSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MultiplayerSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::NetworkChunkPublisherUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::NetworkSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::NetworkStackLatencyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkStackLatencyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::NpcDialoguePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcDialoguePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::NpcRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::OnScreenTextureAnimationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OnScreenTextureAnimationPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::OpenSignPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OpenSignPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PacketViolationWarningPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PacketViolationWarningPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PartyChangedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PartyChangedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PartyDestinationCookieResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PartyDestinationCookieResponsePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PhotoTransferPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PhotoTransferPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlaySoundPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlaySoundPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerActionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerActionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerArmorDamagePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerArmorDamagePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerAuthInputPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerAuthInputPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerEnchantOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerEnchantOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerFogPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerFogPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerHotbarPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerHotbarPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerListPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerListPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerLocationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerLocationPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerSkinPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerSkinPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerStartItemCooldownPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerStartItemCooldownPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2259::PlayerToggleArmorVisibilityRequestPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PlayerToggleArmorVisibilityRequestPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerToggleCrafterSlotRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PlayerToggleCrafterSlotRequestPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerUpdateEntityOverridesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerUpdateEntityOverridesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PlayerVideoCapturePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerVideoCapturePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PositionTrackingDBClientRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBClientRequestPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2259::PositionTrackingDBServerBroadcastPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBServerBroadcastPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::PurchaseReceiptPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PurchaseReceiptPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RecordStartedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RecordStartedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RefreshEntitlementsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RefreshEntitlementsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RemoveActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RemoveObjectivePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveObjectivePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RequestAbilityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestAbilityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RequestChunkRadiusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestChunkRadiusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RequestNetworkSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestNetworkSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RequestPermissionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestPermissionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePackChunkDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePackChunkRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePackClientResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackClientResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePackDataInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackDataInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePackStackPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackStackPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePacksInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePacksInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ResourcePacksReadyForValidationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ResourcePacksReadyForValidationPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::RespawnPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RespawnPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ScriptMessagePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ScriptMessagePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SendPartyDestinationCookiePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SendPartyDestinationCookiePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundCursorItemDragPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundCursorItemDragPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundDataDrivenClosedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataDrivenClosedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundDataStorePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataStorePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundDiagnosticsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDiagnosticsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundLoadingScreenPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundLoadingScreenPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundMatchmakingCancelPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundMatchmakingCancelPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundPackSettingChangePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundPackSettingChangePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundRegisterAudioContentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ServerBoundRegisterAudioContentPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2259::ServerBoundRequestOnlineMusicPlaybackPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ServerBoundRequestOnlineMusicPlaybackPacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerBoundStonecutterSetRecipePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ServerBoundStonecutterSetRecipePacket
                            ),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerPlayerPostMovePositionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerPresenceInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPresenceInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerSettingsRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerSettingsResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerStatsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStatsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerStoreInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStoreInfoPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ServerToClientHandshakePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerToClientHandshakePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetActorDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetActorLinkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorLinkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetActorMotionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorMotionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetCommandsEnabledPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetCommandsEnabledPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetDefaultGameTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDefaultGameTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetDifficultyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDifficultyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetDisplayObjectivePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDisplayObjectivePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetHealthPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHealthPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetHudPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHudPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetLastHurtByPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLastHurtByPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetLocalPlayerAsInitializedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetPassengerOfBlockPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPassengerOfBlockPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetPlayerFurnaceOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerFurnaceOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetPlayerGameTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerGameTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetPlayerInventoryOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetScorePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScorePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetScoreboardIdentityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScoreboardIdentityPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetSpawnPositionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetSpawnPositionPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetTimePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTimePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SetTitlePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTitlePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SettingsCommandPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SettingsCommandPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ShowCreditsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowCreditsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ShowProfilePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowProfilePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ShowStoreOfferPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowStoreOfferPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SimpleEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimpleEventPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SimulationTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimulationTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SpawnExperienceOrbPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnExperienceOrbPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SpawnParticleEffectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnParticleEffectPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::StartGamePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StartGamePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::StopSoundPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StopSoundPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::StructureBlockUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureBlockUpdatePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::StructureDataRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::StructureDataResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataResponsePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SubChunkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SubChunkRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SubClientLoginPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubClientLoginPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SyncActorPropertyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncActorPropertyPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::SyncWorldClocksPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncWorldClocksPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::TakeItemActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TakeItemActorPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::TextPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TextPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::TickingAreaLoadStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TickingAreaLoadStatusPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::ToastRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ToastRequestPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::TransferPlayerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TransferPlayerPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::TrimDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TrimDataPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UnlockedRecipesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UnlockedRecipesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateAbilitiesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAbilitiesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateAdventureSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAdventureSettingsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateAttributesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAttributesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateBlockPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateBlockSyncedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockSyncedPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateClientInputLocksPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientInputLocksPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateClientOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientOptionsPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateEquipPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateEquipPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdatePlayerGameTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdatePlayerGameTypePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateSoftEnumPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSoftEnumPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateSubChunkBlocksPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSubChunkBlocksPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::UpdateTradePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateTradePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::VirtualContainerClosePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VirtualContainerClosePacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::VirtualContainerOpenPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VirtualContainerOpenPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::Packet>::ID => {
                match <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2259::VoxelShapesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_network_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VoxelShapesPacket),
                            packet_id: <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            unknown => {
                let mut buf = Vec::new();
                stream
                    .read_to_end(&mut buf)
                    .map_err(|e| bedrock_network_core::error::PacketCodecError::InvalidPacket {
                        packet_name: "Unknown",
                        packet_id: header.packet_id,
                        error: e.into(),
                    })?;
                V2259::Unknown(
                    Box::new(bedrock_network_core::UnknownPacket {
                        id: unknown,
                        buf: buf.into_boxed_slice(),
                    }),
                )
            }
        };
        Ok((packet, header))
    }
    #[inline]
    fn size_hint(&self, header: &bedrock_network_core::PacketHeader) -> usize {
        <bedrock_network_core::PacketHeader as bedrock_network_core::ProtoCodec>::size_hint(
            header,
        )
            + match self {
                V2259::ActorEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ActorPickRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AddActorPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AddBehaviourTreePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AddItemActorPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AddPaintingPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AddPlayerPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AgentActionEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AgentAnimationPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AnimateEntityPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AnimatePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AnvilDamagePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AutomationClientConnectPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AvailableActorIdentifiersPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AvailableCommandsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::AwardAchievementPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::BiomeDefinitionListPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::BlockActorDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::BlockEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::BlockPickRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::BookEditPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::BossEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraAimAssistActorPriorityPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraAimAssistInstructionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraAimAssistPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraAimAssistPresetsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraInstructionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraPresetsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraShakePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CameraSplinePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ChangeDimensionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ChangeMobPropertyPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ChatPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ChunkRadiusUpdatedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundAttributeLayerSyncPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundCloseFormPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundControlSchemeSetPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundDataDrivenUICloseScreenPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundDataDrivenUIReloadPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundDataDrivenUIShowScreenPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundDataStorePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundDebugRendererPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundMapItemDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundMatchmakingStatePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundMusicPlaybackFailurePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundOnlineMusicPlaybackResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundPlayAudioContentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundStonecutterSetRecipePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundTextureShiftPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientBoundUpdateSoundDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientCacheBlobStatusPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientCacheMissResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientCacheStatusPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ClientToServerHandshakePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CodeBuilderPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CodeBuilderSourcePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CommandBlockUpdatePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CommandOutputPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CommandRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CompletedUsingItemPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ContainerClosePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ContainerOpenPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ContainerRegistryCleanupPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ContainerSetDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CorrectPlayerMovePredictionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CraftingDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CreatePhotoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CreativeContentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::CurrentStructureFeaturePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::DeathInfoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::DebugDrawerPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::DebugInfoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::DimensionDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::DisconnectPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::EditorNetworkPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::EduUriResourcePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::EducationSettingsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::EmoteListPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::EmotePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::FeatureRegistryPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::GameRulesChangedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::GameTestRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::GameTestResultsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::GraphicsParameterOverridePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::GuiDataPickItemPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::HurtArmorPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::InteractPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::InventoryContentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::InventorySlotPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::InventoryTransactionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ItemComponentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ItemStackRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ItemStackResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::JigsawStructureDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LabTablePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LecternUpdatePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LegacyTelemetryEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LessonProgressPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LevelChunkPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LevelEventGenericPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LevelEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LevelSoundEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LocatorBarPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::LoginPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MapCreateLockedCopyPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MapInfoRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MobArmorEquipmentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MobEffectPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MobEquipmentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ModalFormRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ModalFormResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MotionPredictionHintsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MoveActorAbsolutePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MoveActorDeltaPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MovePlayerPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MovementEffectPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MovementPredictionSyncPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::MultiplayerSettingsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::NetworkChunkPublisherUpdatePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::NetworkSettingsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::NetworkStackLatencyPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::NpcDialoguePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::NpcRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::OnScreenTextureAnimationPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::OpenSignPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PacketViolationWarningPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PartyChangedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PartyDestinationCookieResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PhotoTransferPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlaySoundPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayStatusPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerActionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerArmorDamagePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerAuthInputPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerEnchantOptionsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerFogPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerHotbarPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerListPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerLocationPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerSkinPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerStartItemCooldownPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerToggleArmorVisibilityRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerToggleCrafterSlotRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerUpdateEntityOverridesPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PlayerVideoCapturePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PositionTrackingDBClientRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PositionTrackingDBServerBroadcastPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::PurchaseReceiptPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RecordStartedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RefreshEntitlementsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RemoveActorPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RemoveObjectivePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RequestAbilityPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RequestChunkRadiusPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RequestNetworkSettingsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RequestPermissionsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePackChunkDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePackChunkRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePackClientResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePackDataInfoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePackStackPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePacksInfoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ResourcePacksReadyForValidationPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::RespawnPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ScriptMessagePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SendPartyDestinationCookiePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundCursorItemDragPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundDataDrivenClosedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundDataStorePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundDiagnosticsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundLoadingScreenPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundMatchmakingCancelPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundPackSettingChangePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundRegisterAudioContentPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundRequestOnlineMusicPlaybackPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerBoundStonecutterSetRecipePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerPlayerPostMovePositionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerPresenceInfoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerSettingsRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerSettingsResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerStatsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerStoreInfoPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ServerToClientHandshakePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetActorDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetActorLinkPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetActorMotionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetCommandsEnabledPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetDefaultGameTypePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetDifficultyPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetDisplayObjectivePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetHealthPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetHudPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetLastHurtByPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetLocalPlayerAsInitializedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetPassengerOfBlockPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetPlayerFurnaceOptionsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetPlayerGameTypePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetPlayerInventoryOptionsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetScorePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetScoreboardIdentityPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetSpawnPositionPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetTimePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SetTitlePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SettingsCommandPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ShowCreditsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ShowProfilePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ShowStoreOfferPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SimpleEventPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SimulationTypePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SpawnExperienceOrbPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SpawnParticleEffectPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::StartGamePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::StopSoundPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::StructureBlockUpdatePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::StructureDataRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::StructureDataResponsePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SubChunkPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SubChunkRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SubClientLoginPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SyncActorPropertyPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::SyncWorldClocksPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::TakeItemActorPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::TextPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::TickingAreaLoadStatusPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::ToastRequestPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::TransferPlayerPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::TrimDataPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UnlockedRecipesPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateAbilitiesPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateAdventureSettingsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateAttributesPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateBlockPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateBlockSyncedPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateClientInputLocksPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateClientOptionsPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateEquipPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdatePlayerGameTypePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateSoftEnumPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateSubChunkBlocksPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::UpdateTradePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::VirtualContainerClosePacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::VirtualContainerOpenPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::VoxelShapesPacket(pk) => {
                    <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2259::Unknown(pk) => pk.buf.len(),
            }
    }
    #[inline]
    fn id(&self) -> u16 {
        match self {
            V2259::ActorEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ActorEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ActorPickRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AddActorPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AddActorPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AddBehaviourTreePacket(_) => {
                <<V2259 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_network_core::Packet>::ID
            }
            V2259::AddItemActorPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AddItemActorPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AddPaintingPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AddPaintingPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AddPlayerPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AddPlayerPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AgentActionEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AgentAnimationPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AnimateEntityPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AnimatePacket(_) => {
                <<V2259 as ProtoVersionPackets>::AnimatePacket as bedrock_network_core::Packet>::ID
            }
            V2259::AnvilDamagePacket(_) => {
                <<V2259 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_network_core::Packet>::ID
            }
            V2259::AutomationClientConnectPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AvailableActorIdentifiersPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AvailableCommandsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::AwardAchievementPacket(_) => {
                <<V2259 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_network_core::Packet>::ID
            }
            V2259::BiomeDefinitionListPacket(_) => {
                <<V2259 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_network_core::Packet>::ID
            }
            V2259::BlockActorDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::BlockEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::BlockEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::BlockPickRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::BookEditPacket(_) => {
                <<V2259 as ProtoVersionPackets>::BookEditPacket as bedrock_network_core::Packet>::ID
            }
            V2259::BossEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::BossEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraAimAssistActorPriorityPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraAimAssistInstructionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraAimAssistPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraAimAssistPresetsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraInstructionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraPresetsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraShakePacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraShakePacket as bedrock_network_core::Packet>::ID
            }
            V2259::CameraSplinePacket(_) => {
                <<V2259 as ProtoVersionPackets>::CameraSplinePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ChangeDimensionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ChangeMobPropertyPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ChatPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ChatPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ChunkRadiusUpdatedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundAttributeLayerSyncPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundCloseFormPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundControlSchemeSetPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundDataDrivenUICloseScreenPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundDataDrivenUIReloadPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundDataDrivenUIShowScreenPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundDataStorePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundDebugRendererPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundMapItemDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundMatchmakingStatePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundMatchmakingStatePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundMusicPlaybackFailurePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundMusicPlaybackFailurePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundOnlineMusicPlaybackResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundOnlineMusicPlaybackResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundPlayAudioContentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundPlayAudioContentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundStonecutterSetRecipePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundTextureShiftPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientBoundUpdateSoundDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientCacheBlobStatusPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientCacheMissResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientCacheStatusPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ClientToServerHandshakePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_network_core::Packet>::ID
            }
            V2259::CodeBuilderPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CodeBuilderSourcePacket(_) => {
                <<V2259 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_network_core::Packet>::ID
            }
            V2259::CommandBlockUpdatePacket(_) => {
                <<V2259 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_network_core::Packet>::ID
            }
            V2259::CommandOutputPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CommandOutputPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CommandRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CommandRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CompletedUsingItemPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ContainerClosePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ContainerClosePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ContainerOpenPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ContainerRegistryCleanupPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ContainerSetDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CorrectPlayerMovePredictionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CraftingDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CraftingDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CreatePhotoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CreativeContentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::CreativeContentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::CurrentStructureFeaturePacket(_) => {
                <<V2259 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_network_core::Packet>::ID
            }
            V2259::DeathInfoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::DeathInfoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::DebugDrawerPacket(_) => {
                <<V2259 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_network_core::Packet>::ID
            }
            V2259::DebugInfoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::DebugInfoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::DimensionDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::DimensionDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::DisconnectPacket(_) => {
                <<V2259 as ProtoVersionPackets>::DisconnectPacket as bedrock_network_core::Packet>::ID
            }
            V2259::EditorNetworkPacket(_) => {
                <<V2259 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_network_core::Packet>::ID
            }
            V2259::EduUriResourcePacket(_) => {
                <<V2259 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_network_core::Packet>::ID
            }
            V2259::EducationSettingsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::EmoteListPacket(_) => {
                <<V2259 as ProtoVersionPackets>::EmoteListPacket as bedrock_network_core::Packet>::ID
            }
            V2259::EmotePacket(_) => {
                <<V2259 as ProtoVersionPackets>::EmotePacket as bedrock_network_core::Packet>::ID
            }
            V2259::FeatureRegistryPacket(_) => {
                <<V2259 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_network_core::Packet>::ID
            }
            V2259::GameRulesChangedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::GameTestRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::GameTestResultsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::GraphicsParameterOverridePacket(_) => {
                <<V2259 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_network_core::Packet>::ID
            }
            V2259::GuiDataPickItemPacket(_) => {
                <<V2259 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_network_core::Packet>::ID
            }
            V2259::HurtArmorPacket(_) => {
                <<V2259 as ProtoVersionPackets>::HurtArmorPacket as bedrock_network_core::Packet>::ID
            }
            V2259::InteractPacket(_) => {
                <<V2259 as ProtoVersionPackets>::InteractPacket as bedrock_network_core::Packet>::ID
            }
            V2259::InventoryContentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::InventoryContentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::InventorySlotPacket(_) => {
                <<V2259 as ProtoVersionPackets>::InventorySlotPacket as bedrock_network_core::Packet>::ID
            }
            V2259::InventoryTransactionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ItemComponentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ItemComponentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ItemStackRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ItemStackResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::JigsawStructureDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LabTablePacket(_) => {
                <<V2259 as ProtoVersionPackets>::LabTablePacket as bedrock_network_core::Packet>::ID
            }
            V2259::LecternUpdatePacket(_) => {
                <<V2259 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_network_core::Packet>::ID
            }
            V2259::LegacyTelemetryEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LessonProgressPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LessonProgressPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LevelChunkPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LevelChunkPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LevelEventGenericPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LevelEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LevelEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LevelSoundEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LocatorBarPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LocatorBarPacket as bedrock_network_core::Packet>::ID
            }
            V2259::LoginPacket(_) => {
                <<V2259 as ProtoVersionPackets>::LoginPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MapCreateLockedCopyPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MapInfoRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MobArmorEquipmentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MobEffectPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MobEffectPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MobEquipmentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ModalFormRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ModalFormResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::MotionPredictionHintsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MoveActorAbsolutePacket(_) => {
                <<V2259 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_network_core::Packet>::ID
            }
            V2259::MoveActorDeltaPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MovePlayerPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MovePlayerPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MovementEffectPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MovementEffectPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MovementPredictionSyncPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_network_core::Packet>::ID
            }
            V2259::MultiplayerSettingsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::NetworkChunkPublisherUpdatePacket(_) => {
                <<V2259 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_network_core::Packet>::ID
            }
            V2259::NetworkSettingsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::NetworkStackLatencyPacket(_) => {
                <<V2259 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_network_core::Packet>::ID
            }
            V2259::NpcDialoguePacket(_) => {
                <<V2259 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_network_core::Packet>::ID
            }
            V2259::NpcRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::NpcRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::OnScreenTextureAnimationPacket(_) => {
                <<V2259 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_network_core::Packet>::ID
            }
            V2259::OpenSignPacket(_) => {
                <<V2259 as ProtoVersionPackets>::OpenSignPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PacketViolationWarningPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PartyChangedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PartyChangedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PartyDestinationCookieResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::PhotoTransferPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlaySoundPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlaySoundPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayStatusPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayStatusPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerActionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerActionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerArmorDamagePacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerAuthInputPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerEnchantOptionsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerFogPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerFogPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerHotbarPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerListPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerListPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerLocationPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerSkinPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerStartItemCooldownPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerToggleArmorVisibilityRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerToggleArmorVisibilityRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerToggleCrafterSlotRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerUpdateEntityOverridesPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PlayerVideoCapturePacket(_) => {
                <<V2259 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_network_core::Packet>::ID
            }
            V2259::PositionTrackingDBClientRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PositionTrackingDBServerBroadcastPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_network_core::Packet>::ID
            }
            V2259::PurchaseReceiptPacket(_) => {
                <<V2259 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RecordStartedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RecordStartedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RefreshEntitlementsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RemoveActorPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RemoveActorPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RemoveObjectivePacket(_) => {
                <<V2259 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_network_core::Packet>::ID
            }
            V2259::RequestAbilityPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RequestChunkRadiusPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RequestNetworkSettingsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RequestPermissionsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePackChunkDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePackChunkRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePackClientResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePackDataInfoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePackStackPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePacksInfoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ResourcePacksReadyForValidationPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_network_core::Packet>::ID
            }
            V2259::RespawnPacket(_) => {
                <<V2259 as ProtoVersionPackets>::RespawnPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ScriptMessagePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SendPartyDestinationCookiePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundCursorItemDragPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundCursorItemDragPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundDataDrivenClosedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundDataStorePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundDiagnosticsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundLoadingScreenPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundMatchmakingCancelPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundMatchmakingCancelPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundPackSettingChangePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundRegisterAudioContentPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundRegisterAudioContentPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundRequestOnlineMusicPlaybackPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundRequestOnlineMusicPlaybackPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerBoundStonecutterSetRecipePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerBoundStonecutterSetRecipePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerPlayerPostMovePositionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerPresenceInfoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerSettingsRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerSettingsResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerStatsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerStatsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerStoreInfoPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ServerToClientHandshakePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetActorDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetActorDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetActorLinkPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetActorMotionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetCommandsEnabledPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetDefaultGameTypePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetDifficultyPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetDisplayObjectivePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetHealthPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetHealthPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetHudPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetHudPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetLastHurtByPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetLocalPlayerAsInitializedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetPassengerOfBlockPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetPassengerOfBlockPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetPlayerFurnaceOptionsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetPlayerFurnaceOptionsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetPlayerGameTypePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetPlayerInventoryOptionsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetScorePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetScorePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetScoreboardIdentityPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetSpawnPositionPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetTimePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetTimePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SetTitlePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SetTitlePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SettingsCommandPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ShowCreditsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ShowProfilePacket(_) => {
                <<V2259 as ProtoVersionPackets>::ShowProfilePacket as bedrock_network_core::Packet>::ID
            }
            V2259::ShowStoreOfferPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SimpleEventPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SimpleEventPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SimulationTypePacket(_) => {
                <<V2259 as ProtoVersionPackets>::SimulationTypePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SpawnExperienceOrbPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SpawnParticleEffectPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_network_core::Packet>::ID
            }
            V2259::StartGamePacket(_) => {
                <<V2259 as ProtoVersionPackets>::StartGamePacket as bedrock_network_core::Packet>::ID
            }
            V2259::StopSoundPacket(_) => {
                <<V2259 as ProtoVersionPackets>::StopSoundPacket as bedrock_network_core::Packet>::ID
            }
            V2259::StructureBlockUpdatePacket(_) => {
                <<V2259 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_network_core::Packet>::ID
            }
            V2259::StructureDataRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::StructureDataResponsePacket(_) => {
                <<V2259 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_network_core::Packet>::ID
            }
            V2259::SubChunkPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SubChunkPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SubChunkRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SubClientLoginPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SyncActorPropertyPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_network_core::Packet>::ID
            }
            V2259::SyncWorldClocksPacket(_) => {
                <<V2259 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_network_core::Packet>::ID
            }
            V2259::TakeItemActorPacket(_) => {
                <<V2259 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_network_core::Packet>::ID
            }
            V2259::TextPacket(_) => {
                <<V2259 as ProtoVersionPackets>::TextPacket as bedrock_network_core::Packet>::ID
            }
            V2259::TickingAreaLoadStatusPacket(_) => {
                <<V2259 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_network_core::Packet>::ID
            }
            V2259::ToastRequestPacket(_) => {
                <<V2259 as ProtoVersionPackets>::ToastRequestPacket as bedrock_network_core::Packet>::ID
            }
            V2259::TransferPlayerPacket(_) => {
                <<V2259 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_network_core::Packet>::ID
            }
            V2259::TrimDataPacket(_) => {
                <<V2259 as ProtoVersionPackets>::TrimDataPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UnlockedRecipesPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateAbilitiesPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateAdventureSettingsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateAttributesPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateBlockPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateBlockSyncedPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateClientInputLocksPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateClientOptionsPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateEquipPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdatePlayerGameTypePacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateSoftEnumPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateSubChunkBlocksPacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_network_core::Packet>::ID
            }
            V2259::UpdateTradePacket(_) => {
                <<V2259 as ProtoVersionPackets>::UpdateTradePacket as bedrock_network_core::Packet>::ID
            }
            V2259::VirtualContainerClosePacket(_) => {
                <<V2259 as ProtoVersionPackets>::VirtualContainerClosePacket as bedrock_network_core::Packet>::ID
            }
            V2259::VirtualContainerOpenPacket(_) => {
                <<V2259 as ProtoVersionPackets>::VirtualContainerOpenPacket as bedrock_network_core::Packet>::ID
            }
            V2259::VoxelShapesPacket(_) => {
                <<V2259 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_network_core::Packet>::ID
            }
            V2259::Unknown(pk) => pk.id,
        }
    }
}
impl ProtoVersionPackets for V2259 {
    type ActorEventPacket = bedrock_network_version_v975::packets::ActorEventPacket<Self>;
    type ActorPickRequestPacket = bedrock_network_version_v662::packets::ActorPickRequestPacket;
    type AddActorPacket = bedrock_network_version_v2225::packets::AddActorPacket<Self>;
    type AddBehaviourTreePacket = bedrock_network_version_v662::packets::AddBehaviourTreePacket;
    type AddItemActorPacket = bedrock_network_version_v662::packets::AddItemActorPacket<Self>;
    type AddPaintingPacket = bedrock_network_version_v662::packets::AddPaintingPacket<Self>;
    type AddPlayerPacket = bedrock_network_version_v2225::packets::AddPlayerPacket<Self>;
    type AddVolumeEntityPacket = ();
    type AgentActionEventPacket =
        bedrock_network_version_v662::packets::AgentActionEventPacket<Self>;
    type AgentAnimationPacket = bedrock_network_version_v662::packets::AgentAnimationPacket<Self>;
    type AnimateEntityPacket = bedrock_network_version_v662::packets::AnimateEntityPacket<Self>;
    type AnimatePacket = bedrock_network_version_v2225::packets::AnimatePacket<Self>;
    type AnvilDamagePacket = bedrock_network_version_v2168::packets::AnvilDamagePacket<Self>;
    type AutomationClientConnectPacket =
        bedrock_network_version_v662::packets::AutomationClientConnectPacket<Self>;
    type AvailableActorIdentifiersPacket =
        bedrock_network_version_v662::packets::AvailableActorIdentifiersPacket;
    type AvailableCommandsPacket = bedrock_network_version_v898::packets::AvailableCommandsPacket;
    type AwardAchievementPacket = bedrock_network_version_v685::packets::AwardAchievementPacket;
    type BiomeDefinitionListPacket =
        bedrock_network_version_v800::packets::BiomeDefinitionListPacket<Self>;
    type BlockActorDataPacket = bedrock_network_version_v662::packets::BlockActorDataPacket<Self>;
    type BlockEventPacket = bedrock_network_version_v662::packets::BlockEventPacket<Self>;
    type BlockPickRequestPacket =
        bedrock_network_version_v662::packets::BlockPickRequestPacket<Self>;
    type BookEditPacket = bedrock_network_version_v924::packets::BookEditPacket<Self>;
    type BossEventPacket = bedrock_network_version_v2193::packets::BossEventPacket<Self>;
    type CameraAimAssistActorPriorityPacket =
        bedrock_network_version_v924::packets::CameraAimAssistActorPriorityPacket;
    type CameraAimAssistInstructionPacket =
        bedrock_network_version_v776::packets::CameraAimAssistInstructionPacket<Self>;
    type CameraAimAssistPacket = bedrock_network_version_v827::packets::CameraAimAssistPacket<Self>;
    type CameraAimAssistPresetsPacket =
        bedrock_network_version_v800::packets::CameraAimAssistPresetsPacket<Self>;
    type CameraInstructionPacket =
        bedrock_network_version_v924::packets::CameraInstructionPacket<Self>;
    type CameraPacket = bedrock_network_version_v662::packets::CameraPacket<Self>;
    type CameraPresetsPacket = bedrock_network_version_v662::packets::CameraPresetsPacket<Self>;
    type CameraShakePacket = bedrock_network_version_v662::packets::CameraShakePacket<Self>;
    type CameraSplinePacket = bedrock_network_version_v924::packets::CameraSplinePacket<Self>;
    type ChangeDimensionPacket = bedrock_network_version_v712::packets::ChangeDimensionPacket;
    type ChangeMobPropertyPacket =
        bedrock_network_version_v662::packets::ChangeMobPropertyPacket<Self>;
    type ChatPacket = crate::packets::ChatPacket;
    type ChunkRadiusUpdatedPacket = bedrock_network_version_v662::packets::ChunkRadiusUpdatedPacket;
    type ClientBoundAttributeLayerSyncPacket = crate::packets::ClientBoundAttributeLayerSyncPacket;
    type ClientBoundCloseFormPacket =
        bedrock_network_version_v686::packets::ClientBoundCloseFormPacket;
    type ClientBoundControlSchemeSetPacket =
        bedrock_network_version_v800::packets::ClientBoundControlSchemeSetPacket<Self>;
    type ClientBoundDataDrivenUICloseAllScreensPacket = ();
    type ClientBoundDataDrivenUICloseScreenPacket =
        bedrock_network_version_v944::packets::ClientBoundDataDrivenUICloseScreenPacket;
    type ClientBoundDataDrivenUIReloadPacket =
        bedrock_network_version_v924::packets::ClientBoundDataDrivenUIReloadPacket;
    type ClientBoundDataDrivenUIShowScreenPacket =
        bedrock_network_version_v944::packets::ClientBoundDataDrivenUIShowScreenPacket;
    type ClientBoundDataStorePacket = crate::packets::ClientBoundDataStorePacket;
    type ClientBoundDebugRendererPacket =
        bedrock_network_version_v671::packets::ClientBoundDebugRendererPacket;
    type ClientBoundMapItemDataPacket =
        bedrock_network_version_v2168::packets::ClientBoundMapItemDataPacket<Self>;
    type ClientBoundMatchmakingStatePacket =
        bedrock_network_version_v2225::packets::ClientBoundMatchmakingStatePacket;
    type ClientBoundMusicPlaybackFailurePacket =
        crate::packets::ClientBoundMusicPlaybackFailurePacket;
    type ClientBoundOnlineMusicPlaybackResponsePacket =
        crate::packets::ClientBoundOnlineMusicPlaybackResponsePacket<Self>;
    type ClientBoundPlayAudioContentPacket =
        crate::packets::ClientBoundPlayAudioContentPacket<Self>;
    type ClientBoundStonecutterSetRecipePacket =
        bedrock_network_version_v2225::packets::ClientBoundStonecutterSetRecipePacket<Self>;
    type ClientBoundTextureShiftPacket =
        bedrock_network_version_v924::packets::ClientBoundTextureShiftPacket;
    type ClientBoundUpdateSoundDataPacket =
        bedrock_network_version_v2225::packets::ClientBoundUpdateSoundDataPacket<Self>;
    type ClientCacheBlobStatusPacket =
        bedrock_network_version_v1001::packets::ClientCacheBlobStatusPacket;
    type ClientCacheMissResponsePacket =
        bedrock_network_version_v662::packets::ClientCacheMissResponsePacket;
    type ClientCacheStatusPacket = bedrock_network_version_v662::packets::ClientCacheStatusPacket;
    type ClientToServerHandshakePacket =
        bedrock_network_version_v662::packets::ClientToServerHandshakePacket;
    type CodeBuilderPacket = bedrock_network_version_v662::packets::CodeBuilderPacket;
    type CodeBuilderSourcePacket =
        bedrock_network_version_v685::packets::CodeBuilderSourcePacket<Self>;
    type CommandBlockUpdatePacket = crate::packets::CommandBlockUpdatePacket<Self>;
    type CommandOutputPacket = crate::packets::CommandOutputPacket<Self>;
    type CommandRequestPacket = bedrock_network_version_v898::packets::CommandRequestPacket<Self>;
    type CompletedUsingItemPacket =
        bedrock_network_version_v662::packets::CompletedUsingItemPacket<Self>;
    type CompressedBiomeDefinitionListPacket = ();
    type ContainerClosePacket = bedrock_network_version_v685::packets::ContainerClosePacket<Self>;
    type ContainerOpenPacket = bedrock_network_version_v662::packets::ContainerOpenPacket<Self>;
    type ContainerRegistryCleanupPacket =
        bedrock_network_version_v729::packets::ContainerRegistryCleanupPacket<Self>;
    type ContainerSetDataPacket =
        bedrock_network_version_v662::packets::ContainerSetDataPacket<Self>;
    type CorrectPlayerMovePredictionPacket =
        bedrock_network_version_v827::packets::CorrectPlayerMovePredictionPacket<Self>;
    type CraftingDataPacket = bedrock_network_version_v2168::packets::CraftingDataPacket<Self>;
    type CreatePhotoPacket = bedrock_network_version_v662::packets::CreatePhotoPacket;
    type CreativeContentPacket =
        bedrock_network_version_v2168::packets::CreativeContentPacket<Self>;
    type CurrentStructureFeaturePacket =
        bedrock_network_version_v712::packets::CurrentStructureFeaturePacket;
    type DeathInfoPacket = bedrock_network_version_v662::packets::DeathInfoPacket;
    type DebugDrawerPacket = bedrock_network_version_v818::packets::DebugDrawerPacket<Self>;
    type DebugInfoPacket = bedrock_network_version_v662::packets::DebugInfoPacket<Self>;
    type DimensionDataPacket = bedrock_network_version_v662::packets::DimensionDataPacket<Self>;
    type DisconnectPacket = bedrock_network_version_v712::packets::DisconnectPacket<Self>;
    type EditorNetworkPacket = bedrock_network_version_v662::packets::EditorNetworkPacket;
    type EduUriResourcePacket = bedrock_network_version_v662::packets::EduUriResourcePacket<Self>;
    type EducationSettingsPacket =
        bedrock_network_version_v662::packets::EducationSettingsPacket<Self>;
    type EmoteListPacket = bedrock_network_version_v662::packets::EmoteListPacket<Self>;
    type EmotePacket = bedrock_network_version_v729::packets::EmotePacket<Self>;
    type FeatureRegistryPacket = bedrock_network_version_v662::packets::FeatureRegistryPacket;
    type FilterTextPacket = ();
    type GameRulesChangedPacket =
        bedrock_network_version_v662::packets::GameRulesChangedPacket<Self>;
    type GameTestRequestPacket = bedrock_network_version_v662::packets::GameTestRequestPacket<Self>;
    type GameTestResultsPacket = bedrock_network_version_v662::packets::GameTestResultsPacket;
    type GraphicsParameterOverridePacket =
        bedrock_network_version_v1001::packets::GraphicsParameterOverridePacket;
    type GuiDataPickItemPacket = bedrock_network_version_v662::packets::GuiDataPickItemPacket;
    type HurtArmorPacket = bedrock_network_version_v662::packets::HurtArmorPacket;
    type InteractPacket = bedrock_network_version_v898::packets::InteractPacket<Self>;
    type InventoryContentPacket =
        bedrock_network_version_v1001::packets::InventoryContentPacket<Self>;
    type InventorySlotPacket = bedrock_network_version_v975::packets::InventorySlotPacket<Self>;
    type InventoryTransactionPacket =
        bedrock_network_version_v2193::packets::InventoryTransactionPacket<Self>;
    type ItemComponentPacket = bedrock_network_version_v776::packets::ItemComponentPacket<Self>;
    type ItemStackRequestPacket =
        bedrock_network_version_v662::packets::ItemStackRequestPacket<Self>;
    type ItemStackResponsePacket =
        bedrock_network_version_v662::packets::ItemStackResponsePacket<Self>;
    type JigsawStructureDataPacket =
        bedrock_network_version_v712::packets::JigsawStructureDataPacket;
    type LabTablePacket = bedrock_network_version_v662::packets::LabTablePacket<Self>;
    type LecternUpdatePacket = bedrock_network_version_v662::packets::LecternUpdatePacket<Self>;
    type LegacyTelemetryEventPacket =
        bedrock_network_version_v898::packets::LegacyTelemetryEventPacket<Self>;
    type LessonProgressPacket = bedrock_network_version_v662::packets::LessonProgressPacket<Self>;
    type LevelChunkPacket = bedrock_network_version_v2225::packets::LevelChunkPacket<Self>;
    type LevelEventGenericPacket =
        bedrock_network_version_v662::packets::LevelEventGenericPacket<Self>;
    type LevelEventPacket = bedrock_network_version_v662::packets::LevelEventPacket;
    type LevelSoundEventPacket = bedrock_network_version_v1001::packets::LevelSoundEventPacket;
    type LevelSoundEventV1Packet = ();
    type LevelSoundEventV2Packet = ();
    type LocatorBarPacket = bedrock_network_version_v975::packets::LocatorBarPacket;
    type LoginPacket = bedrock_network_version_v662::packets::LoginPacket;
    type MapCreateLockedCopyPacket =
        bedrock_network_version_v662::packets::MapCreateLockedCopyPacket<Self>;
    type MapInfoRequestPacket = bedrock_network_version_v662::packets::MapInfoRequestPacket<Self>;
    type MobArmorEquipmentPacket =
        bedrock_network_version_v1001::packets::MobArmorEquipmentPacket<Self>;
    type MobEffectPacket = bedrock_network_version_v898::packets::MobEffectPacket<Self>;
    type MobEquipmentPacket = bedrock_network_version_v975::packets::MobEquipmentPacket<Self>;
    type ModalFormRequestPacket = bedrock_network_version_v662::packets::ModalFormRequestPacket;
    type ModalFormResponsePacket =
        bedrock_network_version_v662::packets::ModalFormResponsePacket<Self>;
    type MotionPredictionHintsPacket =
        bedrock_network_version_v662::packets::MotionPredictionHintsPacket<Self>;
    type MoveActorAbsolutePacket =
        bedrock_network_version_v662::packets::MoveActorAbsolutePacket<Self>;
    type MoveActorDeltaPacket = bedrock_network_version_v662::packets::MoveActorDeltaPacket<Self>;
    type MovePlayerPacket = bedrock_network_version_v2168::packets::MovePlayerPacket<Self>;
    type MovementEffectPacket = bedrock_network_version_v748::packets::MovementEffectPacket<Self>;
    type MovementPredictionSyncPacket =
        bedrock_network_version_v975::packets::MovementPredictionSyncPacket<Self>;
    type MultiplayerSettingsPacket =
        bedrock_network_version_v662::packets::MultiplayerSettingsPacket<Self>;
    type NetworkChunkPublisherUpdatePacket =
        bedrock_network_version_v662::packets::NetworkChunkPublisherUpdatePacket<Self>;
    type NetworkSettingsPacket = crate::packets::NetworkSettingsPacket<Self>;
    type NetworkStackLatencyPacket =
        bedrock_network_version_v662::packets::NetworkStackLatencyPacket;
    type NpcDialoguePacket = bedrock_network_version_v662::packets::NpcDialoguePacket;
    type NpcRequestPacket = bedrock_network_version_v662::packets::NpcRequestPacket<Self>;
    type OnScreenTextureAnimationPacket =
        bedrock_network_version_v662::packets::OnScreenTextureAnimationPacket;
    type OpenSignPacket = bedrock_network_version_v662::packets::OpenSignPacket<Self>;
    type PacketViolationWarningPacket =
        bedrock_network_version_v662::packets::PacketViolationWarningPacket<Self>;
    type PartyChangedPacket = bedrock_network_version_v975::packets::PartyChangedPacket;
    type PartyDestinationCookieResponsePacket =
        bedrock_network_version_v1001::packets::PartyDestinationCookieResponsePacket;
    type PassengerJumpPacket = ();
    type PhotoTransferPacket = bedrock_network_version_v662::packets::PhotoTransferPacket<Self>;
    type PlaySoundPacket = crate::packets::PlaySoundPacket<Self>;
    type PlayStatusPacket = bedrock_network_version_v662::packets::PlayStatusPacket<Self>;
    type PlayerActionPacket = crate::packets::PlayerActionPacket<Self>;
    type PlayerArmorDamagePacket = bedrock_network_version_v844::packets::PlayerArmorDamagePacket;
    type PlayerAuthInputPacket =
        bedrock_network_version_v2193::packets::PlayerAuthInputPacket<Self>;
    type PlayerEnchantOptionsPacket =
        bedrock_network_version_v662::packets::PlayerEnchantOptionsPacket<Self>;
    type PlayerFogPacket = bedrock_network_version_v662::packets::PlayerFogPacket;
    type PlayerHotbarPacket = bedrock_network_version_v662::packets::PlayerHotbarPacket<Self>;
    type PlayerInputPacket = ();
    type PlayerListPacket = bedrock_network_version_v2225::packets::PlayerListPacket<Self>;
    type PlayerLocationPacket = bedrock_network_version_v2168::packets::PlayerLocationPacket;
    type PlayerSkinPacket = bedrock_network_version_v2168::packets::PlayerSkinPacket<Self>;
    type PlayerStartItemCooldownPacket =
        bedrock_network_version_v662::packets::PlayerStartItemCooldownPacket;
    type PlayerToggleArmorVisibilityRequestPacket =
        crate::packets::PlayerToggleArmorVisibilityRequestPacket;
    type PlayerToggleCrafterSlotRequestPacket =
        bedrock_network_version_v662::packets::PlayerToggleCrafterSlotRequestPacket;
    type PlayerUpdateEntityOverridesPacket =
        bedrock_network_version_v2168::packets::PlayerUpdateEntityOverridesPacket<Self>;
    type PlayerVideoCapturePacket = bedrock_network_version_v786::packets::PlayerVideoCapturePacket;
    type PositionTrackingDBClientRequestPacket =
        bedrock_network_version_v662::packets::PositionTrackingDBClientRequestPacket<Self>;
    type PositionTrackingDBServerBroadcastPacket =
        bedrock_network_version_v662::packets::PositionTrackingDBServerBroadcastPacket<Self>;
    type PurchaseReceiptPacket = bedrock_network_version_v662::packets::PurchaseReceiptPacket;
    type RecordStartedPacket = bedrock_network_version_v2193::packets::RecordStartedPacket<Self>;
    type RefreshEntitlementsPacket =
        bedrock_network_version_v662::packets::RefreshEntitlementsPacket;
    type RemoveActorPacket = bedrock_network_version_v662::packets::RemoveActorPacket<Self>;
    type RemoveObjectivePacket = bedrock_network_version_v662::packets::RemoveObjectivePacket;
    type RemoveVolumeEntityPacket = ();
    type RequestAbilityPacket = bedrock_network_version_v662::packets::RequestAbilityPacket<Self>;
    type RequestChunkRadiusPacket = bedrock_network_version_v662::packets::RequestChunkRadiusPacket;
    type RequestNetworkSettingsPacket =
        bedrock_network_version_v662::packets::RequestNetworkSettingsPacket;
    type RequestPermissionsPacket =
        bedrock_network_version_v662::packets::RequestPermissionsPacket<Self>;
    type ResourcePackChunkDataPacket =
        bedrock_network_version_v662::packets::ResourcePackChunkDataPacket;
    type ResourcePackChunkRequestPacket =
        bedrock_network_version_v662::packets::ResourcePackChunkRequestPacket;
    type ResourcePackClientResponsePacket =
        bedrock_network_version_v2168::packets::ResourcePackClientResponsePacket;
    type ResourcePackDataInfoPacket =
        bedrock_network_version_v662::packets::ResourcePackDataInfoPacket<Self>;
    type ResourcePackStackPacket = crate::packets::ResourcePackStackPacket<Self>;
    type ResourcePacksInfoPacket = bedrock_network_version_v2168::packets::ResourcePacksInfoPacket;
    type ResourcePacksReadyForValidationPacket =
        bedrock_network_version_v944::packets::ResourcePacksReadyForValidationPacket;
    type RespawnPacket = bedrock_network_version_v662::packets::RespawnPacket<Self>;
    type ScriptMessagePacket = bedrock_network_version_v662::packets::ScriptMessagePacket;
    type SendPartyDestinationCookiePacket =
        bedrock_network_version_v1001::packets::SendPartyDestinationCookiePacket;
    type ServerBoundCursorItemDragPacket =
        bedrock_network_version_v2225::packets::ServerBoundCursorItemDragPacket;
    type ServerBoundDataDrivenClosedPacket =
        bedrock_network_version_v944::packets::ServerBoundDataDrivenClosedPacket;
    type ServerBoundDataStorePacket = crate::packets::ServerBoundDataStorePacket;
    type ServerBoundDiagnosticsPacket =
        bedrock_network_version_v2225::packets::ServerBoundDiagnosticsPacket;
    type ServerBoundLoadingScreenPacket =
        bedrock_network_version_v712::packets::ServerBoundLoadingScreenPacket;
    type ServerBoundMatchmakingCancelPacket =
        bedrock_network_version_v2225::packets::ServerBoundMatchmakingCancelPacket;
    type ServerBoundPackSettingChangePacket =
        bedrock_network_version_v2193::packets::ServerBoundPackSettingChangePacket;
    type ServerBoundRegisterAudioContentPacket =
        bedrock_network_version_v2225::packets::ServerBoundRegisterAudioContentPacket<Self>;
    type ServerBoundRequestOnlineMusicPlaybackPacket =
        crate::packets::ServerBoundRequestOnlineMusicPlaybackPacket;
    type ServerBoundStonecutterSetRecipePacket =
        bedrock_network_version_v2225::packets::ServerBoundStonecutterSetRecipePacket;
    type ServerPlayerPostMovePositionPacket =
        bedrock_network_version_v662::packets::ServerPlayerPostMovePositionPacket;
    type ServerPresenceInfoPacket =
        bedrock_network_version_v2168::packets::ServerPresenceInfoPacket;
    type ServerSettingsRequestPacket =
        bedrock_network_version_v662::packets::ServerSettingsRequestPacket;
    type ServerSettingsResponsePacket =
        bedrock_network_version_v662::packets::ServerSettingsResponsePacket;
    type ServerStatsPacket = bedrock_network_version_v662::packets::ServerStatsPacket;
    type ServerStoreInfoPacket = bedrock_network_version_v975::packets::ServerStoreInfoPacket;
    type ServerToClientHandshakePacket =
        bedrock_network_version_v662::packets::ServerToClientHandshakePacket;
    type SetActorDataPacket = bedrock_network_version_v662::packets::SetActorDataPacket<Self>;
    type SetActorLinkPacket = bedrock_network_version_v662::packets::SetActorLinkPacket<Self>;
    type SetActorMotionPacket = bedrock_network_version_v662::packets::SetActorMotionPacket<Self>;
    type SetCommandsEnabledPacket = bedrock_network_version_v662::packets::SetCommandsEnabledPacket;
    type SetDefaultGameTypePacket =
        bedrock_network_version_v662::packets::SetDefaultGameTypePacket<Self>;
    type SetDifficultyPacket = bedrock_network_version_v662::packets::SetDifficultyPacket;
    type SetDisplayObjectivePacket =
        bedrock_network_version_v662::packets::SetDisplayObjectivePacket<Self>;
    type SetHealthPacket = bedrock_network_version_v662::packets::SetHealthPacket;
    type SetHudPacket = bedrock_network_version_v662::packets::SetHudPacket<Self>;
    type SetLastHurtByPacket = bedrock_network_version_v662::packets::SetLastHurtByPacket<Self>;
    type SetLocalPlayerAsInitializedPacket =
        bedrock_network_version_v662::packets::SetLocalPlayerAsInitializedPacket<Self>;
    type SetMovementAuthorityPacket = ();
    type SetPassengerOfBlockPacket =
        bedrock_network_version_v2225::packets::SetPassengerOfBlockPacket<Self>;
    type SetPlayerFurnaceOptionsPacket =
        bedrock_network_version_v2193::packets::SetPlayerFurnaceOptionsPacket;
    type SetPlayerGameTypePacket =
        bedrock_network_version_v662::packets::SetPlayerGameTypePacket<Self>;
    type SetPlayerInventoryOptionsPacket =
        bedrock_network_version_v662::packets::SetPlayerInventoryOptionsPacket<Self>;
    type SetScorePacket = bedrock_network_version_v2168::packets::SetScorePacket<Self>;
    type SetScoreboardIdentityPacket =
        bedrock_network_version_v2168::packets::SetScoreboardIdentityPacket<Self>;
    type SetSpawnPositionPacket =
        bedrock_network_version_v662::packets::SetSpawnPositionPacket<Self>;
    type SetTimePacket = bedrock_network_version_v662::packets::SetTimePacket;
    type SetTitlePacket = bedrock_network_version_v712::packets::SetTitlePacket;
    type SettingsCommandPacket = bedrock_network_version_v662::packets::SettingsCommandPacket;
    type ShowCreditsPacket = bedrock_network_version_v662::packets::ShowCreditsPacket<Self>;
    type ShowProfilePacket = bedrock_network_version_v662::packets::ShowProfilePacket;
    type ShowStoreOfferPacket = bedrock_network_version_v859::packets::ShowStoreOfferPacket<Self>;
    type SimpleEventPacket = bedrock_network_version_v662::packets::SimpleEventPacket;
    type SimulationTypePacket = bedrock_network_version_v662::packets::SimulationTypePacket<Self>;
    type SpawnExperienceOrbPacket = bedrock_network_version_v662::packets::SpawnExperienceOrbPacket;
    type SpawnParticleEffectPacket =
        bedrock_network_version_v662::packets::SpawnParticleEffectPacket<Self>;
    type StartGamePacket = crate::packets::StartGamePacket<Self>;
    type StopSoundPacket = bedrock_network_version_v712::packets::StopSoundPacket;
    type StructureBlockUpdatePacket = crate::packets::StructureBlockUpdatePacket<Self>;
    type StructureDataRequestPacket =
        bedrock_network_version_v662::packets::StructureDataRequestPacket<Self>;
    type StructureDataResponsePacket =
        bedrock_network_version_v662::packets::StructureDataResponsePacket<Self>;
    type SubChunkPacket = bedrock_network_version_v2193::packets::SubChunkPacket<Self>;
    type SubChunkRequestPacket =
        bedrock_network_version_v1001::packets::SubChunkRequestPacket<Self>;
    type SubClientLoginPacket = bedrock_network_version_v662::packets::SubClientLoginPacket;
    type SyncActorPropertyPacket = bedrock_network_version_v662::packets::SyncActorPropertyPacket;
    type SyncWorldClocksPacket = bedrock_network_version_v944::packets::SyncWorldClocksPacket;
    type TakeItemActorPacket = bedrock_network_version_v662::packets::TakeItemActorPacket<Self>;
    type TextPacket = bedrock_network_version_v898::packets::TextPacket<Self>;
    type TickSyncPacket = ();
    type TickingAreaLoadStatusPacket =
        bedrock_network_version_v662::packets::TickingAreaLoadStatusPacket;
    type ToastRequestPacket = bedrock_network_version_v662::packets::ToastRequestPacket;
    type TransferPlayerPacket = crate::packets::TransferPlayerPacket<Self>;
    type TrimDataPacket = bedrock_network_version_v662::packets::TrimDataPacket;
    type UnlockedRecipesPacket = bedrock_network_version_v662::packets::UnlockedRecipesPacket;
    type UpdateAbilitiesPacket = bedrock_network_version_v662::packets::UpdateAbilitiesPacket<Self>;
    type UpdateAdventureSettingsPacket =
        bedrock_network_version_v662::packets::UpdateAdventureSettingsPacket<Self>;
    type UpdateAttributesPacket =
        bedrock_network_version_v729::packets::UpdateAttributesPacket<Self>;
    type UpdateBlockPacket = bedrock_network_version_v662::packets::UpdateBlockPacket<Self>;
    type UpdateBlockSyncedPacket =
        bedrock_network_version_v662::packets::UpdateBlockSyncedPacket<Self>;
    type UpdateClientInputLocksPacket =
        bedrock_network_version_v944::packets::UpdateClientInputLocksPacket;
    type UpdateClientOptionsPacket =
        bedrock_network_version_v975::packets::UpdateClientOptionsPacket;
    type UpdateEquipPacket = bedrock_network_version_v662::packets::UpdateEquipPacket<Self>;
    type UpdatePlayerGameTypePacket =
        bedrock_network_version_v671::packets::UpdatePlayerGameTypePacket<Self>;
    type UpdateSoftEnumPacket = bedrock_network_version_v662::packets::UpdateSoftEnumPacket<Self>;
    type UpdateSubChunkBlocksPacket =
        bedrock_network_version_v662::packets::UpdateSubChunkBlocksPacket<Self>;
    type UpdateTradePacket = bedrock_network_version_v662::packets::UpdateTradePacket<Self>;
    type VirtualContainerClosePacket = crate::packets::VirtualContainerClosePacket;
    type VirtualContainerOpenPacket = crate::packets::VirtualContainerOpenPacket<Self>;
    type VoxelShapesPacket = bedrock_network_version_v944::packets::VoxelShapesPacket;
}
impl ProtoVersionTypes for V2259 {
    type ActorLink = bedrock_network_version_v712::types::ActorLink<Self>;
    type ActorRuntimeID = bedrock_network_version_v662::types::ActorRuntimeID;
    type ActorUniqueID = bedrock_network_version_v662::types::ActorUniqueID;
    type AdventureSettings = bedrock_network_version_v662::types::AdventureSettings;
    type BaseDescription = bedrock_network_version_v662::types::BaseDescription<Self>;
    type BaseGameVersion = bedrock_network_version_v662::types::BaseGameVersion;
    type BiomeCappedSurfaceData = bedrock_network_version_v800::types::BiomeCappedSurfaceData;
    type BiomeClimateData = bedrock_network_version_v844::types::BiomeClimateData;
    type BiomeConditionalTransformationData =
        bedrock_network_version_v800::types::BiomeConditionalTransformationData<Self>;
    type BiomeConsolidatedFeatureList =
        bedrock_network_version_v800::types::BiomeConsolidatedFeatureList<Self>;
    type BiomeCoordinateData = bedrock_network_version_v800::types::BiomeCoordinateData;
    type BiomeDefinition = bedrock_network_version_v844::types::BiomeDefinition<Self>;
    type BiomeDefinitionChunkGenData =
        bedrock_network_version_v975::types::BiomeDefinitionChunkGenData<Self>;
    type BiomeElementData = bedrock_network_version_v800::types::BiomeElementData<Self>;
    type BiomeLegacyWorldGenRulesData =
        bedrock_network_version_v800::types::BiomeLegacyWorldGenRulesData<Self>;
    type BiomeMesaSurfaceData = bedrock_network_version_v800::types::BiomeMesaSurfaceData;
    type BiomeMountainParamsData = bedrock_network_version_v800::types::BiomeMountainParamsData;
    type BiomeMultinoiseGenRulesData =
        bedrock_network_version_v800::types::BiomeMultinoiseGenRulesData;
    type BiomeNoiseGradientSurfaceData =
        bedrock_network_version_v1001::types::BiomeNoiseGradientSurfaceData;
    type BiomeOverworldGenRulesData =
        bedrock_network_version_v800::types::BiomeOverworldGenRulesData<Self>;
    type BiomeReplacementData = bedrock_network_version_v859::types::BiomeReplacementData;
    type BiomeScatterParamData = bedrock_network_version_v800::types::BiomeScatterParamData<Self>;
    type BiomeSurfaceBuilderData =
        bedrock_network_version_v975::types::BiomeSurfaceBuilderData<Self>;
    type BiomeSurfaceMaterialAdjustmentData =
        bedrock_network_version_v800::types::BiomeSurfaceMaterialAdjustmentData<Self>;
    type BiomeSurfaceMaterialData = bedrock_network_version_v800::types::BiomeSurfaceMaterialData;
    type BiomeWeightedData = bedrock_network_version_v800::types::BiomeWeightedData;
    type BiomeWeightedTemperatureData =
        bedrock_network_version_v800::types::BiomeWeightedTemperatureData;
    type BlockPos = bedrock_network_version_v662::types::BlockPos;
    type CameraAimAssistCategories =
        bedrock_network_version_v766::types::CameraAimAssistCategories<Self>;
    type CameraAimAssistCategory =
        bedrock_network_version_v924::types::CameraAimAssistCategory<Self>;
    type CameraAimAssistItemSettings =
        bedrock_network_version_v766::types::CameraAimAssistItemSettings;
    type CameraAimAssistPreset = bedrock_network_version_v766::types::CameraAimAssistPreset;
    type CameraAimAssistPresetDefinition =
        bedrock_network_version_v924::types::CameraAimAssistPresetDefinition<Self>;
    type CameraAimAssistPriority = bedrock_network_version_v766::types::CameraAimAssistPriority;
    type CameraPreset = bedrock_network_version_v2193::types::CameraPreset<Self>;
    type CameraPresets = bedrock_network_version_v662::types::CameraPresets<Self>;
    type CameraSplineInstruction =
        bedrock_network_version_v944::types::CameraSplineInstruction<Self>;
    type ChunkPos = bedrock_network_version_v662::types::ChunkPos;
    type Color = bedrock_network_version_v800::types::Color;
    type CommandOriginData = bedrock_network_version_v898::types::CommandOriginData<Self>;
    type ContainerMixDataEntry = bedrock_network_version_v662::types::ContainerMixDataEntry;
    type CraftingDataEntry = bedrock_network_version_v662::types::CraftingDataEntry<Self>;
    type CraftingRecipeIngredient = bedrock_network_version_v2168::types::CraftingRecipeIngredient;
    type DataItem = bedrock_network_version_v662::types::DataItem<Self>;
    type DebugShape = bedrock_network_version_v2225::types::DebugShape<Self>;
    type DimensionDefinitionGroup = bedrock_network_version_v2225::types::DimensionDefinitionGroup;
    type EduSharedUriResource = bedrock_network_version_v662::types::EduSharedUriResource;
    type EducationLevelSettings = bedrock_network_version_v662::types::EducationLevelSettings;
    type EntityNetID = bedrock_network_version_v662::types::EntityNetID;
    type Experiments = bedrock_network_version_v662::types::Experiments;
    type FullContainerName = bedrock_network_version_v729::types::FullContainerName<Self>;
    type GameRulesChangedPacketData =
        bedrock_network_version_v844::types::GameRulesChangedPacketData;
    type GatheringsConfig = bedrock_network_version_v2168::types::GatheringsConfig;
    type InventoryAction = bedrock_network_version_v1001::types::InventoryAction<Self>;
    type InventorySource = bedrock_network_version_v2193::types::InventorySource;
    type InventoryTransaction = bedrock_network_version_v662::types::InventoryTransaction<Self>;
    type ItemData = bedrock_network_version_v662::types::ItemData;
    type ItemEnchants = bedrock_network_version_v662::types::ItemEnchants<Self>;
    type ItemReleaseTransactionData =
        bedrock_network_version_v2225::types::ItemReleaseTransactionData<Self>;
    type ItemStackRequestNetworkItemInstanceDescriptor =
        bedrock_network_version_v2168::types::ItemStackRequestNetworkItemInstanceDescriptor<Self>;
    type ItemStackRequestSlotInfo =
        bedrock_network_version_v2168::types::ItemStackRequestSlotInfo<Self>;
    type ItemStackResponseContainerInfo =
        bedrock_network_version_v712::types::ItemStackResponseContainerInfo<Self>;
    type ItemStackResponseInfo = bedrock_network_version_v2193::types::ItemStackResponseInfo<Self>;
    type ItemStackResponseSlotInfo =
        bedrock_network_version_v2193::types::ItemStackResponseSlotInfo<Self>;
    type ItemUseOnEntityTransactionData =
        bedrock_network_version_v2225::types::ItemUseOnEntityTransactionData<Self>;
    type ItemUseTransactionData =
        bedrock_network_version_v2193::types::ItemUseTransactionData<Self>;
    type LevelSettings = bedrock_network_version_v2225::types::LevelSettings<Self>;
    type MapDecoration = bedrock_network_version_v2168::types::MapDecoration;
    type MapItemTrackedActorUniqueID =
        bedrock_network_version_v2168::types::MapItemTrackedActorUniqueID<Self>;
    type MaterialReducerDataEntry = bedrock_network_version_v662::types::MaterialReducerDataEntry;
    type MolangVariableMap = bedrock_network_version_v662::types::MolangVariableMap;
    type MoveActorAbsoluteData = bedrock_network_version_v662::types::MoveActorAbsoluteData<Self>;
    type MoveActorDeltaData = bedrock_network_version_v2193::types::MoveActorDeltaData<Self>;
    type MovePlayerTeleportData = bedrock_network_version_v2168::types::MovePlayerTeleportData;
    type MultiRecipe = bedrock_network_version_v2168::types::MultiRecipe;
    type NetworkBlockPosition = bedrock_network_version_v944::types::NetworkBlockPosition;
    type NetworkItemInstanceDescriptor =
        bedrock_network_version_v2168::types::NetworkItemInstanceDescriptor;
    type NetworkItemStackDescriptor =
        bedrock_network_version_v2168::types::NetworkItemStackDescriptor;
    type NetworkItemStackDescriptorV2 =
        bedrock_network_version_v2168::types::NetworkItemStackDescriptorV2;
    type NetworkPermissions = bedrock_network_version_v662::types::NetworkPermissions;
    type PackedItemUseLegacyInventoryTransaction =
        bedrock_network_version_v2193::types::PackedItemUseLegacyInventoryTransaction<Self>;
    type PassengerOfBlockArguments = crate::types::PassengerOfBlockArguments<Self>;
    type PlayerBlockActionData = bedrock_network_version_v2168::types::PlayerBlockActionData<Self>;
    type PositionTrackingId = bedrock_network_version_v662::types::PositionTrackingId;
    type PotionMixDataEntry = bedrock_network_version_v662::types::PotionMixDataEntry;
    type PropertySyncData = bedrock_network_version_v662::types::PropertySyncData;
    type RecipeIngredient = bedrock_network_version_v2168::types::RecipeIngredient<Self>;
    type RecipeUnlockingRequirement =
        bedrock_network_version_v2168::types::RecipeUnlockingRequirement<Self>;
    type RedactableString = bedrock_network_version_v2168::types::RedactableString;
    type ScoreboardId = bedrock_network_version_v662::types::ScoreboardId;
    type SerializedAbilitiesData =
        bedrock_network_version_v776::types::SerializedAbilitiesData<Self>;
    type SerializedSkin = bedrock_network_version_v2225::types::SerializedSkin<Self>;
    type ShapedRecipe = bedrock_network_version_v2168::types::ShapedRecipe<Self>;
    type ShapelessRecipe = bedrock_network_version_v2168::types::ShapelessRecipe<Self>;
    type ShulkerBoxRecipe = bedrock_network_version_v748::types::ShulkerBoxRecipe<Self>;
    type SignedAudioContent = bedrock_network_version_v2225::types::SignedAudioContent;
    type SmithingTransformRecipe =
        bedrock_network_version_v2168::types::SmithingTransformRecipe<Self>;
    type SmithingTrimRecipe = bedrock_network_version_v2168::types::SmithingTrimRecipe<Self>;
    type SoundData = bedrock_network_version_v2225::types::SoundData;
    type SpawnSettings = bedrock_network_version_v662::types::SpawnSettings<Self>;
    type StructureEditorData = bedrock_network_version_v2168::types::StructureEditorData<Self>;
    type StructureSettings = bedrock_network_version_v662::types::StructureSettings<Self>;
    type SubChunkPos = bedrock_network_version_v2168::types::SubChunkPos;
    type SubChunkPosOffset = bedrock_network_version_v662::types::SubChunkPosOffset;
    type SyncedPlayerMovementSettings =
        bedrock_network_version_v818::types::SyncedPlayerMovementSettings;
    type WebSocketPacketData = bedrock_network_version_v662::types::WebSocketPacketData;
}
impl ProtoVersionEnums for V2259 {
    type AbilitiesIndex = bedrock_network_version_v776::enums::AbilitiesIndex;
    type ActorBlockSyncMessageID = bedrock_network_version_v662::enums::ActorBlockSyncMessageID;
    type ActorDamageCause = bedrock_network_version_v662::enums::ActorDamageCause;
    type ActorDataIDs = bedrock_network_version_v924::enums::ActorDataIDs;
    type ActorEvent = bedrock_network_version_v975::enums::ActorEvent;
    type ActorFlags = bedrock_network_version_v2168::enums::ActorFlags;
    type ActorLinkType = bedrock_network_version_v662::enums::ActorLinkType;
    type ActorType = bedrock_network_version_v662::enums::ActorType;
    type AgentActionType = bedrock_network_version_v662::enums::AgentActionType;
    type AimAssistAction = bedrock_network_version_v729::enums::AimAssistAction;
    type AnimatedTextureType = bedrock_network_version_v2168::enums::AnimatedTextureType;
    type AnimationExpression = bedrock_network_version_v2168::enums::AnimationExpression;
    type AnimationMode = bedrock_network_version_v662::enums::AnimationMode;
    type ArmSizeType = bedrock_network_version_v2168::enums::ArmSizeType;
    type AttributeModifierOperation =
        bedrock_network_version_v662::enums::AttributeModifierOperation;
    type AttributeOperands = bedrock_network_version_v662::enums::AttributeOperands;
    type AuthoritativeMovementMode = bedrock_network_version_v748::enums::AuthoritativeMovementMode;
    type BookEditAction = bedrock_network_version_v924::enums::BookEditAction;
    type BossEventUpdateType = bedrock_network_version_v776::enums::BossEventUpdateType<Self>;
    type BuildPlatform = bedrock_network_version_v662::enums::BuildPlatform;
    type CameraAimAssistOperation = bedrock_network_version_v776::enums::CameraAimAssistOperation;
    type CameraShakeAction = bedrock_network_version_v662::enums::CameraShakeAction;
    type CameraShakeType = bedrock_network_version_v662::enums::CameraShakeType;
    type CameraSplineEaseType = bedrock_network_version_v924::enums::CameraSplineEaseType;
    type CameraSplineType = bedrock_network_version_v859::enums::CameraSplineType;
    type ChatRestrictionLevel = bedrock_network_version_v662::enums::ChatRestrictionLevel;
    type CodeBuilderCodeStatus = bedrock_network_version_v685::enums::CodeBuilderCodeStatus;
    type CodeBuilderStorageCategory =
        bedrock_network_version_v662::enums::CodeBuilderStorageCategory;
    type CodeBuilderStorageOperation =
        bedrock_network_version_v662::enums::CodeBuilderStorageOperation;
    type CommandBlockMode = bedrock_network_version_v662::enums::CommandBlockMode;
    type CommandOriginType = bedrock_network_version_v898::enums::CommandOriginType;
    type CommandOutputType = bedrock_network_version_v898::enums::CommandOutputType;
    type CommandParameterOption = bedrock_network_version_v662::enums::CommandParameterOption;
    type CommandPermissionLevel = bedrock_network_version_v662::enums::CommandPermissionLevel;
    type ComplexInventoryTransactionType =
        bedrock_network_version_v662::enums::ComplexInventoryTransactionType;
    type ConnectionFailReason = crate::enums::ConnectionFailReason;
    type ContainerEnumName = crate::enums::ContainerEnumName;
    type ContainerID = bedrock_network_version_v662::enums::ContainerID;
    type ContainerType = crate::enums::ContainerType;
    type ControlScheme = bedrock_network_version_v800::enums::ControlScheme;
    type CraftingDataEntryType = bedrock_network_version_v662::enums::CraftingDataEntryType<Self>;
    type DataItemType = bedrock_network_version_v2168::enums::DataItemType<Self>;
    type Difficulty = bedrock_network_version_v662::enums::Difficulty;
    type EasingType = bedrock_network_version_v662::enums::EasingType;
    type EditorWorldType = bedrock_network_version_v662::enums::EditorWorldType;
    type EducationEditionOffer = bedrock_network_version_v2168::enums::EducationEditionOffer;
    type EnchantType = bedrock_network_version_v662::enums::EnchantType;
    type GamePublishSetting = bedrock_network_version_v662::enums::GamePublishSetting;
    type GameType = bedrock_network_version_v662::enums::GameType;
    type GeneratorType = bedrock_network_version_v662::enums::GeneratorType;
    type HudElement = bedrock_network_version_v786::enums::HudElement;
    type HudVisibility = bedrock_network_version_v786::enums::HudVisibility;
    type IdentityDefinitionType = bedrock_network_version_v662::enums::IdentityDefinitionType<Self>;
    type InputMode = crate::enums::InputMode;
    type InteractionType = bedrock_network_version_v662::enums::InteractionType;
    type InventoryLayout = bedrock_network_version_v662::enums::InventoryLayout;
    type InventoryLeftTabIndex = bedrock_network_version_v662::enums::InventoryLeftTabIndex;
    type InventoryRightTabIndex = bedrock_network_version_v662::enums::InventoryRightTabIndex;
    type InventorySourceFlags = bedrock_network_version_v662::enums::InventorySourceFlags;
    type InventorySourceType = bedrock_network_version_v662::enums::InventorySourceType;
    type ItemDescriptorType = bedrock_network_version_v2168::enums::ItemDescriptorType;
    type ItemReleaseInventoryTransactionType =
        bedrock_network_version_v662::enums::ItemReleaseInventoryTransactionType;
    type ItemStackNetResult = bedrock_network_version_v2168::enums::ItemStackNetResult;
    type ItemStackRequestActionType = crate::enums::ItemStackRequestActionType<Self>;
    type ItemUseInventoryTransactionType =
        bedrock_network_version_v662::enums::ItemUseInventoryTransactionType;
    type ItemUseMethod = bedrock_network_version_v662::enums::ItemUseMethod;
    type ItemUseOnActorInventoryTransactionType =
        bedrock_network_version_v662::enums::ItemUseOnActorInventoryTransactionType;
    type ItemVersion = bedrock_network_version_v776::enums::ItemVersion;
    type LabTableReactionType = bedrock_network_version_v662::enums::LabTableReactionType;
    type LessonAction = bedrock_network_version_v662::enums::LessonAction;
    type LevelEvent = bedrock_network_version_v766::enums::LevelEvent;
    type LevelSoundEventType = crate::enums::LevelSoundEventType;
    type MinecraftPacketIds = bedrock_network_version_v662::enums::MinecraftPacketIds;
    type Mirror = bedrock_network_version_v662::enums::Mirror;
    type ModalFormCancelReason = bedrock_network_version_v662::enums::ModalFormCancelReason;
    type MolangVersion = bedrock_network_version_v662::enums::MolangVersion;
    type MovementEffectType = bedrock_network_version_v1001::enums::MovementEffectType;
    type MultiplayerSettingsPacketType =
        bedrock_network_version_v662::enums::MultiplayerSettingsPacketType;
    type NewInteractionModel = bedrock_network_version_v2168::enums::NewInteractionModel;
    type ObjectiveSortOrder = bedrock_network_version_v662::enums::ObjectiveSortOrder;
    type POIBlockInteractionType = bedrock_network_version_v662::enums::POIBlockInteractionType;
    type PackType = bedrock_network_version_v662::enums::PackType;
    type PacketCompressionAlgorithm =
        bedrock_network_version_v662::enums::PacketCompressionAlgorithm;
    type PacketViolationSeverity = bedrock_network_version_v662::enums::PacketViolationSeverity;
    type PacketViolationType = bedrock_network_version_v662::enums::PacketViolationType;
    type ParticleType = bedrock_network_version_v2168::enums::ParticleType;
    type PersonaPieceType = bedrock_network_version_v2225::enums::PersonaPieceType;
    type PhotoType = bedrock_network_version_v662::enums::PhotoType;
    type PlayStatus = bedrock_network_version_v662::enums::PlayStatus;
    type PlayerAuthInputData = crate::enums::PlayerAuthInputData;
    type PlayerPermissionLevel = bedrock_network_version_v662::enums::PlayerPermissionLevel;
    type PlayerPositionMode = bedrock_network_version_v2168::enums::PlayerPositionMode;
    type PlayerRespawnState = bedrock_network_version_v662::enums::PlayerRespawnState;
    type PredictionType = bedrock_network_version_v827::enums::PredictionType;
    type ResourcePackResponse = bedrock_network_version_v662::enums::ResourcePackResponse;
    type Rotation = bedrock_network_version_v662::enums::Rotation;
    type ServerAuthMovementMode = ();
    type ShowStoreOfferRedirectType =
        bedrock_network_version_v662::enums::ShowStoreOfferRedirectType;
    type SimulationType = bedrock_network_version_v662::enums::SimulationType;
    type SoftEnumUpdateType = bedrock_network_version_v662::enums::SoftEnumUpdateType;
    type SpawnBiomeType = bedrock_network_version_v662::enums::SpawnBiomeType;
    type SpawnPositionType = bedrock_network_version_v662::enums::SpawnPositionType;
    type StructureBlockType = bedrock_network_version_v662::enums::StructureBlockType;
    type StructureRedstoneSaveMode =
        bedrock_network_version_v2168::enums::StructureRedstoneSaveMode;
    type StructureTemplateRequestOperation =
        bedrock_network_version_v662::enums::StructureTemplateRequestOperation;
    type StructureTemplateResponseType =
        bedrock_network_version_v662::enums::StructureTemplateResponseType;
    type TeleportationCause = bedrock_network_version_v662::enums::TeleportationCause;
    type TextPacketType = crate::enums::TextPacketType;
    type TextProcessingEventOrigin = bedrock_network_version_v662::enums::TextProcessingEventOrigin;
    type UIProfile = bedrock_network_version_v662::enums::UIProfile;
}
impl ProtoVersion for V2259 {
    const PROTOCOL_VERSION: u32 = 2259u32;
    const PROTOCOL_BRANCH: &str = "r/26_u7";
    const GAME_VERSION: &str = "1.26.70";
    const RAKNET_VERSION: u8 = 11u8;
}
#[cfg(feature = "packet-dyn")]
impl AsRef<dyn bedrock_network_core::PacketDyn> for V2259 {
    fn as_ref(&self) -> &dyn bedrock_network_core::PacketDyn {
        match self {
            V2259::ActorEventPacket(pk) => pk.as_ref(),
            V2259::ActorPickRequestPacket(pk) => pk.as_ref(),
            V2259::AddActorPacket(pk) => pk.as_ref(),
            V2259::AddBehaviourTreePacket(pk) => pk.as_ref(),
            V2259::AddItemActorPacket(pk) => pk.as_ref(),
            V2259::AddPaintingPacket(pk) => pk.as_ref(),
            V2259::AddPlayerPacket(pk) => pk.as_ref(),
            V2259::AgentActionEventPacket(pk) => pk.as_ref(),
            V2259::AgentAnimationPacket(pk) => pk.as_ref(),
            V2259::AnimateEntityPacket(pk) => pk.as_ref(),
            V2259::AnimatePacket(pk) => pk.as_ref(),
            V2259::AnvilDamagePacket(pk) => pk.as_ref(),
            V2259::AutomationClientConnectPacket(pk) => pk.as_ref(),
            V2259::AvailableActorIdentifiersPacket(pk) => pk.as_ref(),
            V2259::AvailableCommandsPacket(pk) => pk.as_ref(),
            V2259::AwardAchievementPacket(pk) => pk.as_ref(),
            V2259::BiomeDefinitionListPacket(pk) => pk.as_ref(),
            V2259::BlockActorDataPacket(pk) => pk.as_ref(),
            V2259::BlockEventPacket(pk) => pk.as_ref(),
            V2259::BlockPickRequestPacket(pk) => pk.as_ref(),
            V2259::BookEditPacket(pk) => pk.as_ref(),
            V2259::BossEventPacket(pk) => pk.as_ref(),
            V2259::CameraAimAssistActorPriorityPacket(pk) => pk.as_ref(),
            V2259::CameraAimAssistInstructionPacket(pk) => pk.as_ref(),
            V2259::CameraAimAssistPacket(pk) => pk.as_ref(),
            V2259::CameraAimAssistPresetsPacket(pk) => pk.as_ref(),
            V2259::CameraInstructionPacket(pk) => pk.as_ref(),
            V2259::CameraPacket(pk) => pk.as_ref(),
            V2259::CameraPresetsPacket(pk) => pk.as_ref(),
            V2259::CameraShakePacket(pk) => pk.as_ref(),
            V2259::CameraSplinePacket(pk) => pk.as_ref(),
            V2259::ChangeDimensionPacket(pk) => pk.as_ref(),
            V2259::ChangeMobPropertyPacket(pk) => pk.as_ref(),
            V2259::ChatPacket(pk) => pk.as_ref(),
            V2259::ChunkRadiusUpdatedPacket(pk) => pk.as_ref(),
            V2259::ClientBoundAttributeLayerSyncPacket(pk) => pk.as_ref(),
            V2259::ClientBoundCloseFormPacket(pk) => pk.as_ref(),
            V2259::ClientBoundControlSchemeSetPacket(pk) => pk.as_ref(),
            V2259::ClientBoundDataDrivenUICloseScreenPacket(pk) => pk.as_ref(),
            V2259::ClientBoundDataDrivenUIReloadPacket(pk) => pk.as_ref(),
            V2259::ClientBoundDataDrivenUIShowScreenPacket(pk) => pk.as_ref(),
            V2259::ClientBoundDataStorePacket(pk) => pk.as_ref(),
            V2259::ClientBoundDebugRendererPacket(pk) => pk.as_ref(),
            V2259::ClientBoundMapItemDataPacket(pk) => pk.as_ref(),
            V2259::ClientBoundMatchmakingStatePacket(pk) => pk.as_ref(),
            V2259::ClientBoundMusicPlaybackFailurePacket(pk) => pk.as_ref(),
            V2259::ClientBoundOnlineMusicPlaybackResponsePacket(pk) => pk.as_ref(),
            V2259::ClientBoundPlayAudioContentPacket(pk) => pk.as_ref(),
            V2259::ClientBoundStonecutterSetRecipePacket(pk) => pk.as_ref(),
            V2259::ClientBoundTextureShiftPacket(pk) => pk.as_ref(),
            V2259::ClientBoundUpdateSoundDataPacket(pk) => pk.as_ref(),
            V2259::ClientCacheBlobStatusPacket(pk) => pk.as_ref(),
            V2259::ClientCacheMissResponsePacket(pk) => pk.as_ref(),
            V2259::ClientCacheStatusPacket(pk) => pk.as_ref(),
            V2259::ClientToServerHandshakePacket(pk) => pk.as_ref(),
            V2259::CodeBuilderPacket(pk) => pk.as_ref(),
            V2259::CodeBuilderSourcePacket(pk) => pk.as_ref(),
            V2259::CommandBlockUpdatePacket(pk) => pk.as_ref(),
            V2259::CommandOutputPacket(pk) => pk.as_ref(),
            V2259::CommandRequestPacket(pk) => pk.as_ref(),
            V2259::CompletedUsingItemPacket(pk) => pk.as_ref(),
            V2259::ContainerClosePacket(pk) => pk.as_ref(),
            V2259::ContainerOpenPacket(pk) => pk.as_ref(),
            V2259::ContainerRegistryCleanupPacket(pk) => pk.as_ref(),
            V2259::ContainerSetDataPacket(pk) => pk.as_ref(),
            V2259::CorrectPlayerMovePredictionPacket(pk) => pk.as_ref(),
            V2259::CraftingDataPacket(pk) => pk.as_ref(),
            V2259::CreatePhotoPacket(pk) => pk.as_ref(),
            V2259::CreativeContentPacket(pk) => pk.as_ref(),
            V2259::CurrentStructureFeaturePacket(pk) => pk.as_ref(),
            V2259::DeathInfoPacket(pk) => pk.as_ref(),
            V2259::DebugDrawerPacket(pk) => pk.as_ref(),
            V2259::DebugInfoPacket(pk) => pk.as_ref(),
            V2259::DimensionDataPacket(pk) => pk.as_ref(),
            V2259::DisconnectPacket(pk) => pk.as_ref(),
            V2259::EditorNetworkPacket(pk) => pk.as_ref(),
            V2259::EduUriResourcePacket(pk) => pk.as_ref(),
            V2259::EducationSettingsPacket(pk) => pk.as_ref(),
            V2259::EmoteListPacket(pk) => pk.as_ref(),
            V2259::EmotePacket(pk) => pk.as_ref(),
            V2259::FeatureRegistryPacket(pk) => pk.as_ref(),
            V2259::GameRulesChangedPacket(pk) => pk.as_ref(),
            V2259::GameTestRequestPacket(pk) => pk.as_ref(),
            V2259::GameTestResultsPacket(pk) => pk.as_ref(),
            V2259::GraphicsParameterOverridePacket(pk) => pk.as_ref(),
            V2259::GuiDataPickItemPacket(pk) => pk.as_ref(),
            V2259::HurtArmorPacket(pk) => pk.as_ref(),
            V2259::InteractPacket(pk) => pk.as_ref(),
            V2259::InventoryContentPacket(pk) => pk.as_ref(),
            V2259::InventorySlotPacket(pk) => pk.as_ref(),
            V2259::InventoryTransactionPacket(pk) => pk.as_ref(),
            V2259::ItemComponentPacket(pk) => pk.as_ref(),
            V2259::ItemStackRequestPacket(pk) => pk.as_ref(),
            V2259::ItemStackResponsePacket(pk) => pk.as_ref(),
            V2259::JigsawStructureDataPacket(pk) => pk.as_ref(),
            V2259::LabTablePacket(pk) => pk.as_ref(),
            V2259::LecternUpdatePacket(pk) => pk.as_ref(),
            V2259::LegacyTelemetryEventPacket(pk) => pk.as_ref(),
            V2259::LessonProgressPacket(pk) => pk.as_ref(),
            V2259::LevelChunkPacket(pk) => pk.as_ref(),
            V2259::LevelEventGenericPacket(pk) => pk.as_ref(),
            V2259::LevelEventPacket(pk) => pk.as_ref(),
            V2259::LevelSoundEventPacket(pk) => pk.as_ref(),
            V2259::LocatorBarPacket(pk) => pk.as_ref(),
            V2259::LoginPacket(pk) => pk.as_ref(),
            V2259::MapCreateLockedCopyPacket(pk) => pk.as_ref(),
            V2259::MapInfoRequestPacket(pk) => pk.as_ref(),
            V2259::MobArmorEquipmentPacket(pk) => pk.as_ref(),
            V2259::MobEffectPacket(pk) => pk.as_ref(),
            V2259::MobEquipmentPacket(pk) => pk.as_ref(),
            V2259::ModalFormRequestPacket(pk) => pk.as_ref(),
            V2259::ModalFormResponsePacket(pk) => pk.as_ref(),
            V2259::MotionPredictionHintsPacket(pk) => pk.as_ref(),
            V2259::MoveActorAbsolutePacket(pk) => pk.as_ref(),
            V2259::MoveActorDeltaPacket(pk) => pk.as_ref(),
            V2259::MovePlayerPacket(pk) => pk.as_ref(),
            V2259::MovementEffectPacket(pk) => pk.as_ref(),
            V2259::MovementPredictionSyncPacket(pk) => pk.as_ref(),
            V2259::MultiplayerSettingsPacket(pk) => pk.as_ref(),
            V2259::NetworkChunkPublisherUpdatePacket(pk) => pk.as_ref(),
            V2259::NetworkSettingsPacket(pk) => pk.as_ref(),
            V2259::NetworkStackLatencyPacket(pk) => pk.as_ref(),
            V2259::NpcDialoguePacket(pk) => pk.as_ref(),
            V2259::NpcRequestPacket(pk) => pk.as_ref(),
            V2259::OnScreenTextureAnimationPacket(pk) => pk.as_ref(),
            V2259::OpenSignPacket(pk) => pk.as_ref(),
            V2259::PacketViolationWarningPacket(pk) => pk.as_ref(),
            V2259::PartyChangedPacket(pk) => pk.as_ref(),
            V2259::PartyDestinationCookieResponsePacket(pk) => pk.as_ref(),
            V2259::PhotoTransferPacket(pk) => pk.as_ref(),
            V2259::PlaySoundPacket(pk) => pk.as_ref(),
            V2259::PlayStatusPacket(pk) => pk.as_ref(),
            V2259::PlayerActionPacket(pk) => pk.as_ref(),
            V2259::PlayerArmorDamagePacket(pk) => pk.as_ref(),
            V2259::PlayerAuthInputPacket(pk) => pk.as_ref(),
            V2259::PlayerEnchantOptionsPacket(pk) => pk.as_ref(),
            V2259::PlayerFogPacket(pk) => pk.as_ref(),
            V2259::PlayerHotbarPacket(pk) => pk.as_ref(),
            V2259::PlayerListPacket(pk) => pk.as_ref(),
            V2259::PlayerLocationPacket(pk) => pk.as_ref(),
            V2259::PlayerSkinPacket(pk) => pk.as_ref(),
            V2259::PlayerStartItemCooldownPacket(pk) => pk.as_ref(),
            V2259::PlayerToggleArmorVisibilityRequestPacket(pk) => pk.as_ref(),
            V2259::PlayerToggleCrafterSlotRequestPacket(pk) => pk.as_ref(),
            V2259::PlayerUpdateEntityOverridesPacket(pk) => pk.as_ref(),
            V2259::PlayerVideoCapturePacket(pk) => pk.as_ref(),
            V2259::PositionTrackingDBClientRequestPacket(pk) => pk.as_ref(),
            V2259::PositionTrackingDBServerBroadcastPacket(pk) => pk.as_ref(),
            V2259::PurchaseReceiptPacket(pk) => pk.as_ref(),
            V2259::RecordStartedPacket(pk) => pk.as_ref(),
            V2259::RefreshEntitlementsPacket(pk) => pk.as_ref(),
            V2259::RemoveActorPacket(pk) => pk.as_ref(),
            V2259::RemoveObjectivePacket(pk) => pk.as_ref(),
            V2259::RequestAbilityPacket(pk) => pk.as_ref(),
            V2259::RequestChunkRadiusPacket(pk) => pk.as_ref(),
            V2259::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
            V2259::RequestPermissionsPacket(pk) => pk.as_ref(),
            V2259::ResourcePackChunkDataPacket(pk) => pk.as_ref(),
            V2259::ResourcePackChunkRequestPacket(pk) => pk.as_ref(),
            V2259::ResourcePackClientResponsePacket(pk) => pk.as_ref(),
            V2259::ResourcePackDataInfoPacket(pk) => pk.as_ref(),
            V2259::ResourcePackStackPacket(pk) => pk.as_ref(),
            V2259::ResourcePacksInfoPacket(pk) => pk.as_ref(),
            V2259::ResourcePacksReadyForValidationPacket(pk) => pk.as_ref(),
            V2259::RespawnPacket(pk) => pk.as_ref(),
            V2259::ScriptMessagePacket(pk) => pk.as_ref(),
            V2259::SendPartyDestinationCookiePacket(pk) => pk.as_ref(),
            V2259::ServerBoundCursorItemDragPacket(pk) => pk.as_ref(),
            V2259::ServerBoundDataDrivenClosedPacket(pk) => pk.as_ref(),
            V2259::ServerBoundDataStorePacket(pk) => pk.as_ref(),
            V2259::ServerBoundDiagnosticsPacket(pk) => pk.as_ref(),
            V2259::ServerBoundLoadingScreenPacket(pk) => pk.as_ref(),
            V2259::ServerBoundMatchmakingCancelPacket(pk) => pk.as_ref(),
            V2259::ServerBoundPackSettingChangePacket(pk) => pk.as_ref(),
            V2259::ServerBoundRegisterAudioContentPacket(pk) => pk.as_ref(),
            V2259::ServerBoundRequestOnlineMusicPlaybackPacket(pk) => pk.as_ref(),
            V2259::ServerBoundStonecutterSetRecipePacket(pk) => pk.as_ref(),
            V2259::ServerPlayerPostMovePositionPacket(pk) => pk.as_ref(),
            V2259::ServerPresenceInfoPacket(pk) => pk.as_ref(),
            V2259::ServerSettingsRequestPacket(pk) => pk.as_ref(),
            V2259::ServerSettingsResponsePacket(pk) => pk.as_ref(),
            V2259::ServerStatsPacket(pk) => pk.as_ref(),
            V2259::ServerStoreInfoPacket(pk) => pk.as_ref(),
            V2259::ServerToClientHandshakePacket(pk) => pk.as_ref(),
            V2259::SetActorDataPacket(pk) => pk.as_ref(),
            V2259::SetActorLinkPacket(pk) => pk.as_ref(),
            V2259::SetActorMotionPacket(pk) => pk.as_ref(),
            V2259::SetCommandsEnabledPacket(pk) => pk.as_ref(),
            V2259::SetDefaultGameTypePacket(pk) => pk.as_ref(),
            V2259::SetDifficultyPacket(pk) => pk.as_ref(),
            V2259::SetDisplayObjectivePacket(pk) => pk.as_ref(),
            V2259::SetHealthPacket(pk) => pk.as_ref(),
            V2259::SetHudPacket(pk) => pk.as_ref(),
            V2259::SetLastHurtByPacket(pk) => pk.as_ref(),
            V2259::SetLocalPlayerAsInitializedPacket(pk) => pk.as_ref(),
            V2259::SetPassengerOfBlockPacket(pk) => pk.as_ref(),
            V2259::SetPlayerFurnaceOptionsPacket(pk) => pk.as_ref(),
            V2259::SetPlayerGameTypePacket(pk) => pk.as_ref(),
            V2259::SetPlayerInventoryOptionsPacket(pk) => pk.as_ref(),
            V2259::SetScorePacket(pk) => pk.as_ref(),
            V2259::SetScoreboardIdentityPacket(pk) => pk.as_ref(),
            V2259::SetSpawnPositionPacket(pk) => pk.as_ref(),
            V2259::SetTimePacket(pk) => pk.as_ref(),
            V2259::SetTitlePacket(pk) => pk.as_ref(),
            V2259::SettingsCommandPacket(pk) => pk.as_ref(),
            V2259::ShowCreditsPacket(pk) => pk.as_ref(),
            V2259::ShowProfilePacket(pk) => pk.as_ref(),
            V2259::ShowStoreOfferPacket(pk) => pk.as_ref(),
            V2259::SimpleEventPacket(pk) => pk.as_ref(),
            V2259::SimulationTypePacket(pk) => pk.as_ref(),
            V2259::SpawnExperienceOrbPacket(pk) => pk.as_ref(),
            V2259::SpawnParticleEffectPacket(pk) => pk.as_ref(),
            V2259::StartGamePacket(pk) => pk.as_ref(),
            V2259::StopSoundPacket(pk) => pk.as_ref(),
            V2259::StructureBlockUpdatePacket(pk) => pk.as_ref(),
            V2259::StructureDataRequestPacket(pk) => pk.as_ref(),
            V2259::StructureDataResponsePacket(pk) => pk.as_ref(),
            V2259::SubChunkPacket(pk) => pk.as_ref(),
            V2259::SubChunkRequestPacket(pk) => pk.as_ref(),
            V2259::SubClientLoginPacket(pk) => pk.as_ref(),
            V2259::SyncActorPropertyPacket(pk) => pk.as_ref(),
            V2259::SyncWorldClocksPacket(pk) => pk.as_ref(),
            V2259::TakeItemActorPacket(pk) => pk.as_ref(),
            V2259::TextPacket(pk) => pk.as_ref(),
            V2259::TickingAreaLoadStatusPacket(pk) => pk.as_ref(),
            V2259::ToastRequestPacket(pk) => pk.as_ref(),
            V2259::TransferPlayerPacket(pk) => pk.as_ref(),
            V2259::TrimDataPacket(pk) => pk.as_ref(),
            V2259::UnlockedRecipesPacket(pk) => pk.as_ref(),
            V2259::UpdateAbilitiesPacket(pk) => pk.as_ref(),
            V2259::UpdateAdventureSettingsPacket(pk) => pk.as_ref(),
            V2259::UpdateAttributesPacket(pk) => pk.as_ref(),
            V2259::UpdateBlockPacket(pk) => pk.as_ref(),
            V2259::UpdateBlockSyncedPacket(pk) => pk.as_ref(),
            V2259::UpdateClientInputLocksPacket(pk) => pk.as_ref(),
            V2259::UpdateClientOptionsPacket(pk) => pk.as_ref(),
            V2259::UpdateEquipPacket(pk) => pk.as_ref(),
            V2259::UpdatePlayerGameTypePacket(pk) => pk.as_ref(),
            V2259::UpdateSoftEnumPacket(pk) => pk.as_ref(),
            V2259::UpdateSubChunkBlocksPacket(pk) => pk.as_ref(),
            V2259::UpdateTradePacket(pk) => pk.as_ref(),
            V2259::VirtualContainerClosePacket(pk) => pk.as_ref(),
            V2259::VirtualContainerOpenPacket(pk) => pk.as_ref(),
            V2259::VoxelShapesPacket(pk) => pk.as_ref(),
            V2259::Unknown(pk) => pk.as_ref(),
        }
    }
}
#[cfg(feature = "packet-dyn")]
impl From<V2259> for Box<dyn bedrock_network_core::PacketDyn> {
    fn from(val: V2259) -> Box<dyn bedrock_network_core::PacketDyn> {
        match val {
            V2259::ActorEventPacket(pk) => pk,
            V2259::ActorPickRequestPacket(pk) => pk,
            V2259::AddActorPacket(pk) => pk,
            V2259::AddBehaviourTreePacket(pk) => pk,
            V2259::AddItemActorPacket(pk) => pk,
            V2259::AddPaintingPacket(pk) => pk,
            V2259::AddPlayerPacket(pk) => pk,
            V2259::AgentActionEventPacket(pk) => pk,
            V2259::AgentAnimationPacket(pk) => pk,
            V2259::AnimateEntityPacket(pk) => pk,
            V2259::AnimatePacket(pk) => pk,
            V2259::AnvilDamagePacket(pk) => pk,
            V2259::AutomationClientConnectPacket(pk) => pk,
            V2259::AvailableActorIdentifiersPacket(pk) => pk,
            V2259::AvailableCommandsPacket(pk) => pk,
            V2259::AwardAchievementPacket(pk) => pk,
            V2259::BiomeDefinitionListPacket(pk) => pk,
            V2259::BlockActorDataPacket(pk) => pk,
            V2259::BlockEventPacket(pk) => pk,
            V2259::BlockPickRequestPacket(pk) => pk,
            V2259::BookEditPacket(pk) => pk,
            V2259::BossEventPacket(pk) => pk,
            V2259::CameraAimAssistActorPriorityPacket(pk) => pk,
            V2259::CameraAimAssistInstructionPacket(pk) => pk,
            V2259::CameraAimAssistPacket(pk) => pk,
            V2259::CameraAimAssistPresetsPacket(pk) => pk,
            V2259::CameraInstructionPacket(pk) => pk,
            V2259::CameraPacket(pk) => pk,
            V2259::CameraPresetsPacket(pk) => pk,
            V2259::CameraShakePacket(pk) => pk,
            V2259::CameraSplinePacket(pk) => pk,
            V2259::ChangeDimensionPacket(pk) => pk,
            V2259::ChangeMobPropertyPacket(pk) => pk,
            V2259::ChatPacket(pk) => pk,
            V2259::ChunkRadiusUpdatedPacket(pk) => pk,
            V2259::ClientBoundAttributeLayerSyncPacket(pk) => pk,
            V2259::ClientBoundCloseFormPacket(pk) => pk,
            V2259::ClientBoundControlSchemeSetPacket(pk) => pk,
            V2259::ClientBoundDataDrivenUICloseScreenPacket(pk) => pk,
            V2259::ClientBoundDataDrivenUIReloadPacket(pk) => pk,
            V2259::ClientBoundDataDrivenUIShowScreenPacket(pk) => pk,
            V2259::ClientBoundDataStorePacket(pk) => pk,
            V2259::ClientBoundDebugRendererPacket(pk) => pk,
            V2259::ClientBoundMapItemDataPacket(pk) => pk,
            V2259::ClientBoundMatchmakingStatePacket(pk) => pk,
            V2259::ClientBoundMusicPlaybackFailurePacket(pk) => pk,
            V2259::ClientBoundOnlineMusicPlaybackResponsePacket(pk) => pk,
            V2259::ClientBoundPlayAudioContentPacket(pk) => pk,
            V2259::ClientBoundStonecutterSetRecipePacket(pk) => pk,
            V2259::ClientBoundTextureShiftPacket(pk) => pk,
            V2259::ClientBoundUpdateSoundDataPacket(pk) => pk,
            V2259::ClientCacheBlobStatusPacket(pk) => pk,
            V2259::ClientCacheMissResponsePacket(pk) => pk,
            V2259::ClientCacheStatusPacket(pk) => pk,
            V2259::ClientToServerHandshakePacket(pk) => pk,
            V2259::CodeBuilderPacket(pk) => pk,
            V2259::CodeBuilderSourcePacket(pk) => pk,
            V2259::CommandBlockUpdatePacket(pk) => pk,
            V2259::CommandOutputPacket(pk) => pk,
            V2259::CommandRequestPacket(pk) => pk,
            V2259::CompletedUsingItemPacket(pk) => pk,
            V2259::ContainerClosePacket(pk) => pk,
            V2259::ContainerOpenPacket(pk) => pk,
            V2259::ContainerRegistryCleanupPacket(pk) => pk,
            V2259::ContainerSetDataPacket(pk) => pk,
            V2259::CorrectPlayerMovePredictionPacket(pk) => pk,
            V2259::CraftingDataPacket(pk) => pk,
            V2259::CreatePhotoPacket(pk) => pk,
            V2259::CreativeContentPacket(pk) => pk,
            V2259::CurrentStructureFeaturePacket(pk) => pk,
            V2259::DeathInfoPacket(pk) => pk,
            V2259::DebugDrawerPacket(pk) => pk,
            V2259::DebugInfoPacket(pk) => pk,
            V2259::DimensionDataPacket(pk) => pk,
            V2259::DisconnectPacket(pk) => pk,
            V2259::EditorNetworkPacket(pk) => pk,
            V2259::EduUriResourcePacket(pk) => pk,
            V2259::EducationSettingsPacket(pk) => pk,
            V2259::EmoteListPacket(pk) => pk,
            V2259::EmotePacket(pk) => pk,
            V2259::FeatureRegistryPacket(pk) => pk,
            V2259::GameRulesChangedPacket(pk) => pk,
            V2259::GameTestRequestPacket(pk) => pk,
            V2259::GameTestResultsPacket(pk) => pk,
            V2259::GraphicsParameterOverridePacket(pk) => pk,
            V2259::GuiDataPickItemPacket(pk) => pk,
            V2259::HurtArmorPacket(pk) => pk,
            V2259::InteractPacket(pk) => pk,
            V2259::InventoryContentPacket(pk) => pk,
            V2259::InventorySlotPacket(pk) => pk,
            V2259::InventoryTransactionPacket(pk) => pk,
            V2259::ItemComponentPacket(pk) => pk,
            V2259::ItemStackRequestPacket(pk) => pk,
            V2259::ItemStackResponsePacket(pk) => pk,
            V2259::JigsawStructureDataPacket(pk) => pk,
            V2259::LabTablePacket(pk) => pk,
            V2259::LecternUpdatePacket(pk) => pk,
            V2259::LegacyTelemetryEventPacket(pk) => pk,
            V2259::LessonProgressPacket(pk) => pk,
            V2259::LevelChunkPacket(pk) => pk,
            V2259::LevelEventGenericPacket(pk) => pk,
            V2259::LevelEventPacket(pk) => pk,
            V2259::LevelSoundEventPacket(pk) => pk,
            V2259::LocatorBarPacket(pk) => pk,
            V2259::LoginPacket(pk) => pk,
            V2259::MapCreateLockedCopyPacket(pk) => pk,
            V2259::MapInfoRequestPacket(pk) => pk,
            V2259::MobArmorEquipmentPacket(pk) => pk,
            V2259::MobEffectPacket(pk) => pk,
            V2259::MobEquipmentPacket(pk) => pk,
            V2259::ModalFormRequestPacket(pk) => pk,
            V2259::ModalFormResponsePacket(pk) => pk,
            V2259::MotionPredictionHintsPacket(pk) => pk,
            V2259::MoveActorAbsolutePacket(pk) => pk,
            V2259::MoveActorDeltaPacket(pk) => pk,
            V2259::MovePlayerPacket(pk) => pk,
            V2259::MovementEffectPacket(pk) => pk,
            V2259::MovementPredictionSyncPacket(pk) => pk,
            V2259::MultiplayerSettingsPacket(pk) => pk,
            V2259::NetworkChunkPublisherUpdatePacket(pk) => pk,
            V2259::NetworkSettingsPacket(pk) => pk,
            V2259::NetworkStackLatencyPacket(pk) => pk,
            V2259::NpcDialoguePacket(pk) => pk,
            V2259::NpcRequestPacket(pk) => pk,
            V2259::OnScreenTextureAnimationPacket(pk) => pk,
            V2259::OpenSignPacket(pk) => pk,
            V2259::PacketViolationWarningPacket(pk) => pk,
            V2259::PartyChangedPacket(pk) => pk,
            V2259::PartyDestinationCookieResponsePacket(pk) => pk,
            V2259::PhotoTransferPacket(pk) => pk,
            V2259::PlaySoundPacket(pk) => pk,
            V2259::PlayStatusPacket(pk) => pk,
            V2259::PlayerActionPacket(pk) => pk,
            V2259::PlayerArmorDamagePacket(pk) => pk,
            V2259::PlayerAuthInputPacket(pk) => pk,
            V2259::PlayerEnchantOptionsPacket(pk) => pk,
            V2259::PlayerFogPacket(pk) => pk,
            V2259::PlayerHotbarPacket(pk) => pk,
            V2259::PlayerListPacket(pk) => pk,
            V2259::PlayerLocationPacket(pk) => pk,
            V2259::PlayerSkinPacket(pk) => pk,
            V2259::PlayerStartItemCooldownPacket(pk) => pk,
            V2259::PlayerToggleArmorVisibilityRequestPacket(pk) => pk,
            V2259::PlayerToggleCrafterSlotRequestPacket(pk) => pk,
            V2259::PlayerUpdateEntityOverridesPacket(pk) => pk,
            V2259::PlayerVideoCapturePacket(pk) => pk,
            V2259::PositionTrackingDBClientRequestPacket(pk) => pk,
            V2259::PositionTrackingDBServerBroadcastPacket(pk) => pk,
            V2259::PurchaseReceiptPacket(pk) => pk,
            V2259::RecordStartedPacket(pk) => pk,
            V2259::RefreshEntitlementsPacket(pk) => pk,
            V2259::RemoveActorPacket(pk) => pk,
            V2259::RemoveObjectivePacket(pk) => pk,
            V2259::RequestAbilityPacket(pk) => pk,
            V2259::RequestChunkRadiusPacket(pk) => pk,
            V2259::RequestNetworkSettingsPacket(pk) => pk,
            V2259::RequestPermissionsPacket(pk) => pk,
            V2259::ResourcePackChunkDataPacket(pk) => pk,
            V2259::ResourcePackChunkRequestPacket(pk) => pk,
            V2259::ResourcePackClientResponsePacket(pk) => pk,
            V2259::ResourcePackDataInfoPacket(pk) => pk,
            V2259::ResourcePackStackPacket(pk) => pk,
            V2259::ResourcePacksInfoPacket(pk) => pk,
            V2259::ResourcePacksReadyForValidationPacket(pk) => pk,
            V2259::RespawnPacket(pk) => pk,
            V2259::ScriptMessagePacket(pk) => pk,
            V2259::SendPartyDestinationCookiePacket(pk) => pk,
            V2259::ServerBoundCursorItemDragPacket(pk) => pk,
            V2259::ServerBoundDataDrivenClosedPacket(pk) => pk,
            V2259::ServerBoundDataStorePacket(pk) => pk,
            V2259::ServerBoundDiagnosticsPacket(pk) => pk,
            V2259::ServerBoundLoadingScreenPacket(pk) => pk,
            V2259::ServerBoundMatchmakingCancelPacket(pk) => pk,
            V2259::ServerBoundPackSettingChangePacket(pk) => pk,
            V2259::ServerBoundRegisterAudioContentPacket(pk) => pk,
            V2259::ServerBoundRequestOnlineMusicPlaybackPacket(pk) => pk,
            V2259::ServerBoundStonecutterSetRecipePacket(pk) => pk,
            V2259::ServerPlayerPostMovePositionPacket(pk) => pk,
            V2259::ServerPresenceInfoPacket(pk) => pk,
            V2259::ServerSettingsRequestPacket(pk) => pk,
            V2259::ServerSettingsResponsePacket(pk) => pk,
            V2259::ServerStatsPacket(pk) => pk,
            V2259::ServerStoreInfoPacket(pk) => pk,
            V2259::ServerToClientHandshakePacket(pk) => pk,
            V2259::SetActorDataPacket(pk) => pk,
            V2259::SetActorLinkPacket(pk) => pk,
            V2259::SetActorMotionPacket(pk) => pk,
            V2259::SetCommandsEnabledPacket(pk) => pk,
            V2259::SetDefaultGameTypePacket(pk) => pk,
            V2259::SetDifficultyPacket(pk) => pk,
            V2259::SetDisplayObjectivePacket(pk) => pk,
            V2259::SetHealthPacket(pk) => pk,
            V2259::SetHudPacket(pk) => pk,
            V2259::SetLastHurtByPacket(pk) => pk,
            V2259::SetLocalPlayerAsInitializedPacket(pk) => pk,
            V2259::SetPassengerOfBlockPacket(pk) => pk,
            V2259::SetPlayerFurnaceOptionsPacket(pk) => pk,
            V2259::SetPlayerGameTypePacket(pk) => pk,
            V2259::SetPlayerInventoryOptionsPacket(pk) => pk,
            V2259::SetScorePacket(pk) => pk,
            V2259::SetScoreboardIdentityPacket(pk) => pk,
            V2259::SetSpawnPositionPacket(pk) => pk,
            V2259::SetTimePacket(pk) => pk,
            V2259::SetTitlePacket(pk) => pk,
            V2259::SettingsCommandPacket(pk) => pk,
            V2259::ShowCreditsPacket(pk) => pk,
            V2259::ShowProfilePacket(pk) => pk,
            V2259::ShowStoreOfferPacket(pk) => pk,
            V2259::SimpleEventPacket(pk) => pk,
            V2259::SimulationTypePacket(pk) => pk,
            V2259::SpawnExperienceOrbPacket(pk) => pk,
            V2259::SpawnParticleEffectPacket(pk) => pk,
            V2259::StartGamePacket(pk) => pk,
            V2259::StopSoundPacket(pk) => pk,
            V2259::StructureBlockUpdatePacket(pk) => pk,
            V2259::StructureDataRequestPacket(pk) => pk,
            V2259::StructureDataResponsePacket(pk) => pk,
            V2259::SubChunkPacket(pk) => pk,
            V2259::SubChunkRequestPacket(pk) => pk,
            V2259::SubClientLoginPacket(pk) => pk,
            V2259::SyncActorPropertyPacket(pk) => pk,
            V2259::SyncWorldClocksPacket(pk) => pk,
            V2259::TakeItemActorPacket(pk) => pk,
            V2259::TextPacket(pk) => pk,
            V2259::TickingAreaLoadStatusPacket(pk) => pk,
            V2259::ToastRequestPacket(pk) => pk,
            V2259::TransferPlayerPacket(pk) => pk,
            V2259::TrimDataPacket(pk) => pk,
            V2259::UnlockedRecipesPacket(pk) => pk,
            V2259::UpdateAbilitiesPacket(pk) => pk,
            V2259::UpdateAdventureSettingsPacket(pk) => pk,
            V2259::UpdateAttributesPacket(pk) => pk,
            V2259::UpdateBlockPacket(pk) => pk,
            V2259::UpdateBlockSyncedPacket(pk) => pk,
            V2259::UpdateClientInputLocksPacket(pk) => pk,
            V2259::UpdateClientOptionsPacket(pk) => pk,
            V2259::UpdateEquipPacket(pk) => pk,
            V2259::UpdatePlayerGameTypePacket(pk) => pk,
            V2259::UpdateSoftEnumPacket(pk) => pk,
            V2259::UpdateSubChunkBlocksPacket(pk) => pk,
            V2259::UpdateTradePacket(pk) => pk,
            V2259::VirtualContainerClosePacket(pk) => pk,
            V2259::VirtualContainerOpenPacket(pk) => pk,
            V2259::VoxelShapesPacket(pk) => pk,
            V2259::Unknown(pk) => pk,
        }
    }
}
