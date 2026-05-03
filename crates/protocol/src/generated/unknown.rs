#[cfg(feature = "unknown")]
mod inner {
    use crate::ProtoVersion;
    use crate::ProtoVersionEnums;
    use crate::ProtoVersionPackets;
    use crate::ProtoVersionTypes;
    #[derive(Clone, std::fmt::Debug)]
    pub enum Unknown {
        RequestNetworkSettingsPacket(
            Box<<Self as ProtoVersionPackets>::RequestNetworkSettingsPacket>,
        ),
        Unknown(Box<bedrock_protocol_core::UnknownPacket>),
    }
    impl bedrock_protocol_core::DynPacket for Unknown {
        #[inline]
        fn id(&self) -> u16 {
            match self {
                Unknown::RequestNetworkSettingsPacket(_) => {
                    <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                Unknown::Unknown(pk) => pk.id,
            }
        }
    }
    impl bedrock_protocol_core::Packets for Unknown {
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
                Unknown::RequestNetworkSettingsPacket(pk) => {
                    match <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::serialize(
                        pk.as_ref(),
                        stream,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
                                error: err,
                            });
                        }
                    };
                }
                Unknown::Unknown(pk) => stream.write_all(pk.buf.as_ref()).map_err(|e| {
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
                <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID => {
                    match <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::deserialize(
                        stream,
                    ) {
                        Ok(pk) => Unknown::RequestNetworkSettingsPacket(Box::new(pk)),
                        Err(err) => {
                            return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: stringify!(RequestNetworkSettingsPacket),
                                packet_id: <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID,
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
                    Unknown::Unknown(
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
                    Unknown::RequestNetworkSettingsPacket(pk) => {
                        <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::ProtoCodec>::size_hint(
                            pk.as_ref(),
                        )
                    }
                    Unknown::Unknown(pk) => pk.buf.len(),
                }
        }
        #[inline]
        fn as_dyn(&self) -> &dyn bedrock_protocol_core::DynPacket {
            match self {
                Unknown::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                Unknown::Unknown(pk) => pk.as_ref(),
            }
        }
        #[inline]
        fn into_dyn(self) -> Box<dyn bedrock_protocol_core::DynPacket> {
            match self {
                Unknown::RequestNetworkSettingsPacket(pk) => pk,
                Unknown::Unknown(pk) => pk,
            }
        }
    }
    impl ProtoVersionPackets for Unknown {
        type UpdateTradePacket = ();
        type VoxelShapesPacket = ();
        type PacketViolationWarningPacket = ();
        type InteractPacket = ();
        type TransferPlayerPacket = ();
        type SetDifficultyPacket = ();
        type DebugInfoPacket = ();
        type AddVolumeEntityPacket = ();
        type PlayerLocationPacket = ();
        type ModalFormResponsePacket = ();
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type PlayerVideoCapturePacket = ();
        type InventoryTransactionPacket = ();
        type GraphicsParameterOverridePacket = ();
        type TrimDataPacket = ();
        type MobEffectPacket = ();
        type ClientCacheStatusPacket = ();
        type NpcRequestPacket = ();
        type ResourcePackClientResponsePacket = ();
        type ResourcePackDataInfoPacket = ();
        type UpdateBlockSyncedPacket = ();
        type SetLocalPlayerAsInitializedPacket = ();
        type AutomationClientConnectPacket = ();
        type PurchaseReceiptPacket = ();
        type ClientBoundAttributeLayerSyncPacket = ();
        type BlockPickRequestPacket = ();
        type MovementEffectPacket = ();
        type ServerToClientHandshakePacket = ();
        type CameraPresetsPacket = ();
        type ContainerClosePacket = ();
        type AnimatePacket = ();
        type DisconnectPacket = ();
        type SetDefaultGameTypePacket = ();
        type PlayerListPacket = ();
        type AnvilDamagePacket = ();
        type SubClientLoginPacket = ();
        type CameraAimAssistActorPriorityPacket = ();
        type UpdatePlayerGameTypePacket = ();
        type SetScoreboardIdentityPacket = ();
        type NpcDialoguePacket = ();
        type LevelEventPacket = ();
        type SetPlayerGameTypePacket = ();
        type PlaySoundPacket = ();
        type EducationSettingsPacket = ();
        type PlayerEnchantOptionsPacket = ();
        type BiomeDefinitionListPacket = ();
        type BlockEventPacket = ();
        type ShowStoreOfferPacket = ();
        type RefreshEntitlementsPacket = ();
        type AvailableActorIdentifiersPacket = ();
        type SyncActorPropertyPacket = ();
        type AvailableCommandsPacket = ();
        type CommandRequestPacket = ();
        type BlockActorDataPacket = ();
        type MovePlayerPacket = ();
        type RemoveObjectivePacket = ();
        type FeatureRegistryPacket = ();
        type PlayerToggleCrafterSlotRequestPacket = ();
        type CameraInstructionPacket = ();
        type PlayerHotbarPacket = ();
        type AwardAchievementPacket = ();
        type JigsawStructureDataPacket = ();
        type SetPlayerInventoryOptionsPacket = ();
        type MobArmorEquipmentPacket = ();
        type ShowProfilePacket = ();
        type CameraPacket = ();
        type ServerBoundDiagnosticsPacket = ();
        type SetSpawnPositionPacket = ();
        type CodeBuilderSourcePacket = ();
        type MapInfoRequestPacket = ();
        type EduUriResourcePacket = ();
        type PlayerActionPacket = ();
        type UpdateAbilitiesPacket = ();
        type ServerBoundLoadingScreenPacket = ();
        type StopSoundPacket = ();
        type TakeItemActorPacket = ();
        type MoveActorAbsolutePacket = ();
        type MotionPredictionHintsPacket = ();
        type CameraSplinePacket = ();
        type GuiDataPickItemPacket = ();
        type ClientBoundMapItemDataPacket = ();
        type CameraAimAssistPacket = ();
        type HurtArmorPacket = ();
        type PassengerJumpPacket = ();
        type CurrentStructureFeaturePacket = ();
        type CommandBlockUpdatePacket = ();
        type CreativeContentPacket = ();
        type GameTestResultsPacket = ();
        type ItemStackResponsePacket = ();
        type UpdateSubChunkBlocksPacket = ();
        type ClientBoundCloseFormPacket = ();
        type PlayerAuthInputPacket = ();
        type InventorySlotPacket = ();
        type ActorPickRequestPacket = ();
        type ContainerSetDataPacket = ();
        type AddPlayerPacket = ();
        type LevelEventGenericPacket = ();
        type ScriptMessagePacket = ();
        type CameraAimAssistInstructionPacket = ();
        type GameRulesChangedPacket = ();
        type StartGamePacket = ();
        type ClientCacheMissResponsePacket = ();
        type AddBehaviourTreePacket = ();
        type BossEventPacket = ();
        type SetActorDataPacket = ();
        type SetHudPacket = ();
        type ChangeMobPropertyPacket = ();
        type ShowCreditsPacket = ();
        type SetTitlePacket = ();
        type ServerStoreInfoPacket = ();
        type CommandOutputPacket = ();
        type PhotoTransferPacket = ();
        type SyncWorldClocksPacket = ();
        type AgentActionEventPacket = ();
        type UpdateClientOptionsPacket = ();
        type LevelChunkPacket = ();
        type OnScreenTextureAnimationPacket = ();
        type ContainerRegistryCleanupPacket = ();
        type CameraShakePacket = ();
        type MapCreateLockedCopyPacket = ();
        type UpdateClientInputLocksPacket = ();
        type ResourcePackChunkDataPacket = ();
        type PlayerFogPacket = ();
        type ServerPresenceInfoPacket = ();
        type ItemComponentPacket = ();
        type PositionTrackingDBClientRequestPacket = ();
        type LevelSoundEventV2Packet = ();
        type ItemStackRequestPacket = ();
        type ServerSettingsResponsePacket = ();
        type SetCommandsEnabledPacket = ();
        type SpawnExperienceOrbPacket = ();
        type AddPaintingPacket = ();
        type CreatePhotoPacket = ();
        type SetHealthPacket = ();
        type SetDisplayObjectivePacket = ();
        type ServerSettingsRequestPacket = ();
        type NetworkSettingsPacket = ();
        type SettingsCommandPacket = ();
        type LegacyTelemetryEventPacket = ();
        type ServerBoundPackSettingChangePacket = ();
        type ClientBoundDebugRendererPacket = ();
        type ClientBoundControlSchemeSetPacket = ();
        type ServerBoundDataStorePacket = ();
        type GameTestRequestPacket = ();
        type CameraAimAssistPresetsPacket = ();
        type ClientBoundDataDrivenUIShowScreenPacket = ();
        type ContainerOpenPacket = ();
        type RequestPermissionsPacket = ();
        type PlayerUpdateEntityOverridesPacket = ();
        type LecternUpdatePacket = ();
        type RemoveVolumeEntityPacket = ();
        type ServerPlayerPostMovePositionPacket = ();
        type SubChunkPacket = ();
        type RequestAbilityPacket = ();
        type CodeBuilderPacket = ();
        type InventoryContentPacket = ();
        type ClientBoundDataStorePacket = ();
        type PartyChangedPacket = ();
        type DebugDrawerPacket = ();
        type UnlockedRecipesPacket = ();
        type ServerStatsPacket = ();
        type LessonProgressPacket = ();
        type CompletedUsingItemPacket = ();
        type LevelSoundEventPacket = ();
        type SetScorePacket = ();
        type StructureDataResponsePacket = ();
        type PlayStatusPacket = ();
        type ChangeDimensionPacket = ();
        type ClientCacheBlobStatusPacket = ();
        type AgentAnimationPacket = ();
        type ClientBoundTextureShiftPacket = ();
        type UpdateAdventureSettingsPacket = ();
        type NetworkStackLatencyPacket = ();
        type ResourcePacksReadyForValidationPacket = ();
        type NetworkChunkPublisherUpdatePacket = ();
        type UpdateSoftEnumPacket = ();
        type LevelSoundEventV1Packet = ();
        type LoginPacket = ();
        type ChunkRadiusUpdatedPacket = ();
        type ResourcePacksInfoPacket = ();
        type TextPacket = ();
        type DeathInfoPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket = ();
        type BookEditPacket = ();
        type ModalFormRequestPacket = ();
        type SetTimePacket = ();
        type UpdateEquipPacket = ();
        type TickingAreaLoadStatusPacket = ();
        type SetActorLinkPacket = ();
        type LocatorBarPacket = ();
        type ResourcePackChunkRequestPacket = ();
        type RequestChunkRadiusPacket = ();
        type ToastRequestPacket = ();
        type FilterTextPacket = ();
        type AnimateEntityPacket = ();
        type LabTablePacket = ();
        type MultiplayerSettingsPacket = ();
        type SetLastHurtByPacket = ();
        type ResourcePackStackPacket = ();
        type ActorEventPacket = ();
        type SubChunkRequestPacket = ();
        type RequestNetworkSettingsPacket =
            crate::version::unknown::packets::RequestNetworkSettingsPacket;
        type CorrectPlayerMovePredictionPacket = ();
        type AddItemActorPacket = ();
        type MovementPredictionSyncPacket = ();
        type MobEquipmentPacket = ();
        type AddActorPacket = ();
        type MoveActorDeltaPacket = ();
        type PositionTrackingDBServerBroadcastPacket = ();
        type PlayerArmorDamagePacket = ();
        type EmoteListPacket = ();
        type DimensionDataPacket = ();
        type CraftingDataPacket = ();
        type EditorNetworkPacket = ();
        type ServerBoundDataDrivenClosedPacket = ();
        type RemoveActorPacket = ();
        type TickSyncPacket = ();
        type SetMovementAuthorityPacket = ();
        type PlayerSkinPacket = ();
        type StructureBlockUpdatePacket = ();
        type CompressedBiomeDefinitionListPacket = ();
        type SetActorMotionPacket = ();
        type SpawnParticleEffectPacket = ();
        type UpdateAttributesPacket = ();
        type UpdateBlockPacket = ();
        type ClientBoundDataDrivenUIReloadPacket = ();
        type SimpleEventPacket = ();
        type OpenSignPacket = ();
        type EmotePacket = ();
        type PlayerInputPacket = ();
        type PlayerStartItemCooldownPacket = ();
        type ClientToServerHandshakePacket = ();
        type RespawnPacket = ();
        type StructureDataRequestPacket = ();
        type SimulationTypePacket = ();
    }
    impl ProtoVersionTypes for Unknown {
        type ShapelessChemistryRecipe = ();
        type FullContainerName = ();
        type ActorRuntimeID = ();
        type DimensionDefinitionGroup = ();
        type ItemStackResponseContainerInfo = ();
        type NetworkItemInstanceDescriptor = ();
        type CameraAimAssistPreset = ();
        type BaseDescription = ();
        type BiomeClimateData = ();
        type RecipeUnlockingRequirement = ();
        type BiomeDefinitionChunkGenData = ();
        type NetworkPermissions = ();
        type BiomeDefinition = ();
        type CameraAimAssistPriority = ();
        type BiomeSurfaceMaterialAdjustmentData = ();
        type NetworkBlockPosition = ();
        type PositionTrackingId = ();
        type BiomeElementData = ();
        type BiomeSurfaceMaterialData = ();
        type CommandOriginData = ();
        type InventoryAction = ();
        type BiomeCappedSurfaceData = ();
        type ShapedRecipe = ();
        type SerializedSkin = ();
        type EduSharedUriResource = ();
        type DebugShape = ();
        type BiomeOverworldGenRulesData = ();
        type ItemStackRequestSlotInfo = ();
        type MapItemTrackedActorUniqueID = ();
        type ContainerMixDataEntry = ();
        type StructureEditorData = ();
        type SpawnSettings = ();
        type ShapedChemistryRecipe = ();
        type StructureSettings = ();
        type BiomeConditionalTransformationData = ();
        type BiomeConsolidatedFeatureList = ();
        type RecipeIngredient = ();
        type MoveActorAbsoluteData = ();
        type BiomeNoiseGradientSurfaceData = ();
        type BiomeSurfaceBuilderData = ();
        type ChunkPos = ();
        type ItemData = ();
        type SmithingTrimRecipe = ();
        type CameraAimAssistPresetDefinition = ();
        type EntityNetID = ();
        type BlockPos = ();
        type CameraPreset = ();
        type BiomeMultinoiseGenRulesData = ();
        type BiomeScatterParamData = ();
        type CraftingDataEntry = ();
        type PotionMixDataEntry = ();
        type SubChunkPosOffset = ();
        type SyncedPlayerMovementSettings = ();
        type BiomeWeightedData = ();
        type Experiments = ();
        type NetworkItemStackDescriptor = ();
        type Color = ();
        type ActorLink = ();
        type PropertySyncData = ();
        type WebSocketPacketData = ();
        type CameraAimAssistCategories = ();
        type BiomeMesaSurfaceData = ();
        type MapDecoration = ();
        type AdventureSettings = ();
        type DataItem = ();
        type ItemStackResponseInfo = ();
        type MoveActorDeltaData = ();
        type SerializedAbilitiesData = ();
        type BiomeReplacementData = ();
        type InventorySource = ();
        type BiomeMountainParamsData = ();
        type BaseGameVersion = ();
        type PackedItemUseLegacyInventoryTransaction = ();
        type ShulkerBoxRecipe = ();
        type SubChunkPos = ();
        type ItemStackResponseSlotInfo = ();
        type MolangVariableMap = ();
        type CameraAimAssistItemSettings = ();
        type BiomeCoordinateData = ();
        type BiomeLegacyWorldGenRulesData = ();
        type GameRulesChangedPacketData = ();
        type CameraPresets = ();
        type InventoryTransaction = ();
        type CameraInstruction = ();
        type PlayerBlockActionData = ();
        type CameraAimAssistCategory = ();
        type BiomeWeightedTemperatureData = ();
        type EducationLevelSettings = ();
        type ShapelessRecipe = ();
        type CameraSplineInstruction = ();
        type ActorUniqueID = ();
        type SmithingTransformRecipe = ();
        type ItemEnchants = ();
        type LevelSettings = ();
        type ScoreboardId = ();
        type MaterialReducerDataEntry = ();
    }
    impl ProtoVersionEnums for Unknown {
        type ObjectiveSortOrder = ();
        type POIBlockInteractionType = ();
        type PackType = ();
        type AnimationMode = ();
        type PhotoType = ();
        type SoftEnumUpdateType = ();
        type ContainerID = ();
        type IdentityDefinitionType = ();
        type PlayerPositionMode = ();
        type ItemVersion = ();
        type ControlScheme = ();
        type ActorType = ();
        type GeneratorType = ();
        type ItemDescriptorType = ();
        type ActorFlags = ();
        type CraftingDataEntryType = ();
        type DataItemType = ();
        type BossEventUpdateType = ();
        type PacketViolationType = ();
        type CameraAimAssistOperation = ();
        type AttributeOperands = ();
        type ItemUseOnActorInventoryTransactionType = ();
        type StructureTemplateRequestOperation = ();
        type ActorEvent = ();
        type PacketViolationSeverity = ();
        type GameType = ();
        type CommandPermissionLevel = ();
        type CommandParameterOption = ();
        type CameraShakeType = ();
        type PlayStatus = ();
        type PlayerRespawnState = ();
        type ParticleType = ();
        type SpawnBiomeType = ();
        type ActorLinkType = ();
        type CameraSplineType = ();
        type EditorWorldType = ();
        type InventorySourceType = ();
        type BookEditAction = ();
        type LabTableReactionType = ();
        type ConnectionFailReason = ();
        type CodeBuilderStorageCategory = ();
        type MultiplayerSettingsPacketType = ();
        type PredictionType = ();
        type Mirror = ();
        type ServerAuthMovementMode = ();
        type HudElement = ();
        type ChatRestrictionLevel = ();
        type AnimatedTextureType = ();
        type Rotation = ();
        type SimulationType = ();
        type InteractionType = ();
        type StructureTemplateResponseType = ();
        type ComplexInventoryTransactionType = ();
        type TextPacketType = ();
        type TextProcessingEventOrigin = ();
        type LessonAction = ();
        type CommandOriginType = ();
        type AuthoritativeMovementMode = ();
        type HudVisibility = ();
        type EnchantType = ();
        type ItemUseMethod = ();
        type BuildPlatform = ();
        type InventoryLayout = ();
        type LevelEvent = ();
        type CameraShakeAction = ();
        type MinecraftPacketIds = ();
        type CodeBuilderStorageOperation = ();
        type AgentActionType = ();
        type ResourcePackResponse = ();
        type MovementEffectType = ();
        type LevelSoundEventType = ();
        type ActorDamageCause = ();
        type AnimationExpression = ();
        type MolangVersion = ();
        type InventoryLeftTabIndex = ();
        type Difficulty = ();
        type CommandOutputType = ();
        type PlayerPermissionLevel = ();
        type CodeBuilderCodeStatus = ();
        type SpawnPositionType = ();
        type ItemStackNetResult = ();
        type CommandBlockMode = ();
        type InputMode = ();
        type ModalFormCancelReason = ();
        type CameraSplineEaseType = ();
        type AttributeModifierOperation = ();
        type ActorDataIDs = ();
        type InventoryRightTabIndex = ();
        type AimAssistAction = ();
        type ActorBlockSyncMessageID = ();
        type InventorySourceFlags = ();
        type ContainerEnumName = ();
        type EducationEditionOffer = ();
        type ItemReleaseInventoryTransactionType = ();
        type StructureBlockType = ();
        type ItemUseInventoryTransactionType = ();
        type NewInteractionModel = ();
        type UIProfile = ();
        type ShowStoreOfferRedirectType = ();
        type StructureRedstoneSaveMode = ();
        type TeleportationCause = ();
        type PacketCompressionAlgorithm = ();
        type ItemStackRequestActionType = ();
        type EasingType = ();
        type GamePublishSetting = ();
        type AbilitiesIndex = ();
        type ContainerType = ();
    }
    impl ProtoVersion for Unknown {
        const PROTOCOL_VERSION: u32 = 0u32;
        const PROTOCOL_BRANCH: &str = "r/0_u0";
        const GAME_VERSION: &str = "0.0.0";
        const RAKNET_VERSION: u8 = 10u8;
    }
}
#[cfg(feature = "unknown")]
pub use inner::*;
