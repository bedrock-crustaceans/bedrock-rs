#![allow(unused, clippy::too_many_lines)]
use bedrock_protocol_version::ProtoVersion;
use bedrock_protocol_version::ProtoVersionEnums;
use bedrock_protocol_version::ProtoVersionPackets;
use bedrock_protocol_version::ProtoVersionTypes;
#[derive(Clone, std::fmt::Debug)]
pub enum V2168 {
    ActorEventPacket(Box<<Self as ProtoVersionPackets>::ActorEventPacket>),
    ActorPickRequestPacket(Box<<Self as ProtoVersionPackets>::ActorPickRequestPacket>),
    AddActorPacket(Box<<Self as ProtoVersionPackets>::AddActorPacket>),
    AddBehaviourTreePacket(Box<<Self as ProtoVersionPackets>::AddBehaviourTreePacket>),
    AddItemActorPacket(Box<<Self as ProtoVersionPackets>::AddItemActorPacket>),
    AddPaintingPacket(Box<<Self as ProtoVersionPackets>::AddPaintingPacket>),
    AddPlayerPacket(Box<<Self as ProtoVersionPackets>::AddPlayerPacket>),
    AddVolumeEntityPacket(Box<<Self as ProtoVersionPackets>::AddVolumeEntityPacket>),
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
    RefreshEntitlementsPacket(Box<<Self as ProtoVersionPackets>::RefreshEntitlementsPacket>),
    RemoveActorPacket(Box<<Self as ProtoVersionPackets>::RemoveActorPacket>),
    RemoveObjectivePacket(Box<<Self as ProtoVersionPackets>::RemoveObjectivePacket>),
    RemoveVolumeEntityPacket(Box<<Self as ProtoVersionPackets>::RemoveVolumeEntityPacket>),
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
    ServerBoundDataDrivenClosedPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket>,
    ),
    ServerBoundDataStorePacket(Box<<Self as ProtoVersionPackets>::ServerBoundDataStorePacket>),
    ServerBoundDiagnosticsPacket(Box<<Self as ProtoVersionPackets>::ServerBoundDiagnosticsPacket>),
    ServerBoundLoadingScreenPacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundLoadingScreenPacket>,
    ),
    ServerBoundPackSettingChangePacket(
        Box<<Self as ProtoVersionPackets>::ServerBoundPackSettingChangePacket>,
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
    VoxelShapesPacket(Box<<Self as ProtoVersionPackets>::VoxelShapesPacket>),
    Unknown(Box<bedrock_protocol_core::UnknownPacket>),
}
impl bedrock_protocol_core::Packets for V2168 {
    #[inline]
    fn serialize<W: std::io::Write>(
        &self,
        header: &bedrock_protocol_core::PacketHeader,
        stream: &mut W,
    ) -> Result<(), bedrock_protocol_core::error::PacketCodecError> {
        <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::serialize(
            header, stream,
        )
        .map_err(bedrock_protocol_core::error::PacketCodecError::InvalidHeader)?;
        match self {
            V2168::ActorEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ActorPickRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorPickRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AddActorPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AddBehaviourTreePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddBehaviourTreePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AddItemActorPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddItemActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AddPaintingPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPaintingPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AddPlayerPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPlayerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AddVolumeEntityPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddVolumeEntityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AgentActionEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentActionEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AgentAnimationPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentAnimationPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AnimateEntityPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimateEntityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AnimatePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AnvilDamagePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnvilDamagePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AutomationClientConnectPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AutomationClientConnectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AvailableActorIdentifiersPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableActorIdentifiersPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AvailableCommandsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableCommandsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::AwardAchievementPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AwardAchievementPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::BiomeDefinitionListPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BiomeDefinitionListPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::BlockActorDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockActorDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::BlockEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::BlockPickRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockPickRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::BookEditPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BookEditPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::BossEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BossEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraAimAssistActorPriorityPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistActorPriorityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraAimAssistInstructionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistInstructionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraAimAssistPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraAimAssistPresetsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPresetsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraInstructionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraInstructionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraPresetsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPresetsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraShakePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraShakePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CameraSplinePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraSplinePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ChangeDimensionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeDimensionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ChangeMobPropertyPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeMobPropertyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ChunkRadiusUpdatedPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChunkRadiusUpdatedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundAttributeLayerSyncPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundAttributeLayerSyncPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundCloseFormPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundCloseFormPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundControlSchemeSetPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundControlSchemeSetPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundDataDrivenUICloseScreenPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUICloseScreenPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundDataDrivenUIReloadPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataDrivenUIReloadPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundDataDrivenUIShowScreenPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUIShowScreenPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundDataStorePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataStorePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundDebugRendererPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDebugRendererPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundMapItemDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundMapItemDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundTextureShiftPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundTextureShiftPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientBoundUpdateSoundDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundUpdateSoundDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientCacheBlobStatusPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheBlobStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientCacheMissResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheMissResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientCacheStatusPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ClientToServerHandshakePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientToServerHandshakePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CodeBuilderPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CodeBuilderSourcePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderSourcePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CommandBlockUpdatePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandBlockUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CommandOutputPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandOutputPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CommandRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CompletedUsingItemPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CompletedUsingItemPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ContainerClosePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerClosePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ContainerOpenPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerOpenPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ContainerRegistryCleanupPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerRegistryCleanupPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ContainerSetDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerSetDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CorrectPlayerMovePredictionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CraftingDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CraftingDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CreatePhotoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreatePhotoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CreativeContentPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreativeContentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::CurrentStructureFeaturePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CurrentStructureFeaturePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::DeathInfoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DeathInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::DebugDrawerPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugDrawerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::DebugInfoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::DimensionDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DimensionDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::DisconnectPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DisconnectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::EditorNetworkPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EditorNetworkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::EduUriResourcePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EduUriResourcePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::EducationSettingsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EducationSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::EmoteListPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmoteListPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::EmotePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmotePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::FeatureRegistryPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(FeatureRegistryPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::GameRulesChangedPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameRulesChangedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::GameTestRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::GameTestResultsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestResultsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::GraphicsParameterOverridePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GraphicsParameterOverridePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::GuiDataPickItemPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GuiDataPickItemPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::HurtArmorPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(HurtArmorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::InteractPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InteractPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::InventoryContentPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryContentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::InventorySlotPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventorySlotPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::InventoryTransactionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryTransactionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ItemComponentPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemComponentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ItemStackRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ItemStackResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::JigsawStructureDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(JigsawStructureDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LabTablePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LabTablePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LecternUpdatePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LecternUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LegacyTelemetryEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LegacyTelemetryEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LessonProgressPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LessonProgressPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LevelChunkPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelChunkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LevelEventGenericPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventGenericPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LevelEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LevelSoundEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelSoundEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LocatorBarPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LocatorBarPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::LoginPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LoginPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MapCreateLockedCopyPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapCreateLockedCopyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MapInfoRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapInfoRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MobArmorEquipmentPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobArmorEquipmentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MobEffectPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEffectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MobEquipmentPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEquipmentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ModalFormRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ModalFormResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MotionPredictionHintsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MotionPredictionHintsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MoveActorAbsolutePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorAbsolutePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MoveActorDeltaPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorDeltaPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MovePlayerPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovePlayerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MovementEffectPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementEffectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MovementPredictionSyncPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementPredictionSyncPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::MultiplayerSettingsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MultiplayerSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::NetworkChunkPublisherUpdatePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::NetworkSettingsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::NetworkStackLatencyPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkStackLatencyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::NpcDialoguePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcDialoguePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::NpcRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::OnScreenTextureAnimationPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OnScreenTextureAnimationPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::OpenSignPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OpenSignPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PacketViolationWarningPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PacketViolationWarningPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PartyChangedPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PartyChangedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PartyDestinationCookieResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PartyDestinationCookieResponsePacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PhotoTransferPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PhotoTransferPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlaySoundPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlaySoundPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayStatusPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerActionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerActionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerArmorDamagePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerArmorDamagePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerAuthInputPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerAuthInputPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerEnchantOptionsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerEnchantOptionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerFogPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerFogPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerHotbarPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerHotbarPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerListPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerListPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerLocationPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerLocationPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerSkinPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerSkinPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerStartItemCooldownPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerStartItemCooldownPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerToggleCrafterSlotRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PlayerToggleCrafterSlotRequestPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerUpdateEntityOverridesPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerUpdateEntityOverridesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PlayerVideoCapturePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerVideoCapturePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PositionTrackingDBClientRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBClientRequestPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PositionTrackingDBServerBroadcastPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBServerBroadcastPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::PurchaseReceiptPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PurchaseReceiptPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RefreshEntitlementsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RefreshEntitlementsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RemoveActorPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RemoveObjectivePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveObjectivePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RemoveVolumeEntityPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveVolumeEntityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RequestAbilityPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestAbilityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RequestChunkRadiusPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestChunkRadiusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RequestNetworkSettingsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestNetworkSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RequestPermissionsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestPermissionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePackChunkDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePackChunkRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePackClientResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackClientResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePackDataInfoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackDataInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePackStackPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackStackPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePacksInfoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePacksInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ResourcePacksReadyForValidationPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ResourcePacksReadyForValidationPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::RespawnPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RespawnPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ScriptMessagePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ScriptMessagePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SendPartyDestinationCookiePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SendPartyDestinationCookiePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerBoundDataDrivenClosedPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataDrivenClosedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerBoundDataStorePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataStorePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerBoundDiagnosticsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDiagnosticsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerBoundLoadingScreenPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundLoadingScreenPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerBoundPackSettingChangePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundPackSettingChangePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerPlayerPostMovePositionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerPresenceInfoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPresenceInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerSettingsRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerSettingsResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerStatsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStatsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerStoreInfoPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStoreInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ServerToClientHandshakePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerToClientHandshakePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetActorDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetActorLinkPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorLinkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetActorMotionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorMotionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetCommandsEnabledPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetCommandsEnabledPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetDefaultGameTypePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDefaultGameTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetDifficultyPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDifficultyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetDisplayObjectivePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDisplayObjectivePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetHealthPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHealthPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetHudPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHudPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetLastHurtByPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLastHurtByPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetLocalPlayerAsInitializedPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetPlayerGameTypePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerGameTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetPlayerInventoryOptionsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetScorePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScorePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetScoreboardIdentityPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScoreboardIdentityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetSpawnPositionPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetSpawnPositionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetTimePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTimePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SetTitlePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTitlePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SettingsCommandPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SettingsCommandPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ShowCreditsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowCreditsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ShowProfilePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowProfilePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ShowStoreOfferPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowStoreOfferPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SimpleEventPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimpleEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SimulationTypePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimulationTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SpawnExperienceOrbPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnExperienceOrbPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SpawnParticleEffectPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnParticleEffectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::StartGamePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StartGamePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::StopSoundPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StopSoundPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::StructureBlockUpdatePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureBlockUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::StructureDataRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::StructureDataResponsePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SubChunkPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SubChunkRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SubClientLoginPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubClientLoginPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SyncActorPropertyPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncActorPropertyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::SyncWorldClocksPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncWorldClocksPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::TakeItemActorPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TakeItemActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::TextPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TextPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::TickingAreaLoadStatusPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TickingAreaLoadStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::ToastRequestPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ToastRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::TransferPlayerPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TransferPlayerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::TrimDataPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TrimDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UnlockedRecipesPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UnlockedRecipesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateAbilitiesPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAbilitiesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateAdventureSettingsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAdventureSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateAttributesPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAttributesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateBlockPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateBlockSyncedPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockSyncedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateClientInputLocksPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientInputLocksPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateClientOptionsPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientOptionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateEquipPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateEquipPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdatePlayerGameTypePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdatePlayerGameTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateSoftEnumPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSoftEnumPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateSubChunkBlocksPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSubChunkBlocksPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::UpdateTradePacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateTradePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::VoxelShapesPacket(pk) => {
                match <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                    pk.as_ref(),
                    stream,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VoxelShapesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            V2168::Unknown(pk) => {
                stream
                    .write_all(pk.buf.as_ref())
                    .map_err(|e| bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
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
        (Self, bedrock_protocol_core::PacketHeader),
        bedrock_protocol_core::error::PacketCodecError,
    > {
        let header = <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::deserialize(
                stream,
            )
            .map_err(bedrock_protocol_core::error::PacketCodecError::InvalidHeader)?;
        let packet = match header.packet_id {
            <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ActorEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ActorPickRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ActorPickRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AddActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AddBehaviourTreePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddBehaviourTreePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AddItemActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddItemActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AddPaintingPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPaintingPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AddPlayerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddPlayerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AddVolumeEntityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AddVolumeEntityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AgentActionEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentActionEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AgentAnimationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AgentAnimationPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AnimateEntityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimateEntityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AnimatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnimatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AnvilDamagePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AnvilDamagePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AutomationClientConnectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AutomationClientConnectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AvailableActorIdentifiersPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableActorIdentifiersPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AvailableCommandsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AvailableCommandsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::AwardAchievementPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(AwardAchievementPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::BiomeDefinitionListPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BiomeDefinitionListPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::BlockActorDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockActorDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::BlockEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::BlockPickRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BlockPickRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::BookEditPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BookEditPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::BossEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(BossEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraAimAssistActorPriorityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistActorPriorityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraAimAssistInstructionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistInstructionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraAimAssistPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraAimAssistPresetsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraAimAssistPresetsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraInstructionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraInstructionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraPresetsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraPresetsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraShakePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraShakePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CameraSplinePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CameraSplinePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ChangeDimensionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeDimensionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ChangeMobPropertyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChangeMobPropertyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ChunkRadiusUpdatedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ChunkRadiusUpdatedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundAttributeLayerSyncPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundAttributeLayerSyncPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundCloseFormPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundCloseFormPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundControlSchemeSetPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundControlSchemeSetPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2168::ClientBoundDataDrivenUICloseScreenPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUICloseScreenPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundDataDrivenUIReloadPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataDrivenUIReloadPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2168::ClientBoundDataDrivenUIShowScreenPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ClientBoundDataDrivenUIShowScreenPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundDataStorePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDataStorePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundDebugRendererPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundDebugRendererPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundMapItemDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundMapItemDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundTextureShiftPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundTextureShiftPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientBoundUpdateSoundDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientBoundUpdateSoundDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientCacheBlobStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheBlobStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientCacheMissResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheMissResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientCacheStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientCacheStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ClientToServerHandshakePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ClientToServerHandshakePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CodeBuilderPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CodeBuilderSourcePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CodeBuilderSourcePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CommandBlockUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandBlockUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CommandOutputPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandOutputPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CommandRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CommandRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CompletedUsingItemPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CompletedUsingItemPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ContainerClosePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerClosePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ContainerOpenPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerOpenPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ContainerRegistryCleanupPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerRegistryCleanupPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ContainerSetDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ContainerSetDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CorrectPlayerMovePredictionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CraftingDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CraftingDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CreatePhotoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreatePhotoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CreativeContentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CreativeContentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::CurrentStructureFeaturePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(CurrentStructureFeaturePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::DeathInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DeathInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::DebugDrawerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugDrawerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::DebugInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DebugInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::DimensionDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DimensionDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::DisconnectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(DisconnectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::EditorNetworkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EditorNetworkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::EduUriResourcePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EduUriResourcePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::EducationSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EducationSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::EmoteListPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmoteListPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::EmotePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(EmotePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::FeatureRegistryPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(FeatureRegistryPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::GameRulesChangedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameRulesChangedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::GameTestRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::GameTestResultsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GameTestResultsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::GraphicsParameterOverridePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GraphicsParameterOverridePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::GuiDataPickItemPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(GuiDataPickItemPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::HurtArmorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(HurtArmorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::InteractPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InteractPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::InventoryContentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryContentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::InventorySlotPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventorySlotPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::InventoryTransactionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(InventoryTransactionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ItemComponentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemComponentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ItemStackRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ItemStackResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ItemStackResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::JigsawStructureDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(JigsawStructureDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LabTablePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LabTablePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LecternUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LecternUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LegacyTelemetryEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LegacyTelemetryEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LessonProgressPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LessonProgressPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LevelChunkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelChunkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LevelEventGenericPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventGenericPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LevelEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LevelSoundEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LevelSoundEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LocatorBarPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LocatorBarPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::LoginPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(LoginPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MapCreateLockedCopyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapCreateLockedCopyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MapInfoRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MapInfoRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MobArmorEquipmentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobArmorEquipmentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MobEffectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEffectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MobEquipmentPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MobEquipmentPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ModalFormRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ModalFormResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ModalFormResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MotionPredictionHintsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MotionPredictionHintsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MoveActorAbsolutePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorAbsolutePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MoveActorDeltaPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MoveActorDeltaPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MovePlayerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovePlayerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MovementEffectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementEffectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MovementPredictionSyncPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MovementPredictionSyncPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::MultiplayerSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(MultiplayerSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::NetworkChunkPublisherUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::NetworkSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::NetworkStackLatencyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NetworkStackLatencyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::NpcDialoguePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcDialoguePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::NpcRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(NpcRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::OnScreenTextureAnimationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OnScreenTextureAnimationPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::OpenSignPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(OpenSignPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PacketViolationWarningPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PacketViolationWarningPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PartyChangedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PartyChangedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PartyDestinationCookieResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PartyDestinationCookieResponsePacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PhotoTransferPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PhotoTransferPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlaySoundPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlaySoundPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerActionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerActionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerArmorDamagePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerArmorDamagePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerAuthInputPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerAuthInputPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerEnchantOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerEnchantOptionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerFogPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerFogPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerHotbarPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerHotbarPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerListPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerListPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerLocationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerLocationPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerSkinPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerSkinPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerStartItemCooldownPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerStartItemCooldownPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerToggleCrafterSlotRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PlayerToggleCrafterSlotRequestPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerUpdateEntityOverridesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerUpdateEntityOverridesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PlayerVideoCapturePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PlayerVideoCapturePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PositionTrackingDBClientRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBClientRequestPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => {
                        V2168::PositionTrackingDBServerBroadcastPacket(Box::new(pk))
                    }
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                PositionTrackingDBServerBroadcastPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::PurchaseReceiptPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(PurchaseReceiptPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RefreshEntitlementsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RefreshEntitlementsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RemoveActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RemoveObjectivePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveObjectivePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RemoveVolumeEntityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RemoveVolumeEntityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RequestAbilityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestAbilityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RequestChunkRadiusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestChunkRadiusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RequestNetworkSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestNetworkSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RequestPermissionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RequestPermissionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePackChunkDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePackChunkRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackChunkRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePackClientResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackClientResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePackDataInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackDataInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePackStackPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePackStackPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePacksInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ResourcePacksInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ResourcePacksReadyForValidationPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(
                                ResourcePacksReadyForValidationPacket
                            ),
                            packet_id: <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::RespawnPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(RespawnPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ScriptMessagePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ScriptMessagePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SendPartyDestinationCookiePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SendPartyDestinationCookiePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerBoundDataDrivenClosedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataDrivenClosedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerBoundDataStorePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDataStorePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerBoundDiagnosticsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundDiagnosticsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerBoundLoadingScreenPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundLoadingScreenPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerBoundPackSettingChangePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerBoundPackSettingChangePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerPlayerPostMovePositionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerPresenceInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerPresenceInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerSettingsRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerSettingsResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerSettingsResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerStatsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStatsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerStoreInfoPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerStoreInfoPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ServerToClientHandshakePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ServerToClientHandshakePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetActorDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetActorLinkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorLinkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetActorMotionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetActorMotionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetCommandsEnabledPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetCommandsEnabledPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetDefaultGameTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDefaultGameTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetDifficultyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDifficultyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetDisplayObjectivePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetDisplayObjectivePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetHealthPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHealthPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetHudPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetHudPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetLastHurtByPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLastHurtByPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetLocalPlayerAsInitializedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetPlayerGameTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerGameTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetPlayerInventoryOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetScorePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScorePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetScoreboardIdentityPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetScoreboardIdentityPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetSpawnPositionPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetSpawnPositionPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetTimePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTimePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SetTitlePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SetTitlePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SettingsCommandPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SettingsCommandPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ShowCreditsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowCreditsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ShowProfilePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowProfilePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ShowStoreOfferPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ShowStoreOfferPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SimpleEventPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimpleEventPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SimulationTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SimulationTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SpawnExperienceOrbPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnExperienceOrbPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SpawnParticleEffectPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SpawnParticleEffectPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::StartGamePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StartGamePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::StopSoundPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StopSoundPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::StructureBlockUpdatePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureBlockUpdatePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::StructureDataRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::StructureDataResponsePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(StructureDataResponsePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SubChunkPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SubChunkRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubChunkRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SubClientLoginPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SubClientLoginPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SyncActorPropertyPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncActorPropertyPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::SyncWorldClocksPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(SyncWorldClocksPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::TakeItemActorPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TakeItemActorPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::TextPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TextPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::TickingAreaLoadStatusPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TickingAreaLoadStatusPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::ToastRequestPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(ToastRequestPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::TransferPlayerPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TransferPlayerPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::TrimDataPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(TrimDataPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UnlockedRecipesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UnlockedRecipesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateAbilitiesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAbilitiesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateAdventureSettingsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAdventureSettingsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateAttributesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateAttributesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateBlockPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateBlockSyncedPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateBlockSyncedPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateClientInputLocksPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientInputLocksPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateClientOptionsPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateClientOptionsPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateEquipPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateEquipPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdatePlayerGameTypePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdatePlayerGameTypePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateSoftEnumPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSoftEnumPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateSubChunkBlocksPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateSubChunkBlocksPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::UpdateTradePacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(UpdateTradePacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID => {
                match <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                    stream,
                ) {
                    Ok(pk) => V2168::VoxelShapesPacket(Box::new(pk)),
                    Err(err) => {
                        return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(VoxelShapesPacket),
                            packet_id: <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID,
                            error: err,
                        });
                    }
                }
            }
            unknown => {
                let mut buf = Vec::new();
                stream
                    .read_to_end(&mut buf)
                    .map_err(|e| bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                        packet_name: "Unknown",
                        packet_id: header.packet_id,
                        error: e.into(),
                    })?;
                V2168::Unknown(
                    Box::new(bedrock_protocol_core::UnknownPacket {
                        id: unknown,
                        buf: buf.into_boxed_slice(),
                    }),
                )
            }
        };
        Ok((packet, header))
    }
    #[inline]
    fn size_hint(&self, header: &bedrock_protocol_core::PacketHeader) -> usize {
        <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::size_hint(
            header,
        )
            + match self {
                V2168::ActorEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ActorPickRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AddActorPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AddBehaviourTreePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AddItemActorPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AddPaintingPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AddPlayerPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AddVolumeEntityPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AgentActionEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AgentAnimationPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AnimateEntityPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AnimatePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AnvilDamagePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AutomationClientConnectPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AvailableActorIdentifiersPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AvailableCommandsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::AwardAchievementPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::BiomeDefinitionListPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::BlockActorDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::BlockEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::BlockPickRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::BookEditPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::BossEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraAimAssistActorPriorityPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraAimAssistInstructionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraAimAssistPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraAimAssistPresetsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraInstructionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraPresetsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraShakePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CameraSplinePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ChangeDimensionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ChangeMobPropertyPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ChunkRadiusUpdatedPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundAttributeLayerSyncPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundCloseFormPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundControlSchemeSetPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundDataDrivenUICloseScreenPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundDataDrivenUIReloadPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundDataDrivenUIShowScreenPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundDataStorePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundDebugRendererPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundMapItemDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundTextureShiftPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientBoundUpdateSoundDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientCacheBlobStatusPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientCacheMissResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientCacheStatusPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ClientToServerHandshakePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CodeBuilderPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CodeBuilderSourcePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CommandBlockUpdatePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CommandOutputPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CommandRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CompletedUsingItemPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ContainerClosePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ContainerOpenPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ContainerRegistryCleanupPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ContainerSetDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CorrectPlayerMovePredictionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CraftingDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CreatePhotoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CreativeContentPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::CurrentStructureFeaturePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::DeathInfoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::DebugDrawerPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::DebugInfoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::DimensionDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::DisconnectPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::EditorNetworkPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::EduUriResourcePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::EducationSettingsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::EmoteListPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::EmotePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::FeatureRegistryPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::GameRulesChangedPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::GameTestRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::GameTestResultsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::GraphicsParameterOverridePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::GuiDataPickItemPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::HurtArmorPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::InteractPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::InventoryContentPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::InventorySlotPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::InventoryTransactionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ItemComponentPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ItemStackRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ItemStackResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::JigsawStructureDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LabTablePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LecternUpdatePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LegacyTelemetryEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LessonProgressPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LevelChunkPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LevelEventGenericPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LevelEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LevelSoundEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LocatorBarPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::LoginPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MapCreateLockedCopyPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MapInfoRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MobArmorEquipmentPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MobEffectPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MobEquipmentPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ModalFormRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ModalFormResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MotionPredictionHintsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MoveActorAbsolutePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MoveActorDeltaPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MovePlayerPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MovementEffectPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MovementPredictionSyncPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::MultiplayerSettingsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::NetworkChunkPublisherUpdatePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::NetworkSettingsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::NetworkStackLatencyPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::NpcDialoguePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::NpcRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::OnScreenTextureAnimationPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::OpenSignPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PacketViolationWarningPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PartyChangedPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PartyDestinationCookieResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PhotoTransferPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlaySoundPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayStatusPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerActionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerArmorDamagePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerAuthInputPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerEnchantOptionsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerFogPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerHotbarPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerListPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerLocationPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerSkinPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerStartItemCooldownPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerToggleCrafterSlotRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerUpdateEntityOverridesPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PlayerVideoCapturePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PositionTrackingDBClientRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PositionTrackingDBServerBroadcastPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::PurchaseReceiptPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RefreshEntitlementsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RemoveActorPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RemoveObjectivePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RemoveVolumeEntityPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RequestAbilityPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RequestChunkRadiusPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RequestNetworkSettingsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RequestPermissionsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePackChunkDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePackChunkRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePackClientResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePackDataInfoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePackStackPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePacksInfoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ResourcePacksReadyForValidationPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::RespawnPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ScriptMessagePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SendPartyDestinationCookiePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerBoundDataDrivenClosedPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerBoundDataStorePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerBoundDiagnosticsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerBoundLoadingScreenPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerBoundPackSettingChangePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerPlayerPostMovePositionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerPresenceInfoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerSettingsRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerSettingsResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerStatsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerStoreInfoPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ServerToClientHandshakePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetActorDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetActorLinkPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetActorMotionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetCommandsEnabledPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetDefaultGameTypePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetDifficultyPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetDisplayObjectivePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetHealthPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetHudPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetLastHurtByPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetLocalPlayerAsInitializedPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetPlayerGameTypePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetPlayerInventoryOptionsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetScorePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetScoreboardIdentityPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetSpawnPositionPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetTimePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SetTitlePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SettingsCommandPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ShowCreditsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ShowProfilePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ShowStoreOfferPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SimpleEventPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SimulationTypePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SpawnExperienceOrbPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SpawnParticleEffectPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::StartGamePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::StopSoundPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::StructureBlockUpdatePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::StructureDataRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::StructureDataResponsePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SubChunkPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SubChunkRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SubClientLoginPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SyncActorPropertyPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::SyncWorldClocksPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::TakeItemActorPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::TextPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::TickingAreaLoadStatusPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::ToastRequestPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::TransferPlayerPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::TrimDataPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UnlockedRecipesPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateAbilitiesPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateAdventureSettingsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateAttributesPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateBlockPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateBlockSyncedPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateClientInputLocksPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateClientOptionsPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateEquipPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdatePlayerGameTypePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateSoftEnumPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateSubChunkBlocksPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::UpdateTradePacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::VoxelShapesPacket(pk) => {
                    <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                        pk.as_ref(),
                    )
                }
                V2168::Unknown(pk) => pk.buf.len(),
            }
    }
    #[inline]
    fn id(&self) -> u16 {
        match self {
            V2168::ActorEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ActorPickRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AddActorPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AddBehaviourTreePacket(_) => {
                <<V2168 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AddItemActorPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AddPaintingPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AddPlayerPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AddVolumeEntityPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AgentActionEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AgentAnimationPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AnimateEntityPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AnimatePacket(_) => {
                <<V2168 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AnvilDamagePacket(_) => {
                <<V2168 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AutomationClientConnectPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AvailableActorIdentifiersPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AvailableCommandsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::AwardAchievementPacket(_) => {
                <<V2168 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::BiomeDefinitionListPacket(_) => {
                <<V2168 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::BlockActorDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::BlockEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::BlockPickRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::BookEditPacket(_) => {
                <<V2168 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::BossEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraAimAssistActorPriorityPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraAimAssistInstructionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraAimAssistPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraAimAssistPresetsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraInstructionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraPresetsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraShakePacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CameraSplinePacket(_) => {
                <<V2168 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ChangeDimensionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ChangeMobPropertyPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ChunkRadiusUpdatedPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundAttributeLayerSyncPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundCloseFormPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundControlSchemeSetPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundDataDrivenUICloseScreenPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundDataDrivenUIReloadPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundDataDrivenUIShowScreenPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundDataStorePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundDebugRendererPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundMapItemDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundTextureShiftPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientBoundUpdateSoundDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientCacheBlobStatusPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientCacheMissResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientCacheStatusPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ClientToServerHandshakePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CodeBuilderPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CodeBuilderSourcePacket(_) => {
                <<V2168 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CommandBlockUpdatePacket(_) => {
                <<V2168 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CommandOutputPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CommandRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CompletedUsingItemPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ContainerClosePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ContainerOpenPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ContainerRegistryCleanupPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ContainerSetDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CorrectPlayerMovePredictionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CraftingDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CreatePhotoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CreativeContentPacket(_) => {
                <<V2168 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::CurrentStructureFeaturePacket(_) => {
                <<V2168 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::DeathInfoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::DebugDrawerPacket(_) => {
                <<V2168 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::DebugInfoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::DimensionDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::DisconnectPacket(_) => {
                <<V2168 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::EditorNetworkPacket(_) => {
                <<V2168 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::EduUriResourcePacket(_) => {
                <<V2168 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::EducationSettingsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::EmoteListPacket(_) => {
                <<V2168 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::EmotePacket(_) => {
                <<V2168 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::FeatureRegistryPacket(_) => {
                <<V2168 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::GameRulesChangedPacket(_) => {
                <<V2168 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::GameTestRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::GameTestResultsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::GraphicsParameterOverridePacket(_) => {
                <<V2168 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::GuiDataPickItemPacket(_) => {
                <<V2168 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::HurtArmorPacket(_) => {
                <<V2168 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::InteractPacket(_) => {
                <<V2168 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::InventoryContentPacket(_) => {
                <<V2168 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::InventorySlotPacket(_) => {
                <<V2168 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::InventoryTransactionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ItemComponentPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ItemStackRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ItemStackResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::JigsawStructureDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LabTablePacket(_) => {
                <<V2168 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LecternUpdatePacket(_) => {
                <<V2168 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LegacyTelemetryEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LessonProgressPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LevelChunkPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LevelEventGenericPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LevelEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LevelSoundEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LocatorBarPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::LoginPacket(_) => {
                <<V2168 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MapCreateLockedCopyPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MapInfoRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MobArmorEquipmentPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MobEffectPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MobEquipmentPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ModalFormRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ModalFormResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MotionPredictionHintsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MoveActorAbsolutePacket(_) => {
                <<V2168 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MoveActorDeltaPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MovePlayerPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MovementEffectPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MovementPredictionSyncPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::MultiplayerSettingsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::NetworkChunkPublisherUpdatePacket(_) => {
                <<V2168 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::NetworkSettingsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::NetworkStackLatencyPacket(_) => {
                <<V2168 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::NpcDialoguePacket(_) => {
                <<V2168 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::NpcRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::OnScreenTextureAnimationPacket(_) => {
                <<V2168 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::OpenSignPacket(_) => {
                <<V2168 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PacketViolationWarningPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PartyChangedPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PartyDestinationCookieResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PhotoTransferPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlaySoundPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayStatusPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerActionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerArmorDamagePacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerAuthInputPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerEnchantOptionsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerFogPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerHotbarPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerListPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerLocationPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerSkinPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerStartItemCooldownPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerToggleCrafterSlotRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerUpdateEntityOverridesPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PlayerVideoCapturePacket(_) => {
                <<V2168 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PositionTrackingDBClientRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PositionTrackingDBServerBroadcastPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::PurchaseReceiptPacket(_) => {
                <<V2168 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RefreshEntitlementsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RemoveActorPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RemoveObjectivePacket(_) => {
                <<V2168 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RemoveVolumeEntityPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RequestAbilityPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RequestChunkRadiusPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RequestNetworkSettingsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RequestPermissionsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePackChunkDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePackChunkRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePackClientResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePackDataInfoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePackStackPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePacksInfoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ResourcePacksReadyForValidationPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::RespawnPacket(_) => {
                <<V2168 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ScriptMessagePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SendPartyDestinationCookiePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerBoundDataDrivenClosedPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerBoundDataStorePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerBoundDiagnosticsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerBoundLoadingScreenPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerBoundPackSettingChangePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerPlayerPostMovePositionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerPresenceInfoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerSettingsRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerSettingsResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerStatsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerStoreInfoPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ServerToClientHandshakePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetActorDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetActorLinkPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetActorMotionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetCommandsEnabledPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetDefaultGameTypePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetDifficultyPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetDisplayObjectivePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetHealthPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetHudPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetLastHurtByPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetLocalPlayerAsInitializedPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetPlayerGameTypePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetPlayerInventoryOptionsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetScorePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetScoreboardIdentityPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetSpawnPositionPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetTimePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SetTitlePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SettingsCommandPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ShowCreditsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ShowProfilePacket(_) => {
                <<V2168 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ShowStoreOfferPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SimpleEventPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SimulationTypePacket(_) => {
                <<V2168 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SpawnExperienceOrbPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SpawnParticleEffectPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::StartGamePacket(_) => {
                <<V2168 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::StopSoundPacket(_) => {
                <<V2168 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::StructureBlockUpdatePacket(_) => {
                <<V2168 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::StructureDataRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::StructureDataResponsePacket(_) => {
                <<V2168 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SubChunkPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SubChunkRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SubClientLoginPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SyncActorPropertyPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::SyncWorldClocksPacket(_) => {
                <<V2168 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::TakeItemActorPacket(_) => {
                <<V2168 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::TextPacket(_) => {
                <<V2168 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::TickingAreaLoadStatusPacket(_) => {
                <<V2168 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::ToastRequestPacket(_) => {
                <<V2168 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::TransferPlayerPacket(_) => {
                <<V2168 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::TrimDataPacket(_) => {
                <<V2168 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UnlockedRecipesPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateAbilitiesPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateAdventureSettingsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateAttributesPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateBlockPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateBlockSyncedPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateClientInputLocksPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateClientOptionsPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateEquipPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdatePlayerGameTypePacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateSoftEnumPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateSubChunkBlocksPacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::UpdateTradePacket(_) => {
                <<V2168 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::VoxelShapesPacket(_) => {
                <<V2168 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID
            }
            V2168::Unknown(pk) => pk.id,
        }
    }
}
impl ProtoVersionPackets for V2168 {
    type ActorEventPacket = bedrock_protocol_v975::packets::ActorEventPacket<Self>;
    type ActorPickRequestPacket = bedrock_protocol_v662::packets::ActorPickRequestPacket;
    type AddActorPacket = bedrock_protocol_v662::packets::AddActorPacket<Self>;
    type AddBehaviourTreePacket = bedrock_protocol_v662::packets::AddBehaviourTreePacket;
    type AddItemActorPacket = bedrock_protocol_v662::packets::AddItemActorPacket<Self>;
    type AddPaintingPacket = bedrock_protocol_v662::packets::AddPaintingPacket<Self>;
    type AddPlayerPacket = bedrock_protocol_v662::packets::AddPlayerPacket<Self>;
    type AddVolumeEntityPacket = bedrock_protocol_v662::packets::AddVolumeEntityPacket<Self>;
    type AgentActionEventPacket = bedrock_protocol_v662::packets::AgentActionEventPacket<Self>;
    type AgentAnimationPacket = bedrock_protocol_v662::packets::AgentAnimationPacket<Self>;
    type AnimateEntityPacket = bedrock_protocol_v662::packets::AnimateEntityPacket<Self>;
    type AnimatePacket = bedrock_protocol_v898::packets::AnimatePacket<Self>;
    type AnvilDamagePacket = crate::packets::AnvilDamagePacket<Self>;
    type AutomationClientConnectPacket =
        bedrock_protocol_v662::packets::AutomationClientConnectPacket<Self>;
    type AvailableActorIdentifiersPacket =
        bedrock_protocol_v662::packets::AvailableActorIdentifiersPacket;
    type AvailableCommandsPacket = bedrock_protocol_v898::packets::AvailableCommandsPacket;
    type AwardAchievementPacket = bedrock_protocol_v685::packets::AwardAchievementPacket;
    type BiomeDefinitionListPacket =
        bedrock_protocol_v800::packets::BiomeDefinitionListPacket<Self>;
    type BlockActorDataPacket = bedrock_protocol_v662::packets::BlockActorDataPacket<Self>;
    type BlockEventPacket = bedrock_protocol_v662::packets::BlockEventPacket<Self>;
    type BlockPickRequestPacket = bedrock_protocol_v662::packets::BlockPickRequestPacket<Self>;
    type BookEditPacket = bedrock_protocol_v924::packets::BookEditPacket<Self>;
    type BossEventPacket = bedrock_protocol_v1001::packets::BossEventPacket<Self>;
    type CameraAimAssistActorPriorityPacket =
        bedrock_protocol_v924::packets::CameraAimAssistActorPriorityPacket;
    type CameraAimAssistInstructionPacket =
        bedrock_protocol_v776::packets::CameraAimAssistInstructionPacket<Self>;
    type CameraAimAssistPacket = bedrock_protocol_v827::packets::CameraAimAssistPacket<Self>;
    type CameraAimAssistPresetsPacket =
        bedrock_protocol_v800::packets::CameraAimAssistPresetsPacket<Self>;
    type CameraInstructionPacket = bedrock_protocol_v924::packets::CameraInstructionPacket<Self>;
    type CameraPacket = bedrock_protocol_v662::packets::CameraPacket<Self>;
    type CameraPresetsPacket = bedrock_protocol_v662::packets::CameraPresetsPacket<Self>;
    type CameraShakePacket = bedrock_protocol_v662::packets::CameraShakePacket<Self>;
    type CameraSplinePacket = bedrock_protocol_v924::packets::CameraSplinePacket<Self>;
    type ChangeDimensionPacket = bedrock_protocol_v712::packets::ChangeDimensionPacket;
    type ChangeMobPropertyPacket = bedrock_protocol_v662::packets::ChangeMobPropertyPacket<Self>;
    type ChunkRadiusUpdatedPacket = bedrock_protocol_v662::packets::ChunkRadiusUpdatedPacket;
    type ClientBoundAttributeLayerSyncPacket =
        bedrock_protocol_v1001::packets::ClientBoundAttributeLayerSyncPacket;
    type ClientBoundCloseFormPacket = bedrock_protocol_v686::packets::ClientBoundCloseFormPacket;
    type ClientBoundControlSchemeSetPacket =
        bedrock_protocol_v800::packets::ClientBoundControlSchemeSetPacket<Self>;
    type ClientBoundDataDrivenUICloseAllScreensPacket = ();
    type ClientBoundDataDrivenUICloseScreenPacket =
        bedrock_protocol_v944::packets::ClientBoundDataDrivenUICloseScreenPacket;
    type ClientBoundDataDrivenUIReloadPacket =
        bedrock_protocol_v924::packets::ClientBoundDataDrivenUIReloadPacket;
    type ClientBoundDataDrivenUIShowScreenPacket =
        bedrock_protocol_v944::packets::ClientBoundDataDrivenUIShowScreenPacket;
    type ClientBoundDataStorePacket = bedrock_protocol_v924::packets::ClientBoundDataStorePacket;
    type ClientBoundDebugRendererPacket =
        bedrock_protocol_v671::packets::ClientBoundDebugRendererPacket;
    type ClientBoundMapItemDataPacket = crate::packets::ClientBoundMapItemDataPacket<Self>;
    type ClientBoundMatchmakingStatePacket = ();
    type ClientBoundPlayAudioContentPacket = ();
    type ClientBoundStonecutterSetRecipePacket = ();
    type ClientBoundTextureShiftPacket =
        bedrock_protocol_v924::packets::ClientBoundTextureShiftPacket;
    type ClientBoundUpdateSoundDataPacket = crate::packets::ClientBoundUpdateSoundDataPacket<Self>;
    type ClientCacheBlobStatusPacket = bedrock_protocol_v1001::packets::ClientCacheBlobStatusPacket;
    type ClientCacheMissResponsePacket =
        bedrock_protocol_v662::packets::ClientCacheMissResponsePacket;
    type ClientCacheStatusPacket = bedrock_protocol_v662::packets::ClientCacheStatusPacket;
    type ClientToServerHandshakePacket =
        bedrock_protocol_v662::packets::ClientToServerHandshakePacket;
    type CodeBuilderPacket = bedrock_protocol_v662::packets::CodeBuilderPacket;
    type CodeBuilderSourcePacket = bedrock_protocol_v685::packets::CodeBuilderSourcePacket<Self>;
    type CommandBlockUpdatePacket = bedrock_protocol_v776::packets::CommandBlockUpdatePacket<Self>;
    type CommandOutputPacket = bedrock_protocol_v898::packets::CommandOutputPacket<Self>;
    type CommandRequestPacket = bedrock_protocol_v898::packets::CommandRequestPacket<Self>;
    type CompletedUsingItemPacket = bedrock_protocol_v662::packets::CompletedUsingItemPacket<Self>;
    type CompressedBiomeDefinitionListPacket = ();
    type ContainerClosePacket = bedrock_protocol_v685::packets::ContainerClosePacket<Self>;
    type ContainerOpenPacket = bedrock_protocol_v662::packets::ContainerOpenPacket<Self>;
    type ContainerRegistryCleanupPacket =
        bedrock_protocol_v729::packets::ContainerRegistryCleanupPacket<Self>;
    type ContainerSetDataPacket = bedrock_protocol_v662::packets::ContainerSetDataPacket<Self>;
    type CorrectPlayerMovePredictionPacket =
        bedrock_protocol_v827::packets::CorrectPlayerMovePredictionPacket<Self>;
    type CraftingDataPacket = crate::packets::CraftingDataPacket<Self>;
    type CreatePhotoPacket = bedrock_protocol_v662::packets::CreatePhotoPacket;
    type CreativeContentPacket = crate::packets::CreativeContentPacket<Self>;
    type CurrentStructureFeaturePacket =
        bedrock_protocol_v712::packets::CurrentStructureFeaturePacket;
    type DeathInfoPacket = bedrock_protocol_v662::packets::DeathInfoPacket;
    type DebugDrawerPacket = bedrock_protocol_v818::packets::DebugDrawerPacket<Self>;
    type DebugInfoPacket = bedrock_protocol_v662::packets::DebugInfoPacket<Self>;
    type DimensionDataPacket = bedrock_protocol_v662::packets::DimensionDataPacket<Self>;
    type DisconnectPacket = bedrock_protocol_v712::packets::DisconnectPacket<Self>;
    type EditorNetworkPacket = bedrock_protocol_v662::packets::EditorNetworkPacket;
    type EduUriResourcePacket = bedrock_protocol_v662::packets::EduUriResourcePacket<Self>;
    type EducationSettingsPacket = bedrock_protocol_v662::packets::EducationSettingsPacket<Self>;
    type EmoteListPacket = bedrock_protocol_v662::packets::EmoteListPacket<Self>;
    type EmotePacket = bedrock_protocol_v729::packets::EmotePacket<Self>;
    type FeatureRegistryPacket = bedrock_protocol_v662::packets::FeatureRegistryPacket;
    type FilterTextPacket = ();
    type GameRulesChangedPacket = bedrock_protocol_v662::packets::GameRulesChangedPacket<Self>;
    type GameTestRequestPacket = bedrock_protocol_v662::packets::GameTestRequestPacket<Self>;
    type GameTestResultsPacket = bedrock_protocol_v662::packets::GameTestResultsPacket;
    type GraphicsParameterOverridePacket =
        bedrock_protocol_v1001::packets::GraphicsParameterOverridePacket;
    type GuiDataPickItemPacket = bedrock_protocol_v662::packets::GuiDataPickItemPacket;
    type HurtArmorPacket = bedrock_protocol_v662::packets::HurtArmorPacket;
    type InteractPacket = bedrock_protocol_v898::packets::InteractPacket<Self>;
    type InventoryContentPacket = bedrock_protocol_v1001::packets::InventoryContentPacket<Self>;
    type InventorySlotPacket = bedrock_protocol_v975::packets::InventorySlotPacket<Self>;
    type InventoryTransactionPacket =
        bedrock_protocol_v1001::packets::InventoryTransactionPacket<Self>;
    type ItemComponentPacket = bedrock_protocol_v776::packets::ItemComponentPacket<Self>;
    type ItemStackRequestPacket = bedrock_protocol_v662::packets::ItemStackRequestPacket<Self>;
    type ItemStackResponsePacket = bedrock_protocol_v662::packets::ItemStackResponsePacket<Self>;
    type JigsawStructureDataPacket = bedrock_protocol_v712::packets::JigsawStructureDataPacket;
    type LabTablePacket = bedrock_protocol_v662::packets::LabTablePacket<Self>;
    type LecternUpdatePacket = bedrock_protocol_v662::packets::LecternUpdatePacket<Self>;
    type LegacyTelemetryEventPacket =
        bedrock_protocol_v898::packets::LegacyTelemetryEventPacket<Self>;
    type LessonProgressPacket = bedrock_protocol_v662::packets::LessonProgressPacket<Self>;
    type LevelChunkPacket = crate::packets::LevelChunkPacket<Self>;
    type LevelEventGenericPacket = bedrock_protocol_v662::packets::LevelEventGenericPacket<Self>;
    type LevelEventPacket = bedrock_protocol_v662::packets::LevelEventPacket;
    type LevelSoundEventPacket = bedrock_protocol_v1001::packets::LevelSoundEventPacket;
    type LevelSoundEventV1Packet = ();
    type LevelSoundEventV2Packet = ();
    type LocatorBarPacket = bedrock_protocol_v975::packets::LocatorBarPacket;
    type LoginPacket = bedrock_protocol_v662::packets::LoginPacket;
    type MapCreateLockedCopyPacket =
        bedrock_protocol_v662::packets::MapCreateLockedCopyPacket<Self>;
    type MapInfoRequestPacket = bedrock_protocol_v662::packets::MapInfoRequestPacket<Self>;
    type MobArmorEquipmentPacket = bedrock_protocol_v1001::packets::MobArmorEquipmentPacket<Self>;
    type MobEffectPacket = bedrock_protocol_v898::packets::MobEffectPacket<Self>;
    type MobEquipmentPacket = bedrock_protocol_v975::packets::MobEquipmentPacket<Self>;
    type ModalFormRequestPacket = bedrock_protocol_v662::packets::ModalFormRequestPacket;
    type ModalFormResponsePacket = bedrock_protocol_v662::packets::ModalFormResponsePacket<Self>;
    type MotionPredictionHintsPacket =
        bedrock_protocol_v662::packets::MotionPredictionHintsPacket<Self>;
    type MoveActorAbsolutePacket = bedrock_protocol_v662::packets::MoveActorAbsolutePacket<Self>;
    type MoveActorDeltaPacket = bedrock_protocol_v662::packets::MoveActorDeltaPacket<Self>;
    type MovePlayerPacket = crate::packets::MovePlayerPacket<Self>;
    type MovementEffectPacket = bedrock_protocol_v748::packets::MovementEffectPacket<Self>;
    type MovementPredictionSyncPacket =
        bedrock_protocol_v975::packets::MovementPredictionSyncPacket<Self>;
    type MultiplayerSettingsPacket =
        bedrock_protocol_v662::packets::MultiplayerSettingsPacket<Self>;
    type NetworkChunkPublisherUpdatePacket =
        bedrock_protocol_v662::packets::NetworkChunkPublisherUpdatePacket<Self>;
    type NetworkSettingsPacket = bedrock_protocol_v662::packets::NetworkSettingsPacket<Self>;
    type NetworkStackLatencyPacket = bedrock_protocol_v662::packets::NetworkStackLatencyPacket;
    type NpcDialoguePacket = bedrock_protocol_v662::packets::NpcDialoguePacket;
    type NpcRequestPacket = bedrock_protocol_v662::packets::NpcRequestPacket<Self>;
    type OnScreenTextureAnimationPacket =
        bedrock_protocol_v662::packets::OnScreenTextureAnimationPacket;
    type OpenSignPacket = bedrock_protocol_v662::packets::OpenSignPacket<Self>;
    type PacketViolationWarningPacket =
        bedrock_protocol_v662::packets::PacketViolationWarningPacket<Self>;
    type PartyChangedPacket = bedrock_protocol_v975::packets::PartyChangedPacket;
    type PartyDestinationCookieResponsePacket =
        bedrock_protocol_v1001::packets::PartyDestinationCookieResponsePacket;
    type PassengerJumpPacket = ();
    type PhotoTransferPacket = bedrock_protocol_v662::packets::PhotoTransferPacket<Self>;
    type PlaySoundPacket = crate::packets::PlaySoundPacket<Self>;
    type PlayStatusPacket = bedrock_protocol_v662::packets::PlayStatusPacket<Self>;
    type PlayerActionPacket = bedrock_protocol_v662::packets::PlayerActionPacket<Self>;
    type PlayerArmorDamagePacket = bedrock_protocol_v844::packets::PlayerArmorDamagePacket;
    type PlayerAuthInputPacket = crate::packets::PlayerAuthInputPacket<Self>;
    type PlayerEnchantOptionsPacket =
        bedrock_protocol_v662::packets::PlayerEnchantOptionsPacket<Self>;
    type PlayerFogPacket = bedrock_protocol_v662::packets::PlayerFogPacket;
    type PlayerHotbarPacket = bedrock_protocol_v662::packets::PlayerHotbarPacket<Self>;
    type PlayerInputPacket = ();
    type PlayerListPacket = crate::packets::PlayerListPacket<Self>;
    type PlayerLocationPacket = crate::packets::PlayerLocationPacket;
    type PlayerSkinPacket = crate::packets::PlayerSkinPacket<Self>;
    type PlayerStartItemCooldownPacket =
        bedrock_protocol_v662::packets::PlayerStartItemCooldownPacket;
    type PlayerToggleCrafterSlotRequestPacket =
        bedrock_protocol_v662::packets::PlayerToggleCrafterSlotRequestPacket;
    type PlayerUpdateEntityOverridesPacket =
        crate::packets::PlayerUpdateEntityOverridesPacket<Self>;
    type PlayerVideoCapturePacket = bedrock_protocol_v786::packets::PlayerVideoCapturePacket;
    type PositionTrackingDBClientRequestPacket =
        bedrock_protocol_v662::packets::PositionTrackingDBClientRequestPacket<Self>;
    type PositionTrackingDBServerBroadcastPacket =
        bedrock_protocol_v662::packets::PositionTrackingDBServerBroadcastPacket<Self>;
    type PurchaseReceiptPacket = bedrock_protocol_v662::packets::PurchaseReceiptPacket;
    type RecordStartedPacket = ();
    type RefreshEntitlementsPacket = bedrock_protocol_v662::packets::RefreshEntitlementsPacket;
    type RemoveActorPacket = bedrock_protocol_v662::packets::RemoveActorPacket<Self>;
    type RemoveObjectivePacket = bedrock_protocol_v662::packets::RemoveObjectivePacket;
    type RemoveVolumeEntityPacket = bedrock_protocol_v662::packets::RemoveVolumeEntityPacket<Self>;
    type RequestAbilityPacket = bedrock_protocol_v662::packets::RequestAbilityPacket<Self>;
    type RequestChunkRadiusPacket = bedrock_protocol_v662::packets::RequestChunkRadiusPacket;
    type RequestNetworkSettingsPacket =
        bedrock_protocol_v662::packets::RequestNetworkSettingsPacket;
    type RequestPermissionsPacket = bedrock_protocol_v662::packets::RequestPermissionsPacket<Self>;
    type ResourcePackChunkDataPacket = bedrock_protocol_v662::packets::ResourcePackChunkDataPacket;
    type ResourcePackChunkRequestPacket =
        bedrock_protocol_v662::packets::ResourcePackChunkRequestPacket;
    type ResourcePackClientResponsePacket = crate::packets::ResourcePackClientResponsePacket;
    type ResourcePackDataInfoPacket =
        bedrock_protocol_v662::packets::ResourcePackDataInfoPacket<Self>;
    type ResourcePackStackPacket = bedrock_protocol_v898::packets::ResourcePackStackPacket<Self>;
    type ResourcePacksInfoPacket = crate::packets::ResourcePacksInfoPacket;
    type ResourcePacksReadyForValidationPacket =
        bedrock_protocol_v944::packets::ResourcePacksReadyForValidationPacket;
    type RespawnPacket = bedrock_protocol_v662::packets::RespawnPacket<Self>;
    type ScriptMessagePacket = bedrock_protocol_v662::packets::ScriptMessagePacket;
    type SendPartyDestinationCookiePacket =
        bedrock_protocol_v1001::packets::SendPartyDestinationCookiePacket;
    type ServerBoundCursorItemDragPacket = ();
    type ServerBoundDataDrivenClosedPacket =
        bedrock_protocol_v944::packets::ServerBoundDataDrivenClosedPacket;
    type ServerBoundDataStorePacket = bedrock_protocol_v924::packets::ServerBoundDataStorePacket;
    type ServerBoundDiagnosticsPacket = crate::packets::ServerBoundDiagnosticsPacket;
    type ServerBoundLoadingScreenPacket =
        bedrock_protocol_v712::packets::ServerBoundLoadingScreenPacket;
    type ServerBoundMatchmakingCancelPacket = ();
    type ServerBoundPackSettingChangePacket =
        bedrock_protocol_v844::packets::ServerBoundPackSettingChangePacket;
    type ServerBoundRegisterAudioContentPacket = ();
    type ServerBoundStonecutterSetRecipePacket = ();
    type ServerPlayerPostMovePositionPacket =
        bedrock_protocol_v662::packets::ServerPlayerPostMovePositionPacket;
    type ServerPresenceInfoPacket = crate::packets::ServerPresenceInfoPacket;
    type ServerSettingsRequestPacket = bedrock_protocol_v662::packets::ServerSettingsRequestPacket;
    type ServerSettingsResponsePacket =
        bedrock_protocol_v662::packets::ServerSettingsResponsePacket;
    type ServerStatsPacket = bedrock_protocol_v662::packets::ServerStatsPacket;
    type ServerStoreInfoPacket = bedrock_protocol_v975::packets::ServerStoreInfoPacket;
    type ServerToClientHandshakePacket =
        bedrock_protocol_v662::packets::ServerToClientHandshakePacket;
    type SetActorDataPacket = bedrock_protocol_v662::packets::SetActorDataPacket<Self>;
    type SetActorLinkPacket = bedrock_protocol_v662::packets::SetActorLinkPacket<Self>;
    type SetActorMotionPacket = bedrock_protocol_v662::packets::SetActorMotionPacket<Self>;
    type SetCommandsEnabledPacket = bedrock_protocol_v662::packets::SetCommandsEnabledPacket;
    type SetDefaultGameTypePacket = bedrock_protocol_v662::packets::SetDefaultGameTypePacket<Self>;
    type SetDifficultyPacket = bedrock_protocol_v662::packets::SetDifficultyPacket;
    type SetDisplayObjectivePacket =
        bedrock_protocol_v662::packets::SetDisplayObjectivePacket<Self>;
    type SetHealthPacket = bedrock_protocol_v662::packets::SetHealthPacket;
    type SetHudPacket = bedrock_protocol_v662::packets::SetHudPacket<Self>;
    type SetLastHurtByPacket = bedrock_protocol_v662::packets::SetLastHurtByPacket<Self>;
    type SetLocalPlayerAsInitializedPacket =
        bedrock_protocol_v662::packets::SetLocalPlayerAsInitializedPacket<Self>;
    type SetMovementAuthorityPacket = ();
    type SetPassengerOfBlockPacket = ();
    type SetPlayerFurnaceOptionsPacket = ();
    type SetPlayerGameTypePacket = bedrock_protocol_v662::packets::SetPlayerGameTypePacket<Self>;
    type SetPlayerInventoryOptionsPacket =
        bedrock_protocol_v662::packets::SetPlayerInventoryOptionsPacket<Self>;
    type SetScorePacket = crate::packets::SetScorePacket<Self>;
    type SetScoreboardIdentityPacket = crate::packets::SetScoreboardIdentityPacket<Self>;
    type SetSpawnPositionPacket = bedrock_protocol_v662::packets::SetSpawnPositionPacket<Self>;
    type SetTimePacket = bedrock_protocol_v662::packets::SetTimePacket;
    type SetTitlePacket = bedrock_protocol_v712::packets::SetTitlePacket;
    type SettingsCommandPacket = bedrock_protocol_v662::packets::SettingsCommandPacket;
    type ShowCreditsPacket = bedrock_protocol_v662::packets::ShowCreditsPacket<Self>;
    type ShowProfilePacket = bedrock_protocol_v662::packets::ShowProfilePacket;
    type ShowStoreOfferPacket = bedrock_protocol_v859::packets::ShowStoreOfferPacket<Self>;
    type SimpleEventPacket = bedrock_protocol_v662::packets::SimpleEventPacket;
    type SimulationTypePacket = bedrock_protocol_v662::packets::SimulationTypePacket<Self>;
    type SpawnExperienceOrbPacket = bedrock_protocol_v662::packets::SpawnExperienceOrbPacket;
    type SpawnParticleEffectPacket =
        bedrock_protocol_v662::packets::SpawnParticleEffectPacket<Self>;
    type StartGamePacket = crate::packets::StartGamePacket<Self>;
    type StopSoundPacket = bedrock_protocol_v712::packets::StopSoundPacket;
    type StructureBlockUpdatePacket =
        bedrock_protocol_v662::packets::StructureBlockUpdatePacket<Self>;
    type StructureDataRequestPacket =
        bedrock_protocol_v662::packets::StructureDataRequestPacket<Self>;
    type StructureDataResponsePacket =
        bedrock_protocol_v662::packets::StructureDataResponsePacket<Self>;
    type SubChunkPacket = crate::packets::SubChunkPacket<Self>;
    type SubChunkRequestPacket = bedrock_protocol_v1001::packets::SubChunkRequestPacket<Self>;
    type SubClientLoginPacket = bedrock_protocol_v662::packets::SubClientLoginPacket;
    type SyncActorPropertyPacket = bedrock_protocol_v662::packets::SyncActorPropertyPacket;
    type SyncWorldClocksPacket = bedrock_protocol_v944::packets::SyncWorldClocksPacket;
    type TakeItemActorPacket = bedrock_protocol_v662::packets::TakeItemActorPacket<Self>;
    type TextPacket = bedrock_protocol_v898::packets::TextPacket<Self>;
    type TickSyncPacket = ();
    type TickingAreaLoadStatusPacket = bedrock_protocol_v662::packets::TickingAreaLoadStatusPacket;
    type ToastRequestPacket = bedrock_protocol_v662::packets::ToastRequestPacket;
    type TransferPlayerPacket = crate::packets::TransferPlayerPacket<Self>;
    type TrimDataPacket = bedrock_protocol_v662::packets::TrimDataPacket;
    type UnlockedRecipesPacket = bedrock_protocol_v662::packets::UnlockedRecipesPacket;
    type UpdateAbilitiesPacket = bedrock_protocol_v662::packets::UpdateAbilitiesPacket<Self>;
    type UpdateAdventureSettingsPacket =
        bedrock_protocol_v662::packets::UpdateAdventureSettingsPacket<Self>;
    type UpdateAttributesPacket = bedrock_protocol_v729::packets::UpdateAttributesPacket<Self>;
    type UpdateBlockPacket = bedrock_protocol_v662::packets::UpdateBlockPacket<Self>;
    type UpdateBlockSyncedPacket = bedrock_protocol_v662::packets::UpdateBlockSyncedPacket<Self>;
    type UpdateClientInputLocksPacket =
        bedrock_protocol_v944::packets::UpdateClientInputLocksPacket;
    type UpdateClientOptionsPacket = bedrock_protocol_v975::packets::UpdateClientOptionsPacket;
    type UpdateEquipPacket = bedrock_protocol_v662::packets::UpdateEquipPacket<Self>;
    type UpdatePlayerGameTypePacket =
        bedrock_protocol_v671::packets::UpdatePlayerGameTypePacket<Self>;
    type UpdateSoftEnumPacket = bedrock_protocol_v662::packets::UpdateSoftEnumPacket<Self>;
    type UpdateSubChunkBlocksPacket =
        bedrock_protocol_v662::packets::UpdateSubChunkBlocksPacket<Self>;
    type UpdateTradePacket = bedrock_protocol_v662::packets::UpdateTradePacket<Self>;
    type VoxelShapesPacket = bedrock_protocol_v944::packets::VoxelShapesPacket;
}
impl ProtoVersionTypes for V2168 {
    type ActorLink = bedrock_protocol_v712::types::ActorLink<Self>;
    type ActorRuntimeID = bedrock_protocol_v662::types::ActorRuntimeID;
    type ActorUniqueID = bedrock_protocol_v662::types::ActorUniqueID;
    type AdventureSettings = bedrock_protocol_v662::types::AdventureSettings;
    type BaseDescription = bedrock_protocol_v662::types::BaseDescription<Self>;
    type BaseGameVersion = bedrock_protocol_v662::types::BaseGameVersion;
    type BiomeCappedSurfaceData = bedrock_protocol_v800::types::BiomeCappedSurfaceData;
    type BiomeClimateData = bedrock_protocol_v844::types::BiomeClimateData;
    type BiomeConditionalTransformationData =
        bedrock_protocol_v800::types::BiomeConditionalTransformationData<Self>;
    type BiomeConsolidatedFeatureList =
        bedrock_protocol_v800::types::BiomeConsolidatedFeatureList<Self>;
    type BiomeCoordinateData = bedrock_protocol_v800::types::BiomeCoordinateData;
    type BiomeDefinition = bedrock_protocol_v844::types::BiomeDefinition<Self>;
    type BiomeDefinitionChunkGenData =
        bedrock_protocol_v975::types::BiomeDefinitionChunkGenData<Self>;
    type BiomeElementData = bedrock_protocol_v800::types::BiomeElementData<Self>;
    type BiomeLegacyWorldGenRulesData =
        bedrock_protocol_v800::types::BiomeLegacyWorldGenRulesData<Self>;
    type BiomeMesaSurfaceData = bedrock_protocol_v800::types::BiomeMesaSurfaceData;
    type BiomeMountainParamsData = bedrock_protocol_v800::types::BiomeMountainParamsData;
    type BiomeMultinoiseGenRulesData = bedrock_protocol_v800::types::BiomeMultinoiseGenRulesData;
    type BiomeNoiseGradientSurfaceData =
        bedrock_protocol_v1001::types::BiomeNoiseGradientSurfaceData;
    type BiomeOverworldGenRulesData =
        bedrock_protocol_v800::types::BiomeOverworldGenRulesData<Self>;
    type BiomeReplacementData = bedrock_protocol_v859::types::BiomeReplacementData;
    type BiomeScatterParamData = bedrock_protocol_v800::types::BiomeScatterParamData<Self>;
    type BiomeSurfaceBuilderData = bedrock_protocol_v975::types::BiomeSurfaceBuilderData<Self>;
    type BiomeSurfaceMaterialAdjustmentData =
        bedrock_protocol_v800::types::BiomeSurfaceMaterialAdjustmentData<Self>;
    type BiomeSurfaceMaterialData = bedrock_protocol_v800::types::BiomeSurfaceMaterialData;
    type BiomeWeightedData = bedrock_protocol_v800::types::BiomeWeightedData;
    type BiomeWeightedTemperatureData = bedrock_protocol_v800::types::BiomeWeightedTemperatureData;
    type BlockPos = bedrock_protocol_v662::types::BlockPos;
    type CameraAimAssistCategories = bedrock_protocol_v766::types::CameraAimAssistCategories<Self>;
    type CameraAimAssistCategory = bedrock_protocol_v924::types::CameraAimAssistCategory<Self>;
    type CameraAimAssistItemSettings = bedrock_protocol_v766::types::CameraAimAssistItemSettings;
    type CameraAimAssistPreset = bedrock_protocol_v766::types::CameraAimAssistPreset;
    type CameraAimAssistPresetDefinition =
        bedrock_protocol_v924::types::CameraAimAssistPresetDefinition<Self>;
    type CameraAimAssistPriority = bedrock_protocol_v766::types::CameraAimAssistPriority;
    type CameraPreset = bedrock_protocol_v818::types::CameraPreset<Self>;
    type CameraPresets = bedrock_protocol_v662::types::CameraPresets<Self>;
    type CameraSplineInstruction = bedrock_protocol_v944::types::CameraSplineInstruction<Self>;
    type ChunkPos = bedrock_protocol_v662::types::ChunkPos;
    type Color = bedrock_protocol_v800::types::Color;
    type CommandOriginData = bedrock_protocol_v898::types::CommandOriginData<Self>;
    type ContainerMixDataEntry = bedrock_protocol_v662::types::ContainerMixDataEntry;
    type CraftingDataEntry = bedrock_protocol_v662::types::CraftingDataEntry<Self>;
    type CraftingRecipeIngredient = crate::types::CraftingRecipeIngredient;
    type DataItem = bedrock_protocol_v662::types::DataItem<Self>;
    type DebugShape = bedrock_protocol_v1001::types::DebugShape<Self>;
    type DimensionDefinitionGroup = crate::types::DimensionDefinitionGroup;
    type EduSharedUriResource = bedrock_protocol_v662::types::EduSharedUriResource;
    type EducationLevelSettings = bedrock_protocol_v662::types::EducationLevelSettings;
    type EntityNetID = bedrock_protocol_v662::types::EntityNetID;
    type Experiments = bedrock_protocol_v662::types::Experiments;
    type FullContainerName = bedrock_protocol_v729::types::FullContainerName<Self>;
    type GameRulesChangedPacketData = bedrock_protocol_v844::types::GameRulesChangedPacketData;
    type GatheringsConfig = crate::types::GatheringsConfig;
    type InventoryAction = bedrock_protocol_v1001::types::InventoryAction<Self>;
    type InventorySource = bedrock_protocol_v1001::types::InventorySource;
    type InventoryTransaction = bedrock_protocol_v662::types::InventoryTransaction<Self>;
    type ItemData = bedrock_protocol_v662::types::ItemData;
    type ItemEnchants = bedrock_protocol_v662::types::ItemEnchants<Self>;
    type ItemReleaseTransactionData =
        bedrock_protocol_v1001::types::ItemReleaseTransactionData<Self>;
    type ItemStackRequestNetworkItemInstanceDescriptor =
        crate::types::ItemStackRequestNetworkItemInstanceDescriptor<Self>;
    type ItemStackRequestSlotInfo = crate::types::ItemStackRequestSlotInfo<Self>;
    type ItemStackResponseContainerInfo =
        bedrock_protocol_v712::types::ItemStackResponseContainerInfo<Self>;
    type ItemStackResponseInfo = crate::types::ItemStackResponseInfo<Self>;
    type ItemStackResponseSlotInfo = crate::types::ItemStackResponseSlotInfo<Self>;
    type ItemUseOnEntityTransactionData =
        bedrock_protocol_v1001::types::ItemUseOnEntityTransactionData<Self>;
    type ItemUseTransactionData = bedrock_protocol_v1001::types::ItemUseTransactionData<Self>;
    type LevelSettings = crate::types::LevelSettings<Self>;
    type MapDecoration = crate::types::MapDecoration;
    type MapItemTrackedActorUniqueID = crate::types::MapItemTrackedActorUniqueID<Self>;
    type MaterialReducerDataEntry = bedrock_protocol_v662::types::MaterialReducerDataEntry;
    type MolangVariableMap = bedrock_protocol_v662::types::MolangVariableMap;
    type MoveActorAbsoluteData = bedrock_protocol_v662::types::MoveActorAbsoluteData<Self>;
    type MoveActorDeltaData = crate::types::MoveActorDeltaData<Self>;
    type MovePlayerTeleportData = crate::types::MovePlayerTeleportData;
    type MultiRecipe = crate::types::MultiRecipe;
    type NetworkBlockPosition = bedrock_protocol_v944::types::NetworkBlockPosition;
    type NetworkItemInstanceDescriptor = crate::types::NetworkItemInstanceDescriptor;
    type NetworkItemStackDescriptor = crate::types::NetworkItemStackDescriptor;
    type NetworkItemStackDescriptorV2 = crate::types::NetworkItemStackDescriptorV2;
    type NetworkPermissions = bedrock_protocol_v662::types::NetworkPermissions;
    type PackedItemUseLegacyInventoryTransaction =
        crate::types::PackedItemUseLegacyInventoryTransaction<Self>;
    type PassengerOfBlockArguments = ();
    type PlayerBlockActionData = crate::types::PlayerBlockActionData<Self>;
    type PositionTrackingId = bedrock_protocol_v662::types::PositionTrackingId;
    type PotionMixDataEntry = bedrock_protocol_v662::types::PotionMixDataEntry;
    type PropertySyncData = bedrock_protocol_v662::types::PropertySyncData;
    type RecipeIngredient = crate::types::RecipeIngredient<Self>;
    type RecipeUnlockingRequirement = crate::types::RecipeUnlockingRequirement<Self>;
    type RedactableString = crate::types::RedactableString;
    type ScoreboardId = bedrock_protocol_v662::types::ScoreboardId;
    type SerializedAbilitiesData = bedrock_protocol_v776::types::SerializedAbilitiesData<Self>;
    type SerializedSkin = crate::types::SerializedSkin<Self>;
    type ShapedRecipe = crate::types::ShapedRecipe<Self>;
    type ShapelessRecipe = crate::types::ShapelessRecipe<Self>;
    type ShulkerBoxRecipe = bedrock_protocol_v748::types::ShulkerBoxRecipe<Self>;
    type SignedAudioContent = ();
    type SmithingTransformRecipe = crate::types::SmithingTransformRecipe<Self>;
    type SmithingTrimRecipe = crate::types::SmithingTrimRecipe<Self>;
    type SoundData = crate::types::SoundData;
    type SpawnSettings = bedrock_protocol_v662::types::SpawnSettings<Self>;
    type StructureEditorData = crate::types::StructureEditorData<Self>;
    type StructureSettings = bedrock_protocol_v662::types::StructureSettings<Self>;
    type SubChunkPos = crate::types::SubChunkPos;
    type SubChunkPosOffset = bedrock_protocol_v662::types::SubChunkPosOffset;
    type SyncedPlayerMovementSettings = bedrock_protocol_v818::types::SyncedPlayerMovementSettings;
    type WebSocketPacketData = bedrock_protocol_v662::types::WebSocketPacketData;
}
impl ProtoVersionEnums for V2168 {
    type AbilitiesIndex = bedrock_protocol_v776::enums::AbilitiesIndex;
    type ActorBlockSyncMessageID = bedrock_protocol_v662::enums::ActorBlockSyncMessageID;
    type ActorDamageCause = bedrock_protocol_v662::enums::ActorDamageCause;
    type ActorDataIDs = bedrock_protocol_v924::enums::ActorDataIDs;
    type ActorEvent = bedrock_protocol_v975::enums::ActorEvent;
    type ActorFlags = crate::enums::ActorFlags;
    type ActorLinkType = bedrock_protocol_v662::enums::ActorLinkType;
    type ActorType = bedrock_protocol_v662::enums::ActorType;
    type AgentActionType = bedrock_protocol_v662::enums::AgentActionType;
    type AimAssistAction = bedrock_protocol_v729::enums::AimAssistAction;
    type AnimatedTextureType = crate::enums::AnimatedTextureType;
    type AnimationExpression = crate::enums::AnimationExpression;
    type AnimationMode = bedrock_protocol_v662::enums::AnimationMode;
    type ArmSizeType = crate::enums::ArmSizeType;
    type AttributeModifierOperation = bedrock_protocol_v662::enums::AttributeModifierOperation;
    type AttributeOperands = bedrock_protocol_v662::enums::AttributeOperands;
    type AuthoritativeMovementMode = bedrock_protocol_v748::enums::AuthoritativeMovementMode;
    type BookEditAction = bedrock_protocol_v924::enums::BookEditAction;
    type BossEventUpdateType = bedrock_protocol_v776::enums::BossEventUpdateType<Self>;
    type BuildPlatform = bedrock_protocol_v662::enums::BuildPlatform;
    type CameraAimAssistOperation = bedrock_protocol_v776::enums::CameraAimAssistOperation;
    type CameraShakeAction = bedrock_protocol_v662::enums::CameraShakeAction;
    type CameraShakeType = bedrock_protocol_v662::enums::CameraShakeType;
    type CameraSplineEaseType = bedrock_protocol_v924::enums::CameraSplineEaseType;
    type CameraSplineType = bedrock_protocol_v859::enums::CameraSplineType;
    type ChatRestrictionLevel = bedrock_protocol_v662::enums::ChatRestrictionLevel;
    type CodeBuilderCodeStatus = bedrock_protocol_v685::enums::CodeBuilderCodeStatus;
    type CodeBuilderStorageCategory = bedrock_protocol_v662::enums::CodeBuilderStorageCategory;
    type CodeBuilderStorageOperation = bedrock_protocol_v662::enums::CodeBuilderStorageOperation;
    type CommandBlockMode = bedrock_protocol_v662::enums::CommandBlockMode;
    type CommandOriginType = bedrock_protocol_v898::enums::CommandOriginType;
    type CommandOutputType = bedrock_protocol_v898::enums::CommandOutputType;
    type CommandParameterOption = bedrock_protocol_v662::enums::CommandParameterOption;
    type CommandPermissionLevel = bedrock_protocol_v662::enums::CommandPermissionLevel;
    type ComplexInventoryTransactionType =
        bedrock_protocol_v662::enums::ComplexInventoryTransactionType;
    type ConnectionFailReason = bedrock_protocol_v1001::enums::ConnectionFailReason;
    type ContainerEnumName = bedrock_protocol_v944::enums::ContainerEnumName;
    type ContainerID = bedrock_protocol_v662::enums::ContainerID;
    type ContainerType = bedrock_protocol_v662::enums::ContainerType;
    type ControlScheme = bedrock_protocol_v800::enums::ControlScheme;
    type CraftingDataEntryType = bedrock_protocol_v662::enums::CraftingDataEntryType<Self>;
    type DataItemType = crate::enums::DataItemType<Self>;
    type Difficulty = bedrock_protocol_v662::enums::Difficulty;
    type EasingType = bedrock_protocol_v662::enums::EasingType;
    type EditorWorldType = bedrock_protocol_v662::enums::EditorWorldType;
    type EducationEditionOffer = crate::enums::EducationEditionOffer;
    type EnchantType = bedrock_protocol_v662::enums::EnchantType;
    type GamePublishSetting = bedrock_protocol_v662::enums::GamePublishSetting;
    type GameType = bedrock_protocol_v662::enums::GameType;
    type GeneratorType = bedrock_protocol_v662::enums::GeneratorType;
    type HudElement = bedrock_protocol_v786::enums::HudElement;
    type HudVisibility = bedrock_protocol_v786::enums::HudVisibility;
    type IdentityDefinitionType = bedrock_protocol_v662::enums::IdentityDefinitionType<Self>;
    type InputMode = bedrock_protocol_v662::enums::InputMode;
    type InteractionType = bedrock_protocol_v662::enums::InteractionType;
    type InventoryLayout = bedrock_protocol_v662::enums::InventoryLayout;
    type InventoryLeftTabIndex = bedrock_protocol_v662::enums::InventoryLeftTabIndex;
    type InventoryRightTabIndex = bedrock_protocol_v662::enums::InventoryRightTabIndex;
    type InventorySourceFlags = bedrock_protocol_v662::enums::InventorySourceFlags;
    type InventorySourceType = bedrock_protocol_v662::enums::InventorySourceType;
    type ItemDescriptorType = crate::enums::ItemDescriptorType;
    type ItemReleaseInventoryTransactionType =
        bedrock_protocol_v662::enums::ItemReleaseInventoryTransactionType;
    type ItemStackNetResult = crate::enums::ItemStackNetResult;
    type ItemStackRequestActionType = crate::enums::ItemStackRequestActionType<Self>;
    type ItemUseInventoryTransactionType =
        bedrock_protocol_v662::enums::ItemUseInventoryTransactionType;
    type ItemUseMethod = bedrock_protocol_v662::enums::ItemUseMethod;
    type ItemUseOnActorInventoryTransactionType =
        bedrock_protocol_v662::enums::ItemUseOnActorInventoryTransactionType;
    type ItemVersion = bedrock_protocol_v776::enums::ItemVersion;
    type LabTableReactionType = bedrock_protocol_v662::enums::LabTableReactionType;
    type LessonAction = bedrock_protocol_v662::enums::LessonAction;
    type LevelEvent = bedrock_protocol_v766::enums::LevelEvent;
    type LevelSoundEventType = crate::enums::LevelSoundEventType;
    type MinecraftPacketIds = bedrock_protocol_v662::enums::MinecraftPacketIds;
    type Mirror = bedrock_protocol_v662::enums::Mirror;
    type ModalFormCancelReason = bedrock_protocol_v662::enums::ModalFormCancelReason;
    type MolangVersion = bedrock_protocol_v662::enums::MolangVersion;
    type MovementEffectType = bedrock_protocol_v1001::enums::MovementEffectType;
    type MultiplayerSettingsPacketType =
        bedrock_protocol_v662::enums::MultiplayerSettingsPacketType;
    type NewInteractionModel = crate::enums::NewInteractionModel;
    type ObjectiveSortOrder = bedrock_protocol_v662::enums::ObjectiveSortOrder;
    type POIBlockInteractionType = bedrock_protocol_v662::enums::POIBlockInteractionType;
    type PackType = bedrock_protocol_v662::enums::PackType;
    type PacketCompressionAlgorithm = bedrock_protocol_v662::enums::PacketCompressionAlgorithm;
    type PacketViolationSeverity = bedrock_protocol_v662::enums::PacketViolationSeverity;
    type PacketViolationType = bedrock_protocol_v662::enums::PacketViolationType;
    type ParticleType = crate::enums::ParticleType;
    type PersonaPieceType = crate::enums::PersonaPieceType;
    type PhotoType = bedrock_protocol_v662::enums::PhotoType;
    type PlayStatus = bedrock_protocol_v662::enums::PlayStatus;
    type PlayerAuthInputData = crate::enums::PlayerAuthInputData;
    type PlayerPermissionLevel = bedrock_protocol_v662::enums::PlayerPermissionLevel;
    type PlayerPositionMode = crate::enums::PlayerPositionMode;
    type PlayerRespawnState = bedrock_protocol_v662::enums::PlayerRespawnState;
    type PredictionType = bedrock_protocol_v827::enums::PredictionType;
    type ResourcePackResponse = bedrock_protocol_v662::enums::ResourcePackResponse;
    type Rotation = bedrock_protocol_v662::enums::Rotation;
    type ServerAuthMovementMode = ();
    type ShowStoreOfferRedirectType = bedrock_protocol_v662::enums::ShowStoreOfferRedirectType;
    type SimulationType = bedrock_protocol_v662::enums::SimulationType;
    type SoftEnumUpdateType = bedrock_protocol_v662::enums::SoftEnumUpdateType;
    type SpawnBiomeType = bedrock_protocol_v662::enums::SpawnBiomeType;
    type SpawnPositionType = bedrock_protocol_v662::enums::SpawnPositionType;
    type StructureBlockType = bedrock_protocol_v662::enums::StructureBlockType;
    type StructureRedstoneSaveMode = crate::enums::StructureRedstoneSaveMode;
    type StructureTemplateRequestOperation =
        bedrock_protocol_v662::enums::StructureTemplateRequestOperation;
    type StructureTemplateResponseType =
        bedrock_protocol_v662::enums::StructureTemplateResponseType;
    type TeleportationCause = bedrock_protocol_v662::enums::TeleportationCause;
    type TextPacketType = bedrock_protocol_v924::enums::TextPacketType;
    type TextProcessingEventOrigin = bedrock_protocol_v662::enums::TextProcessingEventOrigin;
    type UIProfile = bedrock_protocol_v662::enums::UIProfile;
}
impl ProtoVersion for V2168 {
    const PROTOCOL_VERSION: u32 = 2168u32;
    const PROTOCOL_BRANCH: &str = "r/26_u4";
    const GAME_VERSION: &str = "1.26.40";
    const RAKNET_VERSION: u8 = 11u8;
}
#[cfg(feature = "packet-dyn")]
impl AsRef<dyn bedrock_protocol_core::PacketDyn> for V2168 {
    fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
        match self {
            V2168::ActorEventPacket(pk) => pk.as_ref(),
            V2168::ActorPickRequestPacket(pk) => pk.as_ref(),
            V2168::AddActorPacket(pk) => pk.as_ref(),
            V2168::AddBehaviourTreePacket(pk) => pk.as_ref(),
            V2168::AddItemActorPacket(pk) => pk.as_ref(),
            V2168::AddPaintingPacket(pk) => pk.as_ref(),
            V2168::AddPlayerPacket(pk) => pk.as_ref(),
            V2168::AddVolumeEntityPacket(pk) => pk.as_ref(),
            V2168::AgentActionEventPacket(pk) => pk.as_ref(),
            V2168::AgentAnimationPacket(pk) => pk.as_ref(),
            V2168::AnimateEntityPacket(pk) => pk.as_ref(),
            V2168::AnimatePacket(pk) => pk.as_ref(),
            V2168::AnvilDamagePacket(pk) => pk.as_ref(),
            V2168::AutomationClientConnectPacket(pk) => pk.as_ref(),
            V2168::AvailableActorIdentifiersPacket(pk) => pk.as_ref(),
            V2168::AvailableCommandsPacket(pk) => pk.as_ref(),
            V2168::AwardAchievementPacket(pk) => pk.as_ref(),
            V2168::BiomeDefinitionListPacket(pk) => pk.as_ref(),
            V2168::BlockActorDataPacket(pk) => pk.as_ref(),
            V2168::BlockEventPacket(pk) => pk.as_ref(),
            V2168::BlockPickRequestPacket(pk) => pk.as_ref(),
            V2168::BookEditPacket(pk) => pk.as_ref(),
            V2168::BossEventPacket(pk) => pk.as_ref(),
            V2168::CameraAimAssistActorPriorityPacket(pk) => pk.as_ref(),
            V2168::CameraAimAssistInstructionPacket(pk) => pk.as_ref(),
            V2168::CameraAimAssistPacket(pk) => pk.as_ref(),
            V2168::CameraAimAssistPresetsPacket(pk) => pk.as_ref(),
            V2168::CameraInstructionPacket(pk) => pk.as_ref(),
            V2168::CameraPacket(pk) => pk.as_ref(),
            V2168::CameraPresetsPacket(pk) => pk.as_ref(),
            V2168::CameraShakePacket(pk) => pk.as_ref(),
            V2168::CameraSplinePacket(pk) => pk.as_ref(),
            V2168::ChangeDimensionPacket(pk) => pk.as_ref(),
            V2168::ChangeMobPropertyPacket(pk) => pk.as_ref(),
            V2168::ChunkRadiusUpdatedPacket(pk) => pk.as_ref(),
            V2168::ClientBoundAttributeLayerSyncPacket(pk) => pk.as_ref(),
            V2168::ClientBoundCloseFormPacket(pk) => pk.as_ref(),
            V2168::ClientBoundControlSchemeSetPacket(pk) => pk.as_ref(),
            V2168::ClientBoundDataDrivenUICloseScreenPacket(pk) => pk.as_ref(),
            V2168::ClientBoundDataDrivenUIReloadPacket(pk) => pk.as_ref(),
            V2168::ClientBoundDataDrivenUIShowScreenPacket(pk) => pk.as_ref(),
            V2168::ClientBoundDataStorePacket(pk) => pk.as_ref(),
            V2168::ClientBoundDebugRendererPacket(pk) => pk.as_ref(),
            V2168::ClientBoundMapItemDataPacket(pk) => pk.as_ref(),
            V2168::ClientBoundTextureShiftPacket(pk) => pk.as_ref(),
            V2168::ClientBoundUpdateSoundDataPacket(pk) => pk.as_ref(),
            V2168::ClientCacheBlobStatusPacket(pk) => pk.as_ref(),
            V2168::ClientCacheMissResponsePacket(pk) => pk.as_ref(),
            V2168::ClientCacheStatusPacket(pk) => pk.as_ref(),
            V2168::ClientToServerHandshakePacket(pk) => pk.as_ref(),
            V2168::CodeBuilderPacket(pk) => pk.as_ref(),
            V2168::CodeBuilderSourcePacket(pk) => pk.as_ref(),
            V2168::CommandBlockUpdatePacket(pk) => pk.as_ref(),
            V2168::CommandOutputPacket(pk) => pk.as_ref(),
            V2168::CommandRequestPacket(pk) => pk.as_ref(),
            V2168::CompletedUsingItemPacket(pk) => pk.as_ref(),
            V2168::ContainerClosePacket(pk) => pk.as_ref(),
            V2168::ContainerOpenPacket(pk) => pk.as_ref(),
            V2168::ContainerRegistryCleanupPacket(pk) => pk.as_ref(),
            V2168::ContainerSetDataPacket(pk) => pk.as_ref(),
            V2168::CorrectPlayerMovePredictionPacket(pk) => pk.as_ref(),
            V2168::CraftingDataPacket(pk) => pk.as_ref(),
            V2168::CreatePhotoPacket(pk) => pk.as_ref(),
            V2168::CreativeContentPacket(pk) => pk.as_ref(),
            V2168::CurrentStructureFeaturePacket(pk) => pk.as_ref(),
            V2168::DeathInfoPacket(pk) => pk.as_ref(),
            V2168::DebugDrawerPacket(pk) => pk.as_ref(),
            V2168::DebugInfoPacket(pk) => pk.as_ref(),
            V2168::DimensionDataPacket(pk) => pk.as_ref(),
            V2168::DisconnectPacket(pk) => pk.as_ref(),
            V2168::EditorNetworkPacket(pk) => pk.as_ref(),
            V2168::EduUriResourcePacket(pk) => pk.as_ref(),
            V2168::EducationSettingsPacket(pk) => pk.as_ref(),
            V2168::EmoteListPacket(pk) => pk.as_ref(),
            V2168::EmotePacket(pk) => pk.as_ref(),
            V2168::FeatureRegistryPacket(pk) => pk.as_ref(),
            V2168::GameRulesChangedPacket(pk) => pk.as_ref(),
            V2168::GameTestRequestPacket(pk) => pk.as_ref(),
            V2168::GameTestResultsPacket(pk) => pk.as_ref(),
            V2168::GraphicsParameterOverridePacket(pk) => pk.as_ref(),
            V2168::GuiDataPickItemPacket(pk) => pk.as_ref(),
            V2168::HurtArmorPacket(pk) => pk.as_ref(),
            V2168::InteractPacket(pk) => pk.as_ref(),
            V2168::InventoryContentPacket(pk) => pk.as_ref(),
            V2168::InventorySlotPacket(pk) => pk.as_ref(),
            V2168::InventoryTransactionPacket(pk) => pk.as_ref(),
            V2168::ItemComponentPacket(pk) => pk.as_ref(),
            V2168::ItemStackRequestPacket(pk) => pk.as_ref(),
            V2168::ItemStackResponsePacket(pk) => pk.as_ref(),
            V2168::JigsawStructureDataPacket(pk) => pk.as_ref(),
            V2168::LabTablePacket(pk) => pk.as_ref(),
            V2168::LecternUpdatePacket(pk) => pk.as_ref(),
            V2168::LegacyTelemetryEventPacket(pk) => pk.as_ref(),
            V2168::LessonProgressPacket(pk) => pk.as_ref(),
            V2168::LevelChunkPacket(pk) => pk.as_ref(),
            V2168::LevelEventGenericPacket(pk) => pk.as_ref(),
            V2168::LevelEventPacket(pk) => pk.as_ref(),
            V2168::LevelSoundEventPacket(pk) => pk.as_ref(),
            V2168::LocatorBarPacket(pk) => pk.as_ref(),
            V2168::LoginPacket(pk) => pk.as_ref(),
            V2168::MapCreateLockedCopyPacket(pk) => pk.as_ref(),
            V2168::MapInfoRequestPacket(pk) => pk.as_ref(),
            V2168::MobArmorEquipmentPacket(pk) => pk.as_ref(),
            V2168::MobEffectPacket(pk) => pk.as_ref(),
            V2168::MobEquipmentPacket(pk) => pk.as_ref(),
            V2168::ModalFormRequestPacket(pk) => pk.as_ref(),
            V2168::ModalFormResponsePacket(pk) => pk.as_ref(),
            V2168::MotionPredictionHintsPacket(pk) => pk.as_ref(),
            V2168::MoveActorAbsolutePacket(pk) => pk.as_ref(),
            V2168::MoveActorDeltaPacket(pk) => pk.as_ref(),
            V2168::MovePlayerPacket(pk) => pk.as_ref(),
            V2168::MovementEffectPacket(pk) => pk.as_ref(),
            V2168::MovementPredictionSyncPacket(pk) => pk.as_ref(),
            V2168::MultiplayerSettingsPacket(pk) => pk.as_ref(),
            V2168::NetworkChunkPublisherUpdatePacket(pk) => pk.as_ref(),
            V2168::NetworkSettingsPacket(pk) => pk.as_ref(),
            V2168::NetworkStackLatencyPacket(pk) => pk.as_ref(),
            V2168::NpcDialoguePacket(pk) => pk.as_ref(),
            V2168::NpcRequestPacket(pk) => pk.as_ref(),
            V2168::OnScreenTextureAnimationPacket(pk) => pk.as_ref(),
            V2168::OpenSignPacket(pk) => pk.as_ref(),
            V2168::PacketViolationWarningPacket(pk) => pk.as_ref(),
            V2168::PartyChangedPacket(pk) => pk.as_ref(),
            V2168::PartyDestinationCookieResponsePacket(pk) => pk.as_ref(),
            V2168::PhotoTransferPacket(pk) => pk.as_ref(),
            V2168::PlaySoundPacket(pk) => pk.as_ref(),
            V2168::PlayStatusPacket(pk) => pk.as_ref(),
            V2168::PlayerActionPacket(pk) => pk.as_ref(),
            V2168::PlayerArmorDamagePacket(pk) => pk.as_ref(),
            V2168::PlayerAuthInputPacket(pk) => pk.as_ref(),
            V2168::PlayerEnchantOptionsPacket(pk) => pk.as_ref(),
            V2168::PlayerFogPacket(pk) => pk.as_ref(),
            V2168::PlayerHotbarPacket(pk) => pk.as_ref(),
            V2168::PlayerListPacket(pk) => pk.as_ref(),
            V2168::PlayerLocationPacket(pk) => pk.as_ref(),
            V2168::PlayerSkinPacket(pk) => pk.as_ref(),
            V2168::PlayerStartItemCooldownPacket(pk) => pk.as_ref(),
            V2168::PlayerToggleCrafterSlotRequestPacket(pk) => pk.as_ref(),
            V2168::PlayerUpdateEntityOverridesPacket(pk) => pk.as_ref(),
            V2168::PlayerVideoCapturePacket(pk) => pk.as_ref(),
            V2168::PositionTrackingDBClientRequestPacket(pk) => pk.as_ref(),
            V2168::PositionTrackingDBServerBroadcastPacket(pk) => pk.as_ref(),
            V2168::PurchaseReceiptPacket(pk) => pk.as_ref(),
            V2168::RefreshEntitlementsPacket(pk) => pk.as_ref(),
            V2168::RemoveActorPacket(pk) => pk.as_ref(),
            V2168::RemoveObjectivePacket(pk) => pk.as_ref(),
            V2168::RemoveVolumeEntityPacket(pk) => pk.as_ref(),
            V2168::RequestAbilityPacket(pk) => pk.as_ref(),
            V2168::RequestChunkRadiusPacket(pk) => pk.as_ref(),
            V2168::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
            V2168::RequestPermissionsPacket(pk) => pk.as_ref(),
            V2168::ResourcePackChunkDataPacket(pk) => pk.as_ref(),
            V2168::ResourcePackChunkRequestPacket(pk) => pk.as_ref(),
            V2168::ResourcePackClientResponsePacket(pk) => pk.as_ref(),
            V2168::ResourcePackDataInfoPacket(pk) => pk.as_ref(),
            V2168::ResourcePackStackPacket(pk) => pk.as_ref(),
            V2168::ResourcePacksInfoPacket(pk) => pk.as_ref(),
            V2168::ResourcePacksReadyForValidationPacket(pk) => pk.as_ref(),
            V2168::RespawnPacket(pk) => pk.as_ref(),
            V2168::ScriptMessagePacket(pk) => pk.as_ref(),
            V2168::SendPartyDestinationCookiePacket(pk) => pk.as_ref(),
            V2168::ServerBoundDataDrivenClosedPacket(pk) => pk.as_ref(),
            V2168::ServerBoundDataStorePacket(pk) => pk.as_ref(),
            V2168::ServerBoundDiagnosticsPacket(pk) => pk.as_ref(),
            V2168::ServerBoundLoadingScreenPacket(pk) => pk.as_ref(),
            V2168::ServerBoundPackSettingChangePacket(pk) => pk.as_ref(),
            V2168::ServerPlayerPostMovePositionPacket(pk) => pk.as_ref(),
            V2168::ServerPresenceInfoPacket(pk) => pk.as_ref(),
            V2168::ServerSettingsRequestPacket(pk) => pk.as_ref(),
            V2168::ServerSettingsResponsePacket(pk) => pk.as_ref(),
            V2168::ServerStatsPacket(pk) => pk.as_ref(),
            V2168::ServerStoreInfoPacket(pk) => pk.as_ref(),
            V2168::ServerToClientHandshakePacket(pk) => pk.as_ref(),
            V2168::SetActorDataPacket(pk) => pk.as_ref(),
            V2168::SetActorLinkPacket(pk) => pk.as_ref(),
            V2168::SetActorMotionPacket(pk) => pk.as_ref(),
            V2168::SetCommandsEnabledPacket(pk) => pk.as_ref(),
            V2168::SetDefaultGameTypePacket(pk) => pk.as_ref(),
            V2168::SetDifficultyPacket(pk) => pk.as_ref(),
            V2168::SetDisplayObjectivePacket(pk) => pk.as_ref(),
            V2168::SetHealthPacket(pk) => pk.as_ref(),
            V2168::SetHudPacket(pk) => pk.as_ref(),
            V2168::SetLastHurtByPacket(pk) => pk.as_ref(),
            V2168::SetLocalPlayerAsInitializedPacket(pk) => pk.as_ref(),
            V2168::SetPlayerGameTypePacket(pk) => pk.as_ref(),
            V2168::SetPlayerInventoryOptionsPacket(pk) => pk.as_ref(),
            V2168::SetScorePacket(pk) => pk.as_ref(),
            V2168::SetScoreboardIdentityPacket(pk) => pk.as_ref(),
            V2168::SetSpawnPositionPacket(pk) => pk.as_ref(),
            V2168::SetTimePacket(pk) => pk.as_ref(),
            V2168::SetTitlePacket(pk) => pk.as_ref(),
            V2168::SettingsCommandPacket(pk) => pk.as_ref(),
            V2168::ShowCreditsPacket(pk) => pk.as_ref(),
            V2168::ShowProfilePacket(pk) => pk.as_ref(),
            V2168::ShowStoreOfferPacket(pk) => pk.as_ref(),
            V2168::SimpleEventPacket(pk) => pk.as_ref(),
            V2168::SimulationTypePacket(pk) => pk.as_ref(),
            V2168::SpawnExperienceOrbPacket(pk) => pk.as_ref(),
            V2168::SpawnParticleEffectPacket(pk) => pk.as_ref(),
            V2168::StartGamePacket(pk) => pk.as_ref(),
            V2168::StopSoundPacket(pk) => pk.as_ref(),
            V2168::StructureBlockUpdatePacket(pk) => pk.as_ref(),
            V2168::StructureDataRequestPacket(pk) => pk.as_ref(),
            V2168::StructureDataResponsePacket(pk) => pk.as_ref(),
            V2168::SubChunkPacket(pk) => pk.as_ref(),
            V2168::SubChunkRequestPacket(pk) => pk.as_ref(),
            V2168::SubClientLoginPacket(pk) => pk.as_ref(),
            V2168::SyncActorPropertyPacket(pk) => pk.as_ref(),
            V2168::SyncWorldClocksPacket(pk) => pk.as_ref(),
            V2168::TakeItemActorPacket(pk) => pk.as_ref(),
            V2168::TextPacket(pk) => pk.as_ref(),
            V2168::TickingAreaLoadStatusPacket(pk) => pk.as_ref(),
            V2168::ToastRequestPacket(pk) => pk.as_ref(),
            V2168::TransferPlayerPacket(pk) => pk.as_ref(),
            V2168::TrimDataPacket(pk) => pk.as_ref(),
            V2168::UnlockedRecipesPacket(pk) => pk.as_ref(),
            V2168::UpdateAbilitiesPacket(pk) => pk.as_ref(),
            V2168::UpdateAdventureSettingsPacket(pk) => pk.as_ref(),
            V2168::UpdateAttributesPacket(pk) => pk.as_ref(),
            V2168::UpdateBlockPacket(pk) => pk.as_ref(),
            V2168::UpdateBlockSyncedPacket(pk) => pk.as_ref(),
            V2168::UpdateClientInputLocksPacket(pk) => pk.as_ref(),
            V2168::UpdateClientOptionsPacket(pk) => pk.as_ref(),
            V2168::UpdateEquipPacket(pk) => pk.as_ref(),
            V2168::UpdatePlayerGameTypePacket(pk) => pk.as_ref(),
            V2168::UpdateSoftEnumPacket(pk) => pk.as_ref(),
            V2168::UpdateSubChunkBlocksPacket(pk) => pk.as_ref(),
            V2168::UpdateTradePacket(pk) => pk.as_ref(),
            V2168::VoxelShapesPacket(pk) => pk.as_ref(),
            V2168::Unknown(pk) => pk.as_ref(),
        }
    }
}
#[cfg(feature = "packet-dyn")]
impl From<V2168> for Box<dyn bedrock_protocol_core::PacketDyn> {
    fn from(val: V2168) -> Box<dyn bedrock_protocol_core::PacketDyn> {
        match val {
            V2168::ActorEventPacket(pk) => pk,
            V2168::ActorPickRequestPacket(pk) => pk,
            V2168::AddActorPacket(pk) => pk,
            V2168::AddBehaviourTreePacket(pk) => pk,
            V2168::AddItemActorPacket(pk) => pk,
            V2168::AddPaintingPacket(pk) => pk,
            V2168::AddPlayerPacket(pk) => pk,
            V2168::AddVolumeEntityPacket(pk) => pk,
            V2168::AgentActionEventPacket(pk) => pk,
            V2168::AgentAnimationPacket(pk) => pk,
            V2168::AnimateEntityPacket(pk) => pk,
            V2168::AnimatePacket(pk) => pk,
            V2168::AnvilDamagePacket(pk) => pk,
            V2168::AutomationClientConnectPacket(pk) => pk,
            V2168::AvailableActorIdentifiersPacket(pk) => pk,
            V2168::AvailableCommandsPacket(pk) => pk,
            V2168::AwardAchievementPacket(pk) => pk,
            V2168::BiomeDefinitionListPacket(pk) => pk,
            V2168::BlockActorDataPacket(pk) => pk,
            V2168::BlockEventPacket(pk) => pk,
            V2168::BlockPickRequestPacket(pk) => pk,
            V2168::BookEditPacket(pk) => pk,
            V2168::BossEventPacket(pk) => pk,
            V2168::CameraAimAssistActorPriorityPacket(pk) => pk,
            V2168::CameraAimAssistInstructionPacket(pk) => pk,
            V2168::CameraAimAssistPacket(pk) => pk,
            V2168::CameraAimAssistPresetsPacket(pk) => pk,
            V2168::CameraInstructionPacket(pk) => pk,
            V2168::CameraPacket(pk) => pk,
            V2168::CameraPresetsPacket(pk) => pk,
            V2168::CameraShakePacket(pk) => pk,
            V2168::CameraSplinePacket(pk) => pk,
            V2168::ChangeDimensionPacket(pk) => pk,
            V2168::ChangeMobPropertyPacket(pk) => pk,
            V2168::ChunkRadiusUpdatedPacket(pk) => pk,
            V2168::ClientBoundAttributeLayerSyncPacket(pk) => pk,
            V2168::ClientBoundCloseFormPacket(pk) => pk,
            V2168::ClientBoundControlSchemeSetPacket(pk) => pk,
            V2168::ClientBoundDataDrivenUICloseScreenPacket(pk) => pk,
            V2168::ClientBoundDataDrivenUIReloadPacket(pk) => pk,
            V2168::ClientBoundDataDrivenUIShowScreenPacket(pk) => pk,
            V2168::ClientBoundDataStorePacket(pk) => pk,
            V2168::ClientBoundDebugRendererPacket(pk) => pk,
            V2168::ClientBoundMapItemDataPacket(pk) => pk,
            V2168::ClientBoundTextureShiftPacket(pk) => pk,
            V2168::ClientBoundUpdateSoundDataPacket(pk) => pk,
            V2168::ClientCacheBlobStatusPacket(pk) => pk,
            V2168::ClientCacheMissResponsePacket(pk) => pk,
            V2168::ClientCacheStatusPacket(pk) => pk,
            V2168::ClientToServerHandshakePacket(pk) => pk,
            V2168::CodeBuilderPacket(pk) => pk,
            V2168::CodeBuilderSourcePacket(pk) => pk,
            V2168::CommandBlockUpdatePacket(pk) => pk,
            V2168::CommandOutputPacket(pk) => pk,
            V2168::CommandRequestPacket(pk) => pk,
            V2168::CompletedUsingItemPacket(pk) => pk,
            V2168::ContainerClosePacket(pk) => pk,
            V2168::ContainerOpenPacket(pk) => pk,
            V2168::ContainerRegistryCleanupPacket(pk) => pk,
            V2168::ContainerSetDataPacket(pk) => pk,
            V2168::CorrectPlayerMovePredictionPacket(pk) => pk,
            V2168::CraftingDataPacket(pk) => pk,
            V2168::CreatePhotoPacket(pk) => pk,
            V2168::CreativeContentPacket(pk) => pk,
            V2168::CurrentStructureFeaturePacket(pk) => pk,
            V2168::DeathInfoPacket(pk) => pk,
            V2168::DebugDrawerPacket(pk) => pk,
            V2168::DebugInfoPacket(pk) => pk,
            V2168::DimensionDataPacket(pk) => pk,
            V2168::DisconnectPacket(pk) => pk,
            V2168::EditorNetworkPacket(pk) => pk,
            V2168::EduUriResourcePacket(pk) => pk,
            V2168::EducationSettingsPacket(pk) => pk,
            V2168::EmoteListPacket(pk) => pk,
            V2168::EmotePacket(pk) => pk,
            V2168::FeatureRegistryPacket(pk) => pk,
            V2168::GameRulesChangedPacket(pk) => pk,
            V2168::GameTestRequestPacket(pk) => pk,
            V2168::GameTestResultsPacket(pk) => pk,
            V2168::GraphicsParameterOverridePacket(pk) => pk,
            V2168::GuiDataPickItemPacket(pk) => pk,
            V2168::HurtArmorPacket(pk) => pk,
            V2168::InteractPacket(pk) => pk,
            V2168::InventoryContentPacket(pk) => pk,
            V2168::InventorySlotPacket(pk) => pk,
            V2168::InventoryTransactionPacket(pk) => pk,
            V2168::ItemComponentPacket(pk) => pk,
            V2168::ItemStackRequestPacket(pk) => pk,
            V2168::ItemStackResponsePacket(pk) => pk,
            V2168::JigsawStructureDataPacket(pk) => pk,
            V2168::LabTablePacket(pk) => pk,
            V2168::LecternUpdatePacket(pk) => pk,
            V2168::LegacyTelemetryEventPacket(pk) => pk,
            V2168::LessonProgressPacket(pk) => pk,
            V2168::LevelChunkPacket(pk) => pk,
            V2168::LevelEventGenericPacket(pk) => pk,
            V2168::LevelEventPacket(pk) => pk,
            V2168::LevelSoundEventPacket(pk) => pk,
            V2168::LocatorBarPacket(pk) => pk,
            V2168::LoginPacket(pk) => pk,
            V2168::MapCreateLockedCopyPacket(pk) => pk,
            V2168::MapInfoRequestPacket(pk) => pk,
            V2168::MobArmorEquipmentPacket(pk) => pk,
            V2168::MobEffectPacket(pk) => pk,
            V2168::MobEquipmentPacket(pk) => pk,
            V2168::ModalFormRequestPacket(pk) => pk,
            V2168::ModalFormResponsePacket(pk) => pk,
            V2168::MotionPredictionHintsPacket(pk) => pk,
            V2168::MoveActorAbsolutePacket(pk) => pk,
            V2168::MoveActorDeltaPacket(pk) => pk,
            V2168::MovePlayerPacket(pk) => pk,
            V2168::MovementEffectPacket(pk) => pk,
            V2168::MovementPredictionSyncPacket(pk) => pk,
            V2168::MultiplayerSettingsPacket(pk) => pk,
            V2168::NetworkChunkPublisherUpdatePacket(pk) => pk,
            V2168::NetworkSettingsPacket(pk) => pk,
            V2168::NetworkStackLatencyPacket(pk) => pk,
            V2168::NpcDialoguePacket(pk) => pk,
            V2168::NpcRequestPacket(pk) => pk,
            V2168::OnScreenTextureAnimationPacket(pk) => pk,
            V2168::OpenSignPacket(pk) => pk,
            V2168::PacketViolationWarningPacket(pk) => pk,
            V2168::PartyChangedPacket(pk) => pk,
            V2168::PartyDestinationCookieResponsePacket(pk) => pk,
            V2168::PhotoTransferPacket(pk) => pk,
            V2168::PlaySoundPacket(pk) => pk,
            V2168::PlayStatusPacket(pk) => pk,
            V2168::PlayerActionPacket(pk) => pk,
            V2168::PlayerArmorDamagePacket(pk) => pk,
            V2168::PlayerAuthInputPacket(pk) => pk,
            V2168::PlayerEnchantOptionsPacket(pk) => pk,
            V2168::PlayerFogPacket(pk) => pk,
            V2168::PlayerHotbarPacket(pk) => pk,
            V2168::PlayerListPacket(pk) => pk,
            V2168::PlayerLocationPacket(pk) => pk,
            V2168::PlayerSkinPacket(pk) => pk,
            V2168::PlayerStartItemCooldownPacket(pk) => pk,
            V2168::PlayerToggleCrafterSlotRequestPacket(pk) => pk,
            V2168::PlayerUpdateEntityOverridesPacket(pk) => pk,
            V2168::PlayerVideoCapturePacket(pk) => pk,
            V2168::PositionTrackingDBClientRequestPacket(pk) => pk,
            V2168::PositionTrackingDBServerBroadcastPacket(pk) => pk,
            V2168::PurchaseReceiptPacket(pk) => pk,
            V2168::RefreshEntitlementsPacket(pk) => pk,
            V2168::RemoveActorPacket(pk) => pk,
            V2168::RemoveObjectivePacket(pk) => pk,
            V2168::RemoveVolumeEntityPacket(pk) => pk,
            V2168::RequestAbilityPacket(pk) => pk,
            V2168::RequestChunkRadiusPacket(pk) => pk,
            V2168::RequestNetworkSettingsPacket(pk) => pk,
            V2168::RequestPermissionsPacket(pk) => pk,
            V2168::ResourcePackChunkDataPacket(pk) => pk,
            V2168::ResourcePackChunkRequestPacket(pk) => pk,
            V2168::ResourcePackClientResponsePacket(pk) => pk,
            V2168::ResourcePackDataInfoPacket(pk) => pk,
            V2168::ResourcePackStackPacket(pk) => pk,
            V2168::ResourcePacksInfoPacket(pk) => pk,
            V2168::ResourcePacksReadyForValidationPacket(pk) => pk,
            V2168::RespawnPacket(pk) => pk,
            V2168::ScriptMessagePacket(pk) => pk,
            V2168::SendPartyDestinationCookiePacket(pk) => pk,
            V2168::ServerBoundDataDrivenClosedPacket(pk) => pk,
            V2168::ServerBoundDataStorePacket(pk) => pk,
            V2168::ServerBoundDiagnosticsPacket(pk) => pk,
            V2168::ServerBoundLoadingScreenPacket(pk) => pk,
            V2168::ServerBoundPackSettingChangePacket(pk) => pk,
            V2168::ServerPlayerPostMovePositionPacket(pk) => pk,
            V2168::ServerPresenceInfoPacket(pk) => pk,
            V2168::ServerSettingsRequestPacket(pk) => pk,
            V2168::ServerSettingsResponsePacket(pk) => pk,
            V2168::ServerStatsPacket(pk) => pk,
            V2168::ServerStoreInfoPacket(pk) => pk,
            V2168::ServerToClientHandshakePacket(pk) => pk,
            V2168::SetActorDataPacket(pk) => pk,
            V2168::SetActorLinkPacket(pk) => pk,
            V2168::SetActorMotionPacket(pk) => pk,
            V2168::SetCommandsEnabledPacket(pk) => pk,
            V2168::SetDefaultGameTypePacket(pk) => pk,
            V2168::SetDifficultyPacket(pk) => pk,
            V2168::SetDisplayObjectivePacket(pk) => pk,
            V2168::SetHealthPacket(pk) => pk,
            V2168::SetHudPacket(pk) => pk,
            V2168::SetLastHurtByPacket(pk) => pk,
            V2168::SetLocalPlayerAsInitializedPacket(pk) => pk,
            V2168::SetPlayerGameTypePacket(pk) => pk,
            V2168::SetPlayerInventoryOptionsPacket(pk) => pk,
            V2168::SetScorePacket(pk) => pk,
            V2168::SetScoreboardIdentityPacket(pk) => pk,
            V2168::SetSpawnPositionPacket(pk) => pk,
            V2168::SetTimePacket(pk) => pk,
            V2168::SetTitlePacket(pk) => pk,
            V2168::SettingsCommandPacket(pk) => pk,
            V2168::ShowCreditsPacket(pk) => pk,
            V2168::ShowProfilePacket(pk) => pk,
            V2168::ShowStoreOfferPacket(pk) => pk,
            V2168::SimpleEventPacket(pk) => pk,
            V2168::SimulationTypePacket(pk) => pk,
            V2168::SpawnExperienceOrbPacket(pk) => pk,
            V2168::SpawnParticleEffectPacket(pk) => pk,
            V2168::StartGamePacket(pk) => pk,
            V2168::StopSoundPacket(pk) => pk,
            V2168::StructureBlockUpdatePacket(pk) => pk,
            V2168::StructureDataRequestPacket(pk) => pk,
            V2168::StructureDataResponsePacket(pk) => pk,
            V2168::SubChunkPacket(pk) => pk,
            V2168::SubChunkRequestPacket(pk) => pk,
            V2168::SubClientLoginPacket(pk) => pk,
            V2168::SyncActorPropertyPacket(pk) => pk,
            V2168::SyncWorldClocksPacket(pk) => pk,
            V2168::TakeItemActorPacket(pk) => pk,
            V2168::TextPacket(pk) => pk,
            V2168::TickingAreaLoadStatusPacket(pk) => pk,
            V2168::ToastRequestPacket(pk) => pk,
            V2168::TransferPlayerPacket(pk) => pk,
            V2168::TrimDataPacket(pk) => pk,
            V2168::UnlockedRecipesPacket(pk) => pk,
            V2168::UpdateAbilitiesPacket(pk) => pk,
            V2168::UpdateAdventureSettingsPacket(pk) => pk,
            V2168::UpdateAttributesPacket(pk) => pk,
            V2168::UpdateBlockPacket(pk) => pk,
            V2168::UpdateBlockSyncedPacket(pk) => pk,
            V2168::UpdateClientInputLocksPacket(pk) => pk,
            V2168::UpdateClientOptionsPacket(pk) => pk,
            V2168::UpdateEquipPacket(pk) => pk,
            V2168::UpdatePlayerGameTypePacket(pk) => pk,
            V2168::UpdateSoftEnumPacket(pk) => pk,
            V2168::UpdateSubChunkBlocksPacket(pk) => pk,
            V2168::UpdateTradePacket(pk) => pk,
            V2168::VoxelShapesPacket(pk) => pk,
            V2168::Unknown(pk) => pk,
        }
    }
}
