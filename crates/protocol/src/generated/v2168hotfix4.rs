#![allow(unused)]
#[cfg(feature = "v2168hotfix4")]
mod inner {
    use crate::ProtoVersion;
    use crate::ProtoVersionEnums;
    use crate::ProtoVersionPackets;
    use crate::ProtoVersionTypes;
    #[derive(Clone, std::fmt::Debug)]
    pub enum V2168Hotfix4 {
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
        CameraAimAssistPresetsPacket(
            Box<<Self as ProtoVersionPackets>::CameraAimAssistPresetsPacket>,
        ),
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
        ClientBoundMapItemDataPacket(
            Box<<Self as ProtoVersionPackets>::ClientBoundMapItemDataPacket>,
        ),
        ClientBoundTextureShiftPacket(
            Box<<Self as ProtoVersionPackets>::ClientBoundTextureShiftPacket>,
        ),
        ClientBoundUpdateSoundDataPacket(
            Box<<Self as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket>,
        ),
        ClientCacheBlobStatusPacket(
            Box<<Self as ProtoVersionPackets>::ClientCacheBlobStatusPacket>,
        ),
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
        MotionPredictionHintsPacket(
            Box<<Self as ProtoVersionPackets>::MotionPredictionHintsPacket>,
        ),
        MoveActorAbsolutePacket(Box<<Self as ProtoVersionPackets>::MoveActorAbsolutePacket>),
        MoveActorDeltaPacket(Box<<Self as ProtoVersionPackets>::MoveActorDeltaPacket>),
        MovePlayerPacket(Box<<Self as ProtoVersionPackets>::MovePlayerPacket>),
        MovementEffectPacket(Box<<Self as ProtoVersionPackets>::MovementEffectPacket>),
        MovementPredictionSyncPacket(
            Box<<Self as ProtoVersionPackets>::MovementPredictionSyncPacket>,
        ),
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
        PacketViolationWarningPacket(
            Box<<Self as ProtoVersionPackets>::PacketViolationWarningPacket>,
        ),
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
        RequestNetworkSettingsPacket(
            Box<<Self as ProtoVersionPackets>::RequestNetworkSettingsPacket>,
        ),
        RequestPermissionsPacket(Box<<Self as ProtoVersionPackets>::RequestPermissionsPacket>),
        ResourcePackChunkDataPacket(
            Box<<Self as ProtoVersionPackets>::ResourcePackChunkDataPacket>,
        ),
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
        ServerBoundDiagnosticsPacket(
            Box<<Self as ProtoVersionPackets>::ServerBoundDiagnosticsPacket>,
        ),
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
        ServerSettingsRequestPacket(
            Box<<Self as ProtoVersionPackets>::ServerSettingsRequestPacket>,
        ),
        ServerSettingsResponsePacket(
            Box<<Self as ProtoVersionPackets>::ServerSettingsResponsePacket>,
        ),
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
        SetScoreboardIdentityPacket(
            Box<<Self as ProtoVersionPackets>::SetScoreboardIdentityPacket>,
        ),
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
        StructureDataResponsePacket(
            Box<<Self as ProtoVersionPackets>::StructureDataResponsePacket>,
        ),
        SubChunkPacket(Box<<Self as ProtoVersionPackets>::SubChunkPacket>),
        SubChunkRequestPacket(Box<<Self as ProtoVersionPackets>::SubChunkRequestPacket>),
        SubClientLoginPacket(Box<<Self as ProtoVersionPackets>::SubClientLoginPacket>),
        SyncActorPropertyPacket(Box<<Self as ProtoVersionPackets>::SyncActorPropertyPacket>),
        SyncWorldClocksPacket(Box<<Self as ProtoVersionPackets>::SyncWorldClocksPacket>),
        TakeItemActorPacket(Box<<Self as ProtoVersionPackets>::TakeItemActorPacket>),
        TextPacket(Box<<Self as ProtoVersionPackets>::TextPacket>),
        TickingAreaLoadStatusPacket(
            Box<<Self as ProtoVersionPackets>::TickingAreaLoadStatusPacket>,
        ),
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
        UpdateClientInputLocksPacket(
            Box<<Self as ProtoVersionPackets>::UpdateClientInputLocksPacket>,
        ),
        UpdateClientOptionsPacket(Box<<Self as ProtoVersionPackets>::UpdateClientOptionsPacket>),
        UpdateEquipPacket(Box<<Self as ProtoVersionPackets>::UpdateEquipPacket>),
        UpdatePlayerGameTypePacket(Box<<Self as ProtoVersionPackets>::UpdatePlayerGameTypePacket>),
        UpdateSoftEnumPacket(Box<<Self as ProtoVersionPackets>::UpdateSoftEnumPacket>),
        UpdateSubChunkBlocksPacket(Box<<Self as ProtoVersionPackets>::UpdateSubChunkBlocksPacket>),
        UpdateTradePacket(Box<<Self as ProtoVersionPackets>::UpdateTradePacket>),
        VoxelShapesPacket(Box<<Self as ProtoVersionPackets>::VoxelShapesPacket>),
        Unknown(Box<bedrock_protocol_core::UnknownPacket>),
    }
    impl bedrock_protocol_core::Packets for V2168Hotfix4 {
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
                V2168Hotfix4::ActorEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ActorPickRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AddActorPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AddBehaviourTreePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AddItemActorPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AddPaintingPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AddPlayerPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AddVolumeEntityPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AgentActionEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AgentAnimationPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AnimateEntityPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AnimatePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AnvilDamagePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AutomationClientConnectPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AvailableActorIdentifiersPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AvailableCommandsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::AwardAchievementPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AwardAchievementPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::BiomeDefinitionListPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::BlockActorDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::BlockEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::BlockPickRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::BookEditPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::BossEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraAimAssistActorPriorityPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistActorPriorityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraAimAssistInstructionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistInstructionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraAimAssistPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraAimAssistPresetsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPresetsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraInstructionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraPresetsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraShakePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CameraSplinePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraSplinePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ChangeDimensionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ChangeMobPropertyPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ChunkRadiusUpdatedPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundAttributeLayerSyncPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundAttributeLayerSyncPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundCloseFormPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundCloseFormPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundControlSchemeSetPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundControlSchemeSetPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundDataDrivenUICloseScreenPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundDataDrivenUICloseScreenPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundDataDrivenUIReloadPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundDataDrivenUIReloadPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundDataDrivenUIShowScreenPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundDataDrivenUIShowScreenPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundDataStorePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDataStorePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundDebugRendererPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundMapItemDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundTextureShiftPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundTextureShiftPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientBoundUpdateSoundDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundUpdateSoundDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientCacheBlobStatusPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientCacheMissResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientCacheStatusPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ClientToServerHandshakePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CodeBuilderPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CodeBuilderSourcePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CommandBlockUpdatePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CommandOutputPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CommandRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CompletedUsingItemPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ContainerClosePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ContainerOpenPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ContainerRegistryCleanupPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerRegistryCleanupPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ContainerSetDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CorrectPlayerMovePredictionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CraftingDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CreatePhotoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CreativeContentPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::CurrentStructureFeaturePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CurrentStructureFeaturePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::DeathInfoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::DebugDrawerPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugDrawerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::DebugInfoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::DimensionDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::DisconnectPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::EditorNetworkPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::EduUriResourcePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::EducationSettingsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::EmoteListPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::EmotePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::FeatureRegistryPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::GameRulesChangedPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::GameTestRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::GameTestResultsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::GraphicsParameterOverridePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GraphicsParameterOverridePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::GuiDataPickItemPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::HurtArmorPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::InteractPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::InventoryContentPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::InventorySlotPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::InventoryTransactionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ItemComponentPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ItemStackRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ItemStackResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::JigsawStructureDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(JigsawStructureDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LabTablePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LecternUpdatePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LegacyTelemetryEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LessonProgressPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LevelChunkPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LevelEventGenericPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LevelEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LevelSoundEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LocatorBarPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LocatorBarPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::LoginPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MapCreateLockedCopyPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MapInfoRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MobArmorEquipmentPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MobEffectPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MobEquipmentPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ModalFormRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ModalFormResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MotionPredictionHintsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MoveActorAbsolutePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MoveActorDeltaPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MovePlayerPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MovementEffectPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovementEffectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MovementPredictionSyncPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovementPredictionSyncPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::MultiplayerSettingsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::NetworkChunkPublisherUpdatePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::NetworkSettingsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::NetworkStackLatencyPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::NpcDialoguePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::NpcRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::OnScreenTextureAnimationPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::OpenSignPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PacketViolationWarningPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PartyChangedPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PartyChangedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PartyDestinationCookieResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PartyDestinationCookieResponsePacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PhotoTransferPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlaySoundPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayStatusPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerActionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerArmorDamagePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerAuthInputPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerEnchantOptionsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerFogPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerHotbarPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerListPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerLocationPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerLocationPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerSkinPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerStartItemCooldownPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerToggleCrafterSlotRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerUpdateEntityOverridesPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerUpdateEntityOverridesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PlayerVideoCapturePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerVideoCapturePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PositionTrackingDBClientRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PositionTrackingDBServerBroadcastPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::PurchaseReceiptPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RefreshEntitlementsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RemoveActorPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RemoveObjectivePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RemoveVolumeEntityPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RequestAbilityPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RequestChunkRadiusPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RequestNetworkSettingsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RequestPermissionsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePackChunkDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePackChunkRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePackClientResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePackDataInfoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePackStackPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePacksInfoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ResourcePacksReadyForValidationPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ResourcePacksReadyForValidationPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::RespawnPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ScriptMessagePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SendPartyDestinationCookiePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SendPartyDestinationCookiePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerBoundDataDrivenClosedPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDataDrivenClosedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerBoundDataStorePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDataStorePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerBoundDiagnosticsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDiagnosticsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerBoundLoadingScreenPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundLoadingScreenPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerBoundPackSettingChangePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundPackSettingChangePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerPlayerPostMovePositionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerPresenceInfoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPresenceInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerSettingsRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerSettingsResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerStatsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerStoreInfoPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStoreInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ServerToClientHandshakePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetActorDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetActorLinkPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetActorMotionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetCommandsEnabledPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetDefaultGameTypePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetDifficultyPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetDisplayObjectivePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetHealthPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetHudPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetLastHurtByPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetLocalPlayerAsInitializedPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetPlayerGameTypePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetPlayerInventoryOptionsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetScorePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetScoreboardIdentityPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetSpawnPositionPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetTimePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SetTitlePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SettingsCommandPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ShowCreditsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ShowProfilePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ShowStoreOfferPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SimpleEventPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SimulationTypePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SpawnExperienceOrbPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SpawnParticleEffectPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::StartGamePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::StopSoundPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::StructureBlockUpdatePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::StructureDataRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::StructureDataResponsePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SubChunkPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SubChunkRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SubClientLoginPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SyncActorPropertyPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::SyncWorldClocksPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncWorldClocksPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::TakeItemActorPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::TextPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::TickingAreaLoadStatusPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::ToastRequestPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::TransferPlayerPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::TrimDataPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UnlockedRecipesPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateAbilitiesPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateAdventureSettingsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateAttributesPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateBlockPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateBlockSyncedPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateClientInputLocksPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateClientOptionsPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientOptionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateEquipPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdatePlayerGameTypePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateSoftEnumPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateSubChunkBlocksPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::UpdateTradePacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::VoxelShapesPacket(pk) => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(VoxelShapesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V2168Hotfix4::Unknown(pk) => stream.write_all(pk.buf.as_ref()).map_err(|e| {
                    bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                        packet_name: "Unknown",
                        packet_id: header.packet_id,
                        error: e.into(),
                    }
                })?,
            };
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
                <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ActorEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ActorPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AddActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AddBehaviourTreePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AddItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AddPaintingPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AddPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AddVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AgentActionEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AgentAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AnimateEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AnimatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AnvilDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::AutomationClientConnectPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::AvailableActorIdentifiersPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AvailableCommandsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::AwardAchievementPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AwardAchievementPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::BiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::BlockActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::BlockEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::BlockPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::BookEditPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::BossEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::CameraAimAssistActorPriorityPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistActorPriorityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::CameraAimAssistInstructionPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistInstructionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CameraAimAssistPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::CameraAimAssistPresetsPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPresetsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CameraInstructionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CameraPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CameraPresetsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CameraShakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CameraSplinePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraSplinePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ChangeDimensionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ChangeMobPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ChunkRadiusUpdatedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundAttributeLayerSyncPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundAttributeLayerSyncPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ClientBoundCloseFormPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundCloseFormPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundControlSchemeSetPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundControlSchemeSetPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundDataDrivenUICloseScreenPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundDataDrivenUICloseScreenPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundDataDrivenUIReloadPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundDataDrivenUIReloadPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundDataDrivenUIShowScreenPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ClientBoundDataDrivenUIShowScreenPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ClientBoundDataStorePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDataStorePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundDebugRendererPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundMapItemDataPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundTextureShiftPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundTextureShiftPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientBoundUpdateSoundDataPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundUpdateSoundDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ClientCacheBlobStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientCacheMissResponsePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ClientCacheStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ClientToServerHandshakePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CodeBuilderPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CodeBuilderSourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CommandBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CommandOutputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CommandRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CompletedUsingItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ContainerClosePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ContainerOpenPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ContainerRegistryCleanupPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerRegistryCleanupPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ContainerSetDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::CorrectPlayerMovePredictionPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CraftingDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CreatePhotoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::CreativeContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::CurrentStructureFeaturePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CurrentStructureFeaturePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::DeathInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::DebugDrawerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugDrawerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::DebugInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::DimensionDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::DisconnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::EditorNetworkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::EduUriResourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::EducationSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::EmoteListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::EmotePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::FeatureRegistryPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::GameRulesChangedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::GameTestRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::GameTestResultsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::GraphicsParameterOverridePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GraphicsParameterOverridePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::GuiDataPickItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::HurtArmorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::InteractPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::InventoryContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::InventorySlotPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::InventoryTransactionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ItemComponentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ItemStackRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ItemStackResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::JigsawStructureDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(JigsawStructureDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LabTablePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LecternUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LegacyTelemetryEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LessonProgressPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LevelChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LevelEventGenericPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LevelEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LevelSoundEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LocatorBarPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LocatorBarPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::LoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MapCreateLockedCopyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MapInfoRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MobArmorEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MobEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MobEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ModalFormRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ModalFormResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MotionPredictionHintsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MoveActorAbsolutePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MoveActorDeltaPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MovePlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MovementEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovementEffectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::MovementPredictionSyncPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovementPredictionSyncPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::MultiplayerSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::NetworkChunkPublisherUpdatePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::NetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::NetworkStackLatencyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::NpcDialoguePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::NpcRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::OnScreenTextureAnimationPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::OpenSignPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PacketViolationWarningPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PartyChangedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PartyChangedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PartyDestinationCookieResponsePacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PartyDestinationCookieResponsePacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PhotoTransferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlaySoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerActionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerArmorDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerAuthInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerEnchantOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerFogPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerHotbarPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerLocationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerLocationPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerSkinPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PlayerStartItemCooldownPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PlayerToggleCrafterSlotRequestPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PlayerUpdateEntityOverridesPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerUpdateEntityOverridesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PlayerVideoCapturePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerVideoCapturePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PositionTrackingDBClientRequestPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::PositionTrackingDBServerBroadcastPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::PurchaseReceiptPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RefreshEntitlementsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RemoveActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RemoveObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RemoveVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RequestAbilityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RequestChunkRadiusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::RequestNetworkSettingsPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RequestPermissionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ResourcePackChunkDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ResourcePackChunkRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ResourcePackClientResponsePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ResourcePackDataInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ResourcePackStackPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ResourcePacksInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ResourcePacksReadyForValidationPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    ResourcePacksReadyForValidationPacket
                                ),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::RespawnPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ScriptMessagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::SendPartyDestinationCookiePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SendPartyDestinationCookiePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerBoundDataDrivenClosedPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDataDrivenClosedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ServerBoundDataStorePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDataStorePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerBoundDiagnosticsPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDiagnosticsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerBoundLoadingScreenPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundLoadingScreenPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerBoundPackSettingChangePacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundPackSettingChangePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerPlayerPostMovePositionPacket(
                                Box::new(pk),
                            )
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ServerPresenceInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPresenceInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ServerSettingsRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerSettingsResponsePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ServerStatsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ServerStoreInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStoreInfoPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::ServerToClientHandshakePacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetActorLinkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetActorMotionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetCommandsEnabledPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetDefaultGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetDifficultyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetDisplayObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetHealthPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetHudPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetLastHurtByPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::SetLocalPlayerAsInitializedPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetPlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::SetPlayerInventoryOptionsPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetScorePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetScoreboardIdentityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetSpawnPositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetTimePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SetTitlePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SettingsCommandPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ShowCreditsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ShowProfilePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ShowStoreOfferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SimpleEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SimulationTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SpawnExperienceOrbPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SpawnParticleEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::StartGamePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::StopSoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::StructureBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::StructureDataRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::StructureDataResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SubChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SubChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SubClientLoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SyncActorPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::SyncWorldClocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncWorldClocksPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::TakeItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::TextPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::TickingAreaLoadStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::ToastRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::TransferPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::TrimDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UnlockedRecipesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateAbilitiesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::UpdateAdventureSettingsPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateAttributesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateBlockPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateBlockSyncedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V2168Hotfix4::UpdateClientInputLocksPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateClientOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientOptionsPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateEquipPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdatePlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateSoftEnumPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateSubChunkBlocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::UpdateTradePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V2168Hotfix4::VoxelShapesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(VoxelShapesPacket),
                                packet_id: <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID,
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
                    V2168Hotfix4::Unknown(
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
                    V2168Hotfix4::ActorEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ActorPickRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AddActorPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AddBehaviourTreePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AddItemActorPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AddPaintingPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AddPlayerPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AddVolumeEntityPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AgentActionEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AgentAnimationPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AnimateEntityPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AnimatePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AnvilDamagePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AutomationClientConnectPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AvailableActorIdentifiersPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AvailableCommandsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::AwardAchievementPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::BiomeDefinitionListPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::BlockActorDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::BlockEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::BlockPickRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::BookEditPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::BossEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraAimAssistActorPriorityPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraAimAssistInstructionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraAimAssistPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraAimAssistPresetsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraInstructionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraPresetsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraShakePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CameraSplinePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ChangeDimensionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ChangeMobPropertyPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ChunkRadiusUpdatedPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundAttributeLayerSyncPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundCloseFormPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundControlSchemeSetPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundDataDrivenUICloseScreenPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundDataDrivenUIReloadPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundDataDrivenUIShowScreenPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundDataStorePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundDebugRendererPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundMapItemDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundTextureShiftPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientBoundUpdateSoundDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientCacheBlobStatusPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientCacheMissResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientCacheStatusPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ClientToServerHandshakePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CodeBuilderPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CodeBuilderSourcePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CommandBlockUpdatePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CommandOutputPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CommandRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CompletedUsingItemPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ContainerClosePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ContainerOpenPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ContainerRegistryCleanupPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ContainerSetDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CorrectPlayerMovePredictionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CraftingDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CreatePhotoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CreativeContentPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::CurrentStructureFeaturePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::DeathInfoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::DebugDrawerPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::DebugInfoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::DimensionDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::DisconnectPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::EditorNetworkPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::EduUriResourcePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::EducationSettingsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::EmoteListPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::EmotePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::FeatureRegistryPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::GameRulesChangedPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::GameTestRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::GameTestResultsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::GraphicsParameterOverridePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::GuiDataPickItemPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::HurtArmorPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::InteractPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::InventoryContentPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::InventorySlotPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::InventoryTransactionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ItemComponentPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ItemStackRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ItemStackResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::JigsawStructureDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LabTablePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LecternUpdatePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LegacyTelemetryEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LessonProgressPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LevelChunkPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LevelEventGenericPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LevelEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LevelSoundEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LocatorBarPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::LoginPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MapCreateLockedCopyPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MapInfoRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MobArmorEquipmentPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MobEffectPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MobEquipmentPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ModalFormRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ModalFormResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MotionPredictionHintsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MoveActorAbsolutePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MoveActorDeltaPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MovePlayerPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MovementEffectPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MovementPredictionSyncPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::MultiplayerSettingsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::NetworkChunkPublisherUpdatePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::NetworkSettingsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::NetworkStackLatencyPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::NpcDialoguePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::NpcRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::OnScreenTextureAnimationPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::OpenSignPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PacketViolationWarningPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PartyChangedPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PartyDestinationCookieResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PhotoTransferPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlaySoundPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayStatusPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerActionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerArmorDamagePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerAuthInputPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerEnchantOptionsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerFogPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerHotbarPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerListPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerLocationPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerSkinPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerStartItemCooldownPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerToggleCrafterSlotRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerUpdateEntityOverridesPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PlayerVideoCapturePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PositionTrackingDBClientRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PositionTrackingDBServerBroadcastPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::PurchaseReceiptPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RefreshEntitlementsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RemoveActorPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RemoveObjectivePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RemoveVolumeEntityPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RequestAbilityPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RequestChunkRadiusPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RequestNetworkSettingsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RequestPermissionsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePackChunkDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePackChunkRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePackClientResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePackDataInfoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePackStackPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePacksInfoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ResourcePacksReadyForValidationPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::RespawnPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ScriptMessagePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SendPartyDestinationCookiePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerBoundDataDrivenClosedPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerBoundDataStorePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerBoundDiagnosticsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerBoundLoadingScreenPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerBoundPackSettingChangePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerPlayerPostMovePositionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerPresenceInfoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerSettingsRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerSettingsResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerStatsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerStoreInfoPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ServerToClientHandshakePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetActorDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetActorLinkPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetActorMotionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetCommandsEnabledPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetDefaultGameTypePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetDifficultyPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetDisplayObjectivePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetHealthPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetHudPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetLastHurtByPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetLocalPlayerAsInitializedPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetPlayerGameTypePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetPlayerInventoryOptionsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetScorePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetScoreboardIdentityPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetSpawnPositionPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetTimePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SetTitlePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SettingsCommandPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ShowCreditsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ShowProfilePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ShowStoreOfferPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SimpleEventPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SimulationTypePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SpawnExperienceOrbPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SpawnParticleEffectPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::StartGamePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::StopSoundPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::StructureBlockUpdatePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::StructureDataRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::StructureDataResponsePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SubChunkPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SubChunkRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SubClientLoginPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SyncActorPropertyPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::SyncWorldClocksPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::TakeItemActorPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::TextPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::TickingAreaLoadStatusPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::ToastRequestPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::TransferPlayerPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::TrimDataPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UnlockedRecipesPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateAbilitiesPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateAdventureSettingsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateAttributesPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateBlockPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateBlockSyncedPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateClientInputLocksPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateClientOptionsPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateEquipPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdatePlayerGameTypePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateSoftEnumPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateSubChunkBlocksPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::UpdateTradePacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::VoxelShapesPacket(pk) => {
                        <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V2168Hotfix4::Unknown(pk) => pk.buf.len(),
                }
        }
        #[inline]
        fn id(&self) -> u16 {
            match self {
                V2168Hotfix4::ActorEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ActorPickRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AddActorPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AddBehaviourTreePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AddItemActorPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AddPaintingPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AddPlayerPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AddVolumeEntityPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AgentActionEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AgentAnimationPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AnimateEntityPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AnimatePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AnvilDamagePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AutomationClientConnectPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AvailableActorIdentifiersPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AvailableCommandsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::AwardAchievementPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::BiomeDefinitionListPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::BlockActorDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::BlockEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::BlockPickRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::BookEditPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::BossEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraAimAssistActorPriorityPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistActorPriorityPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraAimAssistInstructionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistInstructionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraAimAssistPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraAimAssistPresetsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraInstructionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraPresetsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraShakePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CameraSplinePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CameraSplinePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ChangeDimensionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ChangeMobPropertyPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ChunkRadiusUpdatedPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundAttributeLayerSyncPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundAttributeLayerSyncPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundCloseFormPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundControlSchemeSetPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundControlSchemeSetPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundDataDrivenUICloseScreenPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUICloseScreenPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundDataDrivenUIReloadPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIReloadPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundDataDrivenUIShowScreenPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataDrivenUIShowScreenPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundDataStorePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDataStorePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundDebugRendererPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundMapItemDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundTextureShiftPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundTextureShiftPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientBoundUpdateSoundDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientBoundUpdateSoundDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientCacheBlobStatusPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientCacheMissResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientCacheStatusPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ClientToServerHandshakePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CodeBuilderPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CodeBuilderSourcePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CommandBlockUpdatePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CommandOutputPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CommandRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CompletedUsingItemPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ContainerClosePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ContainerOpenPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ContainerRegistryCleanupPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ContainerSetDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CorrectPlayerMovePredictionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CraftingDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CreatePhotoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CreativeContentPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::CurrentStructureFeaturePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::DeathInfoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::DebugDrawerPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::DebugDrawerPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::DebugInfoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::DimensionDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::DisconnectPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::EditorNetworkPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::EduUriResourcePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::EducationSettingsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::EmoteListPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::EmotePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::FeatureRegistryPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::GameRulesChangedPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::GameTestRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::GameTestResultsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::GraphicsParameterOverridePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::GraphicsParameterOverridePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::GuiDataPickItemPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::HurtArmorPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::InteractPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::InventoryContentPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::InventorySlotPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::InventoryTransactionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ItemComponentPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ItemStackRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ItemStackResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::JigsawStructureDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LabTablePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LecternUpdatePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LegacyTelemetryEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LessonProgressPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LevelChunkPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LevelEventGenericPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LevelEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LevelSoundEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LocatorBarPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LocatorBarPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::LoginPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MapCreateLockedCopyPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MapInfoRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MobArmorEquipmentPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MobEffectPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MobEquipmentPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ModalFormRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ModalFormResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MotionPredictionHintsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MoveActorAbsolutePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MoveActorDeltaPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MovePlayerPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MovementEffectPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MovementPredictionSyncPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MovementPredictionSyncPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::MultiplayerSettingsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::NetworkChunkPublisherUpdatePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::NetworkSettingsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::NetworkStackLatencyPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::NpcDialoguePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::NpcRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::OnScreenTextureAnimationPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::OpenSignPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PacketViolationWarningPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PartyChangedPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PartyChangedPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PartyDestinationCookieResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PartyDestinationCookieResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PhotoTransferPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlaySoundPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayStatusPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerActionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerArmorDamagePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerAuthInputPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerEnchantOptionsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerFogPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerHotbarPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerListPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerLocationPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerLocationPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerSkinPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerStartItemCooldownPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerToggleCrafterSlotRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerUpdateEntityOverridesPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerUpdateEntityOverridesPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PlayerVideoCapturePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PlayerVideoCapturePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PositionTrackingDBClientRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PositionTrackingDBServerBroadcastPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::PurchaseReceiptPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RefreshEntitlementsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RemoveActorPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RemoveObjectivePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RemoveVolumeEntityPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RequestAbilityPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RequestChunkRadiusPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RequestNetworkSettingsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RequestPermissionsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePackChunkDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePackChunkRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePackClientResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePackDataInfoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePackStackPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePacksInfoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ResourcePacksReadyForValidationPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ResourcePacksReadyForValidationPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::RespawnPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ScriptMessagePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SendPartyDestinationCookiePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SendPartyDestinationCookiePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerBoundDataDrivenClosedPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataDrivenClosedPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerBoundDataStorePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDataStorePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerBoundDiagnosticsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerBoundLoadingScreenPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerBoundPackSettingChangePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerBoundPackSettingChangePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerPlayerPostMovePositionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerPresenceInfoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerPresenceInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerSettingsRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerSettingsResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerStatsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerStoreInfoPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerStoreInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ServerToClientHandshakePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetActorDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetActorLinkPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetActorMotionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetCommandsEnabledPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetDefaultGameTypePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetDifficultyPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetDisplayObjectivePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetHealthPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetHudPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetLastHurtByPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetLocalPlayerAsInitializedPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetPlayerGameTypePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetPlayerInventoryOptionsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetScorePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetScoreboardIdentityPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetSpawnPositionPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetTimePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SetTitlePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SettingsCommandPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ShowCreditsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ShowProfilePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ShowStoreOfferPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SimpleEventPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SimulationTypePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SpawnExperienceOrbPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SpawnParticleEffectPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::StartGamePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::StopSoundPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::StructureBlockUpdatePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::StructureDataRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::StructureDataResponsePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SubChunkPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SubChunkRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SubClientLoginPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SyncActorPropertyPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::SyncWorldClocksPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::SyncWorldClocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::TakeItemActorPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::TextPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::TickingAreaLoadStatusPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::ToastRequestPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::TransferPlayerPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::TrimDataPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UnlockedRecipesPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateAbilitiesPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateAdventureSettingsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateAttributesPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateBlockPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateBlockSyncedPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateClientInputLocksPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateClientOptionsPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateClientOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateEquipPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdatePlayerGameTypePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateSoftEnumPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateSubChunkBlocksPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::UpdateTradePacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::VoxelShapesPacket(_) => {
                    <<V2168Hotfix4 as ProtoVersionPackets>::VoxelShapesPacket as bedrock_protocol_core::Packet>::ID
                }
                V2168Hotfix4::Unknown(pk) => pk.id,
            }
        }
    }
    impl ProtoVersionPackets for V2168Hotfix4 {
        type ActorEventPacket = crate::version::v975::packets::ActorEventPacket<Self>;
        type ActorPickRequestPacket = crate::version::v662::packets::ActorPickRequestPacket;
        type AddActorPacket = crate::version::v662::packets::AddActorPacket<Self>;
        type AddBehaviourTreePacket = crate::version::v662::packets::AddBehaviourTreePacket;
        type AddItemActorPacket = crate::version::v662::packets::AddItemActorPacket<Self>;
        type AddPaintingPacket = crate::version::v662::packets::AddPaintingPacket<Self>;
        type AddPlayerPacket = crate::version::v662::packets::AddPlayerPacket<Self>;
        type AddVolumeEntityPacket = crate::version::v662::packets::AddVolumeEntityPacket<Self>;
        type AgentActionEventPacket = crate::version::v662::packets::AgentActionEventPacket<Self>;
        type AgentAnimationPacket = crate::version::v662::packets::AgentAnimationPacket<Self>;
        type AnimateEntityPacket = crate::version::v662::packets::AnimateEntityPacket<Self>;
        type AnimatePacket = crate::version::v898::packets::AnimatePacket<Self>;
        type AnvilDamagePacket = crate::version::v2168::packets::AnvilDamagePacket<Self>;
        type AutomationClientConnectPacket =
            crate::version::v662::packets::AutomationClientConnectPacket<Self>;
        type AvailableActorIdentifiersPacket =
            crate::version::v662::packets::AvailableActorIdentifiersPacket;
        type AvailableCommandsPacket = crate::version::v898::packets::AvailableCommandsPacket;
        type AwardAchievementPacket = crate::version::v685::packets::AwardAchievementPacket;
        type BiomeDefinitionListPacket =
            crate::version::v800::packets::BiomeDefinitionListPacket<Self>;
        type BlockActorDataPacket = crate::version::v662::packets::BlockActorDataPacket<Self>;
        type BlockEventPacket = crate::version::v662::packets::BlockEventPacket<Self>;
        type BlockPickRequestPacket = crate::version::v662::packets::BlockPickRequestPacket<Self>;
        type BookEditPacket = crate::version::v924::packets::BookEditPacket<Self>;
        type BossEventPacket = crate::version::v1001::packets::BossEventPacket<Self>;
        type CameraAimAssistActorPriorityPacket =
            crate::version::v924::packets::CameraAimAssistActorPriorityPacket;
        type CameraAimAssistInstructionPacket =
            crate::version::v776::packets::CameraAimAssistInstructionPacket<Self>;
        type CameraAimAssistPacket = crate::version::v827::packets::CameraAimAssistPacket<Self>;
        type CameraAimAssistPresetsPacket =
            crate::version::v800::packets::CameraAimAssistPresetsPacket<Self>;
        type CameraInstructionPacket = crate::version::v712::packets::CameraInstructionPacket<Self>;
        type CameraPacket = crate::version::v662::packets::CameraPacket<Self>;
        type CameraPresetsPacket = crate::version::v662::packets::CameraPresetsPacket<Self>;
        type CameraShakePacket = crate::version::v662::packets::CameraShakePacket<Self>;
        type CameraSplinePacket = crate::version::v924::packets::CameraSplinePacket<Self>;
        type ChangeDimensionPacket = crate::version::v712::packets::ChangeDimensionPacket;
        type ChangeMobPropertyPacket = crate::version::v662::packets::ChangeMobPropertyPacket<Self>;
        type ChunkRadiusUpdatedPacket = crate::version::v662::packets::ChunkRadiusUpdatedPacket;
        type ClientBoundAttributeLayerSyncPacket =
            crate::version::v1001::packets::ClientBoundAttributeLayerSyncPacket;
        type ClientBoundCloseFormPacket = crate::version::v686::packets::ClientBoundCloseFormPacket;
        type ClientBoundControlSchemeSetPacket =
            crate::version::v800::packets::ClientBoundControlSchemeSetPacket<Self>;
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket =
            crate::version::v944::packets::ClientBoundDataDrivenUICloseScreenPacket;
        type ClientBoundDataDrivenUIReloadPacket =
            crate::version::v924::packets::ClientBoundDataDrivenUIReloadPacket;
        type ClientBoundDataDrivenUIShowScreenPacket =
            crate::version::v944::packets::ClientBoundDataDrivenUIShowScreenPacket;
        type ClientBoundDataStorePacket = crate::version::v924::packets::ClientBoundDataStorePacket;
        type ClientBoundDebugRendererPacket =
            crate::version::v671::packets::ClientBoundDebugRendererPacket;
        type ClientBoundMapItemDataPacket =
            crate::version::v2168::packets::ClientBoundMapItemDataPacket<Self>;
        type ClientBoundTextureShiftPacket =
            crate::version::v924::packets::ClientBoundTextureShiftPacket;
        type ClientBoundUpdateSoundDataPacket =
            crate::version::v2168::packets::ClientBoundUpdateSoundDataPacket<Self>;
        type ClientCacheBlobStatusPacket =
            crate::version::v1001::packets::ClientCacheBlobStatusPacket;
        type ClientCacheMissResponsePacket =
            crate::version::v662::packets::ClientCacheMissResponsePacket;
        type ClientCacheStatusPacket = crate::version::v662::packets::ClientCacheStatusPacket;
        type ClientToServerHandshakePacket =
            crate::version::v662::packets::ClientToServerHandshakePacket;
        type CodeBuilderPacket = crate::version::v662::packets::CodeBuilderPacket;
        type CodeBuilderSourcePacket = crate::version::v685::packets::CodeBuilderSourcePacket<Self>;
        type CommandBlockUpdatePacket =
            crate::version::v776::packets::CommandBlockUpdatePacket<Self>;
        type CommandOutputPacket = crate::version::v898::packets::CommandOutputPacket<Self>;
        type CommandRequestPacket = crate::version::v898::packets::CommandRequestPacket<Self>;
        type CompletedUsingItemPacket =
            crate::version::v662::packets::CompletedUsingItemPacket<Self>;
        type CompressedBiomeDefinitionListPacket = ();
        type ContainerClosePacket = crate::version::v685::packets::ContainerClosePacket<Self>;
        type ContainerOpenPacket = crate::version::v662::packets::ContainerOpenPacket<Self>;
        type ContainerRegistryCleanupPacket =
            crate::version::v729::packets::ContainerRegistryCleanupPacket<Self>;
        type ContainerSetDataPacket = crate::version::v662::packets::ContainerSetDataPacket<Self>;
        type CorrectPlayerMovePredictionPacket =
            crate::version::v827::packets::CorrectPlayerMovePredictionPacket<Self>;
        type CraftingDataPacket = crate::version::v2168::packets::CraftingDataPacket<Self>;
        type CreatePhotoPacket = crate::version::v662::packets::CreatePhotoPacket;
        type CreativeContentPacket = crate::version::v2168::packets::CreativeContentPacket<Self>;
        type CurrentStructureFeaturePacket =
            crate::version::v712::packets::CurrentStructureFeaturePacket;
        type DeathInfoPacket = crate::version::v662::packets::DeathInfoPacket;
        type DebugDrawerPacket = crate::version::v818::packets::DebugDrawerPacket<Self>;
        type DebugInfoPacket = crate::version::v662::packets::DebugInfoPacket<Self>;
        type DimensionDataPacket = crate::version::v662::packets::DimensionDataPacket<Self>;
        type DisconnectPacket = crate::version::v712::packets::DisconnectPacket<Self>;
        type EditorNetworkPacket = crate::version::v662::packets::EditorNetworkPacket;
        type EduUriResourcePacket = crate::version::v662::packets::EduUriResourcePacket<Self>;
        type EducationSettingsPacket = crate::version::v662::packets::EducationSettingsPacket<Self>;
        type EmoteListPacket = crate::version::v662::packets::EmoteListPacket<Self>;
        type EmotePacket = crate::version::v729::packets::EmotePacket<Self>;
        type FeatureRegistryPacket = crate::version::v662::packets::FeatureRegistryPacket;
        type FilterTextPacket = ();
        type GameRulesChangedPacket = crate::version::v662::packets::GameRulesChangedPacket<Self>;
        type GameTestRequestPacket = crate::version::v662::packets::GameTestRequestPacket<Self>;
        type GameTestResultsPacket = crate::version::v662::packets::GameTestResultsPacket;
        type GraphicsParameterOverridePacket =
            crate::version::v1001::packets::GraphicsParameterOverridePacket;
        type GuiDataPickItemPacket = crate::version::v662::packets::GuiDataPickItemPacket;
        type HurtArmorPacket = crate::version::v662::packets::HurtArmorPacket;
        type InteractPacket = crate::version::v898::packets::InteractPacket<Self>;
        type InventoryContentPacket = crate::version::v1001::packets::InventoryContentPacket<Self>;
        type InventorySlotPacket = crate::version::v975::packets::InventorySlotPacket<Self>;
        type InventoryTransactionPacket =
            crate::version::v1001::packets::InventoryTransactionPacket<Self>;
        type ItemComponentPacket = crate::version::v776::packets::ItemComponentPacket<Self>;
        type ItemStackRequestPacket = crate::version::v662::packets::ItemStackRequestPacket<Self>;
        type ItemStackResponsePacket = crate::version::v662::packets::ItemStackResponsePacket<Self>;
        type JigsawStructureDataPacket = crate::version::v712::packets::JigsawStructureDataPacket;
        type LabTablePacket = crate::version::v662::packets::LabTablePacket<Self>;
        type LecternUpdatePacket = crate::version::v662::packets::LecternUpdatePacket<Self>;
        type LegacyTelemetryEventPacket =
            crate::version::v898::packets::LegacyTelemetryEventPacket<Self>;
        type LessonProgressPacket = crate::version::v662::packets::LessonProgressPacket<Self>;
        type LevelChunkPacket = crate::version::v2168::packets::LevelChunkPacket<Self>;
        type LevelEventGenericPacket = crate::version::v662::packets::LevelEventGenericPacket<Self>;
        type LevelEventPacket = crate::version::v662::packets::LevelEventPacket;
        type LevelSoundEventPacket = crate::version::v1001::packets::LevelSoundEventPacket;
        type LevelSoundEventV1Packet = ();
        type LevelSoundEventV2Packet = ();
        type LocatorBarPacket = crate::version::v975::packets::LocatorBarPacket;
        type LoginPacket = crate::version::v662::packets::LoginPacket;
        type MapCreateLockedCopyPacket =
            crate::version::v662::packets::MapCreateLockedCopyPacket<Self>;
        type MapInfoRequestPacket = crate::version::v662::packets::MapInfoRequestPacket<Self>;
        type MobArmorEquipmentPacket =
            crate::version::v1001::packets::MobArmorEquipmentPacket<Self>;
        type MobEffectPacket = crate::version::v898::packets::MobEffectPacket<Self>;
        type MobEquipmentPacket = crate::version::v975::packets::MobEquipmentPacket<Self>;
        type ModalFormRequestPacket = crate::version::v662::packets::ModalFormRequestPacket;
        type ModalFormResponsePacket = crate::version::v662::packets::ModalFormResponsePacket<Self>;
        type MotionPredictionHintsPacket =
            crate::version::v662::packets::MotionPredictionHintsPacket<Self>;
        type MoveActorAbsolutePacket = crate::version::v662::packets::MoveActorAbsolutePacket<Self>;
        type MoveActorDeltaPacket = crate::version::v662::packets::MoveActorDeltaPacket<Self>;
        type MovePlayerPacket = crate::version::v2168::packets::MovePlayerPacket<Self>;
        type MovementEffectPacket = crate::version::v748::packets::MovementEffectPacket<Self>;
        type MovementPredictionSyncPacket =
            crate::version::v975::packets::MovementPredictionSyncPacket<Self>;
        type MultiplayerSettingsPacket =
            crate::version::v662::packets::MultiplayerSettingsPacket<Self>;
        type NetworkChunkPublisherUpdatePacket =
            crate::version::v662::packets::NetworkChunkPublisherUpdatePacket<Self>;
        type NetworkSettingsPacket = crate::version::v662::packets::NetworkSettingsPacket<Self>;
        type NetworkStackLatencyPacket = crate::version::v662::packets::NetworkStackLatencyPacket;
        type NpcDialoguePacket = crate::version::v662::packets::NpcDialoguePacket;
        type NpcRequestPacket = crate::version::v662::packets::NpcRequestPacket<Self>;
        type OnScreenTextureAnimationPacket =
            crate::version::v662::packets::OnScreenTextureAnimationPacket;
        type OpenSignPacket = crate::version::v662::packets::OpenSignPacket<Self>;
        type PacketViolationWarningPacket =
            crate::version::v662::packets::PacketViolationWarningPacket<Self>;
        type PartyChangedPacket = crate::version::v975::packets::PartyChangedPacket;
        type PartyDestinationCookieResponsePacket =
            crate::version::v1001::packets::PartyDestinationCookieResponsePacket;
        type PassengerJumpPacket = ();
        type PhotoTransferPacket = crate::version::v662::packets::PhotoTransferPacket<Self>;
        type PlaySoundPacket = crate::version::v2168::packets::PlaySoundPacket<Self>;
        type PlayStatusPacket = crate::version::v662::packets::PlayStatusPacket<Self>;
        type PlayerActionPacket = crate::version::v662::packets::PlayerActionPacket<Self>;
        type PlayerArmorDamagePacket = crate::version::v844::packets::PlayerArmorDamagePacket;
        type PlayerAuthInputPacket = crate::version::v2168::packets::PlayerAuthInputPacket<Self>;
        type PlayerEnchantOptionsPacket =
            crate::version::v662::packets::PlayerEnchantOptionsPacket<Self>;
        type PlayerFogPacket = crate::version::v662::packets::PlayerFogPacket;
        type PlayerHotbarPacket = crate::version::v662::packets::PlayerHotbarPacket<Self>;
        type PlayerInputPacket = ();
        type PlayerListPacket = crate::version::v2168::packets::PlayerListPacket<Self>;
        type PlayerLocationPacket = crate::version::v2168::packets::PlayerLocationPacket;
        type PlayerSkinPacket = crate::version::v2168::packets::PlayerSkinPacket<Self>;
        type PlayerStartItemCooldownPacket =
            crate::version::v662::packets::PlayerStartItemCooldownPacket;
        type PlayerToggleCrafterSlotRequestPacket =
            crate::version::v662::packets::PlayerToggleCrafterSlotRequestPacket;
        type PlayerUpdateEntityOverridesPacket =
            crate::version::v2168::packets::PlayerUpdateEntityOverridesPacket<Self>;
        type PlayerVideoCapturePacket = crate::version::v786::packets::PlayerVideoCapturePacket;
        type PositionTrackingDBClientRequestPacket =
            crate::version::v662::packets::PositionTrackingDBClientRequestPacket<Self>;
        type PositionTrackingDBServerBroadcastPacket =
            crate::version::v662::packets::PositionTrackingDBServerBroadcastPacket<Self>;
        type PurchaseReceiptPacket = crate::version::v662::packets::PurchaseReceiptPacket;
        type RefreshEntitlementsPacket = crate::version::v662::packets::RefreshEntitlementsPacket;
        type RemoveActorPacket = crate::version::v662::packets::RemoveActorPacket<Self>;
        type RemoveObjectivePacket = crate::version::v662::packets::RemoveObjectivePacket;
        type RemoveVolumeEntityPacket =
            crate::version::v662::packets::RemoveVolumeEntityPacket<Self>;
        type RequestAbilityPacket = crate::version::v662::packets::RequestAbilityPacket<Self>;
        type RequestChunkRadiusPacket = crate::version::v662::packets::RequestChunkRadiusPacket;
        type RequestNetworkSettingsPacket =
            crate::version::v662::packets::RequestNetworkSettingsPacket;
        type RequestPermissionsPacket =
            crate::version::v662::packets::RequestPermissionsPacket<Self>;
        type ResourcePackChunkDataPacket =
            crate::version::v662::packets::ResourcePackChunkDataPacket;
        type ResourcePackChunkRequestPacket =
            crate::version::v662::packets::ResourcePackChunkRequestPacket;
        type ResourcePackClientResponsePacket =
            crate::version::v2168::packets::ResourcePackClientResponsePacket;
        type ResourcePackDataInfoPacket =
            crate::version::v662::packets::ResourcePackDataInfoPacket<Self>;
        type ResourcePackStackPacket = crate::version::v898::packets::ResourcePackStackPacket<Self>;
        type ResourcePacksInfoPacket = crate::version::v2168::packets::ResourcePacksInfoPacket;
        type ResourcePacksReadyForValidationPacket =
            crate::version::v944::packets::ResourcePacksReadyForValidationPacket;
        type RespawnPacket = crate::version::v662::packets::RespawnPacket<Self>;
        type ScriptMessagePacket = crate::version::v662::packets::ScriptMessagePacket;
        type SendPartyDestinationCookiePacket =
            crate::version::v1001::packets::SendPartyDestinationCookiePacket;
        type ServerBoundDataDrivenClosedPacket =
            crate::version::v944::packets::ServerBoundDataDrivenClosedPacket;
        type ServerBoundDataStorePacket = crate::version::v924::packets::ServerBoundDataStorePacket;
        type ServerBoundDiagnosticsPacket =
            crate::version::v2168::packets::ServerBoundDiagnosticsPacket;
        type ServerBoundLoadingScreenPacket =
            crate::version::v712::packets::ServerBoundLoadingScreenPacket;
        type ServerBoundPackSettingChangePacket =
            crate::version::v844::packets::ServerBoundPackSettingChangePacket;
        type ServerPlayerPostMovePositionPacket =
            crate::version::v662::packets::ServerPlayerPostMovePositionPacket;
        type ServerPresenceInfoPacket = crate::version::v2168::packets::ServerPresenceInfoPacket;
        type ServerSettingsRequestPacket =
            crate::version::v662::packets::ServerSettingsRequestPacket;
        type ServerSettingsResponsePacket =
            crate::version::v662::packets::ServerSettingsResponsePacket;
        type ServerStatsPacket = crate::version::v662::packets::ServerStatsPacket;
        type ServerStoreInfoPacket = crate::version::v975::packets::ServerStoreInfoPacket;
        type ServerToClientHandshakePacket =
            crate::version::v662::packets::ServerToClientHandshakePacket;
        type SetActorDataPacket = crate::version::v662::packets::SetActorDataPacket<Self>;
        type SetActorLinkPacket = crate::version::v662::packets::SetActorLinkPacket<Self>;
        type SetActorMotionPacket = crate::version::v662::packets::SetActorMotionPacket<Self>;
        type SetCommandsEnabledPacket = crate::version::v662::packets::SetCommandsEnabledPacket;
        type SetDefaultGameTypePacket =
            crate::version::v662::packets::SetDefaultGameTypePacket<Self>;
        type SetDifficultyPacket = crate::version::v662::packets::SetDifficultyPacket;
        type SetDisplayObjectivePacket =
            crate::version::v662::packets::SetDisplayObjectivePacket<Self>;
        type SetHealthPacket = crate::version::v662::packets::SetHealthPacket;
        type SetHudPacket = crate::version::v662::packets::SetHudPacket<Self>;
        type SetLastHurtByPacket = crate::version::v662::packets::SetLastHurtByPacket<Self>;
        type SetLocalPlayerAsInitializedPacket =
            crate::version::v662::packets::SetLocalPlayerAsInitializedPacket<Self>;
        type SetMovementAuthorityPacket = ();
        type SetPlayerGameTypePacket = crate::version::v662::packets::SetPlayerGameTypePacket<Self>;
        type SetPlayerInventoryOptionsPacket =
            crate::version::v662::packets::SetPlayerInventoryOptionsPacket<Self>;
        type SetScorePacket = crate::version::v2168_hotfix4::packets::SetScorePacket<Self>;
        type SetScoreboardIdentityPacket =
            crate::version::v2168::packets::SetScoreboardIdentityPacket<Self>;
        type SetSpawnPositionPacket = crate::version::v662::packets::SetSpawnPositionPacket<Self>;
        type SetTimePacket = crate::version::v662::packets::SetTimePacket;
        type SetTitlePacket = crate::version::v712::packets::SetTitlePacket;
        type SettingsCommandPacket = crate::version::v662::packets::SettingsCommandPacket;
        type ShowCreditsPacket = crate::version::v662::packets::ShowCreditsPacket<Self>;
        type ShowProfilePacket = crate::version::v662::packets::ShowProfilePacket;
        type ShowStoreOfferPacket = crate::version::v859::packets::ShowStoreOfferPacket<Self>;
        type SimpleEventPacket = crate::version::v662::packets::SimpleEventPacket;
        type SimulationTypePacket = crate::version::v662::packets::SimulationTypePacket<Self>;
        type SpawnExperienceOrbPacket = crate::version::v662::packets::SpawnExperienceOrbPacket;
        type SpawnParticleEffectPacket =
            crate::version::v662::packets::SpawnParticleEffectPacket<Self>;
        type StartGamePacket = crate::version::v2168::packets::StartGamePacket<Self>;
        type StopSoundPacket = crate::version::v712::packets::StopSoundPacket;
        type StructureBlockUpdatePacket =
            crate::version::v662::packets::StructureBlockUpdatePacket<Self>;
        type StructureDataRequestPacket =
            crate::version::v662::packets::StructureDataRequestPacket<Self>;
        type StructureDataResponsePacket =
            crate::version::v662::packets::StructureDataResponsePacket<Self>;
        type SubChunkPacket = crate::version::v2168::packets::SubChunkPacket<Self>;
        type SubChunkRequestPacket = crate::version::v1001::packets::SubChunkRequestPacket<Self>;
        type SubClientLoginPacket = crate::version::v662::packets::SubClientLoginPacket;
        type SyncActorPropertyPacket = crate::version::v662::packets::SyncActorPropertyPacket;
        type SyncWorldClocksPacket = crate::version::v944::packets::SyncWorldClocksPacket;
        type TakeItemActorPacket = crate::version::v662::packets::TakeItemActorPacket<Self>;
        type TextPacket = crate::version::v898::packets::TextPacket<Self>;
        type TickSyncPacket = ();
        type TickingAreaLoadStatusPacket =
            crate::version::v662::packets::TickingAreaLoadStatusPacket;
        type ToastRequestPacket = crate::version::v662::packets::ToastRequestPacket;
        type TransferPlayerPacket = crate::version::v2168::packets::TransferPlayerPacket<Self>;
        type TrimDataPacket = crate::version::v662::packets::TrimDataPacket;
        type UnlockedRecipesPacket = crate::version::v662::packets::UnlockedRecipesPacket;
        type UpdateAbilitiesPacket = crate::version::v662::packets::UpdateAbilitiesPacket<Self>;
        type UpdateAdventureSettingsPacket =
            crate::version::v662::packets::UpdateAdventureSettingsPacket<Self>;
        type UpdateAttributesPacket = crate::version::v729::packets::UpdateAttributesPacket<Self>;
        type UpdateBlockPacket = crate::version::v662::packets::UpdateBlockPacket<Self>;
        type UpdateBlockSyncedPacket = crate::version::v662::packets::UpdateBlockSyncedPacket<Self>;
        type UpdateClientInputLocksPacket =
            crate::version::v944::packets::UpdateClientInputLocksPacket;
        type UpdateClientOptionsPacket = crate::version::v975::packets::UpdateClientOptionsPacket;
        type UpdateEquipPacket = crate::version::v662::packets::UpdateEquipPacket<Self>;
        type UpdatePlayerGameTypePacket =
            crate::version::v671::packets::UpdatePlayerGameTypePacket<Self>;
        type UpdateSoftEnumPacket = crate::version::v662::packets::UpdateSoftEnumPacket<Self>;
        type UpdateSubChunkBlocksPacket =
            crate::version::v662::packets::UpdateSubChunkBlocksPacket<Self>;
        type UpdateTradePacket = crate::version::v662::packets::UpdateTradePacket<Self>;
        type VoxelShapesPacket = crate::version::v944::packets::VoxelShapesPacket;
    }
    impl ProtoVersionTypes for V2168Hotfix4 {
        type ActorLink = crate::version::v712::types::ActorLink<Self>;
        type ActorRuntimeID = crate::version::v662::types::ActorRuntimeID;
        type ActorUniqueID = crate::version::v662::types::ActorUniqueID;
        type AdventureSettings = crate::version::v662::types::AdventureSettings;
        type BaseDescription = crate::version::v662::types::BaseDescription<Self>;
        type BaseGameVersion = crate::version::v662::types::BaseGameVersion;
        type BiomeCappedSurfaceData = crate::version::v800::types::BiomeCappedSurfaceData;
        type BiomeClimateData = crate::version::v844::types::BiomeClimateData;
        type BiomeConditionalTransformationData =
            crate::version::v800::types::BiomeConditionalTransformationData<Self>;
        type BiomeConsolidatedFeatureList =
            crate::version::v800::types::BiomeConsolidatedFeatureList<Self>;
        type BiomeCoordinateData = crate::version::v800::types::BiomeCoordinateData;
        type BiomeDefinition = crate::version::v844::types::BiomeDefinition<Self>;
        type BiomeDefinitionChunkGenData =
            crate::version::v975::types::BiomeDefinitionChunkGenData<Self>;
        type BiomeElementData = crate::version::v800::types::BiomeElementData<Self>;
        type BiomeLegacyWorldGenRulesData =
            crate::version::v800::types::BiomeLegacyWorldGenRulesData<Self>;
        type BiomeMesaSurfaceData = crate::version::v800::types::BiomeMesaSurfaceData;
        type BiomeMountainParamsData = crate::version::v800::types::BiomeMountainParamsData;
        type BiomeMultinoiseGenRulesData = crate::version::v800::types::BiomeMultinoiseGenRulesData;
        type BiomeNoiseGradientSurfaceData =
            crate::version::v1001::types::BiomeNoiseGradientSurfaceData;
        type BiomeOverworldGenRulesData =
            crate::version::v800::types::BiomeOverworldGenRulesData<Self>;
        type BiomeReplacementData = crate::version::v859::types::BiomeReplacementData;
        type BiomeScatterParamData = crate::version::v800::types::BiomeScatterParamData<Self>;
        type BiomeSurfaceBuilderData = crate::version::v975::types::BiomeSurfaceBuilderData<Self>;
        type BiomeSurfaceMaterialAdjustmentData =
            crate::version::v800::types::BiomeSurfaceMaterialAdjustmentData<Self>;
        type BiomeSurfaceMaterialData = crate::version::v800::types::BiomeSurfaceMaterialData;
        type BiomeWeightedData = crate::version::v800::types::BiomeWeightedData;
        type BiomeWeightedTemperatureData =
            crate::version::v800::types::BiomeWeightedTemperatureData;
        type BlockPos = crate::version::v662::types::BlockPos;
        type CameraAimAssistCategories =
            crate::version::v766::types::CameraAimAssistCategories<Self>;
        type CameraAimAssistCategory = crate::version::v924::types::CameraAimAssistCategory<Self>;
        type CameraAimAssistItemSettings = crate::version::v766::types::CameraAimAssistItemSettings;
        type CameraAimAssistPreset = crate::version::v766::types::CameraAimAssistPreset;
        type CameraAimAssistPresetDefinition =
            crate::version::v924::types::CameraAimAssistPresetDefinition<Self>;
        type CameraAimAssistPriority = crate::version::v766::types::CameraAimAssistPriority;
        type CameraInstruction = crate::version::v924::types::CameraInstruction<Self>;
        type CameraPreset = crate::version::v818::types::CameraPreset<Self>;
        type CameraPresets = crate::version::v662::types::CameraPresets<Self>;
        type CameraSplineInstruction = crate::version::v944::types::CameraSplineInstruction<Self>;
        type ChunkPos = crate::version::v662::types::ChunkPos;
        type Color = crate::version::v800::types::Color;
        type CommandOriginData = crate::version::v898::types::CommandOriginData<Self>;
        type ContainerMixDataEntry = crate::version::v662::types::ContainerMixDataEntry;
        type CraftingDataEntry = crate::version::v662::types::CraftingDataEntry<Self>;
        type CraftingRecipeIngredient = crate::version::v2168::types::CraftingRecipeIngredient;
        type DataItem = crate::version::v662::types::DataItem<Self>;
        type DebugShape = crate::version::v1001::types::DebugShape<Self>;
        type DimensionDefinitionGroup = crate::version::v2168::types::DimensionDefinitionGroup;
        type EduSharedUriResource = crate::version::v662::types::EduSharedUriResource;
        type EducationLevelSettings = crate::version::v662::types::EducationLevelSettings;
        type EntityNetID = crate::version::v662::types::EntityNetID;
        type Experiments = crate::version::v662::types::Experiments;
        type FullContainerName = crate::version::v729::types::FullContainerName<Self>;
        type GameRulesChangedPacketData = crate::version::v844::types::GameRulesChangedPacketData;
        type GatheringsConfig = crate::version::v2168::types::GatheringsConfig;
        type InventoryAction = crate::version::v1001::types::InventoryAction<Self>;
        type InventorySource = crate::version::v662::types::InventorySource<Self>;
        type InventoryTransaction = crate::version::v662::types::InventoryTransaction<Self>;
        type ItemData = crate::version::v662::types::ItemData;
        type ItemEnchants = crate::version::v662::types::ItemEnchants<Self>;
        type ItemStackRequestNetworkItemInstanceDescriptor =
            crate::version::v2168::types::ItemStackRequestNetworkItemInstanceDescriptor<Self>;
        type ItemStackRequestSlotInfo =
            crate::version::v2168::types::ItemStackRequestSlotInfo<Self>;
        type ItemStackResponseContainerInfo =
            crate::version::v712::types::ItemStackResponseContainerInfo<Self>;
        type ItemStackResponseInfo = crate::version::v2168::types::ItemStackResponseInfo<Self>;
        type ItemStackResponseSlotInfo =
            crate::version::v2168::types::ItemStackResponseSlotInfo<Self>;
        type LevelSettings = crate::version::v2168::types::LevelSettings<Self>;
        type MapDecoration = crate::version::v2168::types::MapDecoration;
        type MapItemTrackedActorUniqueID =
            crate::version::v2168::types::MapItemTrackedActorUniqueID<Self>;
        type MaterialReducerDataEntry = crate::version::v662::types::MaterialReducerDataEntry;
        type MolangVariableMap = crate::version::v662::types::MolangVariableMap;
        type MoveActorAbsoluteData = crate::version::v662::types::MoveActorAbsoluteData<Self>;
        type MoveActorDeltaData = crate::version::v2168::types::MoveActorDeltaData<Self>;
        type MovePlayerTeleportData = crate::version::v2168::types::MovePlayerTeleportData;
        type MultiRecipe = crate::version::v2168::types::MultiRecipe;
        type NetworkBlockPosition = crate::version::v944::types::NetworkBlockPosition;
        type NetworkItemInstanceDescriptor =
            crate::version::v2168::types::NetworkItemInstanceDescriptor;
        type NetworkItemStackDescriptor = crate::version::v2168::types::NetworkItemStackDescriptor;
        type NetworkItemStackDescriptorV2 =
            crate::version::v2168::types::NetworkItemStackDescriptorV2;
        type NetworkPermissions = crate::version::v662::types::NetworkPermissions;
        type PackedItemUseLegacyInventoryTransaction =
            crate::version::v944::types::PackedItemUseLegacyInventoryTransaction<Self>;
        type PlayerBlockActionData = crate::version::v2168::types::PlayerBlockActionData<Self>;
        type PositionTrackingId = crate::version::v662::types::PositionTrackingId;
        type PotionMixDataEntry = crate::version::v662::types::PotionMixDataEntry;
        type PropertySyncData = crate::version::v662::types::PropertySyncData;
        type RecipeIngredient = crate::version::v2168::types::RecipeIngredient<Self>;
        type RecipeUnlockingRequirement =
            crate::version::v2168::types::RecipeUnlockingRequirement<Self>;
        type RedactableString = crate::version::v2168::types::RedactableString;
        type ScoreboardId = crate::version::v662::types::ScoreboardId;
        type SerializedAbilitiesData = crate::version::v776::types::SerializedAbilitiesData<Self>;
        type SerializedSkin = crate::version::v2168::types::SerializedSkin<Self>;
        type ShapedRecipe = crate::version::v2168::types::ShapedRecipe<Self>;
        type ShapelessRecipe = crate::version::v2168::types::ShapelessRecipe<Self>;
        type ShulkerBoxRecipe = crate::version::v748::types::ShulkerBoxRecipe<Self>;
        type SmithingTransformRecipe = crate::version::v2168::types::SmithingTransformRecipe<Self>;
        type SmithingTrimRecipe = crate::version::v2168::types::SmithingTrimRecipe<Self>;
        type SoundData = crate::version::v2168::types::SoundData;
        type SpawnSettings = crate::version::v662::types::SpawnSettings<Self>;
        type StructureEditorData = crate::version::v2168::types::StructureEditorData<Self>;
        type StructureSettings = crate::version::v662::types::StructureSettings<Self>;
        type SubChunkPos = crate::version::v2168::types::SubChunkPos;
        type SubChunkPosOffset = crate::version::v662::types::SubChunkPosOffset;
        type SyncedPlayerMovementSettings =
            crate::version::v818::types::SyncedPlayerMovementSettings;
        type WebSocketPacketData = crate::version::v662::types::WebSocketPacketData;
    }
    impl ProtoVersionEnums for V2168Hotfix4 {
        type AbilitiesIndex = crate::version::v776::enums::AbilitiesIndex;
        type ActorBlockSyncMessageID = crate::version::v662::enums::ActorBlockSyncMessageID;
        type ActorDamageCause = crate::version::v662::enums::ActorDamageCause;
        type ActorDataIDs = crate::version::v924::enums::ActorDataIDs;
        type ActorEvent = crate::version::v975::enums::ActorEvent;
        type ActorFlags = crate::version::v2168::enums::ActorFlags;
        type ActorLinkType = crate::version::v662::enums::ActorLinkType;
        type ActorType = crate::version::v662::enums::ActorType;
        type AgentActionType = crate::version::v662::enums::AgentActionType;
        type AimAssistAction = crate::version::v729::enums::AimAssistAction;
        type AnimatedTextureType = crate::version::v2168::enums::AnimatedTextureType;
        type AnimationExpression = crate::version::v2168::enums::AnimationExpression;
        type AnimationMode = crate::version::v662::enums::AnimationMode;
        type ArmSizeType = crate::version::v2168::enums::ArmSizeType;
        type AttributeModifierOperation = crate::version::v662::enums::AttributeModifierOperation;
        type AttributeOperands = crate::version::v662::enums::AttributeOperands;
        type AuthoritativeMovementMode = crate::version::v748::enums::AuthoritativeMovementMode;
        type BookEditAction = crate::version::v924::enums::BookEditAction;
        type BossEventUpdateType = crate::version::v776::enums::BossEventUpdateType<Self>;
        type BuildPlatform = crate::version::v662::enums::BuildPlatform;
        type CameraAimAssistOperation = crate::version::v776::enums::CameraAimAssistOperation;
        type CameraShakeAction = crate::version::v662::enums::CameraShakeAction;
        type CameraShakeType = crate::version::v662::enums::CameraShakeType;
        type CameraSplineEaseType = crate::version::v924::enums::CameraSplineEaseType;
        type CameraSplineType = crate::version::v859::enums::CameraSplineType;
        type ChatRestrictionLevel = crate::version::v662::enums::ChatRestrictionLevel;
        type CodeBuilderCodeStatus = crate::version::v685::enums::CodeBuilderCodeStatus;
        type CodeBuilderStorageCategory = crate::version::v662::enums::CodeBuilderStorageCategory;
        type CodeBuilderStorageOperation = crate::version::v662::enums::CodeBuilderStorageOperation;
        type CommandBlockMode = crate::version::v662::enums::CommandBlockMode;
        type CommandOriginType = crate::version::v898::enums::CommandOriginType;
        type CommandOutputType = crate::version::v898::enums::CommandOutputType;
        type CommandParameterOption = crate::version::v662::enums::CommandParameterOption;
        type CommandPermissionLevel = crate::version::v662::enums::CommandPermissionLevel;
        type ComplexInventoryTransactionType =
            crate::version::v662::enums::ComplexInventoryTransactionType;
        type ConnectionFailReason = crate::version::v1001::enums::ConnectionFailReason;
        type ContainerEnumName = crate::version::v944::enums::ContainerEnumName;
        type ContainerID = crate::version::v662::enums::ContainerID;
        type ContainerType = crate::version::v662::enums::ContainerType;
        type ControlScheme = crate::version::v800::enums::ControlScheme;
        type CraftingDataEntryType = crate::version::v662::enums::CraftingDataEntryType<Self>;
        type DataItemType = crate::version::v2168::enums::DataItemType<Self>;
        type Difficulty = crate::version::v662::enums::Difficulty;
        type EasingType = crate::version::v662::enums::EasingType;
        type EditorWorldType = crate::version::v662::enums::EditorWorldType;
        type EducationEditionOffer = crate::version::v2168::enums::EducationEditionOffer;
        type EnchantType = crate::version::v662::enums::EnchantType;
        type GamePublishSetting = crate::version::v662::enums::GamePublishSetting;
        type GameType = crate::version::v662::enums::GameType;
        type GeneratorType = crate::version::v662::enums::GeneratorType;
        type HudElement = crate::version::v786::enums::HudElement;
        type HudVisibility = crate::version::v786::enums::HudVisibility;
        type IdentityDefinitionType = crate::version::v662::enums::IdentityDefinitionType<Self>;
        type InputMode = crate::version::v662::enums::InputMode;
        type InteractionType = crate::version::v662::enums::InteractionType;
        type InventoryLayout = crate::version::v662::enums::InventoryLayout;
        type InventoryLeftTabIndex = crate::version::v662::enums::InventoryLeftTabIndex;
        type InventoryRightTabIndex = crate::version::v662::enums::InventoryRightTabIndex;
        type InventorySourceFlags = crate::version::v662::enums::InventorySourceFlags;
        type InventorySourceType = crate::version::v662::enums::InventorySourceType<Self>;
        type ItemDescriptorType = crate::version::v2168::enums::ItemDescriptorType;
        type ItemReleaseInventoryTransactionType =
            crate::version::v662::enums::ItemReleaseInventoryTransactionType;
        type ItemStackNetResult = crate::version::v2168::enums::ItemStackNetResult;
        type ItemStackRequestActionType =
            crate::version::v2168::enums::ItemStackRequestActionType<Self>;
        type ItemUseInventoryTransactionType =
            crate::version::v662::enums::ItemUseInventoryTransactionType;
        type ItemUseMethod = crate::version::v662::enums::ItemUseMethod;
        type ItemUseOnActorInventoryTransactionType =
            crate::version::v662::enums::ItemUseOnActorInventoryTransactionType;
        type ItemVersion = crate::version::v776::enums::ItemVersion;
        type LabTableReactionType = crate::version::v662::enums::LabTableReactionType;
        type LessonAction = crate::version::v662::enums::LessonAction;
        type LevelEvent = crate::version::v766::enums::LevelEvent;
        type LevelSoundEventType = crate::version::v2168::enums::LevelSoundEventType;
        type MinecraftPacketIds = crate::version::v662::enums::MinecraftPacketIds;
        type Mirror = crate::version::v662::enums::Mirror;
        type ModalFormCancelReason = crate::version::v662::enums::ModalFormCancelReason;
        type MolangVersion = crate::version::v662::enums::MolangVersion;
        type MovementEffectType = crate::version::v1001::enums::MovementEffectType;
        type MultiplayerSettingsPacketType =
            crate::version::v662::enums::MultiplayerSettingsPacketType;
        type NewInteractionModel = crate::version::v2168::enums::NewInteractionModel;
        type ObjectiveSortOrder = crate::version::v662::enums::ObjectiveSortOrder;
        type POIBlockInteractionType = crate::version::v662::enums::POIBlockInteractionType;
        type PackType = crate::version::v662::enums::PackType;
        type PacketCompressionAlgorithm = crate::version::v662::enums::PacketCompressionAlgorithm;
        type PacketViolationSeverity = crate::version::v662::enums::PacketViolationSeverity;
        type PacketViolationType = crate::version::v662::enums::PacketViolationType;
        type ParticleType = crate::version::v2168::enums::ParticleType;
        type PersonaPieceType = crate::version::v2168::enums::PersonaPieceType;
        type PhotoType = crate::version::v662::enums::PhotoType;
        type PlayStatus = crate::version::v662::enums::PlayStatus;
        type PlayerAuthInputData = crate::version::v2168::enums::PlayerAuthInputData;
        type PlayerPermissionLevel = crate::version::v662::enums::PlayerPermissionLevel;
        type PlayerPositionMode = crate::version::v2168::enums::PlayerPositionMode;
        type PlayerRespawnState = crate::version::v662::enums::PlayerRespawnState;
        type PredictionType = crate::version::v827::enums::PredictionType;
        type ResourcePackResponse = crate::version::v662::enums::ResourcePackResponse;
        type Rotation = crate::version::v662::enums::Rotation;
        type ServerAuthMovementMode = ();
        type ShowStoreOfferRedirectType = crate::version::v662::enums::ShowStoreOfferRedirectType;
        type SimulationType = crate::version::v662::enums::SimulationType;
        type SoftEnumUpdateType = crate::version::v662::enums::SoftEnumUpdateType;
        type SpawnBiomeType = crate::version::v662::enums::SpawnBiomeType;
        type SpawnPositionType = crate::version::v662::enums::SpawnPositionType;
        type StructureBlockType = crate::version::v662::enums::StructureBlockType;
        type StructureRedstoneSaveMode = crate::version::v2168::enums::StructureRedstoneSaveMode;
        type StructureTemplateRequestOperation =
            crate::version::v662::enums::StructureTemplateRequestOperation;
        type StructureTemplateResponseType =
            crate::version::v662::enums::StructureTemplateResponseType;
        type TeleportationCause = crate::version::v662::enums::TeleportationCause;
        type TextPacketType = crate::version::v924::enums::TextPacketType;
        type TextProcessingEventOrigin = crate::version::v662::enums::TextProcessingEventOrigin;
        type UIProfile = crate::version::v662::enums::UIProfile;
    }
    impl ProtoVersion for V2168Hotfix4 {
        const PROTOCOL_VERSION: u32 = 2168u32;
        const PROTOCOL_BRANCH: &str = "r/26_u4-hotfix4";
        const GAME_VERSION: &str = "1.26.44";
        const RAKNET_VERSION: u8 = 11u8;
    }
    #[cfg(feature = "packet-dyn")]
    impl AsRef<dyn bedrock_protocol_core::PacketDyn> for V2168Hotfix4 {
        fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
            match self {
                V2168Hotfix4::ActorEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ActorPickRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AddActorPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AddBehaviourTreePacket(pk) => pk.as_ref(),
                V2168Hotfix4::AddItemActorPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AddPaintingPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AddPlayerPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AddVolumeEntityPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AgentActionEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AgentAnimationPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AnimateEntityPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AnimatePacket(pk) => pk.as_ref(),
                V2168Hotfix4::AnvilDamagePacket(pk) => pk.as_ref(),
                V2168Hotfix4::AutomationClientConnectPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AvailableActorIdentifiersPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AvailableCommandsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::AwardAchievementPacket(pk) => pk.as_ref(),
                V2168Hotfix4::BiomeDefinitionListPacket(pk) => pk.as_ref(),
                V2168Hotfix4::BlockActorDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::BlockEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::BlockPickRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::BookEditPacket(pk) => pk.as_ref(),
                V2168Hotfix4::BossEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraAimAssistActorPriorityPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraAimAssistInstructionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraAimAssistPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraAimAssistPresetsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraInstructionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraPresetsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraShakePacket(pk) => pk.as_ref(),
                V2168Hotfix4::CameraSplinePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ChangeDimensionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ChangeMobPropertyPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ChunkRadiusUpdatedPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundAttributeLayerSyncPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundCloseFormPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundControlSchemeSetPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundDataDrivenUICloseScreenPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundDataDrivenUIReloadPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundDataDrivenUIShowScreenPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundDataStorePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundDebugRendererPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundMapItemDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundTextureShiftPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientBoundUpdateSoundDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientCacheBlobStatusPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientCacheMissResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientCacheStatusPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ClientToServerHandshakePacket(pk) => pk.as_ref(),
                V2168Hotfix4::CodeBuilderPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CodeBuilderSourcePacket(pk) => pk.as_ref(),
                V2168Hotfix4::CommandBlockUpdatePacket(pk) => pk.as_ref(),
                V2168Hotfix4::CommandOutputPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CommandRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CompletedUsingItemPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ContainerClosePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ContainerOpenPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ContainerRegistryCleanupPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ContainerSetDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CorrectPlayerMovePredictionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CraftingDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CreatePhotoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CreativeContentPacket(pk) => pk.as_ref(),
                V2168Hotfix4::CurrentStructureFeaturePacket(pk) => pk.as_ref(),
                V2168Hotfix4::DeathInfoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::DebugDrawerPacket(pk) => pk.as_ref(),
                V2168Hotfix4::DebugInfoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::DimensionDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::DisconnectPacket(pk) => pk.as_ref(),
                V2168Hotfix4::EditorNetworkPacket(pk) => pk.as_ref(),
                V2168Hotfix4::EduUriResourcePacket(pk) => pk.as_ref(),
                V2168Hotfix4::EducationSettingsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::EmoteListPacket(pk) => pk.as_ref(),
                V2168Hotfix4::EmotePacket(pk) => pk.as_ref(),
                V2168Hotfix4::FeatureRegistryPacket(pk) => pk.as_ref(),
                V2168Hotfix4::GameRulesChangedPacket(pk) => pk.as_ref(),
                V2168Hotfix4::GameTestRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::GameTestResultsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::GraphicsParameterOverridePacket(pk) => pk.as_ref(),
                V2168Hotfix4::GuiDataPickItemPacket(pk) => pk.as_ref(),
                V2168Hotfix4::HurtArmorPacket(pk) => pk.as_ref(),
                V2168Hotfix4::InteractPacket(pk) => pk.as_ref(),
                V2168Hotfix4::InventoryContentPacket(pk) => pk.as_ref(),
                V2168Hotfix4::InventorySlotPacket(pk) => pk.as_ref(),
                V2168Hotfix4::InventoryTransactionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ItemComponentPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ItemStackRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ItemStackResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::JigsawStructureDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LabTablePacket(pk) => pk.as_ref(),
                V2168Hotfix4::LecternUpdatePacket(pk) => pk.as_ref(),
                V2168Hotfix4::LegacyTelemetryEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LessonProgressPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LevelChunkPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LevelEventGenericPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LevelEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LevelSoundEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LocatorBarPacket(pk) => pk.as_ref(),
                V2168Hotfix4::LoginPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MapCreateLockedCopyPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MapInfoRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MobArmorEquipmentPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MobEffectPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MobEquipmentPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ModalFormRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ModalFormResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::MotionPredictionHintsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MoveActorAbsolutePacket(pk) => pk.as_ref(),
                V2168Hotfix4::MoveActorDeltaPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MovePlayerPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MovementEffectPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MovementPredictionSyncPacket(pk) => pk.as_ref(),
                V2168Hotfix4::MultiplayerSettingsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::NetworkChunkPublisherUpdatePacket(pk) => pk.as_ref(),
                V2168Hotfix4::NetworkSettingsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::NetworkStackLatencyPacket(pk) => pk.as_ref(),
                V2168Hotfix4::NpcDialoguePacket(pk) => pk.as_ref(),
                V2168Hotfix4::NpcRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::OnScreenTextureAnimationPacket(pk) => pk.as_ref(),
                V2168Hotfix4::OpenSignPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PacketViolationWarningPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PartyChangedPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PartyDestinationCookieResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::PhotoTransferPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlaySoundPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayStatusPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerActionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerArmorDamagePacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerAuthInputPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerEnchantOptionsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerFogPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerHotbarPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerListPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerLocationPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerSkinPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerStartItemCooldownPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerToggleCrafterSlotRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerUpdateEntityOverridesPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PlayerVideoCapturePacket(pk) => pk.as_ref(),
                V2168Hotfix4::PositionTrackingDBClientRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PositionTrackingDBServerBroadcastPacket(pk) => pk.as_ref(),
                V2168Hotfix4::PurchaseReceiptPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RefreshEntitlementsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RemoveActorPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RemoveObjectivePacket(pk) => pk.as_ref(),
                V2168Hotfix4::RemoveVolumeEntityPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RequestAbilityPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RequestChunkRadiusPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RequestPermissionsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePackChunkDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePackChunkRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePackClientResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePackDataInfoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePackStackPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePacksInfoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ResourcePacksReadyForValidationPacket(pk) => pk.as_ref(),
                V2168Hotfix4::RespawnPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ScriptMessagePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SendPartyDestinationCookiePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerBoundDataDrivenClosedPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerBoundDataStorePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerBoundDiagnosticsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerBoundLoadingScreenPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerBoundPackSettingChangePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerPlayerPostMovePositionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerPresenceInfoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerSettingsRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerSettingsResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerStatsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerStoreInfoPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ServerToClientHandshakePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetActorDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetActorLinkPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetActorMotionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetCommandsEnabledPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetDefaultGameTypePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetDifficultyPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetDisplayObjectivePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetHealthPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetHudPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetLastHurtByPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetLocalPlayerAsInitializedPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetPlayerGameTypePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetPlayerInventoryOptionsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetScorePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetScoreboardIdentityPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetSpawnPositionPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetTimePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SetTitlePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SettingsCommandPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ShowCreditsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ShowProfilePacket(pk) => pk.as_ref(),
                V2168Hotfix4::ShowStoreOfferPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SimpleEventPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SimulationTypePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SpawnExperienceOrbPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SpawnParticleEffectPacket(pk) => pk.as_ref(),
                V2168Hotfix4::StartGamePacket(pk) => pk.as_ref(),
                V2168Hotfix4::StopSoundPacket(pk) => pk.as_ref(),
                V2168Hotfix4::StructureBlockUpdatePacket(pk) => pk.as_ref(),
                V2168Hotfix4::StructureDataRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::StructureDataResponsePacket(pk) => pk.as_ref(),
                V2168Hotfix4::SubChunkPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SubChunkRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SubClientLoginPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SyncActorPropertyPacket(pk) => pk.as_ref(),
                V2168Hotfix4::SyncWorldClocksPacket(pk) => pk.as_ref(),
                V2168Hotfix4::TakeItemActorPacket(pk) => pk.as_ref(),
                V2168Hotfix4::TextPacket(pk) => pk.as_ref(),
                V2168Hotfix4::TickingAreaLoadStatusPacket(pk) => pk.as_ref(),
                V2168Hotfix4::ToastRequestPacket(pk) => pk.as_ref(),
                V2168Hotfix4::TransferPlayerPacket(pk) => pk.as_ref(),
                V2168Hotfix4::TrimDataPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UnlockedRecipesPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateAbilitiesPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateAdventureSettingsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateAttributesPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateBlockPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateBlockSyncedPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateClientInputLocksPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateClientOptionsPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateEquipPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdatePlayerGameTypePacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateSoftEnumPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateSubChunkBlocksPacket(pk) => pk.as_ref(),
                V2168Hotfix4::UpdateTradePacket(pk) => pk.as_ref(),
                V2168Hotfix4::VoxelShapesPacket(pk) => pk.as_ref(),
                V2168Hotfix4::Unknown(pk) => pk.as_ref(),
            }
        }
    }
    #[cfg(feature = "packet-dyn")]
    impl From<V2168Hotfix4> for Box<dyn bedrock_protocol_core::PacketDyn> {
        fn from(val: V2168Hotfix4) -> Box<dyn bedrock_protocol_core::PacketDyn> {
            match val {
                V2168Hotfix4::ActorEventPacket(pk) => pk,
                V2168Hotfix4::ActorPickRequestPacket(pk) => pk,
                V2168Hotfix4::AddActorPacket(pk) => pk,
                V2168Hotfix4::AddBehaviourTreePacket(pk) => pk,
                V2168Hotfix4::AddItemActorPacket(pk) => pk,
                V2168Hotfix4::AddPaintingPacket(pk) => pk,
                V2168Hotfix4::AddPlayerPacket(pk) => pk,
                V2168Hotfix4::AddVolumeEntityPacket(pk) => pk,
                V2168Hotfix4::AgentActionEventPacket(pk) => pk,
                V2168Hotfix4::AgentAnimationPacket(pk) => pk,
                V2168Hotfix4::AnimateEntityPacket(pk) => pk,
                V2168Hotfix4::AnimatePacket(pk) => pk,
                V2168Hotfix4::AnvilDamagePacket(pk) => pk,
                V2168Hotfix4::AutomationClientConnectPacket(pk) => pk,
                V2168Hotfix4::AvailableActorIdentifiersPacket(pk) => pk,
                V2168Hotfix4::AvailableCommandsPacket(pk) => pk,
                V2168Hotfix4::AwardAchievementPacket(pk) => pk,
                V2168Hotfix4::BiomeDefinitionListPacket(pk) => pk,
                V2168Hotfix4::BlockActorDataPacket(pk) => pk,
                V2168Hotfix4::BlockEventPacket(pk) => pk,
                V2168Hotfix4::BlockPickRequestPacket(pk) => pk,
                V2168Hotfix4::BookEditPacket(pk) => pk,
                V2168Hotfix4::BossEventPacket(pk) => pk,
                V2168Hotfix4::CameraAimAssistActorPriorityPacket(pk) => pk,
                V2168Hotfix4::CameraAimAssistInstructionPacket(pk) => pk,
                V2168Hotfix4::CameraAimAssistPacket(pk) => pk,
                V2168Hotfix4::CameraAimAssistPresetsPacket(pk) => pk,
                V2168Hotfix4::CameraInstructionPacket(pk) => pk,
                V2168Hotfix4::CameraPacket(pk) => pk,
                V2168Hotfix4::CameraPresetsPacket(pk) => pk,
                V2168Hotfix4::CameraShakePacket(pk) => pk,
                V2168Hotfix4::CameraSplinePacket(pk) => pk,
                V2168Hotfix4::ChangeDimensionPacket(pk) => pk,
                V2168Hotfix4::ChangeMobPropertyPacket(pk) => pk,
                V2168Hotfix4::ChunkRadiusUpdatedPacket(pk) => pk,
                V2168Hotfix4::ClientBoundAttributeLayerSyncPacket(pk) => pk,
                V2168Hotfix4::ClientBoundCloseFormPacket(pk) => pk,
                V2168Hotfix4::ClientBoundControlSchemeSetPacket(pk) => pk,
                V2168Hotfix4::ClientBoundDataDrivenUICloseScreenPacket(pk) => pk,
                V2168Hotfix4::ClientBoundDataDrivenUIReloadPacket(pk) => pk,
                V2168Hotfix4::ClientBoundDataDrivenUIShowScreenPacket(pk) => pk,
                V2168Hotfix4::ClientBoundDataStorePacket(pk) => pk,
                V2168Hotfix4::ClientBoundDebugRendererPacket(pk) => pk,
                V2168Hotfix4::ClientBoundMapItemDataPacket(pk) => pk,
                V2168Hotfix4::ClientBoundTextureShiftPacket(pk) => pk,
                V2168Hotfix4::ClientBoundUpdateSoundDataPacket(pk) => pk,
                V2168Hotfix4::ClientCacheBlobStatusPacket(pk) => pk,
                V2168Hotfix4::ClientCacheMissResponsePacket(pk) => pk,
                V2168Hotfix4::ClientCacheStatusPacket(pk) => pk,
                V2168Hotfix4::ClientToServerHandshakePacket(pk) => pk,
                V2168Hotfix4::CodeBuilderPacket(pk) => pk,
                V2168Hotfix4::CodeBuilderSourcePacket(pk) => pk,
                V2168Hotfix4::CommandBlockUpdatePacket(pk) => pk,
                V2168Hotfix4::CommandOutputPacket(pk) => pk,
                V2168Hotfix4::CommandRequestPacket(pk) => pk,
                V2168Hotfix4::CompletedUsingItemPacket(pk) => pk,
                V2168Hotfix4::ContainerClosePacket(pk) => pk,
                V2168Hotfix4::ContainerOpenPacket(pk) => pk,
                V2168Hotfix4::ContainerRegistryCleanupPacket(pk) => pk,
                V2168Hotfix4::ContainerSetDataPacket(pk) => pk,
                V2168Hotfix4::CorrectPlayerMovePredictionPacket(pk) => pk,
                V2168Hotfix4::CraftingDataPacket(pk) => pk,
                V2168Hotfix4::CreatePhotoPacket(pk) => pk,
                V2168Hotfix4::CreativeContentPacket(pk) => pk,
                V2168Hotfix4::CurrentStructureFeaturePacket(pk) => pk,
                V2168Hotfix4::DeathInfoPacket(pk) => pk,
                V2168Hotfix4::DebugDrawerPacket(pk) => pk,
                V2168Hotfix4::DebugInfoPacket(pk) => pk,
                V2168Hotfix4::DimensionDataPacket(pk) => pk,
                V2168Hotfix4::DisconnectPacket(pk) => pk,
                V2168Hotfix4::EditorNetworkPacket(pk) => pk,
                V2168Hotfix4::EduUriResourcePacket(pk) => pk,
                V2168Hotfix4::EducationSettingsPacket(pk) => pk,
                V2168Hotfix4::EmoteListPacket(pk) => pk,
                V2168Hotfix4::EmotePacket(pk) => pk,
                V2168Hotfix4::FeatureRegistryPacket(pk) => pk,
                V2168Hotfix4::GameRulesChangedPacket(pk) => pk,
                V2168Hotfix4::GameTestRequestPacket(pk) => pk,
                V2168Hotfix4::GameTestResultsPacket(pk) => pk,
                V2168Hotfix4::GraphicsParameterOverridePacket(pk) => pk,
                V2168Hotfix4::GuiDataPickItemPacket(pk) => pk,
                V2168Hotfix4::HurtArmorPacket(pk) => pk,
                V2168Hotfix4::InteractPacket(pk) => pk,
                V2168Hotfix4::InventoryContentPacket(pk) => pk,
                V2168Hotfix4::InventorySlotPacket(pk) => pk,
                V2168Hotfix4::InventoryTransactionPacket(pk) => pk,
                V2168Hotfix4::ItemComponentPacket(pk) => pk,
                V2168Hotfix4::ItemStackRequestPacket(pk) => pk,
                V2168Hotfix4::ItemStackResponsePacket(pk) => pk,
                V2168Hotfix4::JigsawStructureDataPacket(pk) => pk,
                V2168Hotfix4::LabTablePacket(pk) => pk,
                V2168Hotfix4::LecternUpdatePacket(pk) => pk,
                V2168Hotfix4::LegacyTelemetryEventPacket(pk) => pk,
                V2168Hotfix4::LessonProgressPacket(pk) => pk,
                V2168Hotfix4::LevelChunkPacket(pk) => pk,
                V2168Hotfix4::LevelEventGenericPacket(pk) => pk,
                V2168Hotfix4::LevelEventPacket(pk) => pk,
                V2168Hotfix4::LevelSoundEventPacket(pk) => pk,
                V2168Hotfix4::LocatorBarPacket(pk) => pk,
                V2168Hotfix4::LoginPacket(pk) => pk,
                V2168Hotfix4::MapCreateLockedCopyPacket(pk) => pk,
                V2168Hotfix4::MapInfoRequestPacket(pk) => pk,
                V2168Hotfix4::MobArmorEquipmentPacket(pk) => pk,
                V2168Hotfix4::MobEffectPacket(pk) => pk,
                V2168Hotfix4::MobEquipmentPacket(pk) => pk,
                V2168Hotfix4::ModalFormRequestPacket(pk) => pk,
                V2168Hotfix4::ModalFormResponsePacket(pk) => pk,
                V2168Hotfix4::MotionPredictionHintsPacket(pk) => pk,
                V2168Hotfix4::MoveActorAbsolutePacket(pk) => pk,
                V2168Hotfix4::MoveActorDeltaPacket(pk) => pk,
                V2168Hotfix4::MovePlayerPacket(pk) => pk,
                V2168Hotfix4::MovementEffectPacket(pk) => pk,
                V2168Hotfix4::MovementPredictionSyncPacket(pk) => pk,
                V2168Hotfix4::MultiplayerSettingsPacket(pk) => pk,
                V2168Hotfix4::NetworkChunkPublisherUpdatePacket(pk) => pk,
                V2168Hotfix4::NetworkSettingsPacket(pk) => pk,
                V2168Hotfix4::NetworkStackLatencyPacket(pk) => pk,
                V2168Hotfix4::NpcDialoguePacket(pk) => pk,
                V2168Hotfix4::NpcRequestPacket(pk) => pk,
                V2168Hotfix4::OnScreenTextureAnimationPacket(pk) => pk,
                V2168Hotfix4::OpenSignPacket(pk) => pk,
                V2168Hotfix4::PacketViolationWarningPacket(pk) => pk,
                V2168Hotfix4::PartyChangedPacket(pk) => pk,
                V2168Hotfix4::PartyDestinationCookieResponsePacket(pk) => pk,
                V2168Hotfix4::PhotoTransferPacket(pk) => pk,
                V2168Hotfix4::PlaySoundPacket(pk) => pk,
                V2168Hotfix4::PlayStatusPacket(pk) => pk,
                V2168Hotfix4::PlayerActionPacket(pk) => pk,
                V2168Hotfix4::PlayerArmorDamagePacket(pk) => pk,
                V2168Hotfix4::PlayerAuthInputPacket(pk) => pk,
                V2168Hotfix4::PlayerEnchantOptionsPacket(pk) => pk,
                V2168Hotfix4::PlayerFogPacket(pk) => pk,
                V2168Hotfix4::PlayerHotbarPacket(pk) => pk,
                V2168Hotfix4::PlayerListPacket(pk) => pk,
                V2168Hotfix4::PlayerLocationPacket(pk) => pk,
                V2168Hotfix4::PlayerSkinPacket(pk) => pk,
                V2168Hotfix4::PlayerStartItemCooldownPacket(pk) => pk,
                V2168Hotfix4::PlayerToggleCrafterSlotRequestPacket(pk) => pk,
                V2168Hotfix4::PlayerUpdateEntityOverridesPacket(pk) => pk,
                V2168Hotfix4::PlayerVideoCapturePacket(pk) => pk,
                V2168Hotfix4::PositionTrackingDBClientRequestPacket(pk) => pk,
                V2168Hotfix4::PositionTrackingDBServerBroadcastPacket(pk) => pk,
                V2168Hotfix4::PurchaseReceiptPacket(pk) => pk,
                V2168Hotfix4::RefreshEntitlementsPacket(pk) => pk,
                V2168Hotfix4::RemoveActorPacket(pk) => pk,
                V2168Hotfix4::RemoveObjectivePacket(pk) => pk,
                V2168Hotfix4::RemoveVolumeEntityPacket(pk) => pk,
                V2168Hotfix4::RequestAbilityPacket(pk) => pk,
                V2168Hotfix4::RequestChunkRadiusPacket(pk) => pk,
                V2168Hotfix4::RequestNetworkSettingsPacket(pk) => pk,
                V2168Hotfix4::RequestPermissionsPacket(pk) => pk,
                V2168Hotfix4::ResourcePackChunkDataPacket(pk) => pk,
                V2168Hotfix4::ResourcePackChunkRequestPacket(pk) => pk,
                V2168Hotfix4::ResourcePackClientResponsePacket(pk) => pk,
                V2168Hotfix4::ResourcePackDataInfoPacket(pk) => pk,
                V2168Hotfix4::ResourcePackStackPacket(pk) => pk,
                V2168Hotfix4::ResourcePacksInfoPacket(pk) => pk,
                V2168Hotfix4::ResourcePacksReadyForValidationPacket(pk) => pk,
                V2168Hotfix4::RespawnPacket(pk) => pk,
                V2168Hotfix4::ScriptMessagePacket(pk) => pk,
                V2168Hotfix4::SendPartyDestinationCookiePacket(pk) => pk,
                V2168Hotfix4::ServerBoundDataDrivenClosedPacket(pk) => pk,
                V2168Hotfix4::ServerBoundDataStorePacket(pk) => pk,
                V2168Hotfix4::ServerBoundDiagnosticsPacket(pk) => pk,
                V2168Hotfix4::ServerBoundLoadingScreenPacket(pk) => pk,
                V2168Hotfix4::ServerBoundPackSettingChangePacket(pk) => pk,
                V2168Hotfix4::ServerPlayerPostMovePositionPacket(pk) => pk,
                V2168Hotfix4::ServerPresenceInfoPacket(pk) => pk,
                V2168Hotfix4::ServerSettingsRequestPacket(pk) => pk,
                V2168Hotfix4::ServerSettingsResponsePacket(pk) => pk,
                V2168Hotfix4::ServerStatsPacket(pk) => pk,
                V2168Hotfix4::ServerStoreInfoPacket(pk) => pk,
                V2168Hotfix4::ServerToClientHandshakePacket(pk) => pk,
                V2168Hotfix4::SetActorDataPacket(pk) => pk,
                V2168Hotfix4::SetActorLinkPacket(pk) => pk,
                V2168Hotfix4::SetActorMotionPacket(pk) => pk,
                V2168Hotfix4::SetCommandsEnabledPacket(pk) => pk,
                V2168Hotfix4::SetDefaultGameTypePacket(pk) => pk,
                V2168Hotfix4::SetDifficultyPacket(pk) => pk,
                V2168Hotfix4::SetDisplayObjectivePacket(pk) => pk,
                V2168Hotfix4::SetHealthPacket(pk) => pk,
                V2168Hotfix4::SetHudPacket(pk) => pk,
                V2168Hotfix4::SetLastHurtByPacket(pk) => pk,
                V2168Hotfix4::SetLocalPlayerAsInitializedPacket(pk) => pk,
                V2168Hotfix4::SetPlayerGameTypePacket(pk) => pk,
                V2168Hotfix4::SetPlayerInventoryOptionsPacket(pk) => pk,
                V2168Hotfix4::SetScorePacket(pk) => pk,
                V2168Hotfix4::SetScoreboardIdentityPacket(pk) => pk,
                V2168Hotfix4::SetSpawnPositionPacket(pk) => pk,
                V2168Hotfix4::SetTimePacket(pk) => pk,
                V2168Hotfix4::SetTitlePacket(pk) => pk,
                V2168Hotfix4::SettingsCommandPacket(pk) => pk,
                V2168Hotfix4::ShowCreditsPacket(pk) => pk,
                V2168Hotfix4::ShowProfilePacket(pk) => pk,
                V2168Hotfix4::ShowStoreOfferPacket(pk) => pk,
                V2168Hotfix4::SimpleEventPacket(pk) => pk,
                V2168Hotfix4::SimulationTypePacket(pk) => pk,
                V2168Hotfix4::SpawnExperienceOrbPacket(pk) => pk,
                V2168Hotfix4::SpawnParticleEffectPacket(pk) => pk,
                V2168Hotfix4::StartGamePacket(pk) => pk,
                V2168Hotfix4::StopSoundPacket(pk) => pk,
                V2168Hotfix4::StructureBlockUpdatePacket(pk) => pk,
                V2168Hotfix4::StructureDataRequestPacket(pk) => pk,
                V2168Hotfix4::StructureDataResponsePacket(pk) => pk,
                V2168Hotfix4::SubChunkPacket(pk) => pk,
                V2168Hotfix4::SubChunkRequestPacket(pk) => pk,
                V2168Hotfix4::SubClientLoginPacket(pk) => pk,
                V2168Hotfix4::SyncActorPropertyPacket(pk) => pk,
                V2168Hotfix4::SyncWorldClocksPacket(pk) => pk,
                V2168Hotfix4::TakeItemActorPacket(pk) => pk,
                V2168Hotfix4::TextPacket(pk) => pk,
                V2168Hotfix4::TickingAreaLoadStatusPacket(pk) => pk,
                V2168Hotfix4::ToastRequestPacket(pk) => pk,
                V2168Hotfix4::TransferPlayerPacket(pk) => pk,
                V2168Hotfix4::TrimDataPacket(pk) => pk,
                V2168Hotfix4::UnlockedRecipesPacket(pk) => pk,
                V2168Hotfix4::UpdateAbilitiesPacket(pk) => pk,
                V2168Hotfix4::UpdateAdventureSettingsPacket(pk) => pk,
                V2168Hotfix4::UpdateAttributesPacket(pk) => pk,
                V2168Hotfix4::UpdateBlockPacket(pk) => pk,
                V2168Hotfix4::UpdateBlockSyncedPacket(pk) => pk,
                V2168Hotfix4::UpdateClientInputLocksPacket(pk) => pk,
                V2168Hotfix4::UpdateClientOptionsPacket(pk) => pk,
                V2168Hotfix4::UpdateEquipPacket(pk) => pk,
                V2168Hotfix4::UpdatePlayerGameTypePacket(pk) => pk,
                V2168Hotfix4::UpdateSoftEnumPacket(pk) => pk,
                V2168Hotfix4::UpdateSubChunkBlocksPacket(pk) => pk,
                V2168Hotfix4::UpdateTradePacket(pk) => pk,
                V2168Hotfix4::VoxelShapesPacket(pk) => pk,
                V2168Hotfix4::Unknown(pk) => pk,
            }
        }
    }
}
#[cfg(feature = "v2168hotfix4")]
pub use inner::*;
