#![allow(unused)]
#[cfg(feature = "v662")]
mod inner {
    use crate::ProtoVersion;
    use crate::ProtoVersionEnums;
    use crate::ProtoVersionPackets;
    use crate::ProtoVersionTypes;
    #[derive(Clone, std::fmt::Debug)]
    pub enum V662 {
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
        FilterTextPacket(Box<<Self as ProtoVersionPackets>::FilterTextPacket>),
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
        TickSyncPacket(Box<<Self as ProtoVersionPackets>::TickSyncPacket>),
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
    impl bedrock_protocol_core::Packets for V662 {
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
                V662::ActorEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ActorPickRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AddActorPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AddBehaviourTreePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AddItemActorPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AddPaintingPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AddPlayerPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AddVolumeEntityPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AgentActionEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AgentAnimationPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AnimateEntityPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AnimatePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AnvilDamagePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AutomationClientConnectPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AvailableActorIdentifiersPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::AvailableCommandsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::BiomeDefinitionListPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::BlockActorDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::BlockEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::BlockPickRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::BookEditPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::BossEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CameraInstructionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CameraPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CameraPresetsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CameraShakePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ChangeDimensionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ChangeMobPropertyPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ChunkRadiusUpdatedPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ClientBoundDebugRendererPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ClientBoundMapItemDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ClientCacheBlobStatusPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ClientCacheMissResponsePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ClientCacheStatusPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ClientToServerHandshakePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CodeBuilderPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CodeBuilderSourcePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CommandBlockUpdatePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CommandOutputPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CommandRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CompletedUsingItemPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CompressedBiomeDefinitionListPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    CompressedBiomeDefinitionListPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ContainerClosePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ContainerOpenPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ContainerSetDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CorrectPlayerMovePredictionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CraftingDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CreatePhotoPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::CreativeContentPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::DeathInfoPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::DebugInfoPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::DimensionDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::DisconnectPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::EditorNetworkPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::EduUriResourcePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::EducationSettingsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::EmoteListPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::EmotePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::FeatureRegistryPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::FilterTextPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FilterTextPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::GameRulesChangedPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::GameTestRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::GameTestResultsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::GuiDataPickItemPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::HurtArmorPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::InteractPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::InventoryContentPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::InventorySlotPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::InventoryTransactionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ItemComponentPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ItemStackRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ItemStackResponsePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LabTablePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LecternUpdatePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LegacyTelemetryEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LessonProgressPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LevelChunkPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LevelEventGenericPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LevelEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LevelSoundEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LevelSoundEventV1Packet(pk) => {
                    match <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV1Packet),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LevelSoundEventV2Packet(pk) => {
                    match <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV2Packet),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::LoginPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MapCreateLockedCopyPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MapInfoRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MobArmorEquipmentPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MobEffectPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MobEquipmentPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ModalFormRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ModalFormResponsePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MotionPredictionHintsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MoveActorAbsolutePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MoveActorDeltaPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MovePlayerPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::MultiplayerSettingsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::NetworkChunkPublisherUpdatePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::NetworkSettingsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::NetworkStackLatencyPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::NpcDialoguePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::NpcRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::OnScreenTextureAnimationPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::OpenSignPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PacketViolationWarningPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PassengerJumpPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PassengerJumpPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PhotoTransferPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlaySoundPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayStatusPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerActionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerArmorDamagePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerAuthInputPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerEnchantOptionsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerFogPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerHotbarPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerInputPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerInputPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerListPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerSkinPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerStartItemCooldownPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PlayerToggleCrafterSlotRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PositionTrackingDBClientRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PositionTrackingDBServerBroadcastPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::PurchaseReceiptPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RefreshEntitlementsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RemoveActorPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RemoveObjectivePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RemoveVolumeEntityPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RequestAbilityPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RequestChunkRadiusPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RequestNetworkSettingsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RequestPermissionsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ResourcePackChunkDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ResourcePackChunkRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ResourcePackClientResponsePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ResourcePackDataInfoPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ResourcePackStackPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ResourcePacksInfoPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::RespawnPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ScriptMessagePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ServerPlayerPostMovePositionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ServerSettingsRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ServerSettingsResponsePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ServerStatsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ServerToClientHandshakePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetActorDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetActorLinkPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetActorMotionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetCommandsEnabledPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetDefaultGameTypePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetDifficultyPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetDisplayObjectivePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetHealthPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetHudPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetLastHurtByPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetLocalPlayerAsInitializedPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetPlayerGameTypePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetPlayerInventoryOptionsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetScorePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetScoreboardIdentityPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetSpawnPositionPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetTimePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SetTitlePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SettingsCommandPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ShowCreditsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ShowProfilePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ShowStoreOfferPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SimpleEventPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SimulationTypePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SpawnExperienceOrbPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SpawnParticleEffectPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::StartGamePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::StopSoundPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::StructureBlockUpdatePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::StructureDataRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::StructureDataResponsePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SubChunkPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SubChunkRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SubClientLoginPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::SyncActorPropertyPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::TakeItemActorPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::TextPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::TickSyncPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickSyncPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::TickingAreaLoadStatusPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::ToastRequestPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::TransferPlayerPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::TrimDataPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UnlockedRecipesPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateAbilitiesPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateAdventureSettingsPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateAttributesPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateBlockPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateBlockSyncedPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateClientInputLocksPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateEquipPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdatePlayerGameTypePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateSoftEnumPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateSubChunkBlocksPacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::UpdateTradePacket(pk) => {
                    match <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                V662::Unknown(pk) => stream.write_all(pk.buf.as_ref()).map_err(|e| {
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
                <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ActorEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ActorPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ActorPickRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AddActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AddBehaviourTreePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddBehaviourTreePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AddItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddItemActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AddPaintingPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPaintingPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AddPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddPlayerPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AddVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AddVolumeEntityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AgentActionEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentActionEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AgentAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AgentAnimationPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AnimateEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimateEntityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AnimatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnimatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AnvilDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AnvilDamagePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AutomationClientConnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AutomationClientConnectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AvailableActorIdentifiersPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableActorIdentifiersPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::AvailableCommandsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(AvailableCommandsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::BiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BiomeDefinitionListPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::BlockActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockActorDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::BlockEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::BlockPickRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BlockPickRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::BookEditPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BookEditPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::BossEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(BossEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CameraInstructionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraInstructionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CameraPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CameraPresetsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraPresetsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CameraShakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CameraShakePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ChangeDimensionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeDimensionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ChangeMobPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChangeMobPropertyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ChunkRadiusUpdatedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ChunkRadiusUpdatedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ClientBoundDebugRendererPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundDebugRendererPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ClientBoundMapItemDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientBoundMapItemDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ClientCacheBlobStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheBlobStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ClientCacheMissResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheMissResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ClientCacheStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientCacheStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ClientToServerHandshakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ClientToServerHandshakePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CodeBuilderPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CodeBuilderSourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CodeBuilderSourcePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CommandBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandBlockUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CommandOutputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandOutputPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CommandRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CommandRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CompletedUsingItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CompletedUsingItemPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CompressedBiomeDefinitionListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    CompressedBiomeDefinitionListPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ContainerClosePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerClosePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ContainerOpenPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerOpenPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ContainerSetDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ContainerSetDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CorrectPlayerMovePredictionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CorrectPlayerMovePredictionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CraftingDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CraftingDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CreatePhotoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreatePhotoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::CreativeContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(CreativeContentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::DeathInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DeathInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::DebugInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DebugInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::DimensionDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DimensionDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::DisconnectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(DisconnectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::EditorNetworkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EditorNetworkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::EduUriResourcePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EduUriResourcePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::EducationSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EducationSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::EmoteListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmoteListPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::EmotePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(EmotePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::FeatureRegistryPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FeatureRegistryPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::FilterTextPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(FilterTextPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::GameRulesChangedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameRulesChangedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::GameTestRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::GameTestResultsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GameTestResultsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::GuiDataPickItemPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(GuiDataPickItemPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::HurtArmorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(HurtArmorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::InteractPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InteractPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::InventoryContentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryContentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::InventorySlotPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventorySlotPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::InventoryTransactionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(InventoryTransactionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ItemComponentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemComponentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ItemStackRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ItemStackResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ItemStackResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LabTablePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LabTablePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LecternUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LecternUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LegacyTelemetryEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LegacyTelemetryEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LessonProgressPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LessonProgressPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LevelChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelChunkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LevelEventGenericPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventGenericPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LevelEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LevelSoundEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LevelSoundEventV1Packet(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV1Packet),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LevelSoundEventV2Packet(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LevelSoundEventV2Packet),
                                packet_id: <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::LoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(LoginPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MapCreateLockedCopyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapCreateLockedCopyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MapInfoRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MapInfoRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MobArmorEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobArmorEquipmentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MobEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEffectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MobEquipmentPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MobEquipmentPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ModalFormRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ModalFormResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ModalFormResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MotionPredictionHintsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MotionPredictionHintsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MoveActorAbsolutePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorAbsolutePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MoveActorDeltaPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MoveActorDeltaPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MovePlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MovePlayerPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::MultiplayerSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(MultiplayerSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::NetworkChunkPublisherUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkChunkPublisherUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::NetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::NetworkStackLatencyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NetworkStackLatencyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::NpcDialoguePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcDialoguePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::NpcRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(NpcRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::OnScreenTextureAnimationPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OnScreenTextureAnimationPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::OpenSignPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(OpenSignPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PacketViolationWarningPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PacketViolationWarningPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PassengerJumpPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PassengerJumpPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PhotoTransferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PhotoTransferPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlaySoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlaySoundPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerActionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerActionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerArmorDamagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerArmorDamagePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerAuthInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerAuthInputPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerEnchantOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerEnchantOptionsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerFogPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerFogPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerHotbarPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerHotbarPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerInputPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerInputPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerListPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerListPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerSkinPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerSkinPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PlayerStartItemCooldownPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PlayerStartItemCooldownPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V662::PlayerToggleCrafterSlotRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PlayerToggleCrafterSlotRequestPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V662::PositionTrackingDBClientRequestPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBClientRequestPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => {
                            V662::PositionTrackingDBServerBroadcastPacket(Box::new(pk))
                        }
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(
                                    PositionTrackingDBServerBroadcastPacket
                                ),
                                packet_id: <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::PurchaseReceiptPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(PurchaseReceiptPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RefreshEntitlementsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RefreshEntitlementsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RemoveActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RemoveObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveObjectivePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RemoveVolumeEntityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RemoveVolumeEntityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RequestAbilityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestAbilityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RequestChunkRadiusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestChunkRadiusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RequestNetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RequestPermissionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestPermissionsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ResourcePackChunkDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ResourcePackChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackChunkRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ResourcePackClientResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackClientResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ResourcePackDataInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackDataInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ResourcePackStackPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePackStackPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ResourcePacksInfoPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ResourcePacksInfoPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::RespawnPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RespawnPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ScriptMessagePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ScriptMessagePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ServerPlayerPostMovePositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerPlayerPostMovePositionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ServerSettingsRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ServerSettingsResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerSettingsResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ServerStatsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerStatsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ServerToClientHandshakePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ServerToClientHandshakePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetActorDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetActorLinkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorLinkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetActorMotionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetActorMotionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetCommandsEnabledPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetCommandsEnabledPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetDefaultGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDefaultGameTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetDifficultyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDifficultyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetDisplayObjectivePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetDisplayObjectivePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetHealthPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHealthPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetHudPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetHudPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetLastHurtByPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLastHurtByPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetLocalPlayerAsInitializedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetLocalPlayerAsInitializedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetPlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerGameTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetPlayerInventoryOptionsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetPlayerInventoryOptionsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetScorePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScorePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetScoreboardIdentityPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetScoreboardIdentityPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetSpawnPositionPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetSpawnPositionPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetTimePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTimePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SetTitlePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SetTitlePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SettingsCommandPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SettingsCommandPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ShowCreditsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowCreditsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ShowProfilePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowProfilePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ShowStoreOfferPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ShowStoreOfferPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SimpleEventPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimpleEventPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SimulationTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SimulationTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SpawnExperienceOrbPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnExperienceOrbPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SpawnParticleEffectPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SpawnParticleEffectPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::StartGamePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StartGamePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::StopSoundPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StopSoundPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::StructureBlockUpdatePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureBlockUpdatePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::StructureDataRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::StructureDataResponsePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(StructureDataResponsePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SubChunkPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SubChunkRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubChunkRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SubClientLoginPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SubClientLoginPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::SyncActorPropertyPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(SyncActorPropertyPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::TakeItemActorPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TakeItemActorPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::TextPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TextPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::TickSyncPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickSyncPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::TickingAreaLoadStatusPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TickingAreaLoadStatusPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::ToastRequestPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(ToastRequestPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::TransferPlayerPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TransferPlayerPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::TrimDataPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(TrimDataPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UnlockedRecipesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UnlockedRecipesPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateAbilitiesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAbilitiesPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateAdventureSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAdventureSettingsPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateAttributesPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateAttributesPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateBlockPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateBlockSyncedPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateBlockSyncedPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateClientInputLocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateClientInputLocksPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateEquipPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateEquipPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdatePlayerGameTypePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdatePlayerGameTypePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateSoftEnumPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSoftEnumPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateSubChunkBlocksPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateSubChunkBlocksPacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    }
                }
                <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID => {
                    match <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => V662::UpdateTradePacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(UpdateTradePacket),
                                packet_id: <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID,
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
                    V662::Unknown(
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
                    V662::ActorEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ActorPickRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AddActorPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AddBehaviourTreePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AddItemActorPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AddPaintingPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AddPlayerPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AddVolumeEntityPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AgentActionEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AgentAnimationPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AnimateEntityPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AnimatePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AnvilDamagePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AutomationClientConnectPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AvailableActorIdentifiersPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::AvailableCommandsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::BiomeDefinitionListPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::BlockActorDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::BlockEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::BlockPickRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::BookEditPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::BossEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CameraInstructionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CameraPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CameraPresetsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CameraShakePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ChangeDimensionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ChangeMobPropertyPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ChunkRadiusUpdatedPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ClientBoundDebugRendererPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ClientBoundMapItemDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ClientCacheBlobStatusPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ClientCacheMissResponsePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ClientCacheStatusPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ClientToServerHandshakePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CodeBuilderPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CodeBuilderSourcePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CommandBlockUpdatePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CommandOutputPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CommandRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CompletedUsingItemPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CompressedBiomeDefinitionListPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ContainerClosePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ContainerOpenPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ContainerSetDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CorrectPlayerMovePredictionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CraftingDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CreatePhotoPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::CreativeContentPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::DeathInfoPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::DebugInfoPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::DimensionDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::DisconnectPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::EditorNetworkPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::EduUriResourcePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::EducationSettingsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::EmoteListPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::EmotePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::FeatureRegistryPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::FilterTextPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::GameRulesChangedPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::GameTestRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::GameTestResultsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::GuiDataPickItemPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::HurtArmorPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::InteractPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::InventoryContentPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::InventorySlotPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::InventoryTransactionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ItemComponentPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ItemStackRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ItemStackResponsePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LabTablePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LecternUpdatePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LegacyTelemetryEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LessonProgressPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LevelChunkPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LevelEventGenericPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LevelEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LevelSoundEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LevelSoundEventV1Packet(pk) => {
                        <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LevelSoundEventV2Packet(pk) => {
                        <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::LoginPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MapCreateLockedCopyPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MapInfoRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MobArmorEquipmentPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MobEffectPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MobEquipmentPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ModalFormRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ModalFormResponsePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MotionPredictionHintsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MoveActorAbsolutePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MoveActorDeltaPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MovePlayerPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::MultiplayerSettingsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::NetworkChunkPublisherUpdatePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::NetworkSettingsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::NetworkStackLatencyPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::NpcDialoguePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::NpcRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::OnScreenTextureAnimationPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::OpenSignPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PacketViolationWarningPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PassengerJumpPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PhotoTransferPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlaySoundPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayStatusPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerActionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerArmorDamagePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerAuthInputPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerEnchantOptionsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerFogPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerHotbarPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerInputPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerListPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerSkinPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerStartItemCooldownPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PlayerToggleCrafterSlotRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PositionTrackingDBClientRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PositionTrackingDBServerBroadcastPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::PurchaseReceiptPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RefreshEntitlementsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RemoveActorPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RemoveObjectivePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RemoveVolumeEntityPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RequestAbilityPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RequestChunkRadiusPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RequestNetworkSettingsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RequestPermissionsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ResourcePackChunkDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ResourcePackChunkRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ResourcePackClientResponsePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ResourcePackDataInfoPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ResourcePackStackPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ResourcePacksInfoPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::RespawnPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ScriptMessagePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ServerPlayerPostMovePositionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ServerSettingsRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ServerSettingsResponsePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ServerStatsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ServerToClientHandshakePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetActorDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetActorLinkPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetActorMotionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetCommandsEnabledPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetDefaultGameTypePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetDifficultyPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetDisplayObjectivePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetHealthPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetHudPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetLastHurtByPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetLocalPlayerAsInitializedPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetPlayerGameTypePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetPlayerInventoryOptionsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetScorePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetScoreboardIdentityPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetSpawnPositionPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetTimePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SetTitlePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SettingsCommandPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ShowCreditsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ShowProfilePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ShowStoreOfferPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SimpleEventPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SimulationTypePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SpawnExperienceOrbPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SpawnParticleEffectPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::StartGamePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::StopSoundPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::StructureBlockUpdatePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::StructureDataRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::StructureDataResponsePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SubChunkPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SubChunkRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SubClientLoginPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::SyncActorPropertyPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::TakeItemActorPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::TextPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::TickSyncPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::TickingAreaLoadStatusPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::ToastRequestPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::TransferPlayerPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::TrimDataPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UnlockedRecipesPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateAbilitiesPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateAdventureSettingsPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateAttributesPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateBlockPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateBlockSyncedPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateClientInputLocksPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateEquipPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdatePlayerGameTypePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateSoftEnumPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateSubChunkBlocksPacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::UpdateTradePacket(pk) => {
                        <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    V662::Unknown(pk) => pk.buf.len(),
                }
        }
        #[inline]
        fn id(&self) -> u16 {
            match self {
                V662::ActorEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ActorEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ActorPickRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ActorPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AddActorPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AddActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AddBehaviourTreePacket(_) => {
                    <<V662 as ProtoVersionPackets>::AddBehaviourTreePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AddItemActorPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AddItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AddPaintingPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AddPaintingPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AddPlayerPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AddPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AddVolumeEntityPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AddVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AgentActionEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AgentActionEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AgentAnimationPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AgentAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AnimateEntityPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AnimateEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AnimatePacket(_) => {
                    <<V662 as ProtoVersionPackets>::AnimatePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AnvilDamagePacket(_) => {
                    <<V662 as ProtoVersionPackets>::AnvilDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AutomationClientConnectPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AutomationClientConnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AvailableActorIdentifiersPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AvailableActorIdentifiersPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::AvailableCommandsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::AvailableCommandsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::BiomeDefinitionListPacket(_) => {
                    <<V662 as ProtoVersionPackets>::BiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::BlockActorDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::BlockActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::BlockEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::BlockEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::BlockPickRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::BlockPickRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::BookEditPacket(_) => {
                    <<V662 as ProtoVersionPackets>::BookEditPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::BossEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::BossEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CameraInstructionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CameraInstructionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CameraPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CameraPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CameraPresetsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CameraPresetsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CameraShakePacket(_) => {
                    <<V662 as ProtoVersionPackets>::CameraShakePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ChangeDimensionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ChangeDimensionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ChangeMobPropertyPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ChangeMobPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ChunkRadiusUpdatedPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ChunkRadiusUpdatedPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ClientBoundDebugRendererPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ClientBoundDebugRendererPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ClientBoundMapItemDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ClientBoundMapItemDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ClientCacheBlobStatusPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ClientCacheBlobStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ClientCacheMissResponsePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ClientCacheMissResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ClientCacheStatusPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ClientCacheStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ClientToServerHandshakePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ClientToServerHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CodeBuilderPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CodeBuilderPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CodeBuilderSourcePacket(_) => {
                    <<V662 as ProtoVersionPackets>::CodeBuilderSourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CommandBlockUpdatePacket(_) => {
                    <<V662 as ProtoVersionPackets>::CommandBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CommandOutputPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CommandOutputPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CommandRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CommandRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CompletedUsingItemPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CompletedUsingItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CompressedBiomeDefinitionListPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CompressedBiomeDefinitionListPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ContainerClosePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ContainerClosePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ContainerOpenPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ContainerOpenPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ContainerSetDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ContainerSetDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CorrectPlayerMovePredictionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CorrectPlayerMovePredictionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CraftingDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CraftingDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CreatePhotoPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CreatePhotoPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::CreativeContentPacket(_) => {
                    <<V662 as ProtoVersionPackets>::CreativeContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::DeathInfoPacket(_) => {
                    <<V662 as ProtoVersionPackets>::DeathInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::DebugInfoPacket(_) => {
                    <<V662 as ProtoVersionPackets>::DebugInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::DimensionDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::DimensionDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::DisconnectPacket(_) => {
                    <<V662 as ProtoVersionPackets>::DisconnectPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::EditorNetworkPacket(_) => {
                    <<V662 as ProtoVersionPackets>::EditorNetworkPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::EduUriResourcePacket(_) => {
                    <<V662 as ProtoVersionPackets>::EduUriResourcePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::EducationSettingsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::EducationSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::EmoteListPacket(_) => {
                    <<V662 as ProtoVersionPackets>::EmoteListPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::EmotePacket(_) => {
                    <<V662 as ProtoVersionPackets>::EmotePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::FeatureRegistryPacket(_) => {
                    <<V662 as ProtoVersionPackets>::FeatureRegistryPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::FilterTextPacket(_) => {
                    <<V662 as ProtoVersionPackets>::FilterTextPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::GameRulesChangedPacket(_) => {
                    <<V662 as ProtoVersionPackets>::GameRulesChangedPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::GameTestRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::GameTestRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::GameTestResultsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::GameTestResultsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::GuiDataPickItemPacket(_) => {
                    <<V662 as ProtoVersionPackets>::GuiDataPickItemPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::HurtArmorPacket(_) => {
                    <<V662 as ProtoVersionPackets>::HurtArmorPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::InteractPacket(_) => {
                    <<V662 as ProtoVersionPackets>::InteractPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::InventoryContentPacket(_) => {
                    <<V662 as ProtoVersionPackets>::InventoryContentPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::InventorySlotPacket(_) => {
                    <<V662 as ProtoVersionPackets>::InventorySlotPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::InventoryTransactionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::InventoryTransactionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ItemComponentPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ItemComponentPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ItemStackRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ItemStackRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ItemStackResponsePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ItemStackResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LabTablePacket(_) => {
                    <<V662 as ProtoVersionPackets>::LabTablePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LecternUpdatePacket(_) => {
                    <<V662 as ProtoVersionPackets>::LecternUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LegacyTelemetryEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LegacyTelemetryEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LessonProgressPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LessonProgressPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LevelChunkPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LevelChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LevelEventGenericPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LevelEventGenericPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LevelEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LevelEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LevelSoundEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LevelSoundEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::LevelSoundEventV1Packet(_) => {
                    <<V662 as ProtoVersionPackets>::LevelSoundEventV1Packet as bedrock_protocol_core::Packet>::ID
                }
                V662::LevelSoundEventV2Packet(_) => {
                    <<V662 as ProtoVersionPackets>::LevelSoundEventV2Packet as bedrock_protocol_core::Packet>::ID
                }
                V662::LoginPacket(_) => {
                    <<V662 as ProtoVersionPackets>::LoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MapCreateLockedCopyPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MapCreateLockedCopyPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MapInfoRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MapInfoRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MobArmorEquipmentPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MobArmorEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MobEffectPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MobEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MobEquipmentPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MobEquipmentPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ModalFormRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ModalFormRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ModalFormResponsePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ModalFormResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MotionPredictionHintsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MotionPredictionHintsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MoveActorAbsolutePacket(_) => {
                    <<V662 as ProtoVersionPackets>::MoveActorAbsolutePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MoveActorDeltaPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MoveActorDeltaPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MovePlayerPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MovePlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::MultiplayerSettingsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::MultiplayerSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::NetworkChunkPublisherUpdatePacket(_) => {
                    <<V662 as ProtoVersionPackets>::NetworkChunkPublisherUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::NetworkSettingsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::NetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::NetworkStackLatencyPacket(_) => {
                    <<V662 as ProtoVersionPackets>::NetworkStackLatencyPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::NpcDialoguePacket(_) => {
                    <<V662 as ProtoVersionPackets>::NpcDialoguePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::NpcRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::NpcRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::OnScreenTextureAnimationPacket(_) => {
                    <<V662 as ProtoVersionPackets>::OnScreenTextureAnimationPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::OpenSignPacket(_) => {
                    <<V662 as ProtoVersionPackets>::OpenSignPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PacketViolationWarningPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PacketViolationWarningPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PassengerJumpPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PassengerJumpPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PhotoTransferPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PhotoTransferPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlaySoundPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlaySoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayStatusPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerActionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerActionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerArmorDamagePacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerArmorDamagePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerAuthInputPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerAuthInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerEnchantOptionsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerEnchantOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerFogPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerFogPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerHotbarPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerHotbarPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerInputPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerInputPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerListPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerListPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerSkinPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerSkinPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerStartItemCooldownPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerStartItemCooldownPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PlayerToggleCrafterSlotRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PlayerToggleCrafterSlotRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PositionTrackingDBClientRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PositionTrackingDBClientRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PositionTrackingDBServerBroadcastPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PositionTrackingDBServerBroadcastPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::PurchaseReceiptPacket(_) => {
                    <<V662 as ProtoVersionPackets>::PurchaseReceiptPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RefreshEntitlementsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RefreshEntitlementsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RemoveActorPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RemoveActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RemoveObjectivePacket(_) => {
                    <<V662 as ProtoVersionPackets>::RemoveObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RemoveVolumeEntityPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RemoveVolumeEntityPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RequestAbilityPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RequestAbilityPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RequestChunkRadiusPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RequestChunkRadiusPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RequestNetworkSettingsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RequestPermissionsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RequestPermissionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ResourcePackChunkDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ResourcePackChunkDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ResourcePackChunkRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ResourcePackChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ResourcePackClientResponsePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ResourcePackClientResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ResourcePackDataInfoPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ResourcePackDataInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ResourcePackStackPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ResourcePackStackPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ResourcePacksInfoPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ResourcePacksInfoPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::RespawnPacket(_) => {
                    <<V662 as ProtoVersionPackets>::RespawnPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ScriptMessagePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ScriptMessagePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ServerPlayerPostMovePositionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ServerPlayerPostMovePositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ServerSettingsRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ServerSettingsRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ServerSettingsResponsePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ServerSettingsResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ServerStatsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ServerStatsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ServerToClientHandshakePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ServerToClientHandshakePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetActorDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetActorDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetActorLinkPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetActorLinkPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetActorMotionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetActorMotionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetCommandsEnabledPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetCommandsEnabledPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetDefaultGameTypePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetDefaultGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetDifficultyPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetDifficultyPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetDisplayObjectivePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetDisplayObjectivePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetHealthPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetHealthPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetHudPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetHudPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetLastHurtByPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetLastHurtByPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetLocalPlayerAsInitializedPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetLocalPlayerAsInitializedPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetPlayerGameTypePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetPlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetPlayerInventoryOptionsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetPlayerInventoryOptionsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetScorePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetScorePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetScoreboardIdentityPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetScoreboardIdentityPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetSpawnPositionPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetSpawnPositionPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetTimePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetTimePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SetTitlePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SetTitlePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SettingsCommandPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SettingsCommandPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ShowCreditsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ShowCreditsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ShowProfilePacket(_) => {
                    <<V662 as ProtoVersionPackets>::ShowProfilePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ShowStoreOfferPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ShowStoreOfferPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SimpleEventPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SimpleEventPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SimulationTypePacket(_) => {
                    <<V662 as ProtoVersionPackets>::SimulationTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SpawnExperienceOrbPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SpawnExperienceOrbPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SpawnParticleEffectPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SpawnParticleEffectPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::StartGamePacket(_) => {
                    <<V662 as ProtoVersionPackets>::StartGamePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::StopSoundPacket(_) => {
                    <<V662 as ProtoVersionPackets>::StopSoundPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::StructureBlockUpdatePacket(_) => {
                    <<V662 as ProtoVersionPackets>::StructureBlockUpdatePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::StructureDataRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::StructureDataRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::StructureDataResponsePacket(_) => {
                    <<V662 as ProtoVersionPackets>::StructureDataResponsePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SubChunkPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SubChunkPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SubChunkRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SubChunkRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SubClientLoginPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SubClientLoginPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::SyncActorPropertyPacket(_) => {
                    <<V662 as ProtoVersionPackets>::SyncActorPropertyPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::TakeItemActorPacket(_) => {
                    <<V662 as ProtoVersionPackets>::TakeItemActorPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::TextPacket(_) => {
                    <<V662 as ProtoVersionPackets>::TextPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::TickSyncPacket(_) => {
                    <<V662 as ProtoVersionPackets>::TickSyncPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::TickingAreaLoadStatusPacket(_) => {
                    <<V662 as ProtoVersionPackets>::TickingAreaLoadStatusPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::ToastRequestPacket(_) => {
                    <<V662 as ProtoVersionPackets>::ToastRequestPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::TransferPlayerPacket(_) => {
                    <<V662 as ProtoVersionPackets>::TransferPlayerPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::TrimDataPacket(_) => {
                    <<V662 as ProtoVersionPackets>::TrimDataPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UnlockedRecipesPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UnlockedRecipesPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateAbilitiesPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateAbilitiesPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateAdventureSettingsPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateAdventureSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateAttributesPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateAttributesPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateBlockPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateBlockPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateBlockSyncedPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateBlockSyncedPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateClientInputLocksPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateClientInputLocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateEquipPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateEquipPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdatePlayerGameTypePacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdatePlayerGameTypePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateSoftEnumPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateSoftEnumPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateSubChunkBlocksPacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateSubChunkBlocksPacket as bedrock_protocol_core::Packet>::ID
                }
                V662::UpdateTradePacket(_) => {
                    <<V662 as ProtoVersionPackets>::UpdateTradePacket as bedrock_protocol_core::Packet>::ID
                }
                V662::Unknown(pk) => pk.id,
            }
        }
    }
    impl ProtoVersionPackets for V662 {
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
        type AwardAchievementPacket = ();
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
            crate::version::v662::packets::ClientBoundDebugRendererPacket;
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
        type CodeBuilderSourcePacket = crate::version::v662::packets::CodeBuilderSourcePacket<Self>;
        type CommandBlockUpdatePacket =
            crate::version::v662::packets::CommandBlockUpdatePacket<Self>;
        type CommandOutputPacket = crate::version::v662::packets::CommandOutputPacket<Self>;
        type CommandRequestPacket = crate::version::v662::packets::CommandRequestPacket<Self>;
        type CompletedUsingItemPacket =
            crate::version::v662::packets::CompletedUsingItemPacket<Self>;
        type CompressedBiomeDefinitionListPacket =
            crate::version::v662::packets::CompressedBiomeDefinitionListPacket;
        type ContainerClosePacket = crate::version::v662::packets::ContainerClosePacket<Self>;
        type ContainerOpenPacket = crate::version::v662::packets::ContainerOpenPacket<Self>;
        type ContainerRegistryCleanupPacket = ();
        type ContainerSetDataPacket = crate::version::v662::packets::ContainerSetDataPacket<Self>;
        type CorrectPlayerMovePredictionPacket =
            crate::version::v662::packets::CorrectPlayerMovePredictionPacket<Self>;
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
        type FilterTextPacket = crate::version::v662::packets::FilterTextPacket;
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
            crate::version::v662::packets::LegacyTelemetryEventPacket<Self>;
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
        type ResourcePackStackPacket = crate::version::v662::packets::ResourcePackStackPacket<Self>;
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
        type TextPacket = crate::version::v662::packets::TextPacket<Self>;
        type TickSyncPacket = crate::version::v662::packets::TickSyncPacket;
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
            crate::version::v662::packets::UpdatePlayerGameTypePacket<Self>;
        type UpdateSoftEnumPacket = crate::version::v662::packets::UpdateSoftEnumPacket<Self>;
        type UpdateSubChunkBlocksPacket =
            crate::version::v662::packets::UpdateSubChunkBlocksPacket<Self>;
        type UpdateTradePacket = crate::version::v662::packets::UpdateTradePacket<Self>;
        type VoxelShapesPacket = ();
    }
    impl ProtoVersionTypes for V662 {
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
        type LevelSettings = crate::version::v662::types::LevelSettings<Self>;
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
        type RecipeUnlockingRequirement = ();
        type ScoreboardId = crate::version::v662::types::ScoreboardId;
        type SerializedAbilitiesData = crate::version::v662::types::SerializedAbilitiesData<Self>;
        type SerializedSkin = crate::version::v662::types::SerializedSkin<Self>;
        type ShapedRecipe = crate::version::v662::types::ShapedRecipe<Self>;
        type ShapelessRecipe = crate::version::v662::types::ShapelessRecipe<Self>;
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
    impl ProtoVersionEnums for V662 {
        type AbilitiesIndex = crate::version::v662::enums::AbilitiesIndex;
        type ActorBlockSyncMessageID = crate::version::v662::enums::ActorBlockSyncMessageID;
        type ActorDamageCause = crate::version::v662::enums::ActorDamageCause;
        type ActorDataIDs = crate::version::v662::enums::ActorDataIDs;
        type ActorEvent = crate::version::v662::enums::ActorEvent;
        type ActorFlags = crate::version::v662::enums::ActorFlags;
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
        type CodeBuilderCodeStatus = ();
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
        type LevelEvent = crate::version::v662::enums::LevelEvent;
        type LevelSoundEventType = crate::version::v662::enums::LevelSoundEventType;
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
        type ParticleType = crate::version::v662::enums::ParticleType;
        type PhotoType = crate::version::v662::enums::PhotoType;
        type PlayStatus = crate::version::v662::enums::PlayStatus;
        type PlayerPermissionLevel = crate::version::v662::enums::PlayerPermissionLevel;
        type PlayerPositionMode = crate::version::v662::enums::PlayerPositionMode;
        type PlayerRespawnState = crate::version::v662::enums::PlayerRespawnState;
        type PredictionType = crate::version::v662::enums::PredictionType;
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
    impl ProtoVersion for V662 {
        const PROTOCOL_VERSION: u32 = 662u32;
        const PROTOCOL_BRANCH: &str = "r/20_u7";
        const GAME_VERSION: &str = "1.20.70";
        const RAKNET_VERSION: u8 = 11u8;
    }
    #[cfg(feature = "packet-dyn")]
    impl AsRef<dyn bedrock_protocol_core::PacketDyn> for V662 {
        fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
            match self {
                V662::ActorEventPacket(pk) => pk.as_ref(),
                V662::ActorPickRequestPacket(pk) => pk.as_ref(),
                V662::AddActorPacket(pk) => pk.as_ref(),
                V662::AddBehaviourTreePacket(pk) => pk.as_ref(),
                V662::AddItemActorPacket(pk) => pk.as_ref(),
                V662::AddPaintingPacket(pk) => pk.as_ref(),
                V662::AddPlayerPacket(pk) => pk.as_ref(),
                V662::AddVolumeEntityPacket(pk) => pk.as_ref(),
                V662::AgentActionEventPacket(pk) => pk.as_ref(),
                V662::AgentAnimationPacket(pk) => pk.as_ref(),
                V662::AnimateEntityPacket(pk) => pk.as_ref(),
                V662::AnimatePacket(pk) => pk.as_ref(),
                V662::AnvilDamagePacket(pk) => pk.as_ref(),
                V662::AutomationClientConnectPacket(pk) => pk.as_ref(),
                V662::AvailableActorIdentifiersPacket(pk) => pk.as_ref(),
                V662::AvailableCommandsPacket(pk) => pk.as_ref(),
                V662::BiomeDefinitionListPacket(pk) => pk.as_ref(),
                V662::BlockActorDataPacket(pk) => pk.as_ref(),
                V662::BlockEventPacket(pk) => pk.as_ref(),
                V662::BlockPickRequestPacket(pk) => pk.as_ref(),
                V662::BookEditPacket(pk) => pk.as_ref(),
                V662::BossEventPacket(pk) => pk.as_ref(),
                V662::CameraInstructionPacket(pk) => pk.as_ref(),
                V662::CameraPacket(pk) => pk.as_ref(),
                V662::CameraPresetsPacket(pk) => pk.as_ref(),
                V662::CameraShakePacket(pk) => pk.as_ref(),
                V662::ChangeDimensionPacket(pk) => pk.as_ref(),
                V662::ChangeMobPropertyPacket(pk) => pk.as_ref(),
                V662::ChunkRadiusUpdatedPacket(pk) => pk.as_ref(),
                V662::ClientBoundDebugRendererPacket(pk) => pk.as_ref(),
                V662::ClientBoundMapItemDataPacket(pk) => pk.as_ref(),
                V662::ClientCacheBlobStatusPacket(pk) => pk.as_ref(),
                V662::ClientCacheMissResponsePacket(pk) => pk.as_ref(),
                V662::ClientCacheStatusPacket(pk) => pk.as_ref(),
                V662::ClientToServerHandshakePacket(pk) => pk.as_ref(),
                V662::CodeBuilderPacket(pk) => pk.as_ref(),
                V662::CodeBuilderSourcePacket(pk) => pk.as_ref(),
                V662::CommandBlockUpdatePacket(pk) => pk.as_ref(),
                V662::CommandOutputPacket(pk) => pk.as_ref(),
                V662::CommandRequestPacket(pk) => pk.as_ref(),
                V662::CompletedUsingItemPacket(pk) => pk.as_ref(),
                V662::CompressedBiomeDefinitionListPacket(pk) => pk.as_ref(),
                V662::ContainerClosePacket(pk) => pk.as_ref(),
                V662::ContainerOpenPacket(pk) => pk.as_ref(),
                V662::ContainerSetDataPacket(pk) => pk.as_ref(),
                V662::CorrectPlayerMovePredictionPacket(pk) => pk.as_ref(),
                V662::CraftingDataPacket(pk) => pk.as_ref(),
                V662::CreatePhotoPacket(pk) => pk.as_ref(),
                V662::CreativeContentPacket(pk) => pk.as_ref(),
                V662::DeathInfoPacket(pk) => pk.as_ref(),
                V662::DebugInfoPacket(pk) => pk.as_ref(),
                V662::DimensionDataPacket(pk) => pk.as_ref(),
                V662::DisconnectPacket(pk) => pk.as_ref(),
                V662::EditorNetworkPacket(pk) => pk.as_ref(),
                V662::EduUriResourcePacket(pk) => pk.as_ref(),
                V662::EducationSettingsPacket(pk) => pk.as_ref(),
                V662::EmoteListPacket(pk) => pk.as_ref(),
                V662::EmotePacket(pk) => pk.as_ref(),
                V662::FeatureRegistryPacket(pk) => pk.as_ref(),
                V662::FilterTextPacket(pk) => pk.as_ref(),
                V662::GameRulesChangedPacket(pk) => pk.as_ref(),
                V662::GameTestRequestPacket(pk) => pk.as_ref(),
                V662::GameTestResultsPacket(pk) => pk.as_ref(),
                V662::GuiDataPickItemPacket(pk) => pk.as_ref(),
                V662::HurtArmorPacket(pk) => pk.as_ref(),
                V662::InteractPacket(pk) => pk.as_ref(),
                V662::InventoryContentPacket(pk) => pk.as_ref(),
                V662::InventorySlotPacket(pk) => pk.as_ref(),
                V662::InventoryTransactionPacket(pk) => pk.as_ref(),
                V662::ItemComponentPacket(pk) => pk.as_ref(),
                V662::ItemStackRequestPacket(pk) => pk.as_ref(),
                V662::ItemStackResponsePacket(pk) => pk.as_ref(),
                V662::LabTablePacket(pk) => pk.as_ref(),
                V662::LecternUpdatePacket(pk) => pk.as_ref(),
                V662::LegacyTelemetryEventPacket(pk) => pk.as_ref(),
                V662::LessonProgressPacket(pk) => pk.as_ref(),
                V662::LevelChunkPacket(pk) => pk.as_ref(),
                V662::LevelEventGenericPacket(pk) => pk.as_ref(),
                V662::LevelEventPacket(pk) => pk.as_ref(),
                V662::LevelSoundEventPacket(pk) => pk.as_ref(),
                V662::LevelSoundEventV1Packet(pk) => pk.as_ref(),
                V662::LevelSoundEventV2Packet(pk) => pk.as_ref(),
                V662::LoginPacket(pk) => pk.as_ref(),
                V662::MapCreateLockedCopyPacket(pk) => pk.as_ref(),
                V662::MapInfoRequestPacket(pk) => pk.as_ref(),
                V662::MobArmorEquipmentPacket(pk) => pk.as_ref(),
                V662::MobEffectPacket(pk) => pk.as_ref(),
                V662::MobEquipmentPacket(pk) => pk.as_ref(),
                V662::ModalFormRequestPacket(pk) => pk.as_ref(),
                V662::ModalFormResponsePacket(pk) => pk.as_ref(),
                V662::MotionPredictionHintsPacket(pk) => pk.as_ref(),
                V662::MoveActorAbsolutePacket(pk) => pk.as_ref(),
                V662::MoveActorDeltaPacket(pk) => pk.as_ref(),
                V662::MovePlayerPacket(pk) => pk.as_ref(),
                V662::MultiplayerSettingsPacket(pk) => pk.as_ref(),
                V662::NetworkChunkPublisherUpdatePacket(pk) => pk.as_ref(),
                V662::NetworkSettingsPacket(pk) => pk.as_ref(),
                V662::NetworkStackLatencyPacket(pk) => pk.as_ref(),
                V662::NpcDialoguePacket(pk) => pk.as_ref(),
                V662::NpcRequestPacket(pk) => pk.as_ref(),
                V662::OnScreenTextureAnimationPacket(pk) => pk.as_ref(),
                V662::OpenSignPacket(pk) => pk.as_ref(),
                V662::PacketViolationWarningPacket(pk) => pk.as_ref(),
                V662::PassengerJumpPacket(pk) => pk.as_ref(),
                V662::PhotoTransferPacket(pk) => pk.as_ref(),
                V662::PlaySoundPacket(pk) => pk.as_ref(),
                V662::PlayStatusPacket(pk) => pk.as_ref(),
                V662::PlayerActionPacket(pk) => pk.as_ref(),
                V662::PlayerArmorDamagePacket(pk) => pk.as_ref(),
                V662::PlayerAuthInputPacket(pk) => pk.as_ref(),
                V662::PlayerEnchantOptionsPacket(pk) => pk.as_ref(),
                V662::PlayerFogPacket(pk) => pk.as_ref(),
                V662::PlayerHotbarPacket(pk) => pk.as_ref(),
                V662::PlayerInputPacket(pk) => pk.as_ref(),
                V662::PlayerListPacket(pk) => pk.as_ref(),
                V662::PlayerSkinPacket(pk) => pk.as_ref(),
                V662::PlayerStartItemCooldownPacket(pk) => pk.as_ref(),
                V662::PlayerToggleCrafterSlotRequestPacket(pk) => pk.as_ref(),
                V662::PositionTrackingDBClientRequestPacket(pk) => pk.as_ref(),
                V662::PositionTrackingDBServerBroadcastPacket(pk) => pk.as_ref(),
                V662::PurchaseReceiptPacket(pk) => pk.as_ref(),
                V662::RefreshEntitlementsPacket(pk) => pk.as_ref(),
                V662::RemoveActorPacket(pk) => pk.as_ref(),
                V662::RemoveObjectivePacket(pk) => pk.as_ref(),
                V662::RemoveVolumeEntityPacket(pk) => pk.as_ref(),
                V662::RequestAbilityPacket(pk) => pk.as_ref(),
                V662::RequestChunkRadiusPacket(pk) => pk.as_ref(),
                V662::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                V662::RequestPermissionsPacket(pk) => pk.as_ref(),
                V662::ResourcePackChunkDataPacket(pk) => pk.as_ref(),
                V662::ResourcePackChunkRequestPacket(pk) => pk.as_ref(),
                V662::ResourcePackClientResponsePacket(pk) => pk.as_ref(),
                V662::ResourcePackDataInfoPacket(pk) => pk.as_ref(),
                V662::ResourcePackStackPacket(pk) => pk.as_ref(),
                V662::ResourcePacksInfoPacket(pk) => pk.as_ref(),
                V662::RespawnPacket(pk) => pk.as_ref(),
                V662::ScriptMessagePacket(pk) => pk.as_ref(),
                V662::ServerPlayerPostMovePositionPacket(pk) => pk.as_ref(),
                V662::ServerSettingsRequestPacket(pk) => pk.as_ref(),
                V662::ServerSettingsResponsePacket(pk) => pk.as_ref(),
                V662::ServerStatsPacket(pk) => pk.as_ref(),
                V662::ServerToClientHandshakePacket(pk) => pk.as_ref(),
                V662::SetActorDataPacket(pk) => pk.as_ref(),
                V662::SetActorLinkPacket(pk) => pk.as_ref(),
                V662::SetActorMotionPacket(pk) => pk.as_ref(),
                V662::SetCommandsEnabledPacket(pk) => pk.as_ref(),
                V662::SetDefaultGameTypePacket(pk) => pk.as_ref(),
                V662::SetDifficultyPacket(pk) => pk.as_ref(),
                V662::SetDisplayObjectivePacket(pk) => pk.as_ref(),
                V662::SetHealthPacket(pk) => pk.as_ref(),
                V662::SetHudPacket(pk) => pk.as_ref(),
                V662::SetLastHurtByPacket(pk) => pk.as_ref(),
                V662::SetLocalPlayerAsInitializedPacket(pk) => pk.as_ref(),
                V662::SetPlayerGameTypePacket(pk) => pk.as_ref(),
                V662::SetPlayerInventoryOptionsPacket(pk) => pk.as_ref(),
                V662::SetScorePacket(pk) => pk.as_ref(),
                V662::SetScoreboardIdentityPacket(pk) => pk.as_ref(),
                V662::SetSpawnPositionPacket(pk) => pk.as_ref(),
                V662::SetTimePacket(pk) => pk.as_ref(),
                V662::SetTitlePacket(pk) => pk.as_ref(),
                V662::SettingsCommandPacket(pk) => pk.as_ref(),
                V662::ShowCreditsPacket(pk) => pk.as_ref(),
                V662::ShowProfilePacket(pk) => pk.as_ref(),
                V662::ShowStoreOfferPacket(pk) => pk.as_ref(),
                V662::SimpleEventPacket(pk) => pk.as_ref(),
                V662::SimulationTypePacket(pk) => pk.as_ref(),
                V662::SpawnExperienceOrbPacket(pk) => pk.as_ref(),
                V662::SpawnParticleEffectPacket(pk) => pk.as_ref(),
                V662::StartGamePacket(pk) => pk.as_ref(),
                V662::StopSoundPacket(pk) => pk.as_ref(),
                V662::StructureBlockUpdatePacket(pk) => pk.as_ref(),
                V662::StructureDataRequestPacket(pk) => pk.as_ref(),
                V662::StructureDataResponsePacket(pk) => pk.as_ref(),
                V662::SubChunkPacket(pk) => pk.as_ref(),
                V662::SubChunkRequestPacket(pk) => pk.as_ref(),
                V662::SubClientLoginPacket(pk) => pk.as_ref(),
                V662::SyncActorPropertyPacket(pk) => pk.as_ref(),
                V662::TakeItemActorPacket(pk) => pk.as_ref(),
                V662::TextPacket(pk) => pk.as_ref(),
                V662::TickSyncPacket(pk) => pk.as_ref(),
                V662::TickingAreaLoadStatusPacket(pk) => pk.as_ref(),
                V662::ToastRequestPacket(pk) => pk.as_ref(),
                V662::TransferPlayerPacket(pk) => pk.as_ref(),
                V662::TrimDataPacket(pk) => pk.as_ref(),
                V662::UnlockedRecipesPacket(pk) => pk.as_ref(),
                V662::UpdateAbilitiesPacket(pk) => pk.as_ref(),
                V662::UpdateAdventureSettingsPacket(pk) => pk.as_ref(),
                V662::UpdateAttributesPacket(pk) => pk.as_ref(),
                V662::UpdateBlockPacket(pk) => pk.as_ref(),
                V662::UpdateBlockSyncedPacket(pk) => pk.as_ref(),
                V662::UpdateClientInputLocksPacket(pk) => pk.as_ref(),
                V662::UpdateEquipPacket(pk) => pk.as_ref(),
                V662::UpdatePlayerGameTypePacket(pk) => pk.as_ref(),
                V662::UpdateSoftEnumPacket(pk) => pk.as_ref(),
                V662::UpdateSubChunkBlocksPacket(pk) => pk.as_ref(),
                V662::UpdateTradePacket(pk) => pk.as_ref(),
                V662::Unknown(pk) => pk.as_ref(),
            }
        }
    }
    #[cfg(feature = "packet-dyn")]
    impl From<V662> for Box<dyn bedrock_protocol_core::PacketDyn> {
        fn from(val: V662) -> Box<dyn bedrock_protocol_core::PacketDyn> {
            match val {
                V662::ActorEventPacket(pk) => pk,
                V662::ActorPickRequestPacket(pk) => pk,
                V662::AddActorPacket(pk) => pk,
                V662::AddBehaviourTreePacket(pk) => pk,
                V662::AddItemActorPacket(pk) => pk,
                V662::AddPaintingPacket(pk) => pk,
                V662::AddPlayerPacket(pk) => pk,
                V662::AddVolumeEntityPacket(pk) => pk,
                V662::AgentActionEventPacket(pk) => pk,
                V662::AgentAnimationPacket(pk) => pk,
                V662::AnimateEntityPacket(pk) => pk,
                V662::AnimatePacket(pk) => pk,
                V662::AnvilDamagePacket(pk) => pk,
                V662::AutomationClientConnectPacket(pk) => pk,
                V662::AvailableActorIdentifiersPacket(pk) => pk,
                V662::AvailableCommandsPacket(pk) => pk,
                V662::BiomeDefinitionListPacket(pk) => pk,
                V662::BlockActorDataPacket(pk) => pk,
                V662::BlockEventPacket(pk) => pk,
                V662::BlockPickRequestPacket(pk) => pk,
                V662::BookEditPacket(pk) => pk,
                V662::BossEventPacket(pk) => pk,
                V662::CameraInstructionPacket(pk) => pk,
                V662::CameraPacket(pk) => pk,
                V662::CameraPresetsPacket(pk) => pk,
                V662::CameraShakePacket(pk) => pk,
                V662::ChangeDimensionPacket(pk) => pk,
                V662::ChangeMobPropertyPacket(pk) => pk,
                V662::ChunkRadiusUpdatedPacket(pk) => pk,
                V662::ClientBoundDebugRendererPacket(pk) => pk,
                V662::ClientBoundMapItemDataPacket(pk) => pk,
                V662::ClientCacheBlobStatusPacket(pk) => pk,
                V662::ClientCacheMissResponsePacket(pk) => pk,
                V662::ClientCacheStatusPacket(pk) => pk,
                V662::ClientToServerHandshakePacket(pk) => pk,
                V662::CodeBuilderPacket(pk) => pk,
                V662::CodeBuilderSourcePacket(pk) => pk,
                V662::CommandBlockUpdatePacket(pk) => pk,
                V662::CommandOutputPacket(pk) => pk,
                V662::CommandRequestPacket(pk) => pk,
                V662::CompletedUsingItemPacket(pk) => pk,
                V662::CompressedBiomeDefinitionListPacket(pk) => pk,
                V662::ContainerClosePacket(pk) => pk,
                V662::ContainerOpenPacket(pk) => pk,
                V662::ContainerSetDataPacket(pk) => pk,
                V662::CorrectPlayerMovePredictionPacket(pk) => pk,
                V662::CraftingDataPacket(pk) => pk,
                V662::CreatePhotoPacket(pk) => pk,
                V662::CreativeContentPacket(pk) => pk,
                V662::DeathInfoPacket(pk) => pk,
                V662::DebugInfoPacket(pk) => pk,
                V662::DimensionDataPacket(pk) => pk,
                V662::DisconnectPacket(pk) => pk,
                V662::EditorNetworkPacket(pk) => pk,
                V662::EduUriResourcePacket(pk) => pk,
                V662::EducationSettingsPacket(pk) => pk,
                V662::EmoteListPacket(pk) => pk,
                V662::EmotePacket(pk) => pk,
                V662::FeatureRegistryPacket(pk) => pk,
                V662::FilterTextPacket(pk) => pk,
                V662::GameRulesChangedPacket(pk) => pk,
                V662::GameTestRequestPacket(pk) => pk,
                V662::GameTestResultsPacket(pk) => pk,
                V662::GuiDataPickItemPacket(pk) => pk,
                V662::HurtArmorPacket(pk) => pk,
                V662::InteractPacket(pk) => pk,
                V662::InventoryContentPacket(pk) => pk,
                V662::InventorySlotPacket(pk) => pk,
                V662::InventoryTransactionPacket(pk) => pk,
                V662::ItemComponentPacket(pk) => pk,
                V662::ItemStackRequestPacket(pk) => pk,
                V662::ItemStackResponsePacket(pk) => pk,
                V662::LabTablePacket(pk) => pk,
                V662::LecternUpdatePacket(pk) => pk,
                V662::LegacyTelemetryEventPacket(pk) => pk,
                V662::LessonProgressPacket(pk) => pk,
                V662::LevelChunkPacket(pk) => pk,
                V662::LevelEventGenericPacket(pk) => pk,
                V662::LevelEventPacket(pk) => pk,
                V662::LevelSoundEventPacket(pk) => pk,
                V662::LevelSoundEventV1Packet(pk) => pk,
                V662::LevelSoundEventV2Packet(pk) => pk,
                V662::LoginPacket(pk) => pk,
                V662::MapCreateLockedCopyPacket(pk) => pk,
                V662::MapInfoRequestPacket(pk) => pk,
                V662::MobArmorEquipmentPacket(pk) => pk,
                V662::MobEffectPacket(pk) => pk,
                V662::MobEquipmentPacket(pk) => pk,
                V662::ModalFormRequestPacket(pk) => pk,
                V662::ModalFormResponsePacket(pk) => pk,
                V662::MotionPredictionHintsPacket(pk) => pk,
                V662::MoveActorAbsolutePacket(pk) => pk,
                V662::MoveActorDeltaPacket(pk) => pk,
                V662::MovePlayerPacket(pk) => pk,
                V662::MultiplayerSettingsPacket(pk) => pk,
                V662::NetworkChunkPublisherUpdatePacket(pk) => pk,
                V662::NetworkSettingsPacket(pk) => pk,
                V662::NetworkStackLatencyPacket(pk) => pk,
                V662::NpcDialoguePacket(pk) => pk,
                V662::NpcRequestPacket(pk) => pk,
                V662::OnScreenTextureAnimationPacket(pk) => pk,
                V662::OpenSignPacket(pk) => pk,
                V662::PacketViolationWarningPacket(pk) => pk,
                V662::PassengerJumpPacket(pk) => pk,
                V662::PhotoTransferPacket(pk) => pk,
                V662::PlaySoundPacket(pk) => pk,
                V662::PlayStatusPacket(pk) => pk,
                V662::PlayerActionPacket(pk) => pk,
                V662::PlayerArmorDamagePacket(pk) => pk,
                V662::PlayerAuthInputPacket(pk) => pk,
                V662::PlayerEnchantOptionsPacket(pk) => pk,
                V662::PlayerFogPacket(pk) => pk,
                V662::PlayerHotbarPacket(pk) => pk,
                V662::PlayerInputPacket(pk) => pk,
                V662::PlayerListPacket(pk) => pk,
                V662::PlayerSkinPacket(pk) => pk,
                V662::PlayerStartItemCooldownPacket(pk) => pk,
                V662::PlayerToggleCrafterSlotRequestPacket(pk) => pk,
                V662::PositionTrackingDBClientRequestPacket(pk) => pk,
                V662::PositionTrackingDBServerBroadcastPacket(pk) => pk,
                V662::PurchaseReceiptPacket(pk) => pk,
                V662::RefreshEntitlementsPacket(pk) => pk,
                V662::RemoveActorPacket(pk) => pk,
                V662::RemoveObjectivePacket(pk) => pk,
                V662::RemoveVolumeEntityPacket(pk) => pk,
                V662::RequestAbilityPacket(pk) => pk,
                V662::RequestChunkRadiusPacket(pk) => pk,
                V662::RequestNetworkSettingsPacket(pk) => pk,
                V662::RequestPermissionsPacket(pk) => pk,
                V662::ResourcePackChunkDataPacket(pk) => pk,
                V662::ResourcePackChunkRequestPacket(pk) => pk,
                V662::ResourcePackClientResponsePacket(pk) => pk,
                V662::ResourcePackDataInfoPacket(pk) => pk,
                V662::ResourcePackStackPacket(pk) => pk,
                V662::ResourcePacksInfoPacket(pk) => pk,
                V662::RespawnPacket(pk) => pk,
                V662::ScriptMessagePacket(pk) => pk,
                V662::ServerPlayerPostMovePositionPacket(pk) => pk,
                V662::ServerSettingsRequestPacket(pk) => pk,
                V662::ServerSettingsResponsePacket(pk) => pk,
                V662::ServerStatsPacket(pk) => pk,
                V662::ServerToClientHandshakePacket(pk) => pk,
                V662::SetActorDataPacket(pk) => pk,
                V662::SetActorLinkPacket(pk) => pk,
                V662::SetActorMotionPacket(pk) => pk,
                V662::SetCommandsEnabledPacket(pk) => pk,
                V662::SetDefaultGameTypePacket(pk) => pk,
                V662::SetDifficultyPacket(pk) => pk,
                V662::SetDisplayObjectivePacket(pk) => pk,
                V662::SetHealthPacket(pk) => pk,
                V662::SetHudPacket(pk) => pk,
                V662::SetLastHurtByPacket(pk) => pk,
                V662::SetLocalPlayerAsInitializedPacket(pk) => pk,
                V662::SetPlayerGameTypePacket(pk) => pk,
                V662::SetPlayerInventoryOptionsPacket(pk) => pk,
                V662::SetScorePacket(pk) => pk,
                V662::SetScoreboardIdentityPacket(pk) => pk,
                V662::SetSpawnPositionPacket(pk) => pk,
                V662::SetTimePacket(pk) => pk,
                V662::SetTitlePacket(pk) => pk,
                V662::SettingsCommandPacket(pk) => pk,
                V662::ShowCreditsPacket(pk) => pk,
                V662::ShowProfilePacket(pk) => pk,
                V662::ShowStoreOfferPacket(pk) => pk,
                V662::SimpleEventPacket(pk) => pk,
                V662::SimulationTypePacket(pk) => pk,
                V662::SpawnExperienceOrbPacket(pk) => pk,
                V662::SpawnParticleEffectPacket(pk) => pk,
                V662::StartGamePacket(pk) => pk,
                V662::StopSoundPacket(pk) => pk,
                V662::StructureBlockUpdatePacket(pk) => pk,
                V662::StructureDataRequestPacket(pk) => pk,
                V662::StructureDataResponsePacket(pk) => pk,
                V662::SubChunkPacket(pk) => pk,
                V662::SubChunkRequestPacket(pk) => pk,
                V662::SubClientLoginPacket(pk) => pk,
                V662::SyncActorPropertyPacket(pk) => pk,
                V662::TakeItemActorPacket(pk) => pk,
                V662::TextPacket(pk) => pk,
                V662::TickSyncPacket(pk) => pk,
                V662::TickingAreaLoadStatusPacket(pk) => pk,
                V662::ToastRequestPacket(pk) => pk,
                V662::TransferPlayerPacket(pk) => pk,
                V662::TrimDataPacket(pk) => pk,
                V662::UnlockedRecipesPacket(pk) => pk,
                V662::UpdateAbilitiesPacket(pk) => pk,
                V662::UpdateAdventureSettingsPacket(pk) => pk,
                V662::UpdateAttributesPacket(pk) => pk,
                V662::UpdateBlockPacket(pk) => pk,
                V662::UpdateBlockSyncedPacket(pk) => pk,
                V662::UpdateClientInputLocksPacket(pk) => pk,
                V662::UpdateEquipPacket(pk) => pk,
                V662::UpdatePlayerGameTypePacket(pk) => pk,
                V662::UpdateSoftEnumPacket(pk) => pk,
                V662::UpdateSubChunkBlocksPacket(pk) => pk,
                V662::UpdateTradePacket(pk) => pk,
                V662::Unknown(pk) => pk,
            }
        }
    }
}
#[cfg(feature = "v662")]
pub use inner::*;
