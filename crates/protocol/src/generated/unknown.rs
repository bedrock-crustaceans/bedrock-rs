#[cfg(feature = "unknown")]
mod inner {
    use crate::ProtoVersion;
    use crate::ProtoVersionPackets;
    use crate::ProtoVersionTypes;
    use crate::ProtoVersionEnums;
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
                    header,
                    stream,
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
                Unknown::Unknown(pk) => {
                    stream
                        .write_all(pk.buf.as_ref())
                        .map_err(|e| bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: "Unknown",
                            packet_id: header.packet_id,
                            error: e.into(),
                        })?
                }
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
        type UpdateSoftEnumPacket = ();
        type AddPlayerPacket = ();
        type StructureDataRequestPacket = ();
        type LevelSoundEventV2Packet = ();
        type SetDifficultyPacket = ();
        type EmotePacket = ();
        type AwardAchievementPacket = ();
        type AddPaintingPacket = ();
        type ItemStackRequestPacket = ();
        type TextPacket = ();
        type OpenSignPacket = ();
        type ItemStackResponsePacket = ();
        type StopSoundPacket = ();
        type AnimateEntityPacket = ();
        type SetHealthPacket = ();
        type SetPlayerInventoryOptionsPacket = ();
        type ServerStatsPacket = ();
        type ServerPlayerPostMovePositionPacket = ();
        type LevelSoundEventPacket = ();
        type ServerBoundDiagnosticsPacket = ();
        type PlayerUpdateEntityOverridesPacket = ();
        type FeatureRegistryPacket = ();
        type DimensionDataPacket = ();
        type GameTestResultsPacket = ();
        type RequestChunkRadiusPacket = ();
        type ClientCacheMissResponsePacket = ();
        type TransferPlayerPacket = ();
        type ClientToServerHandshakePacket = ();
        type PurchaseReceiptPacket = ();
        type ShowCreditsPacket = ();
        type ContainerOpenPacket = ();
        type CameraPacket = ();
        type SetDefaultGameTypePacket = ();
        type CurrentStructureFeaturePacket = ();
        type CommandOutputPacket = ();
        type MapCreateLockedCopyPacket = ();
        type TickingAreaLoadStatusPacket = ();
        type ServerBoundPackSettingChangePacket = ();
        type CameraInstructionPacket = ();
        type ResourcePackChunkRequestPacket = ();
        type BookEditPacket = ();
        type InventoryContentPacket = ();
        type OnScreenTextureAnimationPacket = ();
        type MovementEffectPacket = ();
        type JigsawStructureDataPacket = ();
        type LocatorBarPacket = ();
        type RemoveVolumeEntityPacket = ();
        type ClientBoundDataDrivenUICloseAllScreensPacket = ();
        type SetTitlePacket = ();
        type BlockPickRequestPacket = ();
        type ClientCacheBlobStatusPacket = ();
        type LoginPacket = ();
        type SubChunkRequestPacket = ();
        type SetMovementAuthorityPacket = ();
        type ClientBoundDataDrivenUIShowScreenPacket = ();
        type UpdateTradePacket = ();
        type ServerBoundDataDrivenClosedPacket = ();
        type BossEventPacket = ();
        type MobEquipmentPacket = ();
        type MovementPredictionSyncPacket = ();
        type PhotoTransferPacket = ();
        type CodeBuilderPacket = ();
        type PassengerJumpPacket = ();
        type ContainerClosePacket = ();
        type StructureDataResponsePacket = ();
        type AnvilDamagePacket = ();
        type SetScorePacket = ();
        type TickSyncPacket = ();
        type ClientBoundDataStorePacket = ();
        type MultiplayerSettingsPacket = ();
        type UpdateBlockPacket = ();
        type SyncActorPropertyPacket = ();
        type UpdateAbilitiesPacket = ();
        type ChangeDimensionPacket = ();
        type UpdatePlayerGameTypePacket = ();
        type PositionTrackingDBClientRequestPacket = ();
        type ClientBoundDebugRendererPacket = ();
        type ModalFormRequestPacket = ();
        type ResourcePackDataInfoPacket = ();
        type StructureBlockUpdatePacket = ();
        type AddActorPacket = ();
        type LegacyTelemetryEventPacket = ();
        type GameTestRequestPacket = ();
        type LevelSoundEventV1Packet = ();
        type BlockActorDataPacket = ();
        type LessonProgressPacket = ();
        type NetworkSettingsPacket = ();
        type PlaySoundPacket = ();
        type SubClientLoginPacket = ();
        type CameraAimAssistActorPriorityPacket = ();
        type ClientBoundMapItemDataPacket = ();
        type ServerStoreInfoPacket = ();
        type ContainerSetDataPacket = ();
        type ServerSettingsResponsePacket = ();
        type SetHudPacket = ();
        type CompressedBiomeDefinitionListPacket = ();
        type SetLastHurtByPacket = ();
        type EduUriResourcePacket = ();
        type ItemComponentPacket = ();
        type PlayerFogPacket = ();
        type NetworkStackLatencyPacket = ();
        type PlayerListPacket = ();
        type CameraShakePacket = ();
        type UnlockedRecipesPacket = ();
        type PlayerArmorDamagePacket = ();
        type SetPlayerGameTypePacket = ();
        type InteractPacket = ();
        type AvailableActorIdentifiersPacket = ();
        type CorrectPlayerMovePredictionPacket = ();
        type MoveActorDeltaPacket = ();
        type StartGamePacket = ();
        type RemoveObjectivePacket = ();
        type RequestPermissionsPacket = ();
        type FilterTextPacket = ();
        type ShowStoreOfferPacket = ();
        type TakeItemActorPacket = ();
        type ToastRequestPacket = ();
        type PlayerHotbarPacket = ();
        type SetSpawnPositionPacket = ();
        type UpdateAttributesPacket = ();
        type ServerPresenceInfoPacket = ();
        type ClientBoundAttributeLayerSyncPacket = ();
        type MoveActorAbsolutePacket = ();
        type RequestAbilityPacket = ();
        type DisconnectPacket = ();
        type AvailableCommandsPacket = ();
        type RemoveActorPacket = ();
        type PlayerSkinPacket = ();
        type MobEffectPacket = ();
        type ActorPickRequestPacket = ();
        type ClientBoundCloseFormPacket = ();
        type RespawnPacket = ();
        type NpcRequestPacket = ();
        type EducationSettingsPacket = ();
        type CameraAimAssistPresetsPacket = ();
        type ResourcePackClientResponsePacket = ();
        type PlayerEnchantOptionsPacket = ();
        type UpdateAdventureSettingsPacket = ();
        type CameraSplinePacket = ();
        type ActorEventPacket = ();
        type ContainerRegistryCleanupPacket = ();
        type CameraPresetsPacket = ();
        type GraphicsParameterOverridePacket = ();
        type PlayerVideoCapturePacket = ();
        type ScriptMessagePacket = ();
        type SetLocalPlayerAsInitializedPacket = ();
        type MovePlayerPacket = ();
        type AddVolumeEntityPacket = ();
        type HurtArmorPacket = ();
        type PacketViolationWarningPacket = ();
        type ResourcePacksInfoPacket = ();
        type SetScoreboardIdentityPacket = ();
        type PlayStatusPacket = ();
        type ServerToClientHandshakePacket = ();
        type PlayerStartItemCooldownPacket = ();
        type SetCommandsEnabledPacket = ();
        type AgentActionEventPacket = ();
        type CameraAimAssistPacket = ();
        type ServerBoundLoadingScreenPacket = ();
        type PlayerLocationPacket = ();
        type LevelChunkPacket = ();
        type AnimatePacket = ();
        type UpdateEquipPacket = ();
        type CreatePhotoPacket = ();
        type PlayerToggleCrafterSlotRequestPacket = ();
        type PartyChangedPacket = ();
        type NpcDialoguePacket = ();
        type SetTimePacket = ();
        type NetworkChunkPublisherUpdatePacket = ();
        type SettingsCommandPacket = ();
        type SetActorLinkPacket = ();
        type UpdateBlockSyncedPacket = ();
        type ResourcePackChunkDataPacket = ();
        type ModalFormResponsePacket = ();
        type CompletedUsingItemPacket = ();
        type ChunkRadiusUpdatedPacket = ();
        type InventorySlotPacket = ();
        type UpdateClientOptionsPacket = ();
        type ChangeMobPropertyPacket = ();
        type SimpleEventPacket = ();
        type VoxelShapesPacket = ();
        type UpdateClientInputLocksPacket = ();
        type CreativeContentPacket = ();
        type UpdateSubChunkBlocksPacket = ();
        type AddBehaviourTreePacket = ();
        type BlockEventPacket = ();
        type LevelEventPacket = ();
        type AgentAnimationPacket = ();
        type GameRulesChangedPacket = ();
        type MobArmorEquipmentPacket = ();
        type MotionPredictionHintsPacket = ();
        type SubChunkPacket = ();
        type PlayerAuthInputPacket = ();
        type ResourcePackStackPacket = ();
        type SetActorDataPacket = ();
        type GuiDataPickItemPacket = ();
        type PlayerInputPacket = ();
        type SetActorMotionPacket = ();
        type CommandRequestPacket = ();
        type ClientBoundDataDrivenUICloseScreenPacket = ();
        type LabTablePacket = ();
        type LecternUpdatePacket = ();
        type ClientCacheStatusPacket = ();
        type ClientBoundDataDrivenUIReloadPacket = ();
        type PlayerActionPacket = ();
        type AddItemActorPacket = ();
        type AutomationClientConnectPacket = ();
        type ClientBoundControlSchemeSetPacket = ();
        type SyncWorldClocksPacket = ();
        type EmoteListPacket = ();
        type PositionTrackingDBServerBroadcastPacket = ();
        type DebugDrawerPacket = ();
        type LevelEventGenericPacket = ();
        type RequestNetworkSettingsPacket = crate::version::unknown::packets::RequestNetworkSettingsPacket;
        type CodeBuilderSourcePacket = ();
        type MapInfoRequestPacket = ();
        type TrimDataPacket = ();
        type CameraAimAssistInstructionPacket = ();
        type DeathInfoPacket = ();
        type EditorNetworkPacket = ();
        type ResourcePacksReadyForValidationPacket = ();
        type DebugInfoPacket = ();
        type SimulationTypePacket = ();
        type SpawnParticleEffectPacket = ();
        type SpawnExperienceOrbPacket = ();
        type SetDisplayObjectivePacket = ();
        type ClientBoundTextureShiftPacket = ();
        type CommandBlockUpdatePacket = ();
        type BiomeDefinitionListPacket = ();
        type ShowProfilePacket = ();
        type CraftingDataPacket = ();
        type ServerBoundDataStorePacket = ();
        type InventoryTransactionPacket = ();
        type RefreshEntitlementsPacket = ();
        type ServerSettingsRequestPacket = ();
    }
    impl ProtoVersionTypes for Unknown {
        type InventorySource = ();
        type ShapedRecipe = ();
        type CameraPresets = ();
        type DataItem = ();
        type PropertySyncData = ();
        type PositionTrackingId = ();
        type MolangVariableMap = ();
        type NetworkPermissions = ();
        type ItemData = ();
        type InventoryAction = ();
        type RecipeIngredient = ();
        type ScoreboardId = ();
        type SpawnSettings = ();
        type StructureSettings = ();
        type ShapelessChemistryRecipe = ();
        type MoveActorAbsoluteData = ();
        type SubChunkPosOffset = ();
        type WebSocketPacketData = ();
        type CameraAimAssistPresetDefinition = ();
        type FullContainerName = ();
        type BiomeConditionalTransformationData = ();
        type BiomeConsolidatedFeatureList = ();
        type BiomeCoordinateData = ();
        type RecipeUnlockingRequirement = ();
        type ItemStackResponseSlotInfo = ();
        type BiomeMultinoiseGenRulesData = ();
        type BiomeSurfaceMaterialAdjustmentData = ();
        type ActorUniqueID = ();
        type ChunkPos = ();
        type ShapelessRecipe = ();
        type PotionMixDataEntry = ();
        type BiomeSurfaceMaterialData = ();
        type CameraAimAssistPreset = ();
        type SmithingTransformRecipe = ();
        type StructureEditorData = ();
        type MapDecoration = ();
        type BiomeDefinitionChunkGenData = ();
        type ItemStackResponseContainerInfo = ();
        type BiomeOverworldGenRulesData = ();
        type BiomeScatterParamData = ();
        type NetworkBlockPosition = ();
        type BiomeMesaSurfaceData = ();
        type EducationLevelSettings = ();
        type BlockPos = ();
        type CameraInstruction = ();
        type SmithingTrimRecipe = ();
        type ItemStackResponseInfo = ();
        type PlayerBlockActionData = ();
        type ItemEnchants = ();
        type CameraSplineInstruction = ();
        type Color = ();
        type ActorRuntimeID = ();
        type InventoryTransaction = ();
        type CraftingDataEntry = ();
        type NetworkItemInstanceDescriptor = ();
        type ShulkerBoxRecipe = ();
        type CameraAimAssistItemSettings = ();
        type GameRulesChangedPacketData = ();
        type SerializedSkin = ();
        type BiomeWeightedTemperatureData = ();
        type Experiments = ();
        type BiomeNoiseGradientSurfaceData = ();
        type LevelSettings = ();
        type BaseGameVersion = ();
        type CameraPreset = ();
        type SubChunkPos = ();
        type PackedItemUseLegacyInventoryTransaction = ();
        type ContainerMixDataEntry = ();
        type SyncedPlayerMovementSettings = ();
        type BiomeLegacyWorldGenRulesData = ();
        type ShapedChemistryRecipe = ();
        type EduSharedUriResource = ();
        type AdventureSettings = ();
        type SerializedAbilitiesData = ();
        type MoveActorDeltaData = ();
        type CameraAimAssistCategories = ();
        type CameraAimAssistCategory = ();
        type BiomeElementData = ();
        type CameraAimAssistPriority = ();
        type BiomeCappedSurfaceData = ();
        type DimensionDefinitionGroup = ();
        type MaterialReducerDataEntry = ();
        type MapItemTrackedActorUniqueID = ();
        type BiomeMountainParamsData = ();
        type BaseDescription = ();
        type BiomeDefinition = ();
        type BiomeSurfaceBuilderData = ();
        type ItemStackRequestSlotInfo = ();
        type ActorLink = ();
        type NetworkItemStackDescriptor = ();
        type CommandOriginData = ();
        type EntityNetID = ();
        type BiomeClimateData = ();
        type DebugShape = ();
        type BiomeReplacementData = ();
        type BiomeWeightedData = ();
    }
    impl ProtoVersionEnums for Unknown {
        type HudVisibility = ();
        type Difficulty = ();
        type PlayerPositionMode = ();
        type InputMode = ();
        type ItemStackNetResult = ();
        type ModalFormCancelReason = ();
        type StructureRedstoneSaveMode = ();
        type LevelEvent = ();
        type InteractionType = ();
        type ComplexInventoryTransactionType = ();
        type InventoryLeftTabIndex = ();
        type ItemStackRequestActionType = ();
        type Mirror = ();
        type POIBlockInteractionType = ();
        type TextProcessingEventOrigin = ();
        type ActorLinkType = ();
        type LabTableReactionType = ();
        type SoftEnumUpdateType = ();
        type BuildPlatform = ();
        type CameraSplineType = ();
        type SpawnBiomeType = ();
        type ActorType = ();
        type ChatRestrictionLevel = ();
        type EducationEditionOffer = ();
        type EnchantType = ();
        type ItemReleaseInventoryTransactionType = ();
        type PacketCompressionAlgorithm = ();
        type ContainerType = ();
        type AgentActionType = ();
        type BossEventUpdateType = ();
        type PhotoType = ();
        type EditorWorldType = ();
        type DataItemType = ();
        type InventoryRightTabIndex = ();
        type UIProfile = ();
        type AbilitiesIndex = ();
        type AnimationExpression = ();
        type PlayerRespawnState = ();
        type ShowStoreOfferRedirectType = ();
        type StructureBlockType = ();
        type CameraSplineEaseType = ();
        type ActorEvent = ();
        type HudElement = ();
        type PredictionType = ();
        type Rotation = ();
        type TextPacketType = ();
        type ItemDescriptorType = ();
        type AttributeOperands = ();
        type CodeBuilderStorageCategory = ();
        type CommandOriginType = ();
        type EasingType = ();
        type CommandParameterOption = ();
        type InventoryLayout = ();
        type LessonAction = ();
        type ObjectiveSortOrder = ();
        type SpawnPositionType = ();
        type ItemUseOnActorInventoryTransactionType = ();
        type StructureTemplateRequestOperation = ();
        type StructureTemplateResponseType = ();
        type GameType = ();
        type CameraAimAssistOperation = ();
        type AnimatedTextureType = ();
        type ActorDamageCause = ();
        type AttributeModifierOperation = ();
        type InventorySourceType = ();
        type ActorFlags = ();
        type SimulationType = ();
        type ActorBlockSyncMessageID = ();
        type CameraShakeType = ();
        type ContainerEnumName = ();
        type ContainerID = ();
        type CommandOutputType = ();
        type CodeBuilderStorageOperation = ();
        type LevelSoundEventType = ();
        type MinecraftPacketIds = ();
        type PackType = ();
        type PacketViolationType = ();
        type TeleportationCause = ();
        type CodeBuilderCodeStatus = ();
        type ItemUseInventoryTransactionType = ();
        type MovementEffectType = ();
        type ItemVersion = ();
        type AuthoritativeMovementMode = ();
        type GeneratorType = ();
        type CraftingDataEntryType = ();
        type CommandBlockMode = ();
        type MolangVersion = ();
        type MultiplayerSettingsPacketType = ();
        type IdentityDefinitionType = ();
        type NewInteractionModel = ();
        type ResourcePackResponse = ();
        type AimAssistAction = ();
        type ControlScheme = ();
        type AnimationMode = ();
        type CommandPermissionLevel = ();
        type ActorDataIDs = ();
        type PacketViolationSeverity = ();
        type ParticleType = ();
        type PlayerPermissionLevel = ();
        type CameraShakeAction = ();
        type BookEditAction = ();
        type ServerAuthMovementMode = ();
        type PlayStatus = ();
        type ItemUseMethod = ();
        type ConnectionFailReason = ();
        type InventorySourceFlags = ();
        type GamePublishSetting = ();
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
