#![allow(unused)]
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
        fn inner(&self) -> &dyn bedrock_protocol_core::DynPacket {
            match self {
                Unknown::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                Unknown::Unknown(pk) => pk.as_ref(),
            }
        }
        #[inline]
        fn into_inner(self) -> Box<dyn bedrock_protocol_core::DynPacket> {
            match self {
                Unknown::RequestNetworkSettingsPacket(pk) => pk,
                Unknown::Unknown(pk) => pk,
            }
        }
    }
    impl ProtoVersionPackets for Unknown {
        type SetTitlePacket = ();
        type SpawnExperienceOrbPacket = ();
        type BlockActorDataPacket = ();
        type AddBehaviourTreePacket = ();
        type AvailableActorIdentifiersPacket = ();
        type CompressedBiomeDefinitionListPacket = ();
        type GuiDataPickItemPacket = ();
        type SetSpawnPositionPacket = ();
        type MapCreateLockedCopyPacket = ();
        type RemoveObjectivePacket = ();
        type ServerToClientHandshakePacket = ();
        type SetLastHurtByPacket = ();
        type UpdateAttributesPacket = ();
        type ServerBoundDataStorePacket = ();
        type ServerPresenceInfoPacket = ();
        type ActorEventPacket = ();
        type HurtArmorPacket = ();
        type LevelEventPacket = ();
        type ModalFormRequestPacket = ();
        type BookEditPacket = ();
        type SetScoreboardIdentityPacket = ();
        type CameraAimAssistActorPriorityPacket = ();
        type EduUriResourcePacket = ();
        type CreativeContentPacket = ();
        type TickSyncPacket = ();
        type PlayerVideoCapturePacket = ();
        type PlayerAuthInputPacket = ();
        type GameTestRequestPacket = ();
        type CurrentStructureFeaturePacket = ();
        type ClientBoundDataDrivenUIReloadPacket = ();
        type LoginPacket = ();
        type ModalFormResponsePacket = ();
        type MultiplayerSettingsPacket = ();
        type ContainerOpenPacket = ();
        type CommandBlockUpdatePacket = ();
        type TextPacket = ();
        type MoveActorDeltaPacket = ();
        type VoxelShapesPacket = ();
        type SetMovementAuthorityPacket = ();
        type ClientBoundDebugRendererPacket = ();
        type ActorPickRequestPacket = ();
        type AnimatePacket = ();
        type RefreshEntitlementsPacket = ();
        type UpdateClientInputLocksPacket = ();
        type EmoteListPacket = ();
        type StructureDataResponsePacket = ();
        type ServerBoundPackSettingChangePacket = ();
        type NpcDialoguePacket = ();
        type CameraPacket = ();
        type ServerStatsPacket = ();
        type SetPlayerGameTypePacket = ();
        type RequestNetworkSettingsPacket =
            crate::version::unknown::packets::RequestNetworkSettingsPacket;
        type StructureBlockUpdatePacket = ();
        type CraftingDataPacket = ();
        type LecternUpdatePacket = ();
        type SetScorePacket = ();
        type LocatorBarPacket = ();
        type CameraInstructionPacket = ();
        type ServerBoundLoadingScreenPacket = ();
        type LevelSoundEventV2Packet = ();
        type UpdateAbilitiesPacket = ();
        type TransferPlayerPacket = ();
        type CodeBuilderSourcePacket = ();
        type CameraAimAssistPacket = ();
        type ResourcePacksInfoPacket = ();
        type ServerBoundDataDrivenClosedPacket = ();
        type MovementEffectPacket = ();
        type ResourcePackDataInfoPacket = ();
        type NetworkStackLatencyPacket = ();
        type ResourcePackClientResponsePacket = ();
        type PlayerUpdateEntityOverridesPacket = ();
        type AutomationClientConnectPacket = ();
        type MapInfoRequestPacket = ();
        type UpdateAdventureSettingsPacket = ();
        type BlockEventPacket = ();
        type ClientBoundDataDrivenUIShowScreenPacket = ();
        type LessonProgressPacket = ();
        type LevelSoundEventPacket = ();
        type PlaySoundPacket = ();
        type SpawnParticleEffectPacket = ();
        type BossEventPacket = ();
        type MobEquipmentPacket = ();
        type PlayerInputPacket = ();
        type ServerPlayerPostMovePositionPacket = ();
        type StructureDataRequestPacket = ();
        type SetHudPacket = ();
        type EditorNetworkPacket = ();
        type ChangeDimensionPacket = ();
        type ShowStoreOfferPacket = ();
        type CameraShakePacket = ();
        type ClientBoundAttributeLayerSyncPacket = ();
        type PositionTrackingDBClientRequestPacket = ();
        type InventorySlotPacket = ();
        type AddPaintingPacket = ();
        type ItemStackResponsePacket = ();
        type ServerStoreInfoPacket = ();
        type SetDisplayObjectivePacket = ();
        type StartGamePacket = ();
        type AgentAnimationPacket = ();
        type RequestChunkRadiusPacket = ();
        type AddPlayerPacket = ();
        type CameraAimAssistInstructionPacket = ();
        type PacketViolationWarningPacket = ();
        type ServerSettingsResponsePacket = ();
        type RemoveActorPacket = ();
        type ContainerClosePacket = ();
        type ClientCacheStatusPacket = ();
        type CompletedUsingItemPacket = ();
        type DisconnectPacket = ();
        type LegacyTelemetryEventPacket = ();
        type SetDifficultyPacket = ();
        type AwardAchievementPacket = ();
        type ServerBoundDiagnosticsPacket = ();
        type FeatureRegistryPacket = ();
        type GameRulesChangedPacket = ();
        type MobEffectPacket = ();
        type UnlockedRecipesPacket = ();
        type SimpleEventPacket = ();
        type AddItemActorPacket = ();
        type FilterTextPacket = ();
        type CodeBuilderPacket = ();
        type PlayerFogPacket = ();
        type UpdateBlockSyncedPacket = ();
        type ClientBoundCloseFormPacket = ();
        type GameTestResultsPacket = ();
        type SetHealthPacket = ();
        type AvailableCommandsPacket = ();
        type DeathInfoPacket = ();
        type PositionTrackingDBServerBroadcastPacket = ();
        type SettingsCommandPacket = ();
        type ClientCacheMissResponsePacket = ();
        type ServerSettingsRequestPacket = ();
        type SetActorLinkPacket = ();
        type CameraPresetsPacket = ();
        type RequestAbilityPacket = ();
        type ShowCreditsPacket = ();
        type MotionPredictionHintsPacket = ();
        type BlockPickRequestPacket = ();
        type MovePlayerPacket = ();
        type OnScreenTextureAnimationPacket = ();
        type SetDefaultGameTypePacket = ();
        type RemoveVolumeEntityPacket = ();
        type ResourcePackChunkRequestPacket = ();
        type SetActorDataPacket = ();
        type CommandOutputPacket = ();
        type DimensionDataPacket = ();
        type ResourcePacksReadyForValidationPacket = ();
        type ItemStackRequestPacket = ();
        type NetworkChunkPublisherUpdatePacket = ();
        type ItemComponentPacket = ();
        type EducationSettingsPacket = ();
        type SetPlayerInventoryOptionsPacket = ();
        type PlayerSkinPacket = ();
        type UpdatePlayerGameTypePacket = ();
        type PlayerStartItemCooldownPacket = ();
        type AnvilDamagePacket = ();
        type AddActorPacket = ();
        type ResourcePackChunkDataPacket = ();
        type SyncWorldClocksPacket = ();
        type CommandRequestPacket = ();
        type InventoryContentPacket = ();
        type LevelSoundEventV1Packet = ();
        type SetLocalPlayerAsInitializedPacket = ();
        type LevelChunkPacket = ();
        type UpdateTradePacket = ();
        type NpcRequestPacket = ();
        type SetActorMotionPacket = ();
        type JigsawStructureDataPacket = ();
        type AddVolumeEntityPacket = ();
        type CreatePhotoPacket = ();
        type ClientBoundDataStorePacket = ();
        type MoveActorAbsolutePacket = ();
        type PlayerActionPacket = ();
        type InventoryTransactionPacket = ();
        type SyncActorPropertyPacket = ();
        type PlayerHotbarPacket = ();
        type PlayerToggleCrafterSlotRequestPacket = ();
        type SetCommandsEnabledPacket = ();
        type ClientCacheBlobStatusPacket = ();
        type CameraAimAssistPresetsPacket = ();
        type BiomeDefinitionListPacket = ();
        type CameraSplinePacket = ();
        type ContainerSetDataPacket = ();
        type PlayStatusPacket = ();
        type MobArmorEquipmentPacket = ();
        type PlayerEnchantOptionsPacket = ();
        type LevelEventGenericPacket = ();
        type ResourcePackStackPacket = ();
        type ToastRequestPacket = ();
        type ChunkRadiusUpdatedPacket = ();
        type PassengerJumpPacket = ();
        type ShowProfilePacket = ();
        type SubChunkPacket = ();
        type SubClientLoginPacket = ();
        type TickingAreaLoadStatusPacket = ();
        type ClientBoundControlSchemeSetPacket = ();
        type PlayerLocationPacket = ();
        type PurchaseReceiptPacket = ();
        type InteractPacket = ();
        type UpdateBlockPacket = ();
        type RequestPermissionsPacket = ();
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type ClientBoundMapItemDataPacket = ();
        type OpenSignPacket = ();
        type AnimateEntityPacket = ();
        type EmotePacket = ();
        type DebugDrawerPacket = ();
        type SubChunkRequestPacket = ();
        type CorrectPlayerMovePredictionPacket = ();
        type ChangeMobPropertyPacket = ();
        type NetworkSettingsPacket = ();
        type UpdateSoftEnumPacket = ();
        type MovementPredictionSyncPacket = ();
        type RespawnPacket = ();
        type UpdateEquipPacket = ();
        type ScriptMessagePacket = ();
        type TakeItemActorPacket = ();
        type PlayerListPacket = ();
        type ClientToServerHandshakePacket = ();
        type SetTimePacket = ();
        type TrimDataPacket = ();
        type ClientBoundTextureShiftPacket = ();
        type PlayerArmorDamagePacket = ();
        type DebugInfoPacket = ();
        type PartyChangedPacket = ();
        type UpdateSubChunkBlocksPacket = ();
        type PhotoTransferPacket = ();
        type AgentActionEventPacket = ();
        type LabTablePacket = ();
        type GraphicsParameterOverridePacket = ();
        type UpdateClientOptionsPacket = ();
        type StopSoundPacket = ();
        type ContainerRegistryCleanupPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket = ();
        type SimulationTypePacket = ();
    }
    impl ProtoVersionTypes for Unknown {
        type SmithingTransformRecipe = ();
        type Color = ();
        type BiomeConditionalTransformationData = ();
        type CameraAimAssistPresetDefinition = ();
        type ItemStackResponseInfo = ();
        type SerializedAbilitiesData = ();
        type Experiments = ();
        type RecipeIngredient = ();
        type ShapelessRecipe = ();
        type StructureEditorData = ();
        type BiomeDefinitionChunkGenData = ();
        type BiomeElementData = ();
        type BiomeSurfaceMaterialData = ();
        type ItemStackResponseSlotInfo = ();
        type CameraPresets = ();
        type ContainerMixDataEntry = ();
        type EduSharedUriResource = ();
        type InventoryAction = ();
        type ItemData = ();
        type BaseGameVersion = ();
        type SmithingTrimRecipe = ();
        type CameraAimAssistPreset = ();
        type BiomeClimateData = ();
        type MoveActorAbsoluteData = ();
        type NetworkPermissions = ();
        type BlockPos = ();
        type SerializedSkin = ();
        type BiomeMesaSurfaceData = ();
        type BiomeSurfaceMaterialAdjustmentData = ();
        type InventorySource = ();
        type BiomeWeightedTemperatureData = ();
        type CommandOriginData = ();
        type BiomeReplacementData = ();
        type BiomeMountainParamsData = ();
        type MapDecoration = ();
        type MoveActorDeltaData = ();
        type ShapelessChemistryRecipe = ();
        type MaterialReducerDataEntry = ();
        type SpawnSettings = ();
        type DataItem = ();
        type PositionTrackingId = ();
        type BiomeCoordinateData = ();
        type CraftingDataEntry = ();
        type LevelSettings = ();
        type NetworkItemStackDescriptor = ();
        type PropertySyncData = ();
        type BiomeDefinition = ();
        type ChunkPos = ();
        type EducationLevelSettings = ();
        type SubChunkPos = ();
        type WebSocketPacketData = ();
        type RecipeUnlockingRequirement = ();
        type BiomeConsolidatedFeatureList = ();
        type EntityNetID = ();
        type SyncedPlayerMovementSettings = ();
        type PlayerBlockActionData = ();
        type MapItemTrackedActorUniqueID = ();
        type MolangVariableMap = ();
        type PotionMixDataEntry = ();
        type CameraInstruction = ();
        type ShapedRecipe = ();
        type GameRulesChangedPacketData = ();
        type InventoryTransaction = ();
        type BaseDescription = ();
        type ActorUniqueID = ();
        type ActorLink = ();
        type DimensionDefinitionGroup = ();
        type ScoreboardId = ();
        type ActorRuntimeID = ();
        type CameraPreset = ();
        type NetworkBlockPosition = ();
        type ShapedChemistryRecipe = ();
        type FullContainerName = ();
        type CameraAimAssistItemSettings = ();
        type CameraSplineInstruction = ();
        type CameraAimAssistPriority = ();
        type BiomeScatterParamData = ();
        type BiomeSurfaceBuilderData = ();
        type PackedItemUseLegacyInventoryTransaction = ();
        type ItemStackResponseContainerInfo = ();
        type StructureSettings = ();
        type BiomeLegacyWorldGenRulesData = ();
        type CameraAimAssistCategory = ();
        type AdventureSettings = ();
        type CameraAimAssistCategories = ();
        type DebugShape = ();
        type SubChunkPosOffset = ();
        type BiomeWeightedData = ();
        type BiomeCappedSurfaceData = ();
        type BiomeOverworldGenRulesData = ();
        type ItemEnchants = ();
        type ItemStackRequestSlotInfo = ();
        type NetworkItemInstanceDescriptor = ();
        type ShulkerBoxRecipe = ();
        type BiomeMultinoiseGenRulesData = ();
        type BiomeNoiseGradientSurfaceData = ();
    }
    impl ProtoVersionEnums for Unknown {
        type IdentityDefinitionType = ();
        type StructureBlockType = ();
        type Difficulty = ();
        type Rotation = ();
        type MultiplayerSettingsPacketType = ();
        type ItemReleaseInventoryTransactionType = ();
        type ContainerType = ();
        type DataItemType = ();
        type InventoryLeftTabIndex = ();
        type LevelEvent = ();
        type AnimationMode = ();
        type CommandBlockMode = ();
        type CodeBuilderStorageOperation = ();
        type ItemUseInventoryTransactionType = ();
        type Mirror = ();
        type StructureTemplateRequestOperation = ();
        type CommandParameterOption = ();
        type HudElement = ();
        type CameraShakeAction = ();
        type StructureTemplateResponseType = ();
        type TextProcessingEventOrigin = ();
        type AimAssistAction = ();
        type AnimationExpression = ();
        type ItemDescriptorType = ();
        type ActorType = ();
        type ModalFormCancelReason = ();
        type PacketViolationSeverity = ();
        type ItemVersion = ();
        type ItemUseMethod = ();
        type PredictionType = ();
        type AbilitiesIndex = ();
        type CameraAimAssistOperation = ();
        type SpawnBiomeType = ();
        type ContainerID = ();
        type InventoryLayout = ();
        type LabTableReactionType = ();
        type CameraShakeType = ();
        type AnimatedTextureType = ();
        type PlayerRespawnState = ();
        type ControlScheme = ();
        type AttributeOperands = ();
        type ItemStackNetResult = ();
        type PackType = ();
        type InputMode = ();
        type PacketCompressionAlgorithm = ();
        type InventorySourceFlags = ();
        type CommandOutputType = ();
        type CodeBuilderCodeStatus = ();
        type EnchantType = ();
        type ActorDamageCause = ();
        type BossEventUpdateType = ();
        type GameType = ();
        type ItemStackRequestActionType = ();
        type BuildPlatform = ();
        type ActorFlags = ();
        type TextPacketType = ();
        type ActorLinkType = ();
        type HudVisibility = ();
        type PlayerPermissionLevel = ();
        type ShowStoreOfferRedirectType = ();
        type InteractionType = ();
        type ActorEvent = ();
        type AuthoritativeMovementMode = ();
        type MovementEffectType = ();
        type ComplexInventoryTransactionType = ();
        type LessonAction = ();
        type ActorBlockSyncMessageID = ();
        type PhotoType = ();
        type POIBlockInteractionType = ();
        type GeneratorType = ();
        type ParticleType = ();
        type CameraSplineEaseType = ();
        type AttributeModifierOperation = ();
        type ItemUseOnActorInventoryTransactionType = ();
        type NewInteractionModel = ();
        type StructureRedstoneSaveMode = ();
        type ActorDataIDs = ();
        type EducationEditionOffer = ();
        type PlayerPositionMode = ();
        type ResourcePackResponse = ();
        type EditorWorldType = ();
        type MinecraftPacketIds = ();
        type SimulationType = ();
        type InventoryRightTabIndex = ();
        type UIProfile = ();
        type CraftingDataEntryType = ();
        type SoftEnumUpdateType = ();
        type MolangVersion = ();
        type ObjectiveSortOrder = ();
        type InventorySourceType = ();
        type CodeBuilderStorageCategory = ();
        type CommandOriginType = ();
        type CommandPermissionLevel = ();
        type PlayStatus = ();
        type SpawnPositionType = ();
        type AgentActionType = ();
        type BookEditAction = ();
        type ContainerEnumName = ();
        type EasingType = ();
        type CameraSplineType = ();
        type ChatRestrictionLevel = ();
        type GamePublishSetting = ();
        type ConnectionFailReason = ();
        type LevelSoundEventType = ();
        type TeleportationCause = ();
        type PacketViolationType = ();
        type ServerAuthMovementMode = ();
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
