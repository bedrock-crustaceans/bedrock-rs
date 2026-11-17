pub trait ProtoVersionPackets {
    type ActorEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorPickRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddActorPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddBehaviourTreePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddItemActorPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddPaintingPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddPlayerPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddVolumeEntityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AgentActionEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AgentAnimationPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimateEntityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimatePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnvilDamagePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AutomationClientConnectPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AvailableActorIdentifiersPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AvailableCommandsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AwardAchievementPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeDefinitionListPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockActorDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockPickRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BookEditPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BossEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistActorPriorityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistInstructionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPresetsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraInstructionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPresetsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraShakePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplinePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChangeDimensionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChangeMobPropertyPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChunkRadiusUpdatedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundAttributeLayerSyncPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundCloseFormPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundControlSchemeSetPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUICloseAllScreensPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUICloseScreenPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUIReloadPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUIShowScreenPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataStorePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDebugRendererPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundMapItemDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundMatchmakingStatePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundPlayAudioContentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundStonecutterSetRecipePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundTextureShiftPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundUpdateSoundDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientCacheBlobStatusPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientCacheMissResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientCacheStatusPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientToServerHandshakePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderSourcePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandBlockUpdatePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandOutputPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CompletedUsingItemPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CompressedBiomeDefinitionListPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerClosePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerOpenPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerRegistryCleanupPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerSetDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CorrectPlayerMovePredictionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CreatePhotoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CreativeContentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CurrentStructureFeaturePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DeathInfoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DebugDrawerPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DebugInfoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DimensionDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DisconnectPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EditorNetworkPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EduUriResourcePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EducationSettingsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EmoteListPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EmotePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type FeatureRegistryPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type FilterTextPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameRulesChangedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameTestRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameTestResultsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GraphicsParameterOverridePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GuiDataPickItemPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type HurtArmorPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InteractPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryContentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySlotPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryTransactionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemComponentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type JigsawStructureDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LabTablePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LecternUpdatePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LegacyTelemetryEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LessonProgressPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelChunkPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelEventGenericPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventV1Packet: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventV2Packet: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LocatorBarPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LoginPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapCreateLockedCopyPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapInfoRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MobArmorEquipmentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MobEffectPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MobEquipmentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ModalFormRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ModalFormResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MotionPredictionHintsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorAbsolutePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorDeltaPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovePlayerPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovementEffectPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovementPredictionSyncPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MultiplayerSettingsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkChunkPublisherUpdatePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkSettingsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkStackLatencyPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NpcDialoguePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NpcRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type OnScreenTextureAnimationPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type OpenSignPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketViolationWarningPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PartyChangedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PartyDestinationCookieResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PassengerJumpPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PhotoTransferPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlaySoundPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayStatusPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerActionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerArmorDamagePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerAuthInputPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerEnchantOptionsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerFogPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerHotbarPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerInputPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerListPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerLocationPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerSkinPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerStartItemCooldownPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerToggleCrafterSlotRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerUpdateEntityOverridesPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerVideoCapturePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PositionTrackingDBClientRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PositionTrackingDBServerBroadcastPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PurchaseReceiptPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RecordStartedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RefreshEntitlementsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RemoveActorPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RemoveObjectivePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RemoveVolumeEntityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestAbilityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestChunkRadiusPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestNetworkSettingsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestPermissionsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackChunkDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackChunkRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackClientResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackDataInfoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackStackPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePacksInfoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePacksReadyForValidationPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RespawnPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ScriptMessagePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SendPartyDestinationCookiePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundCursorItemDragPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundDataDrivenClosedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundDataStorePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundDiagnosticsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundLoadingScreenPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundMatchmakingCancelPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundPackSettingChangePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundRegisterAudioContentPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundStonecutterSetRecipePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerPlayerPostMovePositionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerPresenceInfoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerSettingsRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerSettingsResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerStatsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerStoreInfoPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerToClientHandshakePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetActorDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetActorLinkPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetActorMotionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetCommandsEnabledPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetDefaultGameTypePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetDifficultyPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetDisplayObjectivePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetHealthPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetHudPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetLastHurtByPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetLocalPlayerAsInitializedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetMovementAuthorityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetPassengerOfBlockPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetPlayerFurnaceOptionsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetPlayerGameTypePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetPlayerInventoryOptionsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetScorePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetScoreboardIdentityPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetSpawnPositionPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetTimePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetTitlePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SettingsCommandPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowCreditsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowProfilePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowStoreOfferPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SimpleEventPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SimulationTypePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnExperienceOrbPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnParticleEffectPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StartGamePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StopSoundPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureBlockUpdatePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureDataRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureDataResponsePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubClientLoginPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SyncActorPropertyPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SyncWorldClocksPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TakeItemActorPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TextPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TickSyncPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TickingAreaLoadStatusPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ToastRequestPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TransferPlayerPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TrimDataPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UnlockedRecipesPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateAbilitiesPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateAdventureSettingsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateAttributesPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateBlockPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateBlockSyncedPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateClientInputLocksPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateClientOptionsPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateEquipPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdatePlayerGameTypePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateSoftEnumPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateSubChunkBlocksPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateTradePacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type VoxelShapesPacket: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
}
pub trait ProtoVersionTypes {
    type ActorLink: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorRuntimeID: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorUniqueID: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AdventureSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BaseDescription: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BaseGameVersion: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeCappedSurfaceData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeClimateData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeConditionalTransformationData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeConsolidatedFeatureList: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeCoordinateData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeDefinition: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeDefinitionChunkGenData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeElementData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeLegacyWorldGenRulesData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeMesaSurfaceData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeMountainParamsData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeMultinoiseGenRulesData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeNoiseGradientSurfaceData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeOverworldGenRulesData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeReplacementData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeScatterParamData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeSurfaceBuilderData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeSurfaceMaterialAdjustmentData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeSurfaceMaterialData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeWeightedData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeWeightedTemperatureData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockPos: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistCategories: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistCategory: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistItemSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPreset: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPresetDefinition: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPriority: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPreset: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPresets: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplineInstruction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChunkPos: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Color: bedrock_network_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;
    type CommandOriginData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerMixDataEntry: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingDataEntry: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingRecipeIngredient: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DataItem: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DebugShape: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DimensionDefinitionGroup: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EduSharedUriResource: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EducationLevelSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EntityNetID: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Experiments: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type FullContainerName: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameRulesChangedPacketData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GatheringsConfig: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryAction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySource: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryTransaction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemEnchants: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemReleaseTransactionData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestNetworkItemInstanceDescriptor: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestSlotInfo: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponseContainerInfo: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponseInfo: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponseSlotInfo: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseOnEntityTransactionData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseTransactionData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapDecoration: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapItemTrackedActorUniqueID: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MaterialReducerDataEntry: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MolangVariableMap: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorAbsoluteData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorDeltaData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovePlayerTeleportData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MultiRecipe: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkBlockPosition: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkItemInstanceDescriptor: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkItemStackDescriptor: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkItemStackDescriptorV2: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkPermissions: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PackedItemUseLegacyInventoryTransaction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PassengerOfBlockArguments: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerBlockActionData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PositionTrackingId: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PotionMixDataEntry: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PropertySyncData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RecipeIngredient: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RecipeUnlockingRequirement: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RedactableString: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ScoreboardId: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SerializedAbilitiesData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SerializedSkin: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShapedRecipe: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShapelessRecipe: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShulkerBoxRecipe: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SignedAudioContent: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SmithingTransformRecipe: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SmithingTrimRecipe: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SoundData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureEditorData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkPos: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkPosOffset: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SyncedPlayerMovementSettings: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type WebSocketPacketData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
}
pub trait ProtoVersionEnums {
    type AbilitiesIndex: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorBlockSyncMessageID: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorDamageCause: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorDataIDs: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorEvent: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorFlags: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorLinkType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AgentActionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AimAssistAction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimatedTextureType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimationExpression: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimationMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ArmSizeType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AttributeModifierOperation: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AttributeOperands: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AuthoritativeMovementMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BookEditAction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BossEventUpdateType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BuildPlatform: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistOperation: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraShakeAction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraShakeType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplineEaseType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplineType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChatRestrictionLevel: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderCodeStatus: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderStorageCategory: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderStorageOperation: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandBlockMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandOriginType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandOutputType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandParameterOption: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandPermissionLevel: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ComplexInventoryTransactionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ConnectionFailReason: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerEnumName: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerID: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ControlScheme: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingDataEntryType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DataItemType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Difficulty: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EasingType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EditorWorldType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EducationEditionOffer: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EnchantType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GamePublishSetting: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GeneratorType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type HudElement: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type HudVisibility: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type IdentityDefinitionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InputMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InteractionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryLayout: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryLeftTabIndex: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryRightTabIndex: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySourceFlags: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySourceType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemDescriptorType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemReleaseInventoryTransactionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackNetResult: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestActionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseInventoryTransactionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseMethod: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseOnActorInventoryTransactionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemVersion: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LabTableReactionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LessonAction: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelEvent: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MinecraftPacketIds: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Mirror: bedrock_network_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;
    type ModalFormCancelReason: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MolangVersion: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovementEffectType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MultiplayerSettingsPacketType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NewInteractionModel: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ObjectiveSortOrder: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type POIBlockInteractionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PackType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketCompressionAlgorithm: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketViolationSeverity: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketViolationType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ParticleType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PersonaPieceType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PhotoType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayStatus: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerAuthInputData: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerPermissionLevel: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerPositionMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerRespawnState: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PredictionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackResponse: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Rotation: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerAuthMovementMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowStoreOfferRedirectType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SimulationType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SoftEnumUpdateType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnBiomeType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnPositionType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureBlockType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureRedstoneSaveMode: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureTemplateRequestOperation: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureTemplateResponseType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TeleportationCause: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TextPacketType: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TextProcessingEventOrigin: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UIProfile: bedrock_network_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
}
pub trait ProtoVersion:
    ProtoVersionPackets
    + ProtoVersionTypes
    + ProtoVersionEnums
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
{
    const PROTOCOL_VERSION: u32;
    const PROTOCOL_BRANCH: &str;
    const GAME_VERSION: &str;
    const RAKNET_VERSION: u8;
}
