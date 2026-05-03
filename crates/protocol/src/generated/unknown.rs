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
        type ChunkRadiusUpdatedPacket = ();
        type SetDifficultyPacket = ();
        type BlockPickRequestPacket = ();
        type PlayerToggleCrafterSlotRequestPacket = ();
        type PurchaseReceiptPacket = ();
        type ClientBoundDataDrivenUIShowScreenPacket = ();
        type ItemComponentPacket = ();
        type CameraInstructionPacket = ();
        type LegacyTelemetryEventPacket = ();
        type CameraAimAssistActorPriorityPacket = ();
        type UpdateAbilitiesPacket = ();
        type LessonProgressPacket = ();
        type ClientBoundControlSchemeSetPacket = ();
        type InteractPacket = ();
        type AddPlayerPacket = ();
        type ClientCacheBlobStatusPacket = ();
        type ClientBoundCloseFormPacket = ();
        type TakeItemActorPacket = ();
        type NpcDialoguePacket = ();
        type LevelEventGenericPacket = ();
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type ChangeDimensionPacket = ();
        type GameTestRequestPacket = ();
        type ClientBoundDebugRendererPacket = ();
        type CommandBlockUpdatePacket = ();
        type OnScreenTextureAnimationPacket = ();
        type LecternUpdatePacket = ();
        type AnimateEntityPacket = ();
        type ResourcePackClientResponsePacket = ();
        type ModalFormRequestPacket = ();
        type RequestNetworkSettingsPacket =
            crate::version::unknown::packets::RequestNetworkSettingsPacket;
        type SetPlayerInventoryOptionsPacket = ();
        type StructureDataRequestPacket = ();
        type RespawnPacket = ();
        type ServerToClientHandshakePacket = ();
        type PlaySoundPacket = ();
        type MobArmorEquipmentPacket = ();
        type SettingsCommandPacket = ();
        type CameraAimAssistInstructionPacket = ();
        type PlayerActionPacket = ();
        type NpcRequestPacket = ();
        type PlayerEnchantOptionsPacket = ();
        type ServerSettingsResponsePacket = ();
        type ResourcePacksInfoPacket = ();
        type UpdateClientInputLocksPacket = ();
        type PlayerUpdateEntityOverridesPacket = ();
        type GraphicsParameterOverridePacket = ();
        type ResourcePacksReadyForValidationPacket = ();
        type ServerPresenceInfoPacket = ();
        type MoveActorDeltaPacket = ();
        type ToastRequestPacket = ();
        type ServerStatsPacket = ();
        type CurrentStructureFeaturePacket = ();
        type AnvilDamagePacket = ();
        type MovementEffectPacket = ();
        type BiomeDefinitionListPacket = ();
        type DebugDrawerPacket = ();
        type CameraAimAssistPresetsPacket = ();
        type PlayStatusPacket = ();
        type ServerBoundDiagnosticsPacket = ();
        type RemoveVolumeEntityPacket = ();
        type TickingAreaLoadStatusPacket = ();
        type RemoveActorPacket = ();
        type SetMovementAuthorityPacket = ();
        type PartyChangedPacket = ();
        type SubChunkPacket = ();
        type SetScorePacket = ();
        type SyncActorPropertyPacket = ();
        type ItemStackResponsePacket = ();
        type ResourcePackChunkRequestPacket = ();
        type ClientBoundAttributeLayerSyncPacket = ();
        type LevelSoundEventPacket = ();
        type MobEquipmentPacket = ();
        type NetworkStackLatencyPacket = ();
        type SetHealthPacket = ();
        type ClientBoundMapItemDataPacket = ();
        type SetTitlePacket = ();
        type BookEditPacket = ();
        type ResourcePackChunkDataPacket = ();
        type TextPacket = ();
        type ActorEventPacket = ();
        type UpdateBlockPacket = ();
        type UpdateSoftEnumPacket = ();
        type FilterTextPacket = ();
        type MapCreateLockedCopyPacket = ();
        type PlayerHotbarPacket = ();
        type LevelChunkPacket = ();
        type ServerBoundDataDrivenClosedPacket = ();
        type PlayerArmorDamagePacket = ();
        type SetActorLinkPacket = ();
        type RefreshEntitlementsPacket = ();
        type ServerBoundLoadingScreenPacket = ();
        type SimulationTypePacket = ();
        type CodeBuilderPacket = ();
        type AvailableActorIdentifiersPacket = ();
        type AddVolumeEntityPacket = ();
        type NetworkSettingsPacket = ();
        type ModalFormResponsePacket = ();
        type UpdateBlockSyncedPacket = ();
        type UpdateAdventureSettingsPacket = ();
        type LabTablePacket = ();
        type UpdateAttributesPacket = ();
        type CommandOutputPacket = ();
        type LevelSoundEventV1Packet = ();
        type GameTestResultsPacket = ();
        type ServerBoundPackSettingChangePacket = ();
        type MapInfoRequestPacket = ();
        type UpdateClientOptionsPacket = ();
        type CameraPresetsPacket = ();
        type EditorNetworkPacket = ();
        type ContainerClosePacket = ();
        type CreativeContentPacket = ();
        type SetActorDataPacket = ();
        type MotionPredictionHintsPacket = ();
        type CameraSplinePacket = ();
        type PositionTrackingDBServerBroadcastPacket = ();
        type PlayerInputPacket = ();
        type ClientCacheMissResponsePacket = ();
        type CameraShakePacket = ();
        type CraftingDataPacket = ();
        type RequestAbilityPacket = ();
        type SetLocalPlayerAsInitializedPacket = ();
        type UnlockedRecipesPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket = ();
        type ClientCacheStatusPacket = ();
        type EmotePacket = ();
        type PlayerStartItemCooldownPacket = ();
        type GameRulesChangedPacket = ();
        type AddItemActorPacket = ();
        type AgentAnimationPacket = ();
        type CreatePhotoPacket = ();
        type InventoryTransactionPacket = ();
        type ServerBoundDataStorePacket = ();
        type ContainerSetDataPacket = ();
        type SetDefaultGameTypePacket = ();
        type AwardAchievementPacket = ();
        type ClientBoundDataStorePacket = ();
        type MobEffectPacket = ();
        type DisconnectPacket = ();
        type SetScoreboardIdentityPacket = ();
        type AddPaintingPacket = ();
        type SetSpawnPositionPacket = ();
        type LevelSoundEventV2Packet = ();
        type ServerPlayerPostMovePositionPacket = ();
        type PlayerSkinPacket = ();
        type BlockActorDataPacket = ();
        type StopSoundPacket = ();
        type ActorPickRequestPacket = ();
        type UpdateTradePacket = ();
        type AgentActionEventPacket = ();
        type AddBehaviourTreePacket = ();
        type ResourcePackDataInfoPacket = ();
        type SpawnParticleEffectPacket = ();
        type StructureDataResponsePacket = ();
        type BlockEventPacket = ();
        type SetPlayerGameTypePacket = ();
        type DebugInfoPacket = ();
        type FeatureRegistryPacket = ();
        type CameraPacket = ();
        type TickSyncPacket = ();
        type AnimatePacket = ();
        type VoxelShapesPacket = ();
        type ResourcePackStackPacket = ();
        type SetDisplayObjectivePacket = ();
        type LevelEventPacket = ();
        type UpdateSubChunkBlocksPacket = ();
        type ContainerRegistryCleanupPacket = ();
        type UpdateEquipPacket = ();
        type EducationSettingsPacket = ();
        type SyncWorldClocksPacket = ();
        type AvailableCommandsPacket = ();
        type CorrectPlayerMovePredictionPacket = ();
        type SimpleEventPacket = ();
        type SetCommandsEnabledPacket = ();
        type SubChunkRequestPacket = ();
        type ItemStackRequestPacket = ();
        type MultiplayerSettingsPacket = ();
        type ClientBoundTextureShiftPacket = ();
        type LocatorBarPacket = ();
        type PacketViolationWarningPacket = ();
        type GuiDataPickItemPacket = ();
        type ShowCreditsPacket = ();
        type CodeBuilderSourcePacket = ();
        type PlayerAuthInputPacket = ();
        type RequestPermissionsPacket = ();
        type CameraAimAssistPacket = ();
        type InventorySlotPacket = ();
        type InventoryContentPacket = ();
        type CompressedBiomeDefinitionListPacket = ();
        type PlayerListPacket = ();
        type AutomationClientConnectPacket = ();
        type ContainerOpenPacket = ();
        type StructureBlockUpdatePacket = ();
        type SubClientLoginPacket = ();
        type UpdatePlayerGameTypePacket = ();
        type MoveActorAbsolutePacket = ();
        type BossEventPacket = ();
        type LoginPacket = ();
        type RequestChunkRadiusPacket = ();
        type RemoveObjectivePacket = ();
        type MovePlayerPacket = ();
        type OpenSignPacket = ();
        type ServerSettingsRequestPacket = ();
        type ShowProfilePacket = ();
        type AddActorPacket = ();
        type CommandRequestPacket = ();
        type ChangeMobPropertyPacket = ();
        type ClientBoundDataDrivenUIReloadPacket = ();
        type PhotoTransferPacket = ();
        type DimensionDataPacket = ();
        type SetHudPacket = ();
        type TrimDataPacket = ();
        type PlayerLocationPacket = ();
        type CompletedUsingItemPacket = ();
        type SetTimePacket = ();
        type DeathInfoPacket = ();
        type JigsawStructureDataPacket = ();
        type SpawnExperienceOrbPacket = ();
        type PlayerVideoCapturePacket = ();
        type ServerStoreInfoPacket = ();
        type ShowStoreOfferPacket = ();
        type PassengerJumpPacket = ();
        type EmoteListPacket = ();
        type EduUriResourcePacket = ();
        type PositionTrackingDBClientRequestPacket = ();
        type ScriptMessagePacket = ();
        type TransferPlayerPacket = ();
        type HurtArmorPacket = ();
        type SetActorMotionPacket = ();
        type SetLastHurtByPacket = ();
        type ClientToServerHandshakePacket = ();
        type MovementPredictionSyncPacket = ();
        type NetworkChunkPublisherUpdatePacket = ();
        type StartGamePacket = ();
        type PlayerFogPacket = ();
    }
    impl ProtoVersionTypes for Unknown {
        type NetworkBlockPosition = ();
        type BiomeReplacementData = ();
        type SubChunkPos = ();
        type ShapedChemistryRecipe = ();
        type ChunkPos = ();
        type MolangVariableMap = ();
        type NetworkItemStackDescriptor = ();
        type SpawnSettings = ();
        type BiomeCappedSurfaceData = ();
        type BiomeConsolidatedFeatureList = ();
        type ContainerMixDataEntry = ();
        type ItemStackResponseInfo = ();
        type CameraAimAssistPreset = ();
        type BiomeMountainParamsData = ();
        type DimensionDefinitionGroup = ();
        type ItemStackResponseSlotInfo = ();
        type GameRulesChangedPacketData = ();
        type NetworkItemInstanceDescriptor = ();
        type CameraAimAssistCategory = ();
        type MoveActorDeltaData = ();
        type ShulkerBoxRecipe = ();
        type MapItemTrackedActorUniqueID = ();
        type CameraAimAssistPriority = ();
        type InventoryTransaction = ();
        type CameraInstruction = ();
        type SmithingTransformRecipe = ();
        type BiomeLegacyWorldGenRulesData = ();
        type BiomeWeightedData = ();
        type RecipeUnlockingRequirement = ();
        type CameraAimAssistItemSettings = ();
        type BiomeWeightedTemperatureData = ();
        type EducationLevelSettings = ();
        type BaseGameVersion = ();
        type SerializedSkin = ();
        type Color = ();
        type CommandOriginData = ();
        type InventorySource = ();
        type SyncedPlayerMovementSettings = ();
        type PotionMixDataEntry = ();
        type RecipeIngredient = ();
        type ShapedRecipe = ();
        type BiomeSurfaceBuilderData = ();
        type LevelSettings = ();
        type DataItem = ();
        type MoveActorAbsoluteData = ();
        type MaterialReducerDataEntry = ();
        type SerializedAbilitiesData = ();
        type ShapelessChemistryRecipe = ();
        type BiomeCoordinateData = ();
        type ActorUniqueID = ();
        type PositionTrackingId = ();
        type AdventureSettings = ();
        type CameraPresets = ();
        type BiomeScatterParamData = ();
        type BlockPos = ();
        type BiomeElementData = ();
        type ItemStackResponseContainerInfo = ();
        type BiomeDefinitionChunkGenData = ();
        type BiomeMultinoiseGenRulesData = ();
        type BiomeSurfaceMaterialData = ();
        type PlayerBlockActionData = ();
        type ItemEnchants = ();
        type DebugShape = ();
        type CameraPreset = ();
        type ShapelessRecipe = ();
        type InventoryAction = ();
        type EntityNetID = ();
        type MapDecoration = ();
        type StructureSettings = ();
        type WebSocketPacketData = ();
        type FullContainerName = ();
        type BiomeClimateData = ();
        type SubChunkPosOffset = ();
        type ItemStackRequestSlotInfo = ();
        type BiomeMesaSurfaceData = ();
        type BiomeOverworldGenRulesData = ();
        type PackedItemUseLegacyInventoryTransaction = ();
        type CameraAimAssistPresetDefinition = ();
        type ActorRuntimeID = ();
        type BaseDescription = ();
        type CameraAimAssistCategories = ();
        type CraftingDataEntry = ();
        type ItemData = ();
        type StructureEditorData = ();
        type BiomeDefinition = ();
        type EduSharedUriResource = ();
        type ScoreboardId = ();
        type SmithingTrimRecipe = ();
        type CameraSplineInstruction = ();
        type ActorLink = ();
        type BiomeConditionalTransformationData = ();
        type BiomeNoiseGradientSurfaceData = ();
        type BiomeSurfaceMaterialAdjustmentData = ();
        type Experiments = ();
        type PropertySyncData = ();
        type NetworkPermissions = ();
    }
    impl ProtoVersionEnums for Unknown {
        type StructureTemplateResponseType = ();
        type ParticleType = ();
        type MolangVersion = ();
        type PlayerRespawnState = ();
        type Rotation = ();
        type BossEventUpdateType = ();
        type InteractionType = ();
        type ContainerID = ();
        type DataItemType = ();
        type GamePublishSetting = ();
        type SpawnPositionType = ();
        type ItemVersion = ();
        type ConnectionFailReason = ();
        type AttributeModifierOperation = ();
        type AgentActionType = ();
        type ChatRestrictionLevel = ();
        type MinecraftPacketIds = ();
        type ControlScheme = ();
        type BuildPlatform = ();
        type ItemUseInventoryTransactionType = ();
        type CameraSplineType = ();
        type CommandBlockMode = ();
        type StructureTemplateRequestOperation = ();
        type ActorLinkType = ();
        type PackType = ();
        type SpawnBiomeType = ();
        type SoftEnumUpdateType = ();
        type ItemReleaseInventoryTransactionType = ();
        type PacketViolationSeverity = ();
        type CodeBuilderStorageCategory = ();
        type ShowStoreOfferRedirectType = ();
        type StructureBlockType = ();
        type AnimationMode = ();
        type CommandOriginType = ();
        type Difficulty = ();
        type EditorWorldType = ();
        type LevelEvent = ();
        type CommandParameterOption = ();
        type LabTableReactionType = ();
        type PacketViolationType = ();
        type PredictionType = ();
        type GeneratorType = ();
        type ItemStackRequestActionType = ();
        type HudVisibility = ();
        type CodeBuilderCodeStatus = ();
        type AuthoritativeMovementMode = ();
        type CameraAimAssistOperation = ();
        type ComplexInventoryTransactionType = ();
        type EnchantType = ();
        type CameraShakeAction = ();
        type ItemStackNetResult = ();
        type CraftingDataEntryType = ();
        type HudElement = ();
        type ObjectiveSortOrder = ();
        type CommandPermissionLevel = ();
        type Mirror = ();
        type BookEditAction = ();
        type PlayerPositionMode = ();
        type AbilitiesIndex = ();
        type EducationEditionOffer = ();
        type ActorDataIDs = ();
        type ActorFlags = ();
        type ResourcePackResponse = ();
        type CodeBuilderStorageOperation = ();
        type TeleportationCause = ();
        type TextProcessingEventOrigin = ();
        type CameraSplineEaseType = ();
        type InventoryLayout = ();
        type AnimatedTextureType = ();
        type GameType = ();
        type StructureRedstoneSaveMode = ();
        type ItemUseOnActorInventoryTransactionType = ();
        type CameraShakeType = ();
        type EasingType = ();
        type MultiplayerSettingsPacketType = ();
        type UIProfile = ();
        type ActorBlockSyncMessageID = ();
        type InventoryRightTabIndex = ();
        type LessonAction = ();
        type ActorDamageCause = ();
        type LevelSoundEventType = ();
        type PlayerPermissionLevel = ();
        type TextPacketType = ();
        type ItemUseMethod = ();
        type ActorType = ();
        type InputMode = ();
        type POIBlockInteractionType = ();
        type PlayStatus = ();
        type InventoryLeftTabIndex = ();
        type ContainerType = ();
        type InventorySourceType = ();
        type MovementEffectType = ();
        type ItemDescriptorType = ();
        type AimAssistAction = ();
        type ModalFormCancelReason = ();
        type ActorEvent = ();
        type AttributeOperands = ();
        type ServerAuthMovementMode = ();
        type PhotoType = ();
        type InventorySourceFlags = ();
        type ContainerEnumName = ();
        type IdentityDefinitionType = ();
        type AnimationExpression = ();
        type CommandOutputType = ();
        type NewInteractionModel = ();
        type PacketCompressionAlgorithm = ();
        type SimulationType = ();
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
