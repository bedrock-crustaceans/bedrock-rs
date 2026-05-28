#![allow(unused)]
#[cfg(feature = "v766")]
mod inner {
    use crate::ProtoVersion;
    use crate::ProtoVersionEnums;
    use crate::ProtoVersionPackets;
    use crate::ProtoVersionTypes;
    #[derive(Clone, std::fmt::Debug)]
    pub enum V766 {
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
        CameraAimAssistPacket(Box<<Self as ProtoVersionPackets>::CameraAimAssistPacket>),
        CameraAimAssistPresetsPacket(
            Box<<Self as ProtoVersionPackets>::CameraAimAssistPresetsPacket>,
        ),
        CameraInstructionPacket(Box<<Self as ProtoVersionPackets>::CameraInstructionPacket>),
        CameraPacket(Box<<Self as ProtoVersionPackets>::CameraPacket>),
        CameraPresetsPacket(Box<<Self as ProtoVersionPackets>::CameraPresetsPacket>),
        CameraShakePacket(Box<<Self as ProtoVersionPackets>::CameraShakePacket>),
        ChangeDimensionPacket(Box<<Self as ProtoVersionPackets>::ChangeDimensionPacket>),
        ChangeMobPropertyPacket(Box<<Self as ProtoVersionPackets>::ChangeMobPropertyPacket>),
        ChunkRadiusUpdatedPacket(Box<<Self as ProtoVersionPackets>::ChunkRadiusUpdatedPacket>),
        ClientBoundCloseFormPacket(Box<<Self as ProtoVersionPackets>::ClientBoundCloseFormPacket>),
        ClientBoundDebugRendererPacket(
            Box<<Self as ProtoVersionPackets>::ClientBoundDebugRendererPacket>,
        ),
        ClientBoundMapItemDataPacket(
            Box<<Self as ProtoVersionPackets>::ClientBoundMapItemDataPacket>,
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
        CompressedBiomeDefinitionListPacket(
            Box<<Self as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket>,
        ),
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
        LevelSoundEventV1Packet(Box<<Self as ProtoVersionPackets>::LevelSoundEventV1Packet>),
        LevelSoundEventV2Packet(Box<<Self as ProtoVersionPackets>::LevelSoundEventV2Packet>),
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
        PassengerJumpPacket(Box<<Self as ProtoVersionPackets>::PassengerJumpPacket>),
        PhotoTransferPacket(Box<<Self as ProtoVersionPackets>::PhotoTransferPacket>),
        PlaySoundPacket(Box<<Self as ProtoVersionPackets>::PlaySoundPacket>),
        PlayStatusPacket(Box<<Self as ProtoVersionPackets>::PlayStatusPacket>),
        PlayerActionPacket(Box<<Self as ProtoVersionPackets>::PlayerActionPacket>),
        PlayerArmorDamagePacket(Box<<Self as ProtoVersionPackets>::PlayerArmorDamagePacket>),
        PlayerAuthInputPacket(Box<<Self as ProtoVersionPackets>::PlayerAuthInputPacket>),
        PlayerEnchantOptionsPacket(Box<<Self as ProtoVersionPackets>::PlayerEnchantOptionsPacket>),
        PlayerFogPacket(Box<<Self as ProtoVersionPackets>::PlayerFogPacket>),
        PlayerHotbarPacket(Box<<Self as ProtoVersionPackets>::PlayerHotbarPacket>),
        PlayerInputPacket(Box<<Self as ProtoVersionPackets>::PlayerInputPacket>),
        PlayerListPacket(Box<<Self as ProtoVersionPackets>::PlayerListPacket>),
        PlayerSkinPacket(Box<<Self as ProtoVersionPackets>::PlayerSkinPacket>),
        PlayerStartItemCooldownPacket(
            Box<<Self as ProtoVersionPackets>::PlayerStartItemCooldownPacket>,
        ),
        PlayerToggleCrafterSlotRequestPacket(
            Box<<Self as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket>,
        ),
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
        RespawnPacket(Box<<Self as ProtoVersionPackets>::RespawnPacket>),
        ScriptMessagePacket(Box<<Self as ProtoVersionPackets>::ScriptMessagePacket>),
        ServerBoundDiagnosticsPacket(
            Box<<Self as ProtoVersionPackets>::ServerBoundDiagnosticsPacket>,
        ),
        ServerBoundLoadingScreenPacket(
            Box<<Self as ProtoVersionPackets>::ServerBoundLoadingScreenPacket>,
        ),
        ServerPlayerPostMovePositionPacket(
            Box<<Self as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket>,
        ),
        ServerSettingsRequestPacket(
            Box<<Self as ProtoVersionPackets>::ServerSettingsRequestPacket>,
        ),
        ServerSettingsResponsePacket(
            Box<<Self as ProtoVersionPackets>::ServerSettingsResponsePacket>,
        ),
        ServerStatsPacket(Box<<Self as ProtoVersionPackets>::ServerStatsPacket>),
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
        SetMovementAuthorityPacket(Box<<Self as ProtoVersionPackets>::SetMovementAuthorityPacket>),
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
        UpdateEquipPacket(Box<<Self as ProtoVersionPackets>::UpdateEquipPacket>),
        UpdatePlayerGameTypePacket(Box<<Self as ProtoVersionPackets>::UpdatePlayerGameTypePacket>),
        UpdateSoftEnumPacket(Box<<Self as ProtoVersionPackets>::UpdateSoftEnumPacket>),
        UpdateSubChunkBlocksPacket(Box<<Self as ProtoVersionPackets>::UpdateSubChunkBlocksPacket>),
        UpdateTradePacket(Box<<Self as ProtoVersionPackets>::UpdateTradePacket>),
        Unknown(Box<bedrock_protocol_core::UnknownPacket>),
    }
    impl bedrock_protocol_core::Packets for V766 {
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
                V766::ActorEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ActorPickRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AddActorPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AddBehaviourTreePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AddItemActorPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AddPaintingPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AddPlayerPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AddVolumeEntityPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AgentActionEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AgentAnimationPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AnimateEntityPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AnimatePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AnvilDamagePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AutomationClientConnectPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AvailableActorIdentifiersPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AvailableCommandsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::AwardAchievementPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AwardAchievementPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::BiomeDefinitionListPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::BlockActorDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::BlockEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::BlockPickRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::BookEditPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::BossEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CameraAimAssistPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CameraAimAssistPresetsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPresetsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CameraInstructionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CameraPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CameraPresetsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CameraShakePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ChangeDimensionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ChangeMobPropertyPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ChunkRadiusUpdatedPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientBoundCloseFormPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundCloseFormPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientBoundDebugRendererPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientBoundMapItemDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientCacheBlobStatusPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientCacheMissResponsePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientCacheStatusPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ClientToServerHandshakePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CodeBuilderPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CodeBuilderSourcePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CommandBlockUpdatePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CommandOutputPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CommandRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CompletedUsingItemPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CompressedBiomeDefinitionListPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    CompressedBiomeDefinitionListPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ContainerClosePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ContainerOpenPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ContainerRegistryCleanupPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerRegistryCleanupPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ContainerSetDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CorrectPlayerMovePredictionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CraftingDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CreatePhotoPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CreativeContentPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::CurrentStructureFeaturePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CurrentStructureFeaturePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::DeathInfoPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::DebugInfoPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::DimensionDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::DisconnectPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::EditorNetworkPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::EduUriResourcePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::EducationSettingsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::EmoteListPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::EmotePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::FeatureRegistryPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::GameRulesChangedPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::GameTestRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::GameTestResultsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::GuiDataPickItemPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::HurtArmorPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::InteractPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::InventoryContentPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::InventorySlotPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::InventoryTransactionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ItemComponentPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ItemStackRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ItemStackResponsePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::JigsawStructureDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(JigsawStructureDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LabTablePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LecternUpdatePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LegacyTelemetryEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LessonProgressPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LevelChunkPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LevelEventGenericPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LevelEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LevelSoundEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LevelSoundEventV1Packet(pk) => {
                    match <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV1Packet),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LevelSoundEventV2Packet(pk) => {
                    match <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV2Packet),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::LoginPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MapCreateLockedCopyPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MapInfoRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MobArmorEquipmentPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MobEffectPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MobEquipmentPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ModalFormRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ModalFormResponsePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MotionPredictionHintsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MoveActorAbsolutePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MoveActorDeltaPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MovePlayerPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MovementEffectPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovementEffectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::MultiplayerSettingsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::NetworkChunkPublisherUpdatePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::NetworkSettingsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::NetworkStackLatencyPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::NpcDialoguePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::NpcRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::OnScreenTextureAnimationPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::OpenSignPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PacketViolationWarningPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PassengerJumpPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PassengerJumpPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PhotoTransferPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlaySoundPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayStatusPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerActionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerArmorDamagePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerAuthInputPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerEnchantOptionsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerFogPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerHotbarPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerInputPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerInputPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerListPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerSkinPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerStartItemCooldownPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PlayerToggleCrafterSlotRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PositionTrackingDBClientRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PositionTrackingDBServerBroadcastPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::PurchaseReceiptPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RefreshEntitlementsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RemoveActorPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RemoveObjectivePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RemoveVolumeEntityPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RequestAbilityPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RequestChunkRadiusPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RequestNetworkSettingsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RequestPermissionsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ResourcePackChunkDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ResourcePackChunkRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ResourcePackClientResponsePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ResourcePackDataInfoPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ResourcePackStackPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ResourcePacksInfoPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::RespawnPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ScriptMessagePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerBoundDiagnosticsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDiagnosticsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerBoundLoadingScreenPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundLoadingScreenPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerPlayerPostMovePositionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerSettingsRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerSettingsResponsePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerStatsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ServerToClientHandshakePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetActorDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetActorLinkPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetActorMotionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetCommandsEnabledPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetDefaultGameTypePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetDifficultyPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetDisplayObjectivePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetHealthPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetHudPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetLastHurtByPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetLocalPlayerAsInitializedPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetMovementAuthorityPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetMovementAuthorityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetPlayerGameTypePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetPlayerInventoryOptionsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetScorePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetScoreboardIdentityPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetSpawnPositionPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetTimePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SetTitlePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SettingsCommandPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ShowCreditsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ShowProfilePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ShowStoreOfferPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SimpleEventPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SimulationTypePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SpawnExperienceOrbPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SpawnParticleEffectPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::StartGamePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::StopSoundPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::StructureBlockUpdatePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::StructureDataRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::StructureDataResponsePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SubChunkPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SubChunkRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SubClientLoginPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::SyncActorPropertyPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::TakeItemActorPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::TextPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::TickingAreaLoadStatusPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::ToastRequestPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::TransferPlayerPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::TrimDataPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UnlockedRecipesPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateAbilitiesPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateAdventureSettingsPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateAttributesPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateBlockPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateBlockSyncedPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateClientInputLocksPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateEquipPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdatePlayerGameTypePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateSoftEnumPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateSubChunkBlocksPacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::UpdateTradePacket(pk) => {
                    match <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V766::Unknown(pk) => stream.write_all(pk.buf.as_ref()).map_err(|e| {
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
                <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ActorEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ActorPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AddActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AddBehaviourTreePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AddItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AddPaintingPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AddPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AddVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AgentActionEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AgentAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AnimateEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AnimatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AnvilDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AutomationClientConnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AvailableActorIdentifiersPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AvailableCommandsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::AwardAchievementPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AwardAchievementPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::BiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::BlockActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::BlockEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::BlockPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::BookEditPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::BossEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CameraAimAssistPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CameraAimAssistPresetsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraAimAssistPresetsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CameraInstructionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CameraPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CameraPresetsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CameraShakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ChangeDimensionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ChangeMobPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ChunkRadiusUpdatedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientBoundCloseFormPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundCloseFormPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientBoundDebugRendererPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientBoundMapItemDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientCacheBlobStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientCacheMissResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientCacheStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ClientToServerHandshakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CodeBuilderPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CodeBuilderSourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CommandBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CommandOutputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CommandRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CompletedUsingItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CompressedBiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    CompressedBiomeDefinitionListPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ContainerClosePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ContainerOpenPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ContainerRegistryCleanupPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerRegistryCleanupPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ContainerSetDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CorrectPlayerMovePredictionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CraftingDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CreatePhotoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CreativeContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::CurrentStructureFeaturePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CurrentStructureFeaturePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::DeathInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::DebugInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::DimensionDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::DisconnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::EditorNetworkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::EduUriResourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::EducationSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::EmoteListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::EmotePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::FeatureRegistryPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::GameRulesChangedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::GameTestRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::GameTestResultsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::GuiDataPickItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::HurtArmorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::InteractPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::InventoryContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::InventorySlotPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::InventoryTransactionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ItemComponentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ItemStackRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ItemStackResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::JigsawStructureDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(JigsawStructureDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LabTablePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LecternUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LegacyTelemetryEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LessonProgressPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LevelChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LevelEventGenericPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LevelEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LevelSoundEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LevelSoundEventV1Packet(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV1Packet),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LevelSoundEventV2Packet(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV2Packet),
                                packet_id: <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::LoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MapCreateLockedCopyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MapInfoRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MobArmorEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MobEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MobEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ModalFormRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ModalFormResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MotionPredictionHintsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MoveActorAbsolutePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MoveActorDeltaPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MovePlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MovementEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovementEffectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::MultiplayerSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::NetworkChunkPublisherUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::NetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::NetworkStackLatencyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::NpcDialoguePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::NpcRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::OnScreenTextureAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::OpenSignPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PacketViolationWarningPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PassengerJumpPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PassengerJumpPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PhotoTransferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlaySoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerActionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerArmorDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerAuthInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerEnchantOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerFogPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerHotbarPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerInputPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerSkinPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PlayerStartItemCooldownPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V766::PlayerToggleCrafterSlotRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V766::PositionTrackingDBClientRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V766::PositionTrackingDBServerBroadcastPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::PurchaseReceiptPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RefreshEntitlementsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RemoveActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RemoveObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RemoveVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RequestAbilityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RequestChunkRadiusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RequestNetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RequestPermissionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ResourcePackChunkDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ResourcePackChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ResourcePackClientResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ResourcePackDataInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ResourcePackStackPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ResourcePacksInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::RespawnPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ScriptMessagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerBoundDiagnosticsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundDiagnosticsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerBoundLoadingScreenPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerBoundLoadingScreenPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerPlayerPostMovePositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerSettingsRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerSettingsResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerStatsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ServerToClientHandshakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetActorLinkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetActorMotionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetCommandsEnabledPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetDefaultGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetDifficultyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetDisplayObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetHealthPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetHudPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetLastHurtByPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetLocalPlayerAsInitializedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetMovementAuthorityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetMovementAuthorityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetPlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetPlayerInventoryOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetScorePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetScoreboardIdentityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetSpawnPositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetTimePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SetTitlePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SettingsCommandPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ShowCreditsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ShowProfilePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ShowStoreOfferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SimpleEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SimulationTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SpawnExperienceOrbPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SpawnParticleEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::StartGamePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::StopSoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::StructureBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::StructureDataRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::StructureDataResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SubChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SubChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SubClientLoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::SyncActorPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::TakeItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::TextPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::TickingAreaLoadStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::ToastRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::TransferPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::TrimDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UnlockedRecipesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateAbilitiesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateAdventureSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateAttributesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateBlockPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateBlockSyncedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateClientInputLocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateEquipPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdatePlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateSoftEnumPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateSubChunkBlocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V766::UpdateTradePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
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
                    V766::Unknown(
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
                    V766::ActorEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ActorPickRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AddActorPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AddBehaviourTreePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AddItemActorPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AddPaintingPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AddPlayerPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AddVolumeEntityPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AgentActionEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AgentAnimationPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AnimateEntityPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AnimatePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AnvilDamagePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AutomationClientConnectPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AvailableActorIdentifiersPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AvailableCommandsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::AwardAchievementPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::BiomeDefinitionListPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::BlockActorDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::BlockEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::BlockPickRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::BookEditPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::BossEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CameraAimAssistPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CameraAimAssistPresetsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CameraInstructionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CameraPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CameraPresetsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CameraShakePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ChangeDimensionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ChangeMobPropertyPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ChunkRadiusUpdatedPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientBoundCloseFormPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientBoundDebugRendererPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientBoundMapItemDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientCacheBlobStatusPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientCacheMissResponsePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientCacheStatusPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ClientToServerHandshakePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CodeBuilderPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CodeBuilderSourcePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CommandBlockUpdatePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CommandOutputPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CommandRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CompletedUsingItemPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CompressedBiomeDefinitionListPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ContainerClosePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ContainerOpenPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ContainerRegistryCleanupPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ContainerSetDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CorrectPlayerMovePredictionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CraftingDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CreatePhotoPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CreativeContentPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::CurrentStructureFeaturePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::DeathInfoPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::DebugInfoPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::DimensionDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::DisconnectPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::EditorNetworkPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::EduUriResourcePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::EducationSettingsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::EmoteListPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::EmotePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::FeatureRegistryPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::GameRulesChangedPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::GameTestRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::GameTestResultsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::GuiDataPickItemPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::HurtArmorPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::InteractPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::InventoryContentPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::InventorySlotPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::InventoryTransactionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ItemComponentPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ItemStackRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ItemStackResponsePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::JigsawStructureDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LabTablePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LecternUpdatePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LegacyTelemetryEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LessonProgressPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LevelChunkPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LevelEventGenericPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LevelEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LevelSoundEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LevelSoundEventV1Packet(pk) => {
                        <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LevelSoundEventV2Packet(pk) => {
                        <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::LoginPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MapCreateLockedCopyPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MapInfoRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MobArmorEquipmentPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MobEffectPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MobEquipmentPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ModalFormRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ModalFormResponsePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MotionPredictionHintsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MoveActorAbsolutePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MoveActorDeltaPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MovePlayerPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MovementEffectPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::MultiplayerSettingsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::NetworkChunkPublisherUpdatePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::NetworkSettingsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::NetworkStackLatencyPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::NpcDialoguePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::NpcRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::OnScreenTextureAnimationPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::OpenSignPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PacketViolationWarningPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PassengerJumpPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PhotoTransferPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlaySoundPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayStatusPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerActionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerArmorDamagePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerAuthInputPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerEnchantOptionsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerFogPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerHotbarPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerInputPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerListPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerSkinPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerStartItemCooldownPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PlayerToggleCrafterSlotRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PositionTrackingDBClientRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PositionTrackingDBServerBroadcastPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::PurchaseReceiptPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RefreshEntitlementsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RemoveActorPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RemoveObjectivePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RemoveVolumeEntityPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RequestAbilityPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RequestChunkRadiusPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RequestNetworkSettingsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RequestPermissionsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ResourcePackChunkDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ResourcePackChunkRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ResourcePackClientResponsePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ResourcePackDataInfoPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ResourcePackStackPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ResourcePacksInfoPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::RespawnPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ScriptMessagePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerBoundDiagnosticsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerBoundLoadingScreenPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerPlayerPostMovePositionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerSettingsRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerSettingsResponsePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerStatsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ServerToClientHandshakePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetActorDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetActorLinkPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetActorMotionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetCommandsEnabledPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetDefaultGameTypePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetDifficultyPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetDisplayObjectivePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetHealthPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetHudPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetLastHurtByPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetLocalPlayerAsInitializedPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetMovementAuthorityPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetPlayerGameTypePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetPlayerInventoryOptionsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetScorePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetScoreboardIdentityPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetSpawnPositionPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetTimePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SetTitlePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SettingsCommandPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ShowCreditsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ShowProfilePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ShowStoreOfferPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SimpleEventPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SimulationTypePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SpawnExperienceOrbPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SpawnParticleEffectPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::StartGamePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::StopSoundPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::StructureBlockUpdatePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::StructureDataRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::StructureDataResponsePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SubChunkPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SubChunkRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SubClientLoginPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::SyncActorPropertyPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::TakeItemActorPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::TextPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::TickingAreaLoadStatusPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::ToastRequestPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::TransferPlayerPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::TrimDataPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UnlockedRecipesPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateAbilitiesPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateAdventureSettingsPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateAttributesPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateBlockPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateBlockSyncedPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateClientInputLocksPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateEquipPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdatePlayerGameTypePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateSoftEnumPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateSubChunkBlocksPacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::UpdateTradePacket(pk) => {
                        <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V766::Unknown(pk) => pk.buf.len(),
                }
        }
        #[inline]
        fn id(&self) -> u16 {
            match self {
                V766::ActorEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ActorPickRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AddActorPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AddBehaviourTreePacket(_) => {
                    <<V766 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AddItemActorPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AddPaintingPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AddPlayerPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AddVolumeEntityPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AgentActionEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AgentAnimationPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AnimateEntityPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AnimatePacket(_) => {
                    <<V766 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AnvilDamagePacket(_) => {
                    <<V766 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AutomationClientConnectPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AvailableActorIdentifiersPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AvailableCommandsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::AwardAchievementPacket(_) => {
                    <<V766 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::BiomeDefinitionListPacket(_) => {
                    <<V766 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::BlockActorDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::BlockEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::BlockPickRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::BookEditPacket(_) => {
                    <<V766 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::BossEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CameraAimAssistPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CameraAimAssistPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CameraAimAssistPresetsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CameraAimAssistPresetsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CameraInstructionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CameraPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CameraPresetsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CameraShakePacket(_) => {
                    <<V766 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ChangeDimensionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ChangeMobPropertyPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ChunkRadiusUpdatedPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientBoundCloseFormPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientBoundCloseFormPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientBoundDebugRendererPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientBoundMapItemDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientCacheBlobStatusPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientCacheMissResponsePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientCacheStatusPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ClientToServerHandshakePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CodeBuilderPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CodeBuilderSourcePacket(_) => {
                    <<V766 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CommandBlockUpdatePacket(_) => {
                    <<V766 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CommandOutputPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CommandRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CompletedUsingItemPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CompressedBiomeDefinitionListPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ContainerClosePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ContainerOpenPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ContainerRegistryCleanupPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ContainerRegistryCleanupPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ContainerSetDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CorrectPlayerMovePredictionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CraftingDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CreatePhotoPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CreativeContentPacket(_) => {
                    <<V766 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::CurrentStructureFeaturePacket(_) => {
                    <<V766 as ProtoVersionPackets>::CurrentStructureFeaturePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::DeathInfoPacket(_) => {
                    <<V766 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::DebugInfoPacket(_) => {
                    <<V766 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::DimensionDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::DisconnectPacket(_) => {
                    <<V766 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::EditorNetworkPacket(_) => {
                    <<V766 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::EduUriResourcePacket(_) => {
                    <<V766 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::EducationSettingsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::EmoteListPacket(_) => {
                    <<V766 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::EmotePacket(_) => {
                    <<V766 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::FeatureRegistryPacket(_) => {
                    <<V766 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::GameRulesChangedPacket(_) => {
                    <<V766 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::GameTestRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::GameTestResultsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::GuiDataPickItemPacket(_) => {
                    <<V766 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::HurtArmorPacket(_) => {
                    <<V766 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::InteractPacket(_) => {
                    <<V766 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::InventoryContentPacket(_) => {
                    <<V766 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::InventorySlotPacket(_) => {
                    <<V766 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::InventoryTransactionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ItemComponentPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ItemStackRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ItemStackResponsePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::JigsawStructureDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::JigsawStructureDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LabTablePacket(_) => {
                    <<V766 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LecternUpdatePacket(_) => {
                    <<V766 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LegacyTelemetryEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LessonProgressPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LevelChunkPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LevelEventGenericPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LevelEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LevelSoundEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::LevelSoundEventV1Packet(_) => {
                    <<V766 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID
                }
                V766::LevelSoundEventV2Packet(_) => {
                    <<V766 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID
                }
                V766::LoginPacket(_) => {
                    <<V766 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MapCreateLockedCopyPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MapInfoRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MobArmorEquipmentPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MobEffectPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MobEquipmentPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ModalFormRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ModalFormResponsePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MotionPredictionHintsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MoveActorAbsolutePacket(_) => {
                    <<V766 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MoveActorDeltaPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MovePlayerPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MovementEffectPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MovementEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::MultiplayerSettingsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::NetworkChunkPublisherUpdatePacket(_) => {
                    <<V766 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::NetworkSettingsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::NetworkStackLatencyPacket(_) => {
                    <<V766 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::NpcDialoguePacket(_) => {
                    <<V766 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::NpcRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::OnScreenTextureAnimationPacket(_) => {
                    <<V766 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::OpenSignPacket(_) => {
                    <<V766 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PacketViolationWarningPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PassengerJumpPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PhotoTransferPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlaySoundPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayStatusPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerActionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerArmorDamagePacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerAuthInputPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerEnchantOptionsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerFogPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerHotbarPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerInputPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerListPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerSkinPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerStartItemCooldownPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PlayerToggleCrafterSlotRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PositionTrackingDBClientRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PositionTrackingDBServerBroadcastPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::PurchaseReceiptPacket(_) => {
                    <<V766 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RefreshEntitlementsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RemoveActorPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RemoveObjectivePacket(_) => {
                    <<V766 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RemoveVolumeEntityPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RequestAbilityPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RequestChunkRadiusPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RequestNetworkSettingsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RequestPermissionsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ResourcePackChunkDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ResourcePackChunkRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ResourcePackClientResponsePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ResourcePackDataInfoPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ResourcePackStackPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ResourcePacksInfoPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::RespawnPacket(_) => {
                    <<V766 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ScriptMessagePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerBoundDiagnosticsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerBoundDiagnosticsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerBoundLoadingScreenPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerBoundLoadingScreenPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerPlayerPostMovePositionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerSettingsRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerSettingsResponsePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerStatsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ServerToClientHandshakePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetActorDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetActorLinkPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetActorMotionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetCommandsEnabledPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetDefaultGameTypePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetDifficultyPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetDisplayObjectivePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetHealthPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetHudPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetLastHurtByPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetLocalPlayerAsInitializedPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetMovementAuthorityPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetMovementAuthorityPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetPlayerGameTypePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetPlayerInventoryOptionsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetScorePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetScoreboardIdentityPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetSpawnPositionPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetTimePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SetTitlePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SettingsCommandPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ShowCreditsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ShowProfilePacket(_) => {
                    <<V766 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ShowStoreOfferPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SimpleEventPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SimulationTypePacket(_) => {
                    <<V766 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SpawnExperienceOrbPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SpawnParticleEffectPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::StartGamePacket(_) => {
                    <<V766 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::StopSoundPacket(_) => {
                    <<V766 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::StructureBlockUpdatePacket(_) => {
                    <<V766 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::StructureDataRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::StructureDataResponsePacket(_) => {
                    <<V766 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SubChunkPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SubChunkRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SubClientLoginPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::SyncActorPropertyPacket(_) => {
                    <<V766 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::TakeItemActorPacket(_) => {
                    <<V766 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::TextPacket(_) => {
                    <<V766 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::TickingAreaLoadStatusPacket(_) => {
                    <<V766 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::ToastRequestPacket(_) => {
                    <<V766 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::TransferPlayerPacket(_) => {
                    <<V766 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::TrimDataPacket(_) => {
                    <<V766 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UnlockedRecipesPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateAbilitiesPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateAdventureSettingsPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateAttributesPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateBlockPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateBlockSyncedPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateClientInputLocksPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateEquipPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdatePlayerGameTypePacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateSoftEnumPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateSubChunkBlocksPacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V766::UpdateTradePacket(_) => {
                    <<V766 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID
                }
                V766::Unknown(pk) => pk.id,
            }
        }
    }
    impl ProtoVersionPackets for V766 {
        type ActorEventPacket = crate::version::v662::packets::ActorEventPacket<Self>;
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
        type AnimatePacket = crate::version::v662::packets::AnimatePacket<Self>;
        type AnvilDamagePacket = crate::version::v662::packets::AnvilDamagePacket<Self>;
        type AutomationClientConnectPacket =
            crate::version::v662::packets::AutomationClientConnectPacket<Self>;
        type AvailableActorIdentifiersPacket =
            crate::version::v662::packets::AvailableActorIdentifiersPacket;
        type AvailableCommandsPacket = crate::version::v662::packets::AvailableCommandsPacket<Self>;
        type AwardAchievementPacket = crate::version::v685::packets::AwardAchievementPacket;
        type BiomeDefinitionListPacket = crate::version::v662::packets::BiomeDefinitionListPacket;
        type BlockActorDataPacket = crate::version::v662::packets::BlockActorDataPacket<Self>;
        type BlockEventPacket = crate::version::v662::packets::BlockEventPacket<Self>;
        type BlockPickRequestPacket = crate::version::v662::packets::BlockPickRequestPacket<Self>;
        type BookEditPacket = crate::version::v662::packets::BookEditPacket<Self>;
        type BossEventPacket = crate::version::v662::packets::BossEventPacket<Self>;
        type CameraAimAssistActorPriorityPacket = ();
        type CameraAimAssistInstructionPacket = ();
        type CameraAimAssistPacket = crate::version::v766::packets::CameraAimAssistPacket<Self>;
        type CameraAimAssistPresetsPacket =
            crate::version::v766::packets::CameraAimAssistPresetsPacket<Self>;
        type CameraInstructionPacket = crate::version::v712::packets::CameraInstructionPacket<Self>;
        type CameraPacket = crate::version::v662::packets::CameraPacket<Self>;
        type CameraPresetsPacket = crate::version::v662::packets::CameraPresetsPacket<Self>;
        type CameraShakePacket = crate::version::v662::packets::CameraShakePacket<Self>;
        type CameraSplinePacket = ();
        type ChangeDimensionPacket = crate::version::v712::packets::ChangeDimensionPacket;
        type ChangeMobPropertyPacket = crate::version::v662::packets::ChangeMobPropertyPacket<Self>;
        type ChunkRadiusUpdatedPacket = crate::version::v662::packets::ChunkRadiusUpdatedPacket;
        type ClientBoundAttributeLayerSyncPacket = ();
        type ClientBoundCloseFormPacket = crate::version::v686::packets::ClientBoundCloseFormPacket;
        type ClientBoundControlSchemeSetPacket = ();
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket = ();
        type ClientBoundDataDrivenUIReloadPacket = ();
        type ClientBoundDataDrivenUIShowScreenPacket = ();
        type ClientBoundDataStorePacket = ();
        type ClientBoundDebugRendererPacket =
            crate::version::v671::packets::ClientBoundDebugRendererPacket;
        type ClientBoundMapItemDataPacket =
            crate::version::v662::packets::ClientBoundMapItemDataPacket<Self>;
        type ClientBoundTextureShiftPacket = ();
        type ClientCacheBlobStatusPacket =
            crate::version::v662::packets::ClientCacheBlobStatusPacket;
        type ClientCacheMissResponsePacket =
            crate::version::v662::packets::ClientCacheMissResponsePacket;
        type ClientCacheStatusPacket = crate::version::v662::packets::ClientCacheStatusPacket;
        type ClientToServerHandshakePacket =
            crate::version::v662::packets::ClientToServerHandshakePacket;
        type CodeBuilderPacket = crate::version::v662::packets::CodeBuilderPacket;
        type CodeBuilderSourcePacket = crate::version::v685::packets::CodeBuilderSourcePacket<Self>;
        type CommandBlockUpdatePacket =
            crate::version::v662::packets::CommandBlockUpdatePacket<Self>;
        type CommandOutputPacket = crate::version::v662::packets::CommandOutputPacket<Self>;
        type CommandRequestPacket = crate::version::v662::packets::CommandRequestPacket<Self>;
        type CompletedUsingItemPacket =
            crate::version::v662::packets::CompletedUsingItemPacket<Self>;
        type CompressedBiomeDefinitionListPacket =
            crate::version::v662::packets::CompressedBiomeDefinitionListPacket;
        type ContainerClosePacket = crate::version::v685::packets::ContainerClosePacket<Self>;
        type ContainerOpenPacket = crate::version::v662::packets::ContainerOpenPacket<Self>;
        type ContainerRegistryCleanupPacket =
            crate::version::v729::packets::ContainerRegistryCleanupPacket<Self>;
        type ContainerSetDataPacket = crate::version::v662::packets::ContainerSetDataPacket<Self>;
        type CorrectPlayerMovePredictionPacket =
            crate::version::v671::packets::CorrectPlayerMovePredictionPacket<Self>;
        type CraftingDataPacket = crate::version::v662::packets::CraftingDataPacket<Self>;
        type CreatePhotoPacket = crate::version::v662::packets::CreatePhotoPacket;
        type CreativeContentPacket = crate::version::v662::packets::CreativeContentPacket<Self>;
        type CurrentStructureFeaturePacket =
            crate::version::v712::packets::CurrentStructureFeaturePacket;
        type DeathInfoPacket = crate::version::v662::packets::DeathInfoPacket;
        type DebugDrawerPacket = ();
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
        type GraphicsParameterOverridePacket = ();
        type GuiDataPickItemPacket = crate::version::v662::packets::GuiDataPickItemPacket;
        type HurtArmorPacket = crate::version::v662::packets::HurtArmorPacket;
        type InteractPacket = crate::version::v662::packets::InteractPacket<Self>;
        type InventoryContentPacket = crate::version::v748::packets::InventoryContentPacket<Self>;
        type InventorySlotPacket = crate::version::v748::packets::InventorySlotPacket<Self>;
        type InventoryTransactionPacket =
            crate::version::v662::packets::InventoryTransactionPacket<Self>;
        type ItemComponentPacket = crate::version::v662::packets::ItemComponentPacket;
        type ItemStackRequestPacket = crate::version::v662::packets::ItemStackRequestPacket<Self>;
        type ItemStackResponsePacket = crate::version::v662::packets::ItemStackResponsePacket<Self>;
        type JigsawStructureDataPacket = crate::version::v712::packets::JigsawStructureDataPacket;
        type LabTablePacket = crate::version::v662::packets::LabTablePacket<Self>;
        type LecternUpdatePacket = crate::version::v662::packets::LecternUpdatePacket<Self>;
        type LegacyTelemetryEventPacket =
            crate::version::v685::packets::LegacyTelemetryEventPacket<Self>;
        type LessonProgressPacket = crate::version::v662::packets::LessonProgressPacket<Self>;
        type LevelChunkPacket = crate::version::v662::packets::LevelChunkPacket<Self>;
        type LevelEventGenericPacket = crate::version::v662::packets::LevelEventGenericPacket<Self>;
        type LevelEventPacket = crate::version::v662::packets::LevelEventPacket;
        type LevelSoundEventPacket = crate::version::v662::packets::LevelSoundEventPacket<Self>;
        type LevelSoundEventV1Packet = crate::version::v662::packets::LevelSoundEventV1Packet<Self>;
        type LevelSoundEventV2Packet = crate::version::v662::packets::LevelSoundEventV2Packet<Self>;
        type LocatorBarPacket = ();
        type LoginPacket = crate::version::v662::packets::LoginPacket;
        type MapCreateLockedCopyPacket =
            crate::version::v662::packets::MapCreateLockedCopyPacket<Self>;
        type MapInfoRequestPacket = crate::version::v662::packets::MapInfoRequestPacket<Self>;
        type MobArmorEquipmentPacket = crate::version::v712::packets::MobArmorEquipmentPacket<Self>;
        type MobEffectPacket = crate::version::v748::packets::MobEffectPacket<Self>;
        type MobEquipmentPacket = crate::version::v662::packets::MobEquipmentPacket<Self>;
        type ModalFormRequestPacket = crate::version::v662::packets::ModalFormRequestPacket;
        type ModalFormResponsePacket = crate::version::v662::packets::ModalFormResponsePacket<Self>;
        type MotionPredictionHintsPacket =
            crate::version::v662::packets::MotionPredictionHintsPacket<Self>;
        type MoveActorAbsolutePacket = crate::version::v662::packets::MoveActorAbsolutePacket<Self>;
        type MoveActorDeltaPacket = crate::version::v662::packets::MoveActorDeltaPacket<Self>;
        type MovePlayerPacket = crate::version::v662::packets::MovePlayerPacket<Self>;
        type MovementEffectPacket = crate::version::v748::packets::MovementEffectPacket<Self>;
        type MovementPredictionSyncPacket = ();
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
        type PartyChangedPacket = ();
        type PassengerJumpPacket = crate::version::v662::packets::PassengerJumpPacket;
        type PhotoTransferPacket = crate::version::v662::packets::PhotoTransferPacket<Self>;
        type PlaySoundPacket = crate::version::v662::packets::PlaySoundPacket<Self>;
        type PlayStatusPacket = crate::version::v662::packets::PlayStatusPacket<Self>;
        type PlayerActionPacket = crate::version::v662::packets::PlayerActionPacket<Self>;
        type PlayerArmorDamagePacket = crate::version::v712::packets::PlayerArmorDamagePacket;
        type PlayerAuthInputPacket = crate::version::v766::packets::PlayerAuthInputPacket<Self>;
        type PlayerEnchantOptionsPacket =
            crate::version::v662::packets::PlayerEnchantOptionsPacket<Self>;
        type PlayerFogPacket = crate::version::v662::packets::PlayerFogPacket;
        type PlayerHotbarPacket = crate::version::v662::packets::PlayerHotbarPacket<Self>;
        type PlayerInputPacket = crate::version::v662::packets::PlayerInputPacket;
        type PlayerListPacket = crate::version::v662::packets::PlayerListPacket<Self>;
        type PlayerLocationPacket = ();
        type PlayerSkinPacket = crate::version::v662::packets::PlayerSkinPacket<Self>;
        type PlayerStartItemCooldownPacket =
            crate::version::v662::packets::PlayerStartItemCooldownPacket;
        type PlayerToggleCrafterSlotRequestPacket =
            crate::version::v662::packets::PlayerToggleCrafterSlotRequestPacket;
        type PlayerUpdateEntityOverridesPacket = ();
        type PlayerVideoCapturePacket = ();
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
            crate::version::v662::packets::ResourcePackClientResponsePacket<Self>;
        type ResourcePackDataInfoPacket =
            crate::version::v662::packets::ResourcePackDataInfoPacket<Self>;
        type ResourcePackStackPacket = crate::version::v671::packets::ResourcePackStackPacket<Self>;
        type ResourcePacksInfoPacket = crate::version::v766::packets::ResourcePacksInfoPacket;
        type ResourcePacksReadyForValidationPacket = ();
        type RespawnPacket = crate::version::v662::packets::RespawnPacket<Self>;
        type ScriptMessagePacket = crate::version::v662::packets::ScriptMessagePacket;
        type ServerBoundDataDrivenClosedPacket = ();
        type ServerBoundDataStorePacket = ();
        type ServerBoundDiagnosticsPacket =
            crate::version::v712::packets::ServerBoundDiagnosticsPacket;
        type ServerBoundLoadingScreenPacket =
            crate::version::v712::packets::ServerBoundLoadingScreenPacket;
        type ServerBoundPackSettingChangePacket = ();
        type ServerPlayerPostMovePositionPacket =
            crate::version::v662::packets::ServerPlayerPostMovePositionPacket;
        type ServerPresenceInfoPacket = ();
        type ServerSettingsRequestPacket =
            crate::version::v662::packets::ServerSettingsRequestPacket;
        type ServerSettingsResponsePacket =
            crate::version::v662::packets::ServerSettingsResponsePacket;
        type ServerStatsPacket = crate::version::v662::packets::ServerStatsPacket;
        type ServerStoreInfoPacket = ();
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
        type SetMovementAuthorityPacket =
            crate::version::v748::packets::SetMovementAuthorityPacket<Self>;
        type SetPlayerGameTypePacket = crate::version::v662::packets::SetPlayerGameTypePacket<Self>;
        type SetPlayerInventoryOptionsPacket =
            crate::version::v662::packets::SetPlayerInventoryOptionsPacket<Self>;
        type SetScorePacket = crate::version::v662::packets::SetScorePacket<Self>;
        type SetScoreboardIdentityPacket =
            crate::version::v662::packets::SetScoreboardIdentityPacket<Self>;
        type SetSpawnPositionPacket = crate::version::v662::packets::SetSpawnPositionPacket<Self>;
        type SetTimePacket = crate::version::v662::packets::SetTimePacket;
        type SetTitlePacket = crate::version::v712::packets::SetTitlePacket;
        type SettingsCommandPacket = crate::version::v662::packets::SettingsCommandPacket;
        type ShowCreditsPacket = crate::version::v662::packets::ShowCreditsPacket<Self>;
        type ShowProfilePacket = crate::version::v662::packets::ShowProfilePacket;
        type ShowStoreOfferPacket = crate::version::v662::packets::ShowStoreOfferPacket<Self>;
        type SimpleEventPacket = crate::version::v662::packets::SimpleEventPacket;
        type SimulationTypePacket = crate::version::v662::packets::SimulationTypePacket<Self>;
        type SpawnExperienceOrbPacket = crate::version::v662::packets::SpawnExperienceOrbPacket;
        type SpawnParticleEffectPacket =
            crate::version::v662::packets::SpawnParticleEffectPacket<Self>;
        type StartGamePacket = crate::version::v662::packets::StartGamePacket<Self>;
        type StopSoundPacket = crate::version::v712::packets::StopSoundPacket;
        type StructureBlockUpdatePacket =
            crate::version::v662::packets::StructureBlockUpdatePacket<Self>;
        type StructureDataRequestPacket =
            crate::version::v662::packets::StructureDataRequestPacket<Self>;
        type StructureDataResponsePacket =
            crate::version::v662::packets::StructureDataResponsePacket<Self>;
        type SubChunkPacket = crate::version::v662::packets::SubChunkPacket<Self>;
        type SubChunkRequestPacket = crate::version::v662::packets::SubChunkRequestPacket<Self>;
        type SubClientLoginPacket = crate::version::v662::packets::SubClientLoginPacket;
        type SyncActorPropertyPacket = crate::version::v662::packets::SyncActorPropertyPacket;
        type SyncWorldClocksPacket = ();
        type TakeItemActorPacket = crate::version::v662::packets::TakeItemActorPacket<Self>;
        type TextPacket = crate::version::v685::packets::TextPacket<Self>;
        type TickSyncPacket = ();
        type TickingAreaLoadStatusPacket =
            crate::version::v662::packets::TickingAreaLoadStatusPacket;
        type ToastRequestPacket = crate::version::v662::packets::ToastRequestPacket;
        type TransferPlayerPacket = crate::version::v729::packets::TransferPlayerPacket;
        type TrimDataPacket = crate::version::v662::packets::TrimDataPacket;
        type UnlockedRecipesPacket = crate::version::v662::packets::UnlockedRecipesPacket;
        type UpdateAbilitiesPacket = crate::version::v662::packets::UpdateAbilitiesPacket<Self>;
        type UpdateAdventureSettingsPacket =
            crate::version::v662::packets::UpdateAdventureSettingsPacket<Self>;
        type UpdateAttributesPacket = crate::version::v729::packets::UpdateAttributesPacket<Self>;
        type UpdateBlockPacket = crate::version::v662::packets::UpdateBlockPacket<Self>;
        type UpdateBlockSyncedPacket = crate::version::v662::packets::UpdateBlockSyncedPacket<Self>;
        type UpdateClientInputLocksPacket =
            crate::version::v662::packets::UpdateClientInputLocksPacket;
        type UpdateClientOptionsPacket = ();
        type UpdateEquipPacket = crate::version::v662::packets::UpdateEquipPacket<Self>;
        type UpdatePlayerGameTypePacket =
            crate::version::v671::packets::UpdatePlayerGameTypePacket<Self>;
        type UpdateSoftEnumPacket = crate::version::v662::packets::UpdateSoftEnumPacket<Self>;
        type UpdateSubChunkBlocksPacket =
            crate::version::v662::packets::UpdateSubChunkBlocksPacket<Self>;
        type UpdateTradePacket = crate::version::v662::packets::UpdateTradePacket<Self>;
        type VoxelShapesPacket = ();
    }
    impl ProtoVersionTypes for V766 {
        type ActorLink = crate::version::v712::types::ActorLink<Self>;
        type ActorRuntimeID = crate::version::v662::types::ActorRuntimeID;
        type ActorUniqueID = crate::version::v662::types::ActorUniqueID;
        type AdventureSettings = crate::version::v662::types::AdventureSettings;
        type BaseDescription = crate::version::v662::types::BaseDescription<Self>;
        type BaseGameVersion = crate::version::v662::types::BaseGameVersion;
        type BiomeCappedSurfaceData = ();
        type BiomeClimateData = ();
        type BiomeConditionalTransformationData = ();
        type BiomeConsolidatedFeatureList = ();
        type BiomeCoordinateData = ();
        type BiomeDefinition = ();
        type BiomeDefinitionChunkGenData = ();
        type BiomeElementData = ();
        type BiomeLegacyWorldGenRulesData = ();
        type BiomeMesaSurfaceData = ();
        type BiomeMountainParamsData = ();
        type BiomeMultinoiseGenRulesData = ();
        type BiomeNoiseGradientSurfaceData = ();
        type BiomeOverworldGenRulesData = ();
        type BiomeReplacementData = ();
        type BiomeScatterParamData = ();
        type BiomeSurfaceBuilderData = ();
        type BiomeSurfaceMaterialAdjustmentData = ();
        type BiomeSurfaceMaterialData = ();
        type BiomeWeightedData = ();
        type BiomeWeightedTemperatureData = ();
        type BlockPos = crate::version::v662::types::BlockPos;
        type CameraAimAssistCategories =
            crate::version::v766::types::CameraAimAssistCategories<Self>;
        type CameraAimAssistCategory = crate::version::v766::types::CameraAimAssistCategory<Self>;
        type CameraAimAssistItemSettings = crate::version::v766::types::CameraAimAssistItemSettings;
        type CameraAimAssistPreset = crate::version::v766::types::CameraAimAssistPreset;
        type CameraAimAssistPresetDefinition =
            crate::version::v766::types::CameraAimAssistPresetDefinition<Self>;
        type CameraAimAssistPriority = crate::version::v766::types::CameraAimAssistPriority;
        type CameraInstruction = crate::version::v748::types::CameraInstruction<Self>;
        type CameraPreset = crate::version::v766::types::CameraPreset<Self>;
        type CameraPresets = crate::version::v662::types::CameraPresets<Self>;
        type CameraSplineInstruction = ();
        type ChunkPos = crate::version::v662::types::ChunkPos;
        type Color = ();
        type CommandOriginData = crate::version::v662::types::CommandOriginData<Self>;
        type ContainerMixDataEntry = crate::version::v662::types::ContainerMixDataEntry;
        type CraftingDataEntry = crate::version::v662::types::CraftingDataEntry<Self>;
        type DataItem = crate::version::v662::types::DataItem<Self>;
        type DebugShape = ();
        type DimensionDefinitionGroup = crate::version::v662::types::DimensionDefinitionGroup;
        type EduSharedUriResource = crate::version::v662::types::EduSharedUriResource;
        type EducationLevelSettings = crate::version::v662::types::EducationLevelSettings;
        type EntityNetID = crate::version::v662::types::EntityNetID;
        type Experiments = crate::version::v662::types::Experiments;
        type FullContainerName = crate::version::v729::types::FullContainerName<Self>;
        type GameRulesChangedPacketData = crate::version::v662::types::GameRulesChangedPacketData;
        type InventoryAction = crate::version::v662::types::InventoryAction<Self>;
        type InventorySource = crate::version::v662::types::InventorySource<Self>;
        type InventoryTransaction = crate::version::v662::types::InventoryTransaction<Self>;
        type ItemData = crate::version::v662::types::ItemData;
        type ItemEnchants = crate::version::v662::types::ItemEnchants<Self>;
        type ItemStackRequestSlotInfo = crate::version::v712::types::ItemStackRequestSlotInfo<Self>;
        type ItemStackResponseContainerInfo =
            crate::version::v712::types::ItemStackResponseContainerInfo<Self>;
        type ItemStackResponseInfo = crate::version::v662::types::ItemStackResponseInfo<Self>;
        type ItemStackResponseSlotInfo = crate::version::v766::types::ItemStackResponseSlotInfo;
        type LevelSettings = crate::version::v685::types::LevelSettings<Self>;
        type MapDecoration = crate::version::v662::types::MapDecoration;
        type MapItemTrackedActorUniqueID =
            crate::version::v662::types::MapItemTrackedActorUniqueID<Self>;
        type MaterialReducerDataEntry = crate::version::v662::types::MaterialReducerDataEntry;
        type MolangVariableMap = crate::version::v662::types::MolangVariableMap;
        type MoveActorAbsoluteData = crate::version::v662::types::MoveActorAbsoluteData<Self>;
        type MoveActorDeltaData = crate::version::v662::types::MoveActorDeltaData<Self>;
        type NetworkBlockPosition = crate::version::v662::types::NetworkBlockPosition;
        type NetworkItemInstanceDescriptor =
            crate::version::v662::types::NetworkItemInstanceDescriptor;
        type NetworkItemStackDescriptor = crate::version::v662::types::NetworkItemStackDescriptor;
        type NetworkPermissions = crate::version::v662::types::NetworkPermissions;
        type PackedItemUseLegacyInventoryTransaction =
            crate::version::v712::types::PackedItemUseLegacyInventoryTransaction<Self>;
        type PlayerBlockActionData = crate::version::v662::types::PlayerBlockActionData<Self>;
        type PositionTrackingId = crate::version::v662::types::PositionTrackingId;
        type PotionMixDataEntry = crate::version::v662::types::PotionMixDataEntry;
        type PropertySyncData = crate::version::v662::types::PropertySyncData;
        type RecipeIngredient = crate::version::v662::types::RecipeIngredient<Self>;
        type RecipeUnlockingRequirement =
            crate::version::v685::types::RecipeUnlockingRequirement<Self>;
        type ScoreboardId = crate::version::v662::types::ScoreboardId;
        type SerializedAbilitiesData = crate::version::v662::types::SerializedAbilitiesData<Self>;
        type SerializedSkin = crate::version::v662::types::SerializedSkin<Self>;
        type ShapedRecipe = crate::version::v685::types::ShapedRecipe<Self>;
        type ShapelessRecipe = crate::version::v685::types::ShapelessRecipe<Self>;
        type ShulkerBoxRecipe = crate::version::v748::types::ShulkerBoxRecipe<Self>;
        type SmithingTransformRecipe = crate::version::v662::types::SmithingTransformRecipe<Self>;
        type SmithingTrimRecipe = crate::version::v662::types::SmithingTrimRecipe<Self>;
        type SpawnSettings = crate::version::v662::types::SpawnSettings<Self>;
        type StructureEditorData = crate::version::v662::types::StructureEditorData<Self>;
        type StructureSettings = crate::version::v662::types::StructureSettings<Self>;
        type SubChunkPos = crate::version::v662::types::SubChunkPos;
        type SubChunkPosOffset = crate::version::v662::types::SubChunkPosOffset;
        type SyncedPlayerMovementSettings =
            crate::version::v662::types::SyncedPlayerMovementSettings<Self>;
        type WebSocketPacketData = crate::version::v662::types::WebSocketPacketData;
    }
    impl ProtoVersionEnums for V766 {
        type AbilitiesIndex = crate::version::v662::enums::AbilitiesIndex;
        type ActorBlockSyncMessageID = crate::version::v662::enums::ActorBlockSyncMessageID;
        type ActorDamageCause = crate::version::v662::enums::ActorDamageCause;
        type ActorDataIDs = crate::version::v685::enums::ActorDataIDs;
        type ActorEvent = crate::version::v662::enums::ActorEvent;
        type ActorFlags = crate::version::v671::enums::ActorFlags;
        type ActorLinkType = crate::version::v662::enums::ActorLinkType;
        type ActorType = crate::version::v662::enums::ActorType;
        type AgentActionType = crate::version::v662::enums::AgentActionType;
        type AimAssistAction = crate::version::v729::enums::AimAssistAction;
        type AnimatedTextureType = crate::version::v662::enums::AnimatedTextureType;
        type AnimationExpression = crate::version::v662::enums::AnimationExpression;
        type AnimationMode = crate::version::v662::enums::AnimationMode;
        type AttributeModifierOperation = crate::version::v662::enums::AttributeModifierOperation;
        type AttributeOperands = crate::version::v662::enums::AttributeOperands;
        type AuthoritativeMovementMode = crate::version::v748::enums::AuthoritativeMovementMode;
        type BookEditAction = crate::version::v662::enums::BookEditAction;
        type BossEventUpdateType = crate::version::v662::enums::BossEventUpdateType<Self>;
        type BuildPlatform = crate::version::v662::enums::BuildPlatform;
        type CameraAimAssistOperation = ();
        type CameraShakeAction = crate::version::v662::enums::CameraShakeAction;
        type CameraShakeType = crate::version::v662::enums::CameraShakeType;
        type CameraSplineEaseType = ();
        type CameraSplineType = ();
        type ChatRestrictionLevel = crate::version::v662::enums::ChatRestrictionLevel;
        type CodeBuilderCodeStatus = crate::version::v685::enums::CodeBuilderCodeStatus;
        type CodeBuilderStorageCategory = crate::version::v662::enums::CodeBuilderStorageCategory;
        type CodeBuilderStorageOperation = crate::version::v662::enums::CodeBuilderStorageOperation;
        type CommandBlockMode = crate::version::v662::enums::CommandBlockMode;
        type CommandOriginType = crate::version::v662::enums::CommandOriginType;
        type CommandOutputType = crate::version::v662::enums::CommandOutputType;
        type CommandParameterOption = crate::version::v662::enums::CommandParameterOption;
        type CommandPermissionLevel = crate::version::v662::enums::CommandPermissionLevel;
        type ComplexInventoryTransactionType =
            crate::version::v662::enums::ComplexInventoryTransactionType;
        type ConnectionFailReason = crate::version::v662::enums::ConnectionFailReason;
        type ContainerEnumName = crate::version::v712::enums::ContainerEnumName;
        type ContainerID = crate::version::v662::enums::ContainerID;
        type ContainerType = crate::version::v662::enums::ContainerType;
        type ControlScheme = ();
        type CraftingDataEntryType = crate::version::v662::enums::CraftingDataEntryType<Self>;
        type DataItemType = crate::version::v662::enums::DataItemType<Self>;
        type Difficulty = crate::version::v662::enums::Difficulty;
        type EasingType = crate::version::v662::enums::EasingType;
        type EditorWorldType = crate::version::v662::enums::EditorWorldType;
        type EducationEditionOffer = crate::version::v662::enums::EducationEditionOffer;
        type EnchantType = crate::version::v662::enums::EnchantType;
        type GamePublishSetting = crate::version::v662::enums::GamePublishSetting;
        type GameType = crate::version::v662::enums::GameType;
        type GeneratorType = crate::version::v662::enums::GeneratorType;
        type HudElement = crate::version::v662::enums::HudElement;
        type HudVisibility = crate::version::v662::enums::HudVisibility;
        type IdentityDefinitionType = crate::version::v662::enums::IdentityDefinitionType<Self>;
        type InputMode = crate::version::v662::enums::InputMode;
        type InteractionType = crate::version::v662::enums::InteractionType;
        type InventoryLayout = crate::version::v662::enums::InventoryLayout;
        type InventoryLeftTabIndex = crate::version::v662::enums::InventoryLeftTabIndex;
        type InventoryRightTabIndex = crate::version::v662::enums::InventoryRightTabIndex;
        type InventorySourceFlags = crate::version::v662::enums::InventorySourceFlags;
        type InventorySourceType = crate::version::v662::enums::InventorySourceType<Self>;
        type ItemDescriptorType = crate::version::v662::enums::ItemDescriptorType;
        type ItemReleaseInventoryTransactionType =
            crate::version::v662::enums::ItemReleaseInventoryTransactionType;
        type ItemStackNetResult = crate::version::v662::enums::ItemStackNetResult<Self>;
        type ItemStackRequestActionType =
            crate::version::v712::enums::ItemStackRequestActionType<Self>;
        type ItemUseInventoryTransactionType =
            crate::version::v662::enums::ItemUseInventoryTransactionType;
        type ItemUseMethod = crate::version::v662::enums::ItemUseMethod;
        type ItemUseOnActorInventoryTransactionType =
            crate::version::v662::enums::ItemUseOnActorInventoryTransactionType;
        type ItemVersion = ();
        type LabTableReactionType = crate::version::v662::enums::LabTableReactionType;
        type LessonAction = crate::version::v662::enums::LessonAction;
        type LevelEvent = crate::version::v766::enums::LevelEvent;
        type LevelSoundEventType = crate::version::v766::enums::LevelSoundEventType;
        type MinecraftPacketIds = crate::version::v662::enums::MinecraftPacketIds;
        type Mirror = crate::version::v662::enums::Mirror;
        type ModalFormCancelReason = crate::version::v662::enums::ModalFormCancelReason;
        type MolangVersion = crate::version::v662::enums::MolangVersion;
        type MovementEffectType = crate::version::v748::enums::MovementEffectType;
        type MultiplayerSettingsPacketType =
            crate::version::v662::enums::MultiplayerSettingsPacketType;
        type NewInteractionModel = crate::version::v662::enums::NewInteractionModel;
        type ObjectiveSortOrder = crate::version::v662::enums::ObjectiveSortOrder;
        type POIBlockInteractionType = crate::version::v662::enums::POIBlockInteractionType;
        type PackType = crate::version::v662::enums::PackType;
        type PacketCompressionAlgorithm = crate::version::v662::enums::PacketCompressionAlgorithm;
        type PacketViolationSeverity = crate::version::v662::enums::PacketViolationSeverity;
        type PacketViolationType = crate::version::v662::enums::PacketViolationType;
        type ParticleType = crate::version::v766::enums::ParticleType;
        type PhotoType = crate::version::v662::enums::PhotoType;
        type PlayStatus = crate::version::v662::enums::PlayStatus;
        type PlayerPermissionLevel = crate::version::v662::enums::PlayerPermissionLevel;
        type PlayerPositionMode = crate::version::v662::enums::PlayerPositionMode;
        type PlayerRespawnState = crate::version::v662::enums::PlayerRespawnState;
        type PredictionType = crate::version::v712::enums::PredictionType;
        type ResourcePackResponse = crate::version::v662::enums::ResourcePackResponse;
        type Rotation = crate::version::v662::enums::Rotation;
        type ServerAuthMovementMode = crate::version::v662::enums::ServerAuthMovementMode;
        type ShowStoreOfferRedirectType = crate::version::v662::enums::ShowStoreOfferRedirectType;
        type SimulationType = crate::version::v662::enums::SimulationType;
        type SoftEnumUpdateType = crate::version::v662::enums::SoftEnumUpdateType;
        type SpawnBiomeType = crate::version::v662::enums::SpawnBiomeType;
        type SpawnPositionType = crate::version::v662::enums::SpawnPositionType;
        type StructureBlockType = crate::version::v662::enums::StructureBlockType;
        type StructureRedstoneSaveMode = crate::version::v662::enums::StructureRedstoneSaveMode;
        type StructureTemplateRequestOperation =
            crate::version::v662::enums::StructureTemplateRequestOperation;
        type StructureTemplateResponseType =
            crate::version::v662::enums::StructureTemplateResponseType;
        type TeleportationCause = crate::version::v662::enums::TeleportationCause;
        type TextPacketType = crate::version::v662::enums::TextPacketType;
        type TextProcessingEventOrigin = crate::version::v662::enums::TextProcessingEventOrigin;
        type UIProfile = crate::version::v662::enums::UIProfile;
    }
    impl ProtoVersion for V766 {
        const PROTOCOL_VERSION: u32 = 766u32;
        const PROTOCOL_BRANCH: &str = "r/21_u5";
        const GAME_VERSION: &str = "1.21.50";
        const RAKNET_VERSION: u8 = 11u8;
    }
    #[cfg(feature = "packet-dyn")]
    impl AsRef<dyn bedrock_protocol_core::PacketDyn> for V766 {
        fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
            match self {
                V766::ActorEventPacket(pk) => pk.as_ref(),
                V766::ActorPickRequestPacket(pk) => pk.as_ref(),
                V766::AddActorPacket(pk) => pk.as_ref(),
                V766::AddBehaviourTreePacket(pk) => pk.as_ref(),
                V766::AddItemActorPacket(pk) => pk.as_ref(),
                V766::AddPaintingPacket(pk) => pk.as_ref(),
                V766::AddPlayerPacket(pk) => pk.as_ref(),
                V766::AddVolumeEntityPacket(pk) => pk.as_ref(),
                V766::AgentActionEventPacket(pk) => pk.as_ref(),
                V766::AgentAnimationPacket(pk) => pk.as_ref(),
                V766::AnimateEntityPacket(pk) => pk.as_ref(),
                V766::AnimatePacket(pk) => pk.as_ref(),
                V766::AnvilDamagePacket(pk) => pk.as_ref(),
                V766::AutomationClientConnectPacket(pk) => pk.as_ref(),
                V766::AvailableActorIdentifiersPacket(pk) => pk.as_ref(),
                V766::AvailableCommandsPacket(pk) => pk.as_ref(),
                V766::AwardAchievementPacket(pk) => pk.as_ref(),
                V766::BiomeDefinitionListPacket(pk) => pk.as_ref(),
                V766::BlockActorDataPacket(pk) => pk.as_ref(),
                V766::BlockEventPacket(pk) => pk.as_ref(),
                V766::BlockPickRequestPacket(pk) => pk.as_ref(),
                V766::BookEditPacket(pk) => pk.as_ref(),
                V766::BossEventPacket(pk) => pk.as_ref(),
                V766::CameraAimAssistPacket(pk) => pk.as_ref(),
                V766::CameraAimAssistPresetsPacket(pk) => pk.as_ref(),
                V766::CameraInstructionPacket(pk) => pk.as_ref(),
                V766::CameraPacket(pk) => pk.as_ref(),
                V766::CameraPresetsPacket(pk) => pk.as_ref(),
                V766::CameraShakePacket(pk) => pk.as_ref(),
                V766::ChangeDimensionPacket(pk) => pk.as_ref(),
                V766::ChangeMobPropertyPacket(pk) => pk.as_ref(),
                V766::ChunkRadiusUpdatedPacket(pk) => pk.as_ref(),
                V766::ClientBoundCloseFormPacket(pk) => pk.as_ref(),
                V766::ClientBoundDebugRendererPacket(pk) => pk.as_ref(),
                V766::ClientBoundMapItemDataPacket(pk) => pk.as_ref(),
                V766::ClientCacheBlobStatusPacket(pk) => pk.as_ref(),
                V766::ClientCacheMissResponsePacket(pk) => pk.as_ref(),
                V766::ClientCacheStatusPacket(pk) => pk.as_ref(),
                V766::ClientToServerHandshakePacket(pk) => pk.as_ref(),
                V766::CodeBuilderPacket(pk) => pk.as_ref(),
                V766::CodeBuilderSourcePacket(pk) => pk.as_ref(),
                V766::CommandBlockUpdatePacket(pk) => pk.as_ref(),
                V766::CommandOutputPacket(pk) => pk.as_ref(),
                V766::CommandRequestPacket(pk) => pk.as_ref(),
                V766::CompletedUsingItemPacket(pk) => pk.as_ref(),
                V766::CompressedBiomeDefinitionListPacket(pk) => pk.as_ref(),
                V766::ContainerClosePacket(pk) => pk.as_ref(),
                V766::ContainerOpenPacket(pk) => pk.as_ref(),
                V766::ContainerRegistryCleanupPacket(pk) => pk.as_ref(),
                V766::ContainerSetDataPacket(pk) => pk.as_ref(),
                V766::CorrectPlayerMovePredictionPacket(pk) => pk.as_ref(),
                V766::CraftingDataPacket(pk) => pk.as_ref(),
                V766::CreatePhotoPacket(pk) => pk.as_ref(),
                V766::CreativeContentPacket(pk) => pk.as_ref(),
                V766::CurrentStructureFeaturePacket(pk) => pk.as_ref(),
                V766::DeathInfoPacket(pk) => pk.as_ref(),
                V766::DebugInfoPacket(pk) => pk.as_ref(),
                V766::DimensionDataPacket(pk) => pk.as_ref(),
                V766::DisconnectPacket(pk) => pk.as_ref(),
                V766::EditorNetworkPacket(pk) => pk.as_ref(),
                V766::EduUriResourcePacket(pk) => pk.as_ref(),
                V766::EducationSettingsPacket(pk) => pk.as_ref(),
                V766::EmoteListPacket(pk) => pk.as_ref(),
                V766::EmotePacket(pk) => pk.as_ref(),
                V766::FeatureRegistryPacket(pk) => pk.as_ref(),
                V766::GameRulesChangedPacket(pk) => pk.as_ref(),
                V766::GameTestRequestPacket(pk) => pk.as_ref(),
                V766::GameTestResultsPacket(pk) => pk.as_ref(),
                V766::GuiDataPickItemPacket(pk) => pk.as_ref(),
                V766::HurtArmorPacket(pk) => pk.as_ref(),
                V766::InteractPacket(pk) => pk.as_ref(),
                V766::InventoryContentPacket(pk) => pk.as_ref(),
                V766::InventorySlotPacket(pk) => pk.as_ref(),
                V766::InventoryTransactionPacket(pk) => pk.as_ref(),
                V766::ItemComponentPacket(pk) => pk.as_ref(),
                V766::ItemStackRequestPacket(pk) => pk.as_ref(),
                V766::ItemStackResponsePacket(pk) => pk.as_ref(),
                V766::JigsawStructureDataPacket(pk) => pk.as_ref(),
                V766::LabTablePacket(pk) => pk.as_ref(),
                V766::LecternUpdatePacket(pk) => pk.as_ref(),
                V766::LegacyTelemetryEventPacket(pk) => pk.as_ref(),
                V766::LessonProgressPacket(pk) => pk.as_ref(),
                V766::LevelChunkPacket(pk) => pk.as_ref(),
                V766::LevelEventGenericPacket(pk) => pk.as_ref(),
                V766::LevelEventPacket(pk) => pk.as_ref(),
                V766::LevelSoundEventPacket(pk) => pk.as_ref(),
                V766::LevelSoundEventV1Packet(pk) => pk.as_ref(),
                V766::LevelSoundEventV2Packet(pk) => pk.as_ref(),
                V766::LoginPacket(pk) => pk.as_ref(),
                V766::MapCreateLockedCopyPacket(pk) => pk.as_ref(),
                V766::MapInfoRequestPacket(pk) => pk.as_ref(),
                V766::MobArmorEquipmentPacket(pk) => pk.as_ref(),
                V766::MobEffectPacket(pk) => pk.as_ref(),
                V766::MobEquipmentPacket(pk) => pk.as_ref(),
                V766::ModalFormRequestPacket(pk) => pk.as_ref(),
                V766::ModalFormResponsePacket(pk) => pk.as_ref(),
                V766::MotionPredictionHintsPacket(pk) => pk.as_ref(),
                V766::MoveActorAbsolutePacket(pk) => pk.as_ref(),
                V766::MoveActorDeltaPacket(pk) => pk.as_ref(),
                V766::MovePlayerPacket(pk) => pk.as_ref(),
                V766::MovementEffectPacket(pk) => pk.as_ref(),
                V766::MultiplayerSettingsPacket(pk) => pk.as_ref(),
                V766::NetworkChunkPublisherUpdatePacket(pk) => pk.as_ref(),
                V766::NetworkSettingsPacket(pk) => pk.as_ref(),
                V766::NetworkStackLatencyPacket(pk) => pk.as_ref(),
                V766::NpcDialoguePacket(pk) => pk.as_ref(),
                V766::NpcRequestPacket(pk) => pk.as_ref(),
                V766::OnScreenTextureAnimationPacket(pk) => pk.as_ref(),
                V766::OpenSignPacket(pk) => pk.as_ref(),
                V766::PacketViolationWarningPacket(pk) => pk.as_ref(),
                V766::PassengerJumpPacket(pk) => pk.as_ref(),
                V766::PhotoTransferPacket(pk) => pk.as_ref(),
                V766::PlaySoundPacket(pk) => pk.as_ref(),
                V766::PlayStatusPacket(pk) => pk.as_ref(),
                V766::PlayerActionPacket(pk) => pk.as_ref(),
                V766::PlayerArmorDamagePacket(pk) => pk.as_ref(),
                V766::PlayerAuthInputPacket(pk) => pk.as_ref(),
                V766::PlayerEnchantOptionsPacket(pk) => pk.as_ref(),
                V766::PlayerFogPacket(pk) => pk.as_ref(),
                V766::PlayerHotbarPacket(pk) => pk.as_ref(),
                V766::PlayerInputPacket(pk) => pk.as_ref(),
                V766::PlayerListPacket(pk) => pk.as_ref(),
                V766::PlayerSkinPacket(pk) => pk.as_ref(),
                V766::PlayerStartItemCooldownPacket(pk) => pk.as_ref(),
                V766::PlayerToggleCrafterSlotRequestPacket(pk) => pk.as_ref(),
                V766::PositionTrackingDBClientRequestPacket(pk) => pk.as_ref(),
                V766::PositionTrackingDBServerBroadcastPacket(pk) => pk.as_ref(),
                V766::PurchaseReceiptPacket(pk) => pk.as_ref(),
                V766::RefreshEntitlementsPacket(pk) => pk.as_ref(),
                V766::RemoveActorPacket(pk) => pk.as_ref(),
                V766::RemoveObjectivePacket(pk) => pk.as_ref(),
                V766::RemoveVolumeEntityPacket(pk) => pk.as_ref(),
                V766::RequestAbilityPacket(pk) => pk.as_ref(),
                V766::RequestChunkRadiusPacket(pk) => pk.as_ref(),
                V766::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                V766::RequestPermissionsPacket(pk) => pk.as_ref(),
                V766::ResourcePackChunkDataPacket(pk) => pk.as_ref(),
                V766::ResourcePackChunkRequestPacket(pk) => pk.as_ref(),
                V766::ResourcePackClientResponsePacket(pk) => pk.as_ref(),
                V766::ResourcePackDataInfoPacket(pk) => pk.as_ref(),
                V766::ResourcePackStackPacket(pk) => pk.as_ref(),
                V766::ResourcePacksInfoPacket(pk) => pk.as_ref(),
                V766::RespawnPacket(pk) => pk.as_ref(),
                V766::ScriptMessagePacket(pk) => pk.as_ref(),
                V766::ServerBoundDiagnosticsPacket(pk) => pk.as_ref(),
                V766::ServerBoundLoadingScreenPacket(pk) => pk.as_ref(),
                V766::ServerPlayerPostMovePositionPacket(pk) => pk.as_ref(),
                V766::ServerSettingsRequestPacket(pk) => pk.as_ref(),
                V766::ServerSettingsResponsePacket(pk) => pk.as_ref(),
                V766::ServerStatsPacket(pk) => pk.as_ref(),
                V766::ServerToClientHandshakePacket(pk) => pk.as_ref(),
                V766::SetActorDataPacket(pk) => pk.as_ref(),
                V766::SetActorLinkPacket(pk) => pk.as_ref(),
                V766::SetActorMotionPacket(pk) => pk.as_ref(),
                V766::SetCommandsEnabledPacket(pk) => pk.as_ref(),
                V766::SetDefaultGameTypePacket(pk) => pk.as_ref(),
                V766::SetDifficultyPacket(pk) => pk.as_ref(),
                V766::SetDisplayObjectivePacket(pk) => pk.as_ref(),
                V766::SetHealthPacket(pk) => pk.as_ref(),
                V766::SetHudPacket(pk) => pk.as_ref(),
                V766::SetLastHurtByPacket(pk) => pk.as_ref(),
                V766::SetLocalPlayerAsInitializedPacket(pk) => pk.as_ref(),
                V766::SetMovementAuthorityPacket(pk) => pk.as_ref(),
                V766::SetPlayerGameTypePacket(pk) => pk.as_ref(),
                V766::SetPlayerInventoryOptionsPacket(pk) => pk.as_ref(),
                V766::SetScorePacket(pk) => pk.as_ref(),
                V766::SetScoreboardIdentityPacket(pk) => pk.as_ref(),
                V766::SetSpawnPositionPacket(pk) => pk.as_ref(),
                V766::SetTimePacket(pk) => pk.as_ref(),
                V766::SetTitlePacket(pk) => pk.as_ref(),
                V766::SettingsCommandPacket(pk) => pk.as_ref(),
                V766::ShowCreditsPacket(pk) => pk.as_ref(),
                V766::ShowProfilePacket(pk) => pk.as_ref(),
                V766::ShowStoreOfferPacket(pk) => pk.as_ref(),
                V766::SimpleEventPacket(pk) => pk.as_ref(),
                V766::SimulationTypePacket(pk) => pk.as_ref(),
                V766::SpawnExperienceOrbPacket(pk) => pk.as_ref(),
                V766::SpawnParticleEffectPacket(pk) => pk.as_ref(),
                V766::StartGamePacket(pk) => pk.as_ref(),
                V766::StopSoundPacket(pk) => pk.as_ref(),
                V766::StructureBlockUpdatePacket(pk) => pk.as_ref(),
                V766::StructureDataRequestPacket(pk) => pk.as_ref(),
                V766::StructureDataResponsePacket(pk) => pk.as_ref(),
                V766::SubChunkPacket(pk) => pk.as_ref(),
                V766::SubChunkRequestPacket(pk) => pk.as_ref(),
                V766::SubClientLoginPacket(pk) => pk.as_ref(),
                V766::SyncActorPropertyPacket(pk) => pk.as_ref(),
                V766::TakeItemActorPacket(pk) => pk.as_ref(),
                V766::TextPacket(pk) => pk.as_ref(),
                V766::TickingAreaLoadStatusPacket(pk) => pk.as_ref(),
                V766::ToastRequestPacket(pk) => pk.as_ref(),
                V766::TransferPlayerPacket(pk) => pk.as_ref(),
                V766::TrimDataPacket(pk) => pk.as_ref(),
                V766::UnlockedRecipesPacket(pk) => pk.as_ref(),
                V766::UpdateAbilitiesPacket(pk) => pk.as_ref(),
                V766::UpdateAdventureSettingsPacket(pk) => pk.as_ref(),
                V766::UpdateAttributesPacket(pk) => pk.as_ref(),
                V766::UpdateBlockPacket(pk) => pk.as_ref(),
                V766::UpdateBlockSyncedPacket(pk) => pk.as_ref(),
                V766::UpdateClientInputLocksPacket(pk) => pk.as_ref(),
                V766::UpdateEquipPacket(pk) => pk.as_ref(),
                V766::UpdatePlayerGameTypePacket(pk) => pk.as_ref(),
                V766::UpdateSoftEnumPacket(pk) => pk.as_ref(),
                V766::UpdateSubChunkBlocksPacket(pk) => pk.as_ref(),
                V766::UpdateTradePacket(pk) => pk.as_ref(),
                V766::Unknown(pk) => pk.as_ref(),
            }
        }
    }
    #[cfg(feature = "packet-dyn")]
    impl From<V766> for Box<dyn bedrock_protocol_core::PacketDyn> {
        fn from(val: V766) -> Box<dyn bedrock_protocol_core::PacketDyn> {
            match val {
                V766::ActorEventPacket(pk) => pk,
                V766::ActorPickRequestPacket(pk) => pk,
                V766::AddActorPacket(pk) => pk,
                V766::AddBehaviourTreePacket(pk) => pk,
                V766::AddItemActorPacket(pk) => pk,
                V766::AddPaintingPacket(pk) => pk,
                V766::AddPlayerPacket(pk) => pk,
                V766::AddVolumeEntityPacket(pk) => pk,
                V766::AgentActionEventPacket(pk) => pk,
                V766::AgentAnimationPacket(pk) => pk,
                V766::AnimateEntityPacket(pk) => pk,
                V766::AnimatePacket(pk) => pk,
                V766::AnvilDamagePacket(pk) => pk,
                V766::AutomationClientConnectPacket(pk) => pk,
                V766::AvailableActorIdentifiersPacket(pk) => pk,
                V766::AvailableCommandsPacket(pk) => pk,
                V766::AwardAchievementPacket(pk) => pk,
                V766::BiomeDefinitionListPacket(pk) => pk,
                V766::BlockActorDataPacket(pk) => pk,
                V766::BlockEventPacket(pk) => pk,
                V766::BlockPickRequestPacket(pk) => pk,
                V766::BookEditPacket(pk) => pk,
                V766::BossEventPacket(pk) => pk,
                V766::CameraAimAssistPacket(pk) => pk,
                V766::CameraAimAssistPresetsPacket(pk) => pk,
                V766::CameraInstructionPacket(pk) => pk,
                V766::CameraPacket(pk) => pk,
                V766::CameraPresetsPacket(pk) => pk,
                V766::CameraShakePacket(pk) => pk,
                V766::ChangeDimensionPacket(pk) => pk,
                V766::ChangeMobPropertyPacket(pk) => pk,
                V766::ChunkRadiusUpdatedPacket(pk) => pk,
                V766::ClientBoundCloseFormPacket(pk) => pk,
                V766::ClientBoundDebugRendererPacket(pk) => pk,
                V766::ClientBoundMapItemDataPacket(pk) => pk,
                V766::ClientCacheBlobStatusPacket(pk) => pk,
                V766::ClientCacheMissResponsePacket(pk) => pk,
                V766::ClientCacheStatusPacket(pk) => pk,
                V766::ClientToServerHandshakePacket(pk) => pk,
                V766::CodeBuilderPacket(pk) => pk,
                V766::CodeBuilderSourcePacket(pk) => pk,
                V766::CommandBlockUpdatePacket(pk) => pk,
                V766::CommandOutputPacket(pk) => pk,
                V766::CommandRequestPacket(pk) => pk,
                V766::CompletedUsingItemPacket(pk) => pk,
                V766::CompressedBiomeDefinitionListPacket(pk) => pk,
                V766::ContainerClosePacket(pk) => pk,
                V766::ContainerOpenPacket(pk) => pk,
                V766::ContainerRegistryCleanupPacket(pk) => pk,
                V766::ContainerSetDataPacket(pk) => pk,
                V766::CorrectPlayerMovePredictionPacket(pk) => pk,
                V766::CraftingDataPacket(pk) => pk,
                V766::CreatePhotoPacket(pk) => pk,
                V766::CreativeContentPacket(pk) => pk,
                V766::CurrentStructureFeaturePacket(pk) => pk,
                V766::DeathInfoPacket(pk) => pk,
                V766::DebugInfoPacket(pk) => pk,
                V766::DimensionDataPacket(pk) => pk,
                V766::DisconnectPacket(pk) => pk,
                V766::EditorNetworkPacket(pk) => pk,
                V766::EduUriResourcePacket(pk) => pk,
                V766::EducationSettingsPacket(pk) => pk,
                V766::EmoteListPacket(pk) => pk,
                V766::EmotePacket(pk) => pk,
                V766::FeatureRegistryPacket(pk) => pk,
                V766::GameRulesChangedPacket(pk) => pk,
                V766::GameTestRequestPacket(pk) => pk,
                V766::GameTestResultsPacket(pk) => pk,
                V766::GuiDataPickItemPacket(pk) => pk,
                V766::HurtArmorPacket(pk) => pk,
                V766::InteractPacket(pk) => pk,
                V766::InventoryContentPacket(pk) => pk,
                V766::InventorySlotPacket(pk) => pk,
                V766::InventoryTransactionPacket(pk) => pk,
                V766::ItemComponentPacket(pk) => pk,
                V766::ItemStackRequestPacket(pk) => pk,
                V766::ItemStackResponsePacket(pk) => pk,
                V766::JigsawStructureDataPacket(pk) => pk,
                V766::LabTablePacket(pk) => pk,
                V766::LecternUpdatePacket(pk) => pk,
                V766::LegacyTelemetryEventPacket(pk) => pk,
                V766::LessonProgressPacket(pk) => pk,
                V766::LevelChunkPacket(pk) => pk,
                V766::LevelEventGenericPacket(pk) => pk,
                V766::LevelEventPacket(pk) => pk,
                V766::LevelSoundEventPacket(pk) => pk,
                V766::LevelSoundEventV1Packet(pk) => pk,
                V766::LevelSoundEventV2Packet(pk) => pk,
                V766::LoginPacket(pk) => pk,
                V766::MapCreateLockedCopyPacket(pk) => pk,
                V766::MapInfoRequestPacket(pk) => pk,
                V766::MobArmorEquipmentPacket(pk) => pk,
                V766::MobEffectPacket(pk) => pk,
                V766::MobEquipmentPacket(pk) => pk,
                V766::ModalFormRequestPacket(pk) => pk,
                V766::ModalFormResponsePacket(pk) => pk,
                V766::MotionPredictionHintsPacket(pk) => pk,
                V766::MoveActorAbsolutePacket(pk) => pk,
                V766::MoveActorDeltaPacket(pk) => pk,
                V766::MovePlayerPacket(pk) => pk,
                V766::MovementEffectPacket(pk) => pk,
                V766::MultiplayerSettingsPacket(pk) => pk,
                V766::NetworkChunkPublisherUpdatePacket(pk) => pk,
                V766::NetworkSettingsPacket(pk) => pk,
                V766::NetworkStackLatencyPacket(pk) => pk,
                V766::NpcDialoguePacket(pk) => pk,
                V766::NpcRequestPacket(pk) => pk,
                V766::OnScreenTextureAnimationPacket(pk) => pk,
                V766::OpenSignPacket(pk) => pk,
                V766::PacketViolationWarningPacket(pk) => pk,
                V766::PassengerJumpPacket(pk) => pk,
                V766::PhotoTransferPacket(pk) => pk,
                V766::PlaySoundPacket(pk) => pk,
                V766::PlayStatusPacket(pk) => pk,
                V766::PlayerActionPacket(pk) => pk,
                V766::PlayerArmorDamagePacket(pk) => pk,
                V766::PlayerAuthInputPacket(pk) => pk,
                V766::PlayerEnchantOptionsPacket(pk) => pk,
                V766::PlayerFogPacket(pk) => pk,
                V766::PlayerHotbarPacket(pk) => pk,
                V766::PlayerInputPacket(pk) => pk,
                V766::PlayerListPacket(pk) => pk,
                V766::PlayerSkinPacket(pk) => pk,
                V766::PlayerStartItemCooldownPacket(pk) => pk,
                V766::PlayerToggleCrafterSlotRequestPacket(pk) => pk,
                V766::PositionTrackingDBClientRequestPacket(pk) => pk,
                V766::PositionTrackingDBServerBroadcastPacket(pk) => pk,
                V766::PurchaseReceiptPacket(pk) => pk,
                V766::RefreshEntitlementsPacket(pk) => pk,
                V766::RemoveActorPacket(pk) => pk,
                V766::RemoveObjectivePacket(pk) => pk,
                V766::RemoveVolumeEntityPacket(pk) => pk,
                V766::RequestAbilityPacket(pk) => pk,
                V766::RequestChunkRadiusPacket(pk) => pk,
                V766::RequestNetworkSettingsPacket(pk) => pk,
                V766::RequestPermissionsPacket(pk) => pk,
                V766::ResourcePackChunkDataPacket(pk) => pk,
                V766::ResourcePackChunkRequestPacket(pk) => pk,
                V766::ResourcePackClientResponsePacket(pk) => pk,
                V766::ResourcePackDataInfoPacket(pk) => pk,
                V766::ResourcePackStackPacket(pk) => pk,
                V766::ResourcePacksInfoPacket(pk) => pk,
                V766::RespawnPacket(pk) => pk,
                V766::ScriptMessagePacket(pk) => pk,
                V766::ServerBoundDiagnosticsPacket(pk) => pk,
                V766::ServerBoundLoadingScreenPacket(pk) => pk,
                V766::ServerPlayerPostMovePositionPacket(pk) => pk,
                V766::ServerSettingsRequestPacket(pk) => pk,
                V766::ServerSettingsResponsePacket(pk) => pk,
                V766::ServerStatsPacket(pk) => pk,
                V766::ServerToClientHandshakePacket(pk) => pk,
                V766::SetActorDataPacket(pk) => pk,
                V766::SetActorLinkPacket(pk) => pk,
                V766::SetActorMotionPacket(pk) => pk,
                V766::SetCommandsEnabledPacket(pk) => pk,
                V766::SetDefaultGameTypePacket(pk) => pk,
                V766::SetDifficultyPacket(pk) => pk,
                V766::SetDisplayObjectivePacket(pk) => pk,
                V766::SetHealthPacket(pk) => pk,
                V766::SetHudPacket(pk) => pk,
                V766::SetLastHurtByPacket(pk) => pk,
                V766::SetLocalPlayerAsInitializedPacket(pk) => pk,
                V766::SetMovementAuthorityPacket(pk) => pk,
                V766::SetPlayerGameTypePacket(pk) => pk,
                V766::SetPlayerInventoryOptionsPacket(pk) => pk,
                V766::SetScorePacket(pk) => pk,
                V766::SetScoreboardIdentityPacket(pk) => pk,
                V766::SetSpawnPositionPacket(pk) => pk,
                V766::SetTimePacket(pk) => pk,
                V766::SetTitlePacket(pk) => pk,
                V766::SettingsCommandPacket(pk) => pk,
                V766::ShowCreditsPacket(pk) => pk,
                V766::ShowProfilePacket(pk) => pk,
                V766::ShowStoreOfferPacket(pk) => pk,
                V766::SimpleEventPacket(pk) => pk,
                V766::SimulationTypePacket(pk) => pk,
                V766::SpawnExperienceOrbPacket(pk) => pk,
                V766::SpawnParticleEffectPacket(pk) => pk,
                V766::StartGamePacket(pk) => pk,
                V766::StopSoundPacket(pk) => pk,
                V766::StructureBlockUpdatePacket(pk) => pk,
                V766::StructureDataRequestPacket(pk) => pk,
                V766::StructureDataResponsePacket(pk) => pk,
                V766::SubChunkPacket(pk) => pk,
                V766::SubChunkRequestPacket(pk) => pk,
                V766::SubClientLoginPacket(pk) => pk,
                V766::SyncActorPropertyPacket(pk) => pk,
                V766::TakeItemActorPacket(pk) => pk,
                V766::TextPacket(pk) => pk,
                V766::TickingAreaLoadStatusPacket(pk) => pk,
                V766::ToastRequestPacket(pk) => pk,
                V766::TransferPlayerPacket(pk) => pk,
                V766::TrimDataPacket(pk) => pk,
                V766::UnlockedRecipesPacket(pk) => pk,
                V766::UpdateAbilitiesPacket(pk) => pk,
                V766::UpdateAdventureSettingsPacket(pk) => pk,
                V766::UpdateAttributesPacket(pk) => pk,
                V766::UpdateBlockPacket(pk) => pk,
                V766::UpdateBlockSyncedPacket(pk) => pk,
                V766::UpdateClientInputLocksPacket(pk) => pk,
                V766::UpdateEquipPacket(pk) => pk,
                V766::UpdatePlayerGameTypePacket(pk) => pk,
                V766::UpdateSoftEnumPacket(pk) => pk,
                V766::UpdateSubChunkBlocksPacket(pk) => pk,
                V766::UpdateTradePacket(pk) => pk,
                V766::Unknown(pk) => pk,
            }
        }
    }
}
#[cfg(feature = "v766")]
pub use inner::*;
