#![allow(unused)]
#[cfg(feature = "v685")]
mod inner {
    use crate::ProtoVersion;
    use crate::ProtoVersionEnums;
    use crate::ProtoVersionPackets;
    use crate::ProtoVersionTypes;
    #[derive(Clone, std::fmt::Debug)]
    pub enum V685 {
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
        CameraInstructionPacket(Box<<Self as ProtoVersionPackets>::CameraInstructionPacket>),
        CameraPacket(Box<<Self as ProtoVersionPackets>::CameraPacket>),
        CameraPresetsPacket(Box<<Self as ProtoVersionPackets>::CameraPresetsPacket>),
        CameraShakePacket(Box<<Self as ProtoVersionPackets>::CameraShakePacket>),
        ChangeDimensionPacket(Box<<Self as ProtoVersionPackets>::ChangeDimensionPacket>),
        ChangeMobPropertyPacket(Box<<Self as ProtoVersionPackets>::ChangeMobPropertyPacket>),
        ChunkRadiusUpdatedPacket(Box<<Self as ProtoVersionPackets>::ChunkRadiusUpdatedPacket>),
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
        ContainerSetDataPacket(Box<<Self as ProtoVersionPackets>::ContainerSetDataPacket>),
        CorrectPlayerMovePredictionPacket(
            Box<<Self as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket>,
        ),
        CraftingDataPacket(Box<<Self as ProtoVersionPackets>::CraftingDataPacket>),
        CreatePhotoPacket(Box<<Self as ProtoVersionPackets>::CreatePhotoPacket>),
        CreativeContentPacket(Box<<Self as ProtoVersionPackets>::CreativeContentPacket>),
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
    impl bedrock_protocol_core::Packets for V685 {
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
                V685::ActorEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ActorPickRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AddActorPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AddBehaviourTreePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AddItemActorPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AddPaintingPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AddPlayerPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AddVolumeEntityPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AgentActionEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AgentAnimationPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AnimateEntityPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AnimatePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AnvilDamagePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AutomationClientConnectPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AvailableActorIdentifiersPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AvailableCommandsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::AwardAchievementPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AwardAchievementPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::BiomeDefinitionListPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::BlockActorDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::BlockEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::BlockPickRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::BookEditPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::BossEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CameraInstructionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CameraPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CameraPresetsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CameraShakePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ChangeDimensionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ChangeMobPropertyPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ChunkRadiusUpdatedPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ClientBoundDebugRendererPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ClientBoundMapItemDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ClientCacheBlobStatusPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ClientCacheMissResponsePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ClientCacheStatusPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ClientToServerHandshakePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CodeBuilderPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CodeBuilderSourcePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CommandBlockUpdatePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CommandOutputPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CommandRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CompletedUsingItemPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CompressedBiomeDefinitionListPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    CompressedBiomeDefinitionListPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ContainerClosePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ContainerOpenPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ContainerSetDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CorrectPlayerMovePredictionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CraftingDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CreatePhotoPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::CreativeContentPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::DeathInfoPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::DebugInfoPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::DimensionDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::DisconnectPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::EditorNetworkPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::EduUriResourcePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::EducationSettingsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::EmoteListPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::EmotePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::FeatureRegistryPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::GameRulesChangedPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::GameTestRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::GameTestResultsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::GuiDataPickItemPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::HurtArmorPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::InteractPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::InventoryContentPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::InventorySlotPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::InventoryTransactionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ItemComponentPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ItemStackRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ItemStackResponsePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LabTablePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LecternUpdatePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LegacyTelemetryEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LessonProgressPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LevelChunkPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LevelEventGenericPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LevelEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LevelSoundEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LevelSoundEventV1Packet(pk) => {
                    match <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV1Packet),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LevelSoundEventV2Packet(pk) => {
                    match <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV2Packet),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::LoginPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MapCreateLockedCopyPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MapInfoRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MobArmorEquipmentPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MobEffectPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MobEquipmentPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ModalFormRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ModalFormResponsePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MotionPredictionHintsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MoveActorAbsolutePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MoveActorDeltaPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MovePlayerPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::MultiplayerSettingsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::NetworkChunkPublisherUpdatePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::NetworkSettingsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::NetworkStackLatencyPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::NpcDialoguePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::NpcRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::OnScreenTextureAnimationPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::OpenSignPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PacketViolationWarningPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PassengerJumpPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PassengerJumpPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PhotoTransferPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlaySoundPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayStatusPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerActionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerArmorDamagePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerAuthInputPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerEnchantOptionsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerFogPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerHotbarPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerInputPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerInputPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerListPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerSkinPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerStartItemCooldownPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PlayerToggleCrafterSlotRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PositionTrackingDBClientRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PositionTrackingDBServerBroadcastPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::PurchaseReceiptPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RefreshEntitlementsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RemoveActorPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RemoveObjectivePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RemoveVolumeEntityPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RequestAbilityPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RequestChunkRadiusPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RequestNetworkSettingsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RequestPermissionsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ResourcePackChunkDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ResourcePackChunkRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ResourcePackClientResponsePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ResourcePackDataInfoPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ResourcePackStackPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ResourcePacksInfoPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::RespawnPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ScriptMessagePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ServerPlayerPostMovePositionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ServerSettingsRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ServerSettingsResponsePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ServerStatsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ServerToClientHandshakePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetActorDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetActorLinkPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetActorMotionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetCommandsEnabledPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetDefaultGameTypePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetDifficultyPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetDisplayObjectivePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetHealthPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetHudPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetLastHurtByPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetLocalPlayerAsInitializedPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetPlayerGameTypePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetPlayerInventoryOptionsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetScorePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetScoreboardIdentityPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetSpawnPositionPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetTimePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SetTitlePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SettingsCommandPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ShowCreditsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ShowProfilePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ShowStoreOfferPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SimpleEventPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SimulationTypePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SpawnExperienceOrbPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SpawnParticleEffectPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::StartGamePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::StopSoundPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::StructureBlockUpdatePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::StructureDataRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::StructureDataResponsePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SubChunkPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SubChunkRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SubClientLoginPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::SyncActorPropertyPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::TakeItemActorPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::TextPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::TickingAreaLoadStatusPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::ToastRequestPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::TransferPlayerPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::TrimDataPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UnlockedRecipesPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateAbilitiesPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateAdventureSettingsPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateAttributesPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateBlockPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateBlockSyncedPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateClientInputLocksPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateEquipPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdatePlayerGameTypePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateSoftEnumPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateSubChunkBlocksPacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::UpdateTradePacket(pk) => {
                    match <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V685::Unknown(pk) => stream.write_all(pk.buf.as_ref()).map_err(|e| {
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
                <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ActorEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ActorPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AddActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AddBehaviourTreePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AddItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AddPaintingPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AddPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AddVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AgentActionEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AgentAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AnimateEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AnimatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AnvilDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AutomationClientConnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AvailableActorIdentifiersPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AvailableCommandsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::AwardAchievementPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AwardAchievementPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::BiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::BlockActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::BlockEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::BlockPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::BookEditPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::BossEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CameraInstructionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CameraPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CameraPresetsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CameraShakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ChangeDimensionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ChangeMobPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ChunkRadiusUpdatedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ClientBoundDebugRendererPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ClientBoundMapItemDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ClientCacheBlobStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ClientCacheMissResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ClientCacheStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ClientToServerHandshakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CodeBuilderPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CodeBuilderSourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CommandBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CommandOutputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CommandRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CompletedUsingItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CompressedBiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    CompressedBiomeDefinitionListPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ContainerClosePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ContainerOpenPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ContainerSetDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CorrectPlayerMovePredictionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CraftingDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CreatePhotoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::CreativeContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::DeathInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::DebugInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::DimensionDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::DisconnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::EditorNetworkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::EduUriResourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::EducationSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::EmoteListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::EmotePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::FeatureRegistryPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::GameRulesChangedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::GameTestRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::GameTestResultsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::GuiDataPickItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::HurtArmorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::InteractPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::InventoryContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::InventorySlotPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::InventoryTransactionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ItemComponentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ItemStackRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ItemStackResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LabTablePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LecternUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LegacyTelemetryEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LessonProgressPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LevelChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LevelEventGenericPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LevelEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LevelSoundEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LevelSoundEventV1Packet(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV1Packet),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LevelSoundEventV2Packet(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV2Packet),
                                packet_id: <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::LoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MapCreateLockedCopyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MapInfoRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MobArmorEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MobEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MobEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ModalFormRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ModalFormResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MotionPredictionHintsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MoveActorAbsolutePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MoveActorDeltaPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MovePlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::MultiplayerSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::NetworkChunkPublisherUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::NetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::NetworkStackLatencyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::NpcDialoguePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::NpcRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::OnScreenTextureAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::OpenSignPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PacketViolationWarningPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PassengerJumpPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PassengerJumpPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PhotoTransferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlaySoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerActionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerArmorDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerAuthInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerEnchantOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerFogPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerHotbarPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerInputPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerSkinPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PlayerStartItemCooldownPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V685::PlayerToggleCrafterSlotRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V685::PositionTrackingDBClientRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V685::PositionTrackingDBServerBroadcastPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::PurchaseReceiptPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RefreshEntitlementsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RemoveActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RemoveObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RemoveVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RequestAbilityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RequestChunkRadiusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RequestNetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RequestPermissionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ResourcePackChunkDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ResourcePackChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ResourcePackClientResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ResourcePackDataInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ResourcePackStackPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ResourcePacksInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::RespawnPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ScriptMessagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ServerPlayerPostMovePositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ServerSettingsRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ServerSettingsResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ServerStatsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ServerToClientHandshakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetActorLinkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetActorMotionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetCommandsEnabledPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetDefaultGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetDifficultyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetDisplayObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetHealthPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetHudPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetLastHurtByPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetLocalPlayerAsInitializedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetPlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetPlayerInventoryOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetScorePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetScoreboardIdentityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetSpawnPositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetTimePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SetTitlePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SettingsCommandPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ShowCreditsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ShowProfilePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ShowStoreOfferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SimpleEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SimulationTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SpawnExperienceOrbPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SpawnParticleEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::StartGamePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::StopSoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::StructureBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::StructureDataRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::StructureDataResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SubChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SubChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SubClientLoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::SyncActorPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::TakeItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::TextPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::TickingAreaLoadStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::ToastRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::TransferPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::TrimDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UnlockedRecipesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateAbilitiesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateAdventureSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateAttributesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateBlockPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateBlockSyncedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateClientInputLocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateEquipPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdatePlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateSoftEnumPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateSubChunkBlocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V685::UpdateTradePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
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
                    V685::Unknown(
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
                    V685::ActorEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ActorPickRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AddActorPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AddBehaviourTreePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AddItemActorPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AddPaintingPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AddPlayerPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AddVolumeEntityPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AgentActionEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AgentAnimationPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AnimateEntityPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AnimatePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AnvilDamagePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AutomationClientConnectPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AvailableActorIdentifiersPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AvailableCommandsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::AwardAchievementPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::BiomeDefinitionListPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::BlockActorDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::BlockEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::BlockPickRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::BookEditPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::BossEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CameraInstructionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CameraPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CameraPresetsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CameraShakePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ChangeDimensionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ChangeMobPropertyPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ChunkRadiusUpdatedPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ClientBoundDebugRendererPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ClientBoundMapItemDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ClientCacheBlobStatusPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ClientCacheMissResponsePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ClientCacheStatusPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ClientToServerHandshakePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CodeBuilderPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CodeBuilderSourcePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CommandBlockUpdatePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CommandOutputPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CommandRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CompletedUsingItemPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CompressedBiomeDefinitionListPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ContainerClosePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ContainerOpenPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ContainerSetDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CorrectPlayerMovePredictionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CraftingDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CreatePhotoPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::CreativeContentPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::DeathInfoPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::DebugInfoPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::DimensionDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::DisconnectPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::EditorNetworkPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::EduUriResourcePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::EducationSettingsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::EmoteListPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::EmotePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::FeatureRegistryPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::GameRulesChangedPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::GameTestRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::GameTestResultsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::GuiDataPickItemPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::HurtArmorPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::InteractPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::InventoryContentPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::InventorySlotPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::InventoryTransactionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ItemComponentPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ItemStackRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ItemStackResponsePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LabTablePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LecternUpdatePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LegacyTelemetryEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LessonProgressPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LevelChunkPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LevelEventGenericPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LevelEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LevelSoundEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LevelSoundEventV1Packet(pk) => {
                        <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LevelSoundEventV2Packet(pk) => {
                        <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::LoginPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MapCreateLockedCopyPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MapInfoRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MobArmorEquipmentPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MobEffectPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MobEquipmentPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ModalFormRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ModalFormResponsePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MotionPredictionHintsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MoveActorAbsolutePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MoveActorDeltaPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MovePlayerPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::MultiplayerSettingsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::NetworkChunkPublisherUpdatePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::NetworkSettingsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::NetworkStackLatencyPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::NpcDialoguePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::NpcRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::OnScreenTextureAnimationPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::OpenSignPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PacketViolationWarningPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PassengerJumpPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PhotoTransferPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlaySoundPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayStatusPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerActionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerArmorDamagePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerAuthInputPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerEnchantOptionsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerFogPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerHotbarPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerInputPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerListPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerSkinPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerStartItemCooldownPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PlayerToggleCrafterSlotRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PositionTrackingDBClientRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PositionTrackingDBServerBroadcastPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::PurchaseReceiptPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RefreshEntitlementsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RemoveActorPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RemoveObjectivePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RemoveVolumeEntityPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RequestAbilityPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RequestChunkRadiusPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RequestNetworkSettingsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RequestPermissionsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ResourcePackChunkDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ResourcePackChunkRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ResourcePackClientResponsePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ResourcePackDataInfoPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ResourcePackStackPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ResourcePacksInfoPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::RespawnPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ScriptMessagePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ServerPlayerPostMovePositionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ServerSettingsRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ServerSettingsResponsePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ServerStatsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ServerToClientHandshakePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetActorDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetActorLinkPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetActorMotionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetCommandsEnabledPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetDefaultGameTypePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetDifficultyPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetDisplayObjectivePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetHealthPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetHudPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetLastHurtByPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetLocalPlayerAsInitializedPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetPlayerGameTypePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetPlayerInventoryOptionsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetScorePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetScoreboardIdentityPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetSpawnPositionPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetTimePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SetTitlePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SettingsCommandPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ShowCreditsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ShowProfilePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ShowStoreOfferPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SimpleEventPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SimulationTypePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SpawnExperienceOrbPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SpawnParticleEffectPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::StartGamePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::StopSoundPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::StructureBlockUpdatePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::StructureDataRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::StructureDataResponsePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SubChunkPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SubChunkRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SubClientLoginPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::SyncActorPropertyPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::TakeItemActorPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::TextPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::TickingAreaLoadStatusPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::ToastRequestPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::TransferPlayerPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::TrimDataPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UnlockedRecipesPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateAbilitiesPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateAdventureSettingsPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateAttributesPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateBlockPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateBlockSyncedPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateClientInputLocksPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateEquipPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdatePlayerGameTypePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateSoftEnumPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateSubChunkBlocksPacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::UpdateTradePacket(pk) => {
                        <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V685::Unknown(pk) => pk.buf.len(),
                }
        }
        #[inline]
        fn id(&self) -> u16 {
            match self {
                V685::ActorEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ActorPickRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AddActorPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AddBehaviourTreePacket(_) => {
                    <<V685 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AddItemActorPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AddPaintingPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AddPlayerPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AddVolumeEntityPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AgentActionEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AgentAnimationPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AnimateEntityPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AnimatePacket(_) => {
                    <<V685 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AnvilDamagePacket(_) => {
                    <<V685 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AutomationClientConnectPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AvailableActorIdentifiersPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AvailableCommandsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::AwardAchievementPacket(_) => {
                    <<V685 as ProtoVersionPackets>::AwardAchievementPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::BiomeDefinitionListPacket(_) => {
                    <<V685 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::BlockActorDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::BlockEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::BlockPickRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::BookEditPacket(_) => {
                    <<V685 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::BossEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CameraInstructionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CameraPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CameraPresetsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CameraShakePacket(_) => {
                    <<V685 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ChangeDimensionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ChangeMobPropertyPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ChunkRadiusUpdatedPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ClientBoundDebugRendererPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ClientBoundMapItemDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ClientCacheBlobStatusPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ClientCacheMissResponsePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ClientCacheStatusPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ClientToServerHandshakePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CodeBuilderPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CodeBuilderSourcePacket(_) => {
                    <<V685 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CommandBlockUpdatePacket(_) => {
                    <<V685 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CommandOutputPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CommandRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CompletedUsingItemPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CompressedBiomeDefinitionListPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ContainerClosePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ContainerOpenPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ContainerSetDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CorrectPlayerMovePredictionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CraftingDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CreatePhotoPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::CreativeContentPacket(_) => {
                    <<V685 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::DeathInfoPacket(_) => {
                    <<V685 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::DebugInfoPacket(_) => {
                    <<V685 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::DimensionDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::DisconnectPacket(_) => {
                    <<V685 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::EditorNetworkPacket(_) => {
                    <<V685 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::EduUriResourcePacket(_) => {
                    <<V685 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::EducationSettingsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::EmoteListPacket(_) => {
                    <<V685 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::EmotePacket(_) => {
                    <<V685 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::FeatureRegistryPacket(_) => {
                    <<V685 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::GameRulesChangedPacket(_) => {
                    <<V685 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::GameTestRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::GameTestResultsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::GuiDataPickItemPacket(_) => {
                    <<V685 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::HurtArmorPacket(_) => {
                    <<V685 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::InteractPacket(_) => {
                    <<V685 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::InventoryContentPacket(_) => {
                    <<V685 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::InventorySlotPacket(_) => {
                    <<V685 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::InventoryTransactionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ItemComponentPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ItemStackRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ItemStackResponsePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LabTablePacket(_) => {
                    <<V685 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LecternUpdatePacket(_) => {
                    <<V685 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LegacyTelemetryEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LessonProgressPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LevelChunkPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LevelEventGenericPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LevelEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LevelSoundEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::LevelSoundEventV1Packet(_) => {
                    <<V685 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID
                }
                V685::LevelSoundEventV2Packet(_) => {
                    <<V685 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID
                }
                V685::LoginPacket(_) => {
                    <<V685 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MapCreateLockedCopyPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MapInfoRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MobArmorEquipmentPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MobEffectPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MobEquipmentPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ModalFormRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ModalFormResponsePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MotionPredictionHintsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MoveActorAbsolutePacket(_) => {
                    <<V685 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MoveActorDeltaPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MovePlayerPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::MultiplayerSettingsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::NetworkChunkPublisherUpdatePacket(_) => {
                    <<V685 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::NetworkSettingsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::NetworkStackLatencyPacket(_) => {
                    <<V685 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::NpcDialoguePacket(_) => {
                    <<V685 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::NpcRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::OnScreenTextureAnimationPacket(_) => {
                    <<V685 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::OpenSignPacket(_) => {
                    <<V685 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PacketViolationWarningPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PassengerJumpPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PhotoTransferPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlaySoundPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayStatusPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerActionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerArmorDamagePacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerAuthInputPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerEnchantOptionsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerFogPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerHotbarPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerInputPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerListPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerSkinPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerStartItemCooldownPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PlayerToggleCrafterSlotRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PositionTrackingDBClientRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PositionTrackingDBServerBroadcastPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::PurchaseReceiptPacket(_) => {
                    <<V685 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RefreshEntitlementsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RemoveActorPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RemoveObjectivePacket(_) => {
                    <<V685 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RemoveVolumeEntityPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RequestAbilityPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RequestChunkRadiusPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RequestNetworkSettingsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RequestPermissionsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ResourcePackChunkDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ResourcePackChunkRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ResourcePackClientResponsePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ResourcePackDataInfoPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ResourcePackStackPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ResourcePacksInfoPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::RespawnPacket(_) => {
                    <<V685 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ScriptMessagePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ServerPlayerPostMovePositionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ServerSettingsRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ServerSettingsResponsePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ServerStatsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ServerToClientHandshakePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetActorDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetActorLinkPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetActorMotionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetCommandsEnabledPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetDefaultGameTypePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetDifficultyPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetDisplayObjectivePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetHealthPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetHudPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetLastHurtByPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetLocalPlayerAsInitializedPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetPlayerGameTypePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetPlayerInventoryOptionsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetScorePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetScoreboardIdentityPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetSpawnPositionPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetTimePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SetTitlePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SettingsCommandPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ShowCreditsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ShowProfilePacket(_) => {
                    <<V685 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ShowStoreOfferPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SimpleEventPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SimulationTypePacket(_) => {
                    <<V685 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SpawnExperienceOrbPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SpawnParticleEffectPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::StartGamePacket(_) => {
                    <<V685 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::StopSoundPacket(_) => {
                    <<V685 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::StructureBlockUpdatePacket(_) => {
                    <<V685 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::StructureDataRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::StructureDataResponsePacket(_) => {
                    <<V685 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SubChunkPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SubChunkRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SubClientLoginPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::SyncActorPropertyPacket(_) => {
                    <<V685 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::TakeItemActorPacket(_) => {
                    <<V685 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::TextPacket(_) => {
                    <<V685 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::TickingAreaLoadStatusPacket(_) => {
                    <<V685 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::ToastRequestPacket(_) => {
                    <<V685 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::TransferPlayerPacket(_) => {
                    <<V685 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::TrimDataPacket(_) => {
                    <<V685 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UnlockedRecipesPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateAbilitiesPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateAdventureSettingsPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateAttributesPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateBlockPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateBlockSyncedPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateClientInputLocksPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateEquipPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdatePlayerGameTypePacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateSoftEnumPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateSubChunkBlocksPacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V685::UpdateTradePacket(_) => {
                    <<V685 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID
                }
                V685::Unknown(pk) => pk.id,
            }
        }
    }
    impl ProtoVersionPackets for V685 {
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
        type CameraAimAssistPacket = ();
        type CameraAimAssistPresetsPacket = ();
        type CameraInstructionPacket = crate::version::v662::packets::CameraInstructionPacket<Self>;
        type CameraPacket = crate::version::v662::packets::CameraPacket<Self>;
        type CameraPresetsPacket = crate::version::v662::packets::CameraPresetsPacket<Self>;
        type CameraShakePacket = crate::version::v662::packets::CameraShakePacket<Self>;
        type CameraSplinePacket = ();
        type ChangeDimensionPacket = crate::version::v662::packets::ChangeDimensionPacket;
        type ChangeMobPropertyPacket = crate::version::v662::packets::ChangeMobPropertyPacket<Self>;
        type ChunkRadiusUpdatedPacket = crate::version::v662::packets::ChunkRadiusUpdatedPacket;
        type ClientBoundAttributeLayerSyncPacket = ();
        type ClientBoundCloseFormPacket = ();
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
        type ContainerRegistryCleanupPacket = ();
        type ContainerSetDataPacket = crate::version::v662::packets::ContainerSetDataPacket<Self>;
        type CorrectPlayerMovePredictionPacket =
            crate::version::v671::packets::CorrectPlayerMovePredictionPacket<Self>;
        type CraftingDataPacket = crate::version::v662::packets::CraftingDataPacket<Self>;
        type CreatePhotoPacket = crate::version::v662::packets::CreatePhotoPacket;
        type CreativeContentPacket = crate::version::v662::packets::CreativeContentPacket<Self>;
        type CurrentStructureFeaturePacket = ();
        type DeathInfoPacket = crate::version::v662::packets::DeathInfoPacket;
        type DebugDrawerPacket = ();
        type DebugInfoPacket = crate::version::v662::packets::DebugInfoPacket<Self>;
        type DimensionDataPacket = crate::version::v662::packets::DimensionDataPacket<Self>;
        type DisconnectPacket = crate::version::v662::packets::DisconnectPacket<Self>;
        type EditorNetworkPacket = crate::version::v662::packets::EditorNetworkPacket;
        type EduUriResourcePacket = crate::version::v662::packets::EduUriResourcePacket<Self>;
        type EducationSettingsPacket = crate::version::v662::packets::EducationSettingsPacket<Self>;
        type EmoteListPacket = crate::version::v662::packets::EmoteListPacket<Self>;
        type EmotePacket = crate::version::v662::packets::EmotePacket<Self>;
        type FeatureRegistryPacket = crate::version::v662::packets::FeatureRegistryPacket;
        type FilterTextPacket = ();
        type GameRulesChangedPacket = crate::version::v662::packets::GameRulesChangedPacket<Self>;
        type GameTestRequestPacket = crate::version::v662::packets::GameTestRequestPacket<Self>;
        type GameTestResultsPacket = crate::version::v662::packets::GameTestResultsPacket;
        type GraphicsParameterOverridePacket = ();
        type GuiDataPickItemPacket = crate::version::v662::packets::GuiDataPickItemPacket;
        type HurtArmorPacket = crate::version::v662::packets::HurtArmorPacket;
        type InteractPacket = crate::version::v662::packets::InteractPacket<Self>;
        type InventoryContentPacket = crate::version::v662::packets::InventoryContentPacket<Self>;
        type InventorySlotPacket = crate::version::v662::packets::InventorySlotPacket<Self>;
        type InventoryTransactionPacket =
            crate::version::v662::packets::InventoryTransactionPacket<Self>;
        type ItemComponentPacket = crate::version::v662::packets::ItemComponentPacket;
        type ItemStackRequestPacket = crate::version::v662::packets::ItemStackRequestPacket<Self>;
        type ItemStackResponsePacket = crate::version::v662::packets::ItemStackResponsePacket<Self>;
        type JigsawStructureDataPacket = ();
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
        type MobArmorEquipmentPacket = crate::version::v662::packets::MobArmorEquipmentPacket<Self>;
        type MobEffectPacket = crate::version::v662::packets::MobEffectPacket<Self>;
        type MobEquipmentPacket = crate::version::v662::packets::MobEquipmentPacket<Self>;
        type ModalFormRequestPacket = crate::version::v662::packets::ModalFormRequestPacket;
        type ModalFormResponsePacket = crate::version::v662::packets::ModalFormResponsePacket<Self>;
        type MotionPredictionHintsPacket =
            crate::version::v662::packets::MotionPredictionHintsPacket<Self>;
        type MoveActorAbsolutePacket = crate::version::v662::packets::MoveActorAbsolutePacket<Self>;
        type MoveActorDeltaPacket = crate::version::v662::packets::MoveActorDeltaPacket<Self>;
        type MovePlayerPacket = crate::version::v662::packets::MovePlayerPacket<Self>;
        type MovementEffectPacket = ();
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
        type PlayerArmorDamagePacket = crate::version::v662::packets::PlayerArmorDamagePacket;
        type PlayerAuthInputPacket = crate::version::v662::packets::PlayerAuthInputPacket<Self>;
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
        type ResourcePacksInfoPacket = crate::version::v662::packets::ResourcePacksInfoPacket;
        type ResourcePacksReadyForValidationPacket = ();
        type RespawnPacket = crate::version::v662::packets::RespawnPacket<Self>;
        type ScriptMessagePacket = crate::version::v662::packets::ScriptMessagePacket;
        type ServerBoundDataDrivenClosedPacket = ();
        type ServerBoundDataStorePacket = ();
        type ServerBoundDiagnosticsPacket = ();
        type ServerBoundLoadingScreenPacket = ();
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
        type SetMovementAuthorityPacket = ();
        type SetPlayerGameTypePacket = crate::version::v662::packets::SetPlayerGameTypePacket<Self>;
        type SetPlayerInventoryOptionsPacket =
            crate::version::v662::packets::SetPlayerInventoryOptionsPacket<Self>;
        type SetScorePacket = crate::version::v662::packets::SetScorePacket<Self>;
        type SetScoreboardIdentityPacket =
            crate::version::v662::packets::SetScoreboardIdentityPacket<Self>;
        type SetSpawnPositionPacket = crate::version::v662::packets::SetSpawnPositionPacket<Self>;
        type SetTimePacket = crate::version::v662::packets::SetTimePacket;
        type SetTitlePacket = crate::version::v662::packets::SetTitlePacket;
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
        type StopSoundPacket = crate::version::v662::packets::StopSoundPacket;
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
        type TransferPlayerPacket = crate::version::v662::packets::TransferPlayerPacket;
        type TrimDataPacket = crate::version::v662::packets::TrimDataPacket;
        type UnlockedRecipesPacket = crate::version::v662::packets::UnlockedRecipesPacket;
        type UpdateAbilitiesPacket = crate::version::v662::packets::UpdateAbilitiesPacket<Self>;
        type UpdateAdventureSettingsPacket =
            crate::version::v662::packets::UpdateAdventureSettingsPacket<Self>;
        type UpdateAttributesPacket = crate::version::v662::packets::UpdateAttributesPacket<Self>;
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
    impl ProtoVersionTypes for V685 {
        type ActorLink = crate::version::v662::types::ActorLink<Self>;
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
        type CameraAimAssistCategories = ();
        type CameraAimAssistCategory = ();
        type CameraAimAssistItemSettings = ();
        type CameraAimAssistPreset = ();
        type CameraAimAssistPresetDefinition = ();
        type CameraAimAssistPriority = ();
        type CameraInstruction = crate::version::v662::types::CameraInstruction<Self>;
        type CameraPreset = crate::version::v662::types::CameraPreset;
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
        type FullContainerName = ();
        type GameRulesChangedPacketData = crate::version::v662::types::GameRulesChangedPacketData;
        type InventoryAction = crate::version::v662::types::InventoryAction<Self>;
        type InventorySource = crate::version::v662::types::InventorySource<Self>;
        type InventoryTransaction = crate::version::v662::types::InventoryTransaction<Self>;
        type ItemData = crate::version::v662::types::ItemData;
        type ItemEnchants = crate::version::v662::types::ItemEnchants<Self>;
        type ItemStackRequestSlotInfo = crate::version::v662::types::ItemStackRequestSlotInfo<Self>;
        type ItemStackResponseContainerInfo =
            crate::version::v662::types::ItemStackResponseContainerInfo<Self>;
        type ItemStackResponseInfo = crate::version::v662::types::ItemStackResponseInfo<Self>;
        type ItemStackResponseSlotInfo = crate::version::v662::types::ItemStackResponseSlotInfo;
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
            crate::version::v662::types::PackedItemUseLegacyInventoryTransaction<Self>;
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
        type ShulkerBoxRecipe = crate::version::v662::types::ShulkerBoxRecipe<Self>;
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
    impl ProtoVersionEnums for V685 {
        type AbilitiesIndex = crate::version::v662::enums::AbilitiesIndex;
        type ActorBlockSyncMessageID = crate::version::v662::enums::ActorBlockSyncMessageID;
        type ActorDamageCause = crate::version::v662::enums::ActorDamageCause;
        type ActorDataIDs = crate::version::v685::enums::ActorDataIDs;
        type ActorEvent = crate::version::v662::enums::ActorEvent;
        type ActorFlags = crate::version::v671::enums::ActorFlags;
        type ActorLinkType = crate::version::v662::enums::ActorLinkType;
        type ActorType = crate::version::v662::enums::ActorType;
        type AgentActionType = crate::version::v662::enums::AgentActionType;
        type AimAssistAction = ();
        type AnimatedTextureType = crate::version::v662::enums::AnimatedTextureType;
        type AnimationExpression = crate::version::v662::enums::AnimationExpression;
        type AnimationMode = crate::version::v662::enums::AnimationMode;
        type AttributeModifierOperation = crate::version::v662::enums::AttributeModifierOperation;
        type AttributeOperands = crate::version::v662::enums::AttributeOperands;
        type AuthoritativeMovementMode = ();
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
        type ContainerEnumName = crate::version::v662::enums::ContainerEnumName;
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
            crate::version::v662::enums::ItemStackRequestActionType<Self>;
        type ItemUseInventoryTransactionType =
            crate::version::v662::enums::ItemUseInventoryTransactionType;
        type ItemUseMethod = crate::version::v662::enums::ItemUseMethod;
        type ItemUseOnActorInventoryTransactionType =
            crate::version::v662::enums::ItemUseOnActorInventoryTransactionType;
        type ItemVersion = ();
        type LabTableReactionType = crate::version::v662::enums::LabTableReactionType;
        type LessonAction = crate::version::v662::enums::LessonAction;
        type LevelEvent = crate::version::v685::enums::LevelEvent;
        type LevelSoundEventType = crate::version::v685::enums::LevelSoundEventType;
        type MinecraftPacketIds = crate::version::v662::enums::MinecraftPacketIds;
        type Mirror = crate::version::v662::enums::Mirror;
        type ModalFormCancelReason = crate::version::v662::enums::ModalFormCancelReason;
        type MolangVersion = crate::version::v662::enums::MolangVersion;
        type MovementEffectType = ();
        type MultiplayerSettingsPacketType =
            crate::version::v662::enums::MultiplayerSettingsPacketType;
        type NewInteractionModel = crate::version::v662::enums::NewInteractionModel;
        type ObjectiveSortOrder = crate::version::v662::enums::ObjectiveSortOrder;
        type POIBlockInteractionType = crate::version::v662::enums::POIBlockInteractionType;
        type PackType = crate::version::v662::enums::PackType;
        type PacketCompressionAlgorithm = crate::version::v662::enums::PacketCompressionAlgorithm;
        type PacketViolationSeverity = crate::version::v662::enums::PacketViolationSeverity;
        type PacketViolationType = crate::version::v662::enums::PacketViolationType;
        type ParticleType = crate::version::v685::enums::ParticleType;
        type PhotoType = crate::version::v662::enums::PhotoType;
        type PlayStatus = crate::version::v662::enums::PlayStatus;
        type PlayerPermissionLevel = crate::version::v662::enums::PlayerPermissionLevel;
        type PlayerPositionMode = crate::version::v662::enums::PlayerPositionMode;
        type PlayerRespawnState = crate::version::v662::enums::PlayerRespawnState;
        type PredictionType = crate::version::v671::enums::PredictionType;
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
    impl ProtoVersion for V685 {
        const PROTOCOL_VERSION: u32 = 685u32;
        const PROTOCOL_BRANCH: &str = "r/21_u0";
        const GAME_VERSION: &str = "1.21.0";
        const RAKNET_VERSION: u8 = 11u8;
    }
    #[cfg(feature = "packet-dyn")]
    impl AsRef<dyn bedrock_protocol_core::PacketDyn> for V685 {
        fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
            match self {
                V685::ActorEventPacket(pk) => pk.as_ref(),
                V685::ActorPickRequestPacket(pk) => pk.as_ref(),
                V685::AddActorPacket(pk) => pk.as_ref(),
                V685::AddBehaviourTreePacket(pk) => pk.as_ref(),
                V685::AddItemActorPacket(pk) => pk.as_ref(),
                V685::AddPaintingPacket(pk) => pk.as_ref(),
                V685::AddPlayerPacket(pk) => pk.as_ref(),
                V685::AddVolumeEntityPacket(pk) => pk.as_ref(),
                V685::AgentActionEventPacket(pk) => pk.as_ref(),
                V685::AgentAnimationPacket(pk) => pk.as_ref(),
                V685::AnimateEntityPacket(pk) => pk.as_ref(),
                V685::AnimatePacket(pk) => pk.as_ref(),
                V685::AnvilDamagePacket(pk) => pk.as_ref(),
                V685::AutomationClientConnectPacket(pk) => pk.as_ref(),
                V685::AvailableActorIdentifiersPacket(pk) => pk.as_ref(),
                V685::AvailableCommandsPacket(pk) => pk.as_ref(),
                V685::AwardAchievementPacket(pk) => pk.as_ref(),
                V685::BiomeDefinitionListPacket(pk) => pk.as_ref(),
                V685::BlockActorDataPacket(pk) => pk.as_ref(),
                V685::BlockEventPacket(pk) => pk.as_ref(),
                V685::BlockPickRequestPacket(pk) => pk.as_ref(),
                V685::BookEditPacket(pk) => pk.as_ref(),
                V685::BossEventPacket(pk) => pk.as_ref(),
                V685::CameraInstructionPacket(pk) => pk.as_ref(),
                V685::CameraPacket(pk) => pk.as_ref(),
                V685::CameraPresetsPacket(pk) => pk.as_ref(),
                V685::CameraShakePacket(pk) => pk.as_ref(),
                V685::ChangeDimensionPacket(pk) => pk.as_ref(),
                V685::ChangeMobPropertyPacket(pk) => pk.as_ref(),
                V685::ChunkRadiusUpdatedPacket(pk) => pk.as_ref(),
                V685::ClientBoundDebugRendererPacket(pk) => pk.as_ref(),
                V685::ClientBoundMapItemDataPacket(pk) => pk.as_ref(),
                V685::ClientCacheBlobStatusPacket(pk) => pk.as_ref(),
                V685::ClientCacheMissResponsePacket(pk) => pk.as_ref(),
                V685::ClientCacheStatusPacket(pk) => pk.as_ref(),
                V685::ClientToServerHandshakePacket(pk) => pk.as_ref(),
                V685::CodeBuilderPacket(pk) => pk.as_ref(),
                V685::CodeBuilderSourcePacket(pk) => pk.as_ref(),
                V685::CommandBlockUpdatePacket(pk) => pk.as_ref(),
                V685::CommandOutputPacket(pk) => pk.as_ref(),
                V685::CommandRequestPacket(pk) => pk.as_ref(),
                V685::CompletedUsingItemPacket(pk) => pk.as_ref(),
                V685::CompressedBiomeDefinitionListPacket(pk) => pk.as_ref(),
                V685::ContainerClosePacket(pk) => pk.as_ref(),
                V685::ContainerOpenPacket(pk) => pk.as_ref(),
                V685::ContainerSetDataPacket(pk) => pk.as_ref(),
                V685::CorrectPlayerMovePredictionPacket(pk) => pk.as_ref(),
                V685::CraftingDataPacket(pk) => pk.as_ref(),
                V685::CreatePhotoPacket(pk) => pk.as_ref(),
                V685::CreativeContentPacket(pk) => pk.as_ref(),
                V685::DeathInfoPacket(pk) => pk.as_ref(),
                V685::DebugInfoPacket(pk) => pk.as_ref(),
                V685::DimensionDataPacket(pk) => pk.as_ref(),
                V685::DisconnectPacket(pk) => pk.as_ref(),
                V685::EditorNetworkPacket(pk) => pk.as_ref(),
                V685::EduUriResourcePacket(pk) => pk.as_ref(),
                V685::EducationSettingsPacket(pk) => pk.as_ref(),
                V685::EmoteListPacket(pk) => pk.as_ref(),
                V685::EmotePacket(pk) => pk.as_ref(),
                V685::FeatureRegistryPacket(pk) => pk.as_ref(),
                V685::GameRulesChangedPacket(pk) => pk.as_ref(),
                V685::GameTestRequestPacket(pk) => pk.as_ref(),
                V685::GameTestResultsPacket(pk) => pk.as_ref(),
                V685::GuiDataPickItemPacket(pk) => pk.as_ref(),
                V685::HurtArmorPacket(pk) => pk.as_ref(),
                V685::InteractPacket(pk) => pk.as_ref(),
                V685::InventoryContentPacket(pk) => pk.as_ref(),
                V685::InventorySlotPacket(pk) => pk.as_ref(),
                V685::InventoryTransactionPacket(pk) => pk.as_ref(),
                V685::ItemComponentPacket(pk) => pk.as_ref(),
                V685::ItemStackRequestPacket(pk) => pk.as_ref(),
                V685::ItemStackResponsePacket(pk) => pk.as_ref(),
                V685::LabTablePacket(pk) => pk.as_ref(),
                V685::LecternUpdatePacket(pk) => pk.as_ref(),
                V685::LegacyTelemetryEventPacket(pk) => pk.as_ref(),
                V685::LessonProgressPacket(pk) => pk.as_ref(),
                V685::LevelChunkPacket(pk) => pk.as_ref(),
                V685::LevelEventGenericPacket(pk) => pk.as_ref(),
                V685::LevelEventPacket(pk) => pk.as_ref(),
                V685::LevelSoundEventPacket(pk) => pk.as_ref(),
                V685::LevelSoundEventV1Packet(pk) => pk.as_ref(),
                V685::LevelSoundEventV2Packet(pk) => pk.as_ref(),
                V685::LoginPacket(pk) => pk.as_ref(),
                V685::MapCreateLockedCopyPacket(pk) => pk.as_ref(),
                V685::MapInfoRequestPacket(pk) => pk.as_ref(),
                V685::MobArmorEquipmentPacket(pk) => pk.as_ref(),
                V685::MobEffectPacket(pk) => pk.as_ref(),
                V685::MobEquipmentPacket(pk) => pk.as_ref(),
                V685::ModalFormRequestPacket(pk) => pk.as_ref(),
                V685::ModalFormResponsePacket(pk) => pk.as_ref(),
                V685::MotionPredictionHintsPacket(pk) => pk.as_ref(),
                V685::MoveActorAbsolutePacket(pk) => pk.as_ref(),
                V685::MoveActorDeltaPacket(pk) => pk.as_ref(),
                V685::MovePlayerPacket(pk) => pk.as_ref(),
                V685::MultiplayerSettingsPacket(pk) => pk.as_ref(),
                V685::NetworkChunkPublisherUpdatePacket(pk) => pk.as_ref(),
                V685::NetworkSettingsPacket(pk) => pk.as_ref(),
                V685::NetworkStackLatencyPacket(pk) => pk.as_ref(),
                V685::NpcDialoguePacket(pk) => pk.as_ref(),
                V685::NpcRequestPacket(pk) => pk.as_ref(),
                V685::OnScreenTextureAnimationPacket(pk) => pk.as_ref(),
                V685::OpenSignPacket(pk) => pk.as_ref(),
                V685::PacketViolationWarningPacket(pk) => pk.as_ref(),
                V685::PassengerJumpPacket(pk) => pk.as_ref(),
                V685::PhotoTransferPacket(pk) => pk.as_ref(),
                V685::PlaySoundPacket(pk) => pk.as_ref(),
                V685::PlayStatusPacket(pk) => pk.as_ref(),
                V685::PlayerActionPacket(pk) => pk.as_ref(),
                V685::PlayerArmorDamagePacket(pk) => pk.as_ref(),
                V685::PlayerAuthInputPacket(pk) => pk.as_ref(),
                V685::PlayerEnchantOptionsPacket(pk) => pk.as_ref(),
                V685::PlayerFogPacket(pk) => pk.as_ref(),
                V685::PlayerHotbarPacket(pk) => pk.as_ref(),
                V685::PlayerInputPacket(pk) => pk.as_ref(),
                V685::PlayerListPacket(pk) => pk.as_ref(),
                V685::PlayerSkinPacket(pk) => pk.as_ref(),
                V685::PlayerStartItemCooldownPacket(pk) => pk.as_ref(),
                V685::PlayerToggleCrafterSlotRequestPacket(pk) => pk.as_ref(),
                V685::PositionTrackingDBClientRequestPacket(pk) => pk.as_ref(),
                V685::PositionTrackingDBServerBroadcastPacket(pk) => pk.as_ref(),
                V685::PurchaseReceiptPacket(pk) => pk.as_ref(),
                V685::RefreshEntitlementsPacket(pk) => pk.as_ref(),
                V685::RemoveActorPacket(pk) => pk.as_ref(),
                V685::RemoveObjectivePacket(pk) => pk.as_ref(),
                V685::RemoveVolumeEntityPacket(pk) => pk.as_ref(),
                V685::RequestAbilityPacket(pk) => pk.as_ref(),
                V685::RequestChunkRadiusPacket(pk) => pk.as_ref(),
                V685::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                V685::RequestPermissionsPacket(pk) => pk.as_ref(),
                V685::ResourcePackChunkDataPacket(pk) => pk.as_ref(),
                V685::ResourcePackChunkRequestPacket(pk) => pk.as_ref(),
                V685::ResourcePackClientResponsePacket(pk) => pk.as_ref(),
                V685::ResourcePackDataInfoPacket(pk) => pk.as_ref(),
                V685::ResourcePackStackPacket(pk) => pk.as_ref(),
                V685::ResourcePacksInfoPacket(pk) => pk.as_ref(),
                V685::RespawnPacket(pk) => pk.as_ref(),
                V685::ScriptMessagePacket(pk) => pk.as_ref(),
                V685::ServerPlayerPostMovePositionPacket(pk) => pk.as_ref(),
                V685::ServerSettingsRequestPacket(pk) => pk.as_ref(),
                V685::ServerSettingsResponsePacket(pk) => pk.as_ref(),
                V685::ServerStatsPacket(pk) => pk.as_ref(),
                V685::ServerToClientHandshakePacket(pk) => pk.as_ref(),
                V685::SetActorDataPacket(pk) => pk.as_ref(),
                V685::SetActorLinkPacket(pk) => pk.as_ref(),
                V685::SetActorMotionPacket(pk) => pk.as_ref(),
                V685::SetCommandsEnabledPacket(pk) => pk.as_ref(),
                V685::SetDefaultGameTypePacket(pk) => pk.as_ref(),
                V685::SetDifficultyPacket(pk) => pk.as_ref(),
                V685::SetDisplayObjectivePacket(pk) => pk.as_ref(),
                V685::SetHealthPacket(pk) => pk.as_ref(),
                V685::SetHudPacket(pk) => pk.as_ref(),
                V685::SetLastHurtByPacket(pk) => pk.as_ref(),
                V685::SetLocalPlayerAsInitializedPacket(pk) => pk.as_ref(),
                V685::SetPlayerGameTypePacket(pk) => pk.as_ref(),
                V685::SetPlayerInventoryOptionsPacket(pk) => pk.as_ref(),
                V685::SetScorePacket(pk) => pk.as_ref(),
                V685::SetScoreboardIdentityPacket(pk) => pk.as_ref(),
                V685::SetSpawnPositionPacket(pk) => pk.as_ref(),
                V685::SetTimePacket(pk) => pk.as_ref(),
                V685::SetTitlePacket(pk) => pk.as_ref(),
                V685::SettingsCommandPacket(pk) => pk.as_ref(),
                V685::ShowCreditsPacket(pk) => pk.as_ref(),
                V685::ShowProfilePacket(pk) => pk.as_ref(),
                V685::ShowStoreOfferPacket(pk) => pk.as_ref(),
                V685::SimpleEventPacket(pk) => pk.as_ref(),
                V685::SimulationTypePacket(pk) => pk.as_ref(),
                V685::SpawnExperienceOrbPacket(pk) => pk.as_ref(),
                V685::SpawnParticleEffectPacket(pk) => pk.as_ref(),
                V685::StartGamePacket(pk) => pk.as_ref(),
                V685::StopSoundPacket(pk) => pk.as_ref(),
                V685::StructureBlockUpdatePacket(pk) => pk.as_ref(),
                V685::StructureDataRequestPacket(pk) => pk.as_ref(),
                V685::StructureDataResponsePacket(pk) => pk.as_ref(),
                V685::SubChunkPacket(pk) => pk.as_ref(),
                V685::SubChunkRequestPacket(pk) => pk.as_ref(),
                V685::SubClientLoginPacket(pk) => pk.as_ref(),
                V685::SyncActorPropertyPacket(pk) => pk.as_ref(),
                V685::TakeItemActorPacket(pk) => pk.as_ref(),
                V685::TextPacket(pk) => pk.as_ref(),
                V685::TickingAreaLoadStatusPacket(pk) => pk.as_ref(),
                V685::ToastRequestPacket(pk) => pk.as_ref(),
                V685::TransferPlayerPacket(pk) => pk.as_ref(),
                V685::TrimDataPacket(pk) => pk.as_ref(),
                V685::UnlockedRecipesPacket(pk) => pk.as_ref(),
                V685::UpdateAbilitiesPacket(pk) => pk.as_ref(),
                V685::UpdateAdventureSettingsPacket(pk) => pk.as_ref(),
                V685::UpdateAttributesPacket(pk) => pk.as_ref(),
                V685::UpdateBlockPacket(pk) => pk.as_ref(),
                V685::UpdateBlockSyncedPacket(pk) => pk.as_ref(),
                V685::UpdateClientInputLocksPacket(pk) => pk.as_ref(),
                V685::UpdateEquipPacket(pk) => pk.as_ref(),
                V685::UpdatePlayerGameTypePacket(pk) => pk.as_ref(),
                V685::UpdateSoftEnumPacket(pk) => pk.as_ref(),
                V685::UpdateSubChunkBlocksPacket(pk) => pk.as_ref(),
                V685::UpdateTradePacket(pk) => pk.as_ref(),
                V685::Unknown(pk) => pk.as_ref(),
            }
        }
    }
    #[cfg(feature = "packet-dyn")]
    impl From<V685> for Box<dyn bedrock_protocol_core::PacketDyn> {
        fn from(val: V685) -> Box<dyn bedrock_protocol_core::PacketDyn> {
            match val {
                V685::ActorEventPacket(pk) => pk,
                V685::ActorPickRequestPacket(pk) => pk,
                V685::AddActorPacket(pk) => pk,
                V685::AddBehaviourTreePacket(pk) => pk,
                V685::AddItemActorPacket(pk) => pk,
                V685::AddPaintingPacket(pk) => pk,
                V685::AddPlayerPacket(pk) => pk,
                V685::AddVolumeEntityPacket(pk) => pk,
                V685::AgentActionEventPacket(pk) => pk,
                V685::AgentAnimationPacket(pk) => pk,
                V685::AnimateEntityPacket(pk) => pk,
                V685::AnimatePacket(pk) => pk,
                V685::AnvilDamagePacket(pk) => pk,
                V685::AutomationClientConnectPacket(pk) => pk,
                V685::AvailableActorIdentifiersPacket(pk) => pk,
                V685::AvailableCommandsPacket(pk) => pk,
                V685::AwardAchievementPacket(pk) => pk,
                V685::BiomeDefinitionListPacket(pk) => pk,
                V685::BlockActorDataPacket(pk) => pk,
                V685::BlockEventPacket(pk) => pk,
                V685::BlockPickRequestPacket(pk) => pk,
                V685::BookEditPacket(pk) => pk,
                V685::BossEventPacket(pk) => pk,
                V685::CameraInstructionPacket(pk) => pk,
                V685::CameraPacket(pk) => pk,
                V685::CameraPresetsPacket(pk) => pk,
                V685::CameraShakePacket(pk) => pk,
                V685::ChangeDimensionPacket(pk) => pk,
                V685::ChangeMobPropertyPacket(pk) => pk,
                V685::ChunkRadiusUpdatedPacket(pk) => pk,
                V685::ClientBoundDebugRendererPacket(pk) => pk,
                V685::ClientBoundMapItemDataPacket(pk) => pk,
                V685::ClientCacheBlobStatusPacket(pk) => pk,
                V685::ClientCacheMissResponsePacket(pk) => pk,
                V685::ClientCacheStatusPacket(pk) => pk,
                V685::ClientToServerHandshakePacket(pk) => pk,
                V685::CodeBuilderPacket(pk) => pk,
                V685::CodeBuilderSourcePacket(pk) => pk,
                V685::CommandBlockUpdatePacket(pk) => pk,
                V685::CommandOutputPacket(pk) => pk,
                V685::CommandRequestPacket(pk) => pk,
                V685::CompletedUsingItemPacket(pk) => pk,
                V685::CompressedBiomeDefinitionListPacket(pk) => pk,
                V685::ContainerClosePacket(pk) => pk,
                V685::ContainerOpenPacket(pk) => pk,
                V685::ContainerSetDataPacket(pk) => pk,
                V685::CorrectPlayerMovePredictionPacket(pk) => pk,
                V685::CraftingDataPacket(pk) => pk,
                V685::CreatePhotoPacket(pk) => pk,
                V685::CreativeContentPacket(pk) => pk,
                V685::DeathInfoPacket(pk) => pk,
                V685::DebugInfoPacket(pk) => pk,
                V685::DimensionDataPacket(pk) => pk,
                V685::DisconnectPacket(pk) => pk,
                V685::EditorNetworkPacket(pk) => pk,
                V685::EduUriResourcePacket(pk) => pk,
                V685::EducationSettingsPacket(pk) => pk,
                V685::EmoteListPacket(pk) => pk,
                V685::EmotePacket(pk) => pk,
                V685::FeatureRegistryPacket(pk) => pk,
                V685::GameRulesChangedPacket(pk) => pk,
                V685::GameTestRequestPacket(pk) => pk,
                V685::GameTestResultsPacket(pk) => pk,
                V685::GuiDataPickItemPacket(pk) => pk,
                V685::HurtArmorPacket(pk) => pk,
                V685::InteractPacket(pk) => pk,
                V685::InventoryContentPacket(pk) => pk,
                V685::InventorySlotPacket(pk) => pk,
                V685::InventoryTransactionPacket(pk) => pk,
                V685::ItemComponentPacket(pk) => pk,
                V685::ItemStackRequestPacket(pk) => pk,
                V685::ItemStackResponsePacket(pk) => pk,
                V685::LabTablePacket(pk) => pk,
                V685::LecternUpdatePacket(pk) => pk,
                V685::LegacyTelemetryEventPacket(pk) => pk,
                V685::LessonProgressPacket(pk) => pk,
                V685::LevelChunkPacket(pk) => pk,
                V685::LevelEventGenericPacket(pk) => pk,
                V685::LevelEventPacket(pk) => pk,
                V685::LevelSoundEventPacket(pk) => pk,
                V685::LevelSoundEventV1Packet(pk) => pk,
                V685::LevelSoundEventV2Packet(pk) => pk,
                V685::LoginPacket(pk) => pk,
                V685::MapCreateLockedCopyPacket(pk) => pk,
                V685::MapInfoRequestPacket(pk) => pk,
                V685::MobArmorEquipmentPacket(pk) => pk,
                V685::MobEffectPacket(pk) => pk,
                V685::MobEquipmentPacket(pk) => pk,
                V685::ModalFormRequestPacket(pk) => pk,
                V685::ModalFormResponsePacket(pk) => pk,
                V685::MotionPredictionHintsPacket(pk) => pk,
                V685::MoveActorAbsolutePacket(pk) => pk,
                V685::MoveActorDeltaPacket(pk) => pk,
                V685::MovePlayerPacket(pk) => pk,
                V685::MultiplayerSettingsPacket(pk) => pk,
                V685::NetworkChunkPublisherUpdatePacket(pk) => pk,
                V685::NetworkSettingsPacket(pk) => pk,
                V685::NetworkStackLatencyPacket(pk) => pk,
                V685::NpcDialoguePacket(pk) => pk,
                V685::NpcRequestPacket(pk) => pk,
                V685::OnScreenTextureAnimationPacket(pk) => pk,
                V685::OpenSignPacket(pk) => pk,
                V685::PacketViolationWarningPacket(pk) => pk,
                V685::PassengerJumpPacket(pk) => pk,
                V685::PhotoTransferPacket(pk) => pk,
                V685::PlaySoundPacket(pk) => pk,
                V685::PlayStatusPacket(pk) => pk,
                V685::PlayerActionPacket(pk) => pk,
                V685::PlayerArmorDamagePacket(pk) => pk,
                V685::PlayerAuthInputPacket(pk) => pk,
                V685::PlayerEnchantOptionsPacket(pk) => pk,
                V685::PlayerFogPacket(pk) => pk,
                V685::PlayerHotbarPacket(pk) => pk,
                V685::PlayerInputPacket(pk) => pk,
                V685::PlayerListPacket(pk) => pk,
                V685::PlayerSkinPacket(pk) => pk,
                V685::PlayerStartItemCooldownPacket(pk) => pk,
                V685::PlayerToggleCrafterSlotRequestPacket(pk) => pk,
                V685::PositionTrackingDBClientRequestPacket(pk) => pk,
                V685::PositionTrackingDBServerBroadcastPacket(pk) => pk,
                V685::PurchaseReceiptPacket(pk) => pk,
                V685::RefreshEntitlementsPacket(pk) => pk,
                V685::RemoveActorPacket(pk) => pk,
                V685::RemoveObjectivePacket(pk) => pk,
                V685::RemoveVolumeEntityPacket(pk) => pk,
                V685::RequestAbilityPacket(pk) => pk,
                V685::RequestChunkRadiusPacket(pk) => pk,
                V685::RequestNetworkSettingsPacket(pk) => pk,
                V685::RequestPermissionsPacket(pk) => pk,
                V685::ResourcePackChunkDataPacket(pk) => pk,
                V685::ResourcePackChunkRequestPacket(pk) => pk,
                V685::ResourcePackClientResponsePacket(pk) => pk,
                V685::ResourcePackDataInfoPacket(pk) => pk,
                V685::ResourcePackStackPacket(pk) => pk,
                V685::ResourcePacksInfoPacket(pk) => pk,
                V685::RespawnPacket(pk) => pk,
                V685::ScriptMessagePacket(pk) => pk,
                V685::ServerPlayerPostMovePositionPacket(pk) => pk,
                V685::ServerSettingsRequestPacket(pk) => pk,
                V685::ServerSettingsResponsePacket(pk) => pk,
                V685::ServerStatsPacket(pk) => pk,
                V685::ServerToClientHandshakePacket(pk) => pk,
                V685::SetActorDataPacket(pk) => pk,
                V685::SetActorLinkPacket(pk) => pk,
                V685::SetActorMotionPacket(pk) => pk,
                V685::SetCommandsEnabledPacket(pk) => pk,
                V685::SetDefaultGameTypePacket(pk) => pk,
                V685::SetDifficultyPacket(pk) => pk,
                V685::SetDisplayObjectivePacket(pk) => pk,
                V685::SetHealthPacket(pk) => pk,
                V685::SetHudPacket(pk) => pk,
                V685::SetLastHurtByPacket(pk) => pk,
                V685::SetLocalPlayerAsInitializedPacket(pk) => pk,
                V685::SetPlayerGameTypePacket(pk) => pk,
                V685::SetPlayerInventoryOptionsPacket(pk) => pk,
                V685::SetScorePacket(pk) => pk,
                V685::SetScoreboardIdentityPacket(pk) => pk,
                V685::SetSpawnPositionPacket(pk) => pk,
                V685::SetTimePacket(pk) => pk,
                V685::SetTitlePacket(pk) => pk,
                V685::SettingsCommandPacket(pk) => pk,
                V685::ShowCreditsPacket(pk) => pk,
                V685::ShowProfilePacket(pk) => pk,
                V685::ShowStoreOfferPacket(pk) => pk,
                V685::SimpleEventPacket(pk) => pk,
                V685::SimulationTypePacket(pk) => pk,
                V685::SpawnExperienceOrbPacket(pk) => pk,
                V685::SpawnParticleEffectPacket(pk) => pk,
                V685::StartGamePacket(pk) => pk,
                V685::StopSoundPacket(pk) => pk,
                V685::StructureBlockUpdatePacket(pk) => pk,
                V685::StructureDataRequestPacket(pk) => pk,
                V685::StructureDataResponsePacket(pk) => pk,
                V685::SubChunkPacket(pk) => pk,
                V685::SubChunkRequestPacket(pk) => pk,
                V685::SubClientLoginPacket(pk) => pk,
                V685::SyncActorPropertyPacket(pk) => pk,
                V685::TakeItemActorPacket(pk) => pk,
                V685::TextPacket(pk) => pk,
                V685::TickingAreaLoadStatusPacket(pk) => pk,
                V685::ToastRequestPacket(pk) => pk,
                V685::TransferPlayerPacket(pk) => pk,
                V685::TrimDataPacket(pk) => pk,
                V685::UnlockedRecipesPacket(pk) => pk,
                V685::UpdateAbilitiesPacket(pk) => pk,
                V685::UpdateAdventureSettingsPacket(pk) => pk,
                V685::UpdateAttributesPacket(pk) => pk,
                V685::UpdateBlockPacket(pk) => pk,
                V685::UpdateBlockSyncedPacket(pk) => pk,
                V685::UpdateClientInputLocksPacket(pk) => pk,
                V685::UpdateEquipPacket(pk) => pk,
                V685::UpdatePlayerGameTypePacket(pk) => pk,
                V685::UpdateSoftEnumPacket(pk) => pk,
                V685::UpdateSubChunkBlocksPacket(pk) => pk,
                V685::UpdateTradePacket(pk) => pk,
                V685::Unknown(pk) => pk,
            }
        }
    }
}
#[cfg(feature = "v685")]
pub use inner::*;
