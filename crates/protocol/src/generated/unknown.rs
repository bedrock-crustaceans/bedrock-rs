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
        fn id(&self) -> u16 {
            match self {
                Unknown::RequestNetworkSettingsPacket(_) => {
                    <<Unknown as ProtoVersionPackets>::RequestNetworkSettingsPacket as bedrock_protocol_core::Packet>::ID
                }
                Unknown::Unknown(pk) => pk.id,
            }
        }
    }
    impl ProtoVersionPackets for Unknown {
        type ActorEventPacket = ();
        type ActorPickRequestPacket = ();
        type AddActorPacket = ();
        type AddBehaviourTreePacket = ();
        type AddItemActorPacket = ();
        type AddPaintingPacket = ();
        type AddPlayerPacket = ();
        type AddVolumeEntityPacket = ();
        type AgentActionEventPacket = ();
        type AgentAnimationPacket = ();
        type AnimateEntityPacket = ();
        type AnimatePacket = ();
        type AnvilDamagePacket = ();
        type AutomationClientConnectPacket = ();
        type AvailableActorIdentifiersPacket = ();
        type AvailableCommandsPacket = ();
        type AwardAchievementPacket = ();
        type BiomeDefinitionListPacket = ();
        type BlockActorDataPacket = ();
        type BlockEventPacket = ();
        type BlockPickRequestPacket = ();
        type BookEditPacket = ();
        type BossEventPacket = ();
        type CameraAimAssistActorPriorityPacket = ();
        type CameraAimAssistInstructionPacket = ();
        type CameraAimAssistPacket = ();
        type CameraAimAssistPresetsPacket = ();
        type CameraInstructionPacket = ();
        type CameraPacket = ();
        type CameraPresetsPacket = ();
        type CameraShakePacket = ();
        type CameraSplinePacket = ();
        type ChangeDimensionPacket = ();
        type ChangeMobPropertyPacket = ();
        type ChunkRadiusUpdatedPacket = ();
        type ClientBoundAttributeLayerSyncPacket = ();
        type ClientBoundCloseFormPacket = ();
        type ClientBoundControlSchemeSetPacket = ();
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket = ();
        type ClientBoundDataDrivenUIReloadPacket = ();
        type ClientBoundDataDrivenUIShowScreenPacket = ();
        type ClientBoundDataStorePacket = ();
        type ClientBoundDebugRendererPacket = ();
        type ClientBoundMapItemDataPacket = ();
        type ClientBoundTextureShiftPacket = ();
        type ClientCacheBlobStatusPacket = ();
        type ClientCacheMissResponsePacket = ();
        type ClientCacheStatusPacket = ();
        type ClientToServerHandshakePacket = ();
        type CodeBuilderPacket = ();
        type CodeBuilderSourcePacket = ();
        type CommandBlockUpdatePacket = ();
        type CommandOutputPacket = ();
        type CommandRequestPacket = ();
        type CompletedUsingItemPacket = ();
        type CompressedBiomeDefinitionListPacket = ();
        type ContainerClosePacket = ();
        type ContainerOpenPacket = ();
        type ContainerRegistryCleanupPacket = ();
        type ContainerSetDataPacket = ();
        type CorrectPlayerMovePredictionPacket = ();
        type CraftingDataPacket = ();
        type CreatePhotoPacket = ();
        type CreativeContentPacket = ();
        type CurrentStructureFeaturePacket = ();
        type DeathInfoPacket = ();
        type DebugDrawerPacket = ();
        type DebugInfoPacket = ();
        type DimensionDataPacket = ();
        type DisconnectPacket = ();
        type EditorNetworkPacket = ();
        type EduUriResourcePacket = ();
        type EducationSettingsPacket = ();
        type EmoteListPacket = ();
        type EmotePacket = ();
        type FeatureRegistryPacket = ();
        type FilterTextPacket = ();
        type GameRulesChangedPacket = ();
        type GameTestRequestPacket = ();
        type GameTestResultsPacket = ();
        type GraphicsParameterOverridePacket = ();
        type GuiDataPickItemPacket = ();
        type HurtArmorPacket = ();
        type InteractPacket = ();
        type InventoryContentPacket = ();
        type InventorySlotPacket = ();
        type InventoryTransactionPacket = ();
        type ItemComponentPacket = ();
        type ItemStackRequestPacket = ();
        type ItemStackResponsePacket = ();
        type JigsawStructureDataPacket = ();
        type LabTablePacket = ();
        type LecternUpdatePacket = ();
        type LegacyTelemetryEventPacket = ();
        type LessonProgressPacket = ();
        type LevelChunkPacket = ();
        type LevelEventGenericPacket = ();
        type LevelEventPacket = ();
        type LevelSoundEventPacket = ();
        type LevelSoundEventV1Packet = ();
        type LevelSoundEventV2Packet = ();
        type LocatorBarPacket = ();
        type LoginPacket = ();
        type MapCreateLockedCopyPacket = ();
        type MapInfoRequestPacket = ();
        type MobArmorEquipmentPacket = ();
        type MobEffectPacket = ();
        type MobEquipmentPacket = ();
        type ModalFormRequestPacket = ();
        type ModalFormResponsePacket = ();
        type MotionPredictionHintsPacket = ();
        type MoveActorAbsolutePacket = ();
        type MoveActorDeltaPacket = ();
        type MovePlayerPacket = ();
        type MovementEffectPacket = ();
        type MovementPredictionSyncPacket = ();
        type MultiplayerSettingsPacket = ();
        type NetworkChunkPublisherUpdatePacket = ();
        type NetworkSettingsPacket = ();
        type NetworkStackLatencyPacket = ();
        type NpcDialoguePacket = ();
        type NpcRequestPacket = ();
        type OnScreenTextureAnimationPacket = ();
        type OpenSignPacket = ();
        type PacketViolationWarningPacket = ();
        type PartyChangedPacket = ();
        type PassengerJumpPacket = ();
        type PhotoTransferPacket = ();
        type PlaySoundPacket = ();
        type PlayStatusPacket = ();
        type PlayerActionPacket = ();
        type PlayerArmorDamagePacket = ();
        type PlayerAuthInputPacket = ();
        type PlayerEnchantOptionsPacket = ();
        type PlayerFogPacket = ();
        type PlayerHotbarPacket = ();
        type PlayerInputPacket = ();
        type PlayerListPacket = ();
        type PlayerLocationPacket = ();
        type PlayerSkinPacket = ();
        type PlayerStartItemCooldownPacket = ();
        type PlayerToggleCrafterSlotRequestPacket = ();
        type PlayerUpdateEntityOverridesPacket = ();
        type PlayerVideoCapturePacket = ();
        type PositionTrackingDBClientRequestPacket = ();
        type PositionTrackingDBServerBroadcastPacket = ();
        type PurchaseReceiptPacket = ();
        type RefreshEntitlementsPacket = ();
        type RemoveActorPacket = ();
        type RemoveObjectivePacket = ();
        type RemoveVolumeEntityPacket = ();
        type RequestAbilityPacket = ();
        type RequestChunkRadiusPacket = ();
        type RequestNetworkSettingsPacket =
            crate::version::unknown::packets::RequestNetworkSettingsPacket;
        type RequestPermissionsPacket = ();
        type ResourcePackChunkDataPacket = ();
        type ResourcePackChunkRequestPacket = ();
        type ResourcePackClientResponsePacket = ();
        type ResourcePackDataInfoPacket = ();
        type ResourcePackStackPacket = ();
        type ResourcePacksInfoPacket = ();
        type ResourcePacksReadyForValidationPacket = ();
        type RespawnPacket = ();
        type ScriptMessagePacket = ();
        type ServerBoundDataDrivenClosedPacket = ();
        type ServerBoundDataStorePacket = ();
        type ServerBoundDiagnosticsPacket = ();
        type ServerBoundLoadingScreenPacket = ();
        type ServerBoundPackSettingChangePacket = ();
        type ServerPlayerPostMovePositionPacket = ();
        type ServerPresenceInfoPacket = ();
        type ServerSettingsRequestPacket = ();
        type ServerSettingsResponsePacket = ();
        type ServerStatsPacket = ();
        type ServerStoreInfoPacket = ();
        type ServerToClientHandshakePacket = ();
        type SetActorDataPacket = ();
        type SetActorLinkPacket = ();
        type SetActorMotionPacket = ();
        type SetCommandsEnabledPacket = ();
        type SetDefaultGameTypePacket = ();
        type SetDifficultyPacket = ();
        type SetDisplayObjectivePacket = ();
        type SetHealthPacket = ();
        type SetHudPacket = ();
        type SetLastHurtByPacket = ();
        type SetLocalPlayerAsInitializedPacket = ();
        type SetMovementAuthorityPacket = ();
        type SetPlayerGameTypePacket = ();
        type SetPlayerInventoryOptionsPacket = ();
        type SetScorePacket = ();
        type SetScoreboardIdentityPacket = ();
        type SetSpawnPositionPacket = ();
        type SetTimePacket = ();
        type SetTitlePacket = ();
        type SettingsCommandPacket = ();
        type ShowCreditsPacket = ();
        type ShowProfilePacket = ();
        type ShowStoreOfferPacket = ();
        type SimpleEventPacket = ();
        type SimulationTypePacket = ();
        type SpawnExperienceOrbPacket = ();
        type SpawnParticleEffectPacket = ();
        type StartGamePacket = ();
        type StopSoundPacket = ();
        type StructureBlockUpdatePacket = ();
        type StructureDataRequestPacket = ();
        type StructureDataResponsePacket = ();
        type SubChunkPacket = ();
        type SubChunkRequestPacket = ();
        type SubClientLoginPacket = ();
        type SyncActorPropertyPacket = ();
        type SyncWorldClocksPacket = ();
        type TakeItemActorPacket = ();
        type TextPacket = ();
        type TickSyncPacket = ();
        type TickingAreaLoadStatusPacket = ();
        type ToastRequestPacket = ();
        type TransferPlayerPacket = ();
        type TrimDataPacket = ();
        type UnlockedRecipesPacket = ();
        type UpdateAbilitiesPacket = ();
        type UpdateAdventureSettingsPacket = ();
        type UpdateAttributesPacket = ();
        type UpdateBlockPacket = ();
        type UpdateBlockSyncedPacket = ();
        type UpdateClientInputLocksPacket = ();
        type UpdateClientOptionsPacket = ();
        type UpdateEquipPacket = ();
        type UpdatePlayerGameTypePacket = ();
        type UpdateSoftEnumPacket = ();
        type UpdateSubChunkBlocksPacket = ();
        type UpdateTradePacket = ();
        type VoxelShapesPacket = ();
    }
    impl ProtoVersionTypes for Unknown {
        type ActorLink = ();
        type ActorRuntimeID = ();
        type ActorUniqueID = ();
        type AdventureSettings = ();
        type BaseDescription = ();
        type BaseGameVersion = ();
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
        type BlockPos = ();
        type CameraAimAssistCategories = ();
        type CameraAimAssistCategory = ();
        type CameraAimAssistItemSettings = ();
        type CameraAimAssistPreset = ();
        type CameraAimAssistPresetDefinition = ();
        type CameraAimAssistPriority = ();
        type CameraInstruction = ();
        type CameraPreset = ();
        type CameraPresets = ();
        type CameraSplineInstruction = ();
        type ChunkPos = ();
        type Color = ();
        type CommandOriginData = ();
        type ContainerMixDataEntry = ();
        type CraftingDataEntry = ();
        type DataItem = ();
        type DebugShape = ();
        type DimensionDefinitionGroup = ();
        type EduSharedUriResource = ();
        type EducationLevelSettings = ();
        type EntityNetID = ();
        type Experiments = ();
        type FullContainerName = ();
        type GameRulesChangedPacketData = ();
        type InventoryAction = ();
        type InventorySource = ();
        type InventoryTransaction = ();
        type ItemData = ();
        type ItemEnchants = ();
        type ItemStackRequestSlotInfo = ();
        type ItemStackResponseContainerInfo = ();
        type ItemStackResponseInfo = ();
        type ItemStackResponseSlotInfo = ();
        type LevelSettings = ();
        type MapDecoration = ();
        type MapItemTrackedActorUniqueID = ();
        type MaterialReducerDataEntry = ();
        type MolangVariableMap = ();
        type MoveActorAbsoluteData = ();
        type MoveActorDeltaData = ();
        type NetworkBlockPosition = ();
        type NetworkItemInstanceDescriptor = ();
        type NetworkItemStackDescriptor = ();
        type NetworkPermissions = ();
        type PackedItemUseLegacyInventoryTransaction = ();
        type PlayerBlockActionData = ();
        type PositionTrackingId = ();
        type PotionMixDataEntry = ();
        type PropertySyncData = ();
        type RecipeIngredient = ();
        type RecipeUnlockingRequirement = ();
        type ScoreboardId = ();
        type SerializedAbilitiesData = ();
        type SerializedSkin = ();
        type ShapedRecipe = ();
        type ShapelessRecipe = ();
        type ShulkerBoxRecipe = ();
        type SmithingTransformRecipe = ();
        type SmithingTrimRecipe = ();
        type SpawnSettings = ();
        type StructureEditorData = ();
        type StructureSettings = ();
        type SubChunkPos = ();
        type SubChunkPosOffset = ();
        type SyncedPlayerMovementSettings = ();
        type WebSocketPacketData = ();
    }
    impl ProtoVersionEnums for Unknown {
        type AbilitiesIndex = ();
        type ActorBlockSyncMessageID = ();
        type ActorDamageCause = ();
        type ActorDataIDs = ();
        type ActorEvent = ();
        type ActorFlags = ();
        type ActorLinkType = ();
        type ActorType = ();
        type AgentActionType = ();
        type AimAssistAction = ();
        type AnimatedTextureType = ();
        type AnimationExpression = ();
        type AnimationMode = ();
        type AttributeModifierOperation = ();
        type AttributeOperands = ();
        type AuthoritativeMovementMode = ();
        type BookEditAction = ();
        type BossEventUpdateType = ();
        type BuildPlatform = ();
        type CameraAimAssistOperation = ();
        type CameraShakeAction = ();
        type CameraShakeType = ();
        type CameraSplineEaseType = ();
        type CameraSplineType = ();
        type ChatRestrictionLevel = ();
        type CodeBuilderCodeStatus = ();
        type CodeBuilderStorageCategory = ();
        type CodeBuilderStorageOperation = ();
        type CommandBlockMode = ();
        type CommandOriginType = ();
        type CommandOutputType = ();
        type CommandParameterOption = ();
        type CommandPermissionLevel = ();
        type ComplexInventoryTransactionType = ();
        type ConnectionFailReason = ();
        type ContainerEnumName = ();
        type ContainerID = ();
        type ContainerType = ();
        type ControlScheme = ();
        type CraftingDataEntryType = ();
        type DataItemType = ();
        type Difficulty = ();
        type EasingType = ();
        type EditorWorldType = ();
        type EducationEditionOffer = ();
        type EnchantType = ();
        type GamePublishSetting = ();
        type GameType = ();
        type GeneratorType = ();
        type HudElement = ();
        type HudVisibility = ();
        type IdentityDefinitionType = ();
        type InputMode = ();
        type InteractionType = ();
        type InventoryLayout = ();
        type InventoryLeftTabIndex = ();
        type InventoryRightTabIndex = ();
        type InventorySourceFlags = ();
        type InventorySourceType = ();
        type ItemDescriptorType = ();
        type ItemReleaseInventoryTransactionType = ();
        type ItemStackNetResult = ();
        type ItemStackRequestActionType = ();
        type ItemUseInventoryTransactionType = ();
        type ItemUseMethod = ();
        type ItemUseOnActorInventoryTransactionType = ();
        type ItemVersion = ();
        type LabTableReactionType = ();
        type LessonAction = ();
        type LevelEvent = ();
        type LevelSoundEventType = ();
        type MinecraftPacketIds = ();
        type Mirror = ();
        type ModalFormCancelReason = ();
        type MolangVersion = ();
        type MovementEffectType = ();
        type MultiplayerSettingsPacketType = ();
        type NewInteractionModel = ();
        type ObjectiveSortOrder = ();
        type POIBlockInteractionType = ();
        type PackType = ();
        type PacketCompressionAlgorithm = ();
        type PacketViolationSeverity = ();
        type PacketViolationType = ();
        type ParticleType = ();
        type PhotoType = ();
        type PlayStatus = ();
        type PlayerPermissionLevel = ();
        type PlayerPositionMode = ();
        type PlayerRespawnState = ();
        type PredictionType = ();
        type ResourcePackResponse = ();
        type Rotation = ();
        type ServerAuthMovementMode = ();
        type ShowStoreOfferRedirectType = ();
        type SimulationType = ();
        type SoftEnumUpdateType = ();
        type SpawnBiomeType = ();
        type SpawnPositionType = ();
        type StructureBlockType = ();
        type StructureRedstoneSaveMode = ();
        type StructureTemplateRequestOperation = ();
        type StructureTemplateResponseType = ();
        type TeleportationCause = ();
        type TextPacketType = ();
        type TextProcessingEventOrigin = ();
        type UIProfile = ();
    }
    impl ProtoVersion for Unknown {
        const PROTOCOL_VERSION: u32 = 0u32;
        const PROTOCOL_BRANCH: &str = "r/0_u0";
        const GAME_VERSION: &str = "0.0.0";
        const RAKNET_VERSION: u8 = 10u8;
    }
    #[cfg(feature = "packet-dyn")]
    impl AsRef<dyn bedrock_protocol_core::PacketDyn> for Unknown {
        fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
            match self {
                Unknown::RequestNetworkSettingsPacket(pk) => pk.as_ref(),
                Unknown::Unknown(pk) => pk.as_ref(),
            }
        }
    }
    #[cfg(feature = "packet-dyn")]
    impl From<Unknown> for Box<dyn bedrock_protocol_core::PacketDyn> {
        fn from(val: Unknown) -> Box<dyn bedrock_protocol_core::PacketDyn> {
            match val {
                Unknown::RequestNetworkSettingsPacket(pk) => pk,
                Unknown::Unknown(pk) => pk,
            }
        }
    }
}
#[cfg(feature = "unknown")]
pub use inner::*;
