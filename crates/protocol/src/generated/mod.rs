#![allow(unused)]
pub trait ProtoVersionPackets {
    type ActorEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorPickRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddActorPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddBehaviourTreePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddItemActorPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddPaintingPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddPlayerPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AddVolumeEntityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AgentActionEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AgentAnimationPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimateEntityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimatePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnvilDamagePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AutomationClientConnectPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AvailableActorIdentifiersPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AvailableCommandsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AwardAchievementPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeDefinitionListPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockActorDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockPickRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BookEditPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BossEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistActorPriorityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistInstructionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPresetsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraInstructionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPresetsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraShakePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplinePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChangeDimensionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChangeMobPropertyPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChunkRadiusUpdatedPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundAttributeLayerSyncPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundCloseFormPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundControlSchemeSetPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUICloseAllScreensPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUICloseScreenPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUIReloadPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataDrivenUIShowScreenPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDataStorePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundDebugRendererPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundMapItemDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundTextureShiftPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientBoundUpdateSoundDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientCacheBlobStatusPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientCacheMissResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientCacheStatusPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ClientToServerHandshakePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderSourcePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandBlockUpdatePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandOutputPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CompletedUsingItemPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CompressedBiomeDefinitionListPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerClosePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerOpenPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerRegistryCleanupPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerSetDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CorrectPlayerMovePredictionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CreatePhotoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CreativeContentPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CurrentStructureFeaturePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DeathInfoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DebugDrawerPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DebugInfoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DimensionDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DisconnectPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EditorNetworkPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EduUriResourcePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EducationSettingsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EmoteListPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EmotePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type FeatureRegistryPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type FilterTextPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameRulesChangedPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameTestRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameTestResultsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GraphicsParameterOverridePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GuiDataPickItemPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type HurtArmorPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InteractPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryContentPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySlotPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryTransactionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemComponentPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type JigsawStructureDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LabTablePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LecternUpdatePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LegacyTelemetryEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LessonProgressPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelChunkPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelEventGenericPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventV1Packet: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventV2Packet: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LocatorBarPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LoginPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapCreateLockedCopyPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapInfoRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MobArmorEquipmentPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MobEffectPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MobEquipmentPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ModalFormRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ModalFormResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MotionPredictionHintsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorAbsolutePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorDeltaPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovePlayerPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovementEffectPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovementPredictionSyncPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MultiplayerSettingsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkChunkPublisherUpdatePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkSettingsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkStackLatencyPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NpcDialoguePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NpcRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type OnScreenTextureAnimationPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type OpenSignPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketViolationWarningPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PartyChangedPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PartyDestinationCookieResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PassengerJumpPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PhotoTransferPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlaySoundPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayStatusPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerActionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerArmorDamagePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerAuthInputPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerEnchantOptionsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerFogPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerHotbarPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerInputPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerListPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerLocationPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerSkinPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerStartItemCooldownPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerToggleCrafterSlotRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerUpdateEntityOverridesPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerVideoCapturePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PositionTrackingDBClientRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PositionTrackingDBServerBroadcastPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PurchaseReceiptPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RefreshEntitlementsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RemoveActorPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RemoveObjectivePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RemoveVolumeEntityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestAbilityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestChunkRadiusPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestNetworkSettingsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RequestPermissionsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackChunkDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackChunkRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackClientResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackDataInfoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackStackPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePacksInfoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePacksReadyForValidationPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RespawnPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ScriptMessagePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SendPartyDestinationCookiePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundDataDrivenClosedPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundDataStorePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundDiagnosticsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundLoadingScreenPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerBoundPackSettingChangePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerPlayerPostMovePositionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerPresenceInfoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerSettingsRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerSettingsResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerStatsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerStoreInfoPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerToClientHandshakePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetActorDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetActorLinkPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetActorMotionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetCommandsEnabledPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetDefaultGameTypePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetDifficultyPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetDisplayObjectivePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetHealthPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetHudPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetLastHurtByPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetLocalPlayerAsInitializedPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetMovementAuthorityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetPlayerGameTypePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetPlayerInventoryOptionsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetScorePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetScoreboardIdentityPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetSpawnPositionPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetTimePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SetTitlePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SettingsCommandPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowCreditsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowProfilePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowStoreOfferPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SimpleEventPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SimulationTypePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnExperienceOrbPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnParticleEffectPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StartGamePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StopSoundPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureBlockUpdatePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureDataRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureDataResponsePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubClientLoginPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SyncActorPropertyPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SyncWorldClocksPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TakeItemActorPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TextPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TickSyncPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TickingAreaLoadStatusPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ToastRequestPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TransferPlayerPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TrimDataPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UnlockedRecipesPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateAbilitiesPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateAdventureSettingsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateAttributesPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateBlockPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateBlockSyncedPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateClientInputLocksPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateClientOptionsPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateEquipPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdatePlayerGameTypePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateSoftEnumPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateSubChunkBlocksPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UpdateTradePacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type VoxelShapesPacket: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
}
pub trait ProtoVersionTypes {
    type ActorLink: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorRuntimeID: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorUniqueID: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AdventureSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BaseDescription: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BaseGameVersion: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeCappedSurfaceData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeClimateData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeConditionalTransformationData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeConsolidatedFeatureList: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeCoordinateData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeDefinition: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeDefinitionChunkGenData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeElementData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeLegacyWorldGenRulesData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeMesaSurfaceData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeMountainParamsData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeMultinoiseGenRulesData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeNoiseGradientSurfaceData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeOverworldGenRulesData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeReplacementData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeScatterParamData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeSurfaceBuilderData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeSurfaceMaterialAdjustmentData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeSurfaceMaterialData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeWeightedData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BiomeWeightedTemperatureData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BlockPos: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistCategories: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistCategory: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistItemSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPreset: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPresetDefinition: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistPriority: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraInstruction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPreset: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraPresets: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplineInstruction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChunkPos: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Color: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;
    type CommandOriginData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerMixDataEntry: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingDataEntry: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DataItem: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DebugShape: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DimensionDefinitionGroup: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EduSharedUriResource: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EducationLevelSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EntityNetID: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Experiments: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type FullContainerName: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameRulesChangedPacketData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryAction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySource: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryTransaction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemEnchants: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestSlotInfo: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponseContainerInfo: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponseInfo: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackResponseSlotInfo: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapDecoration: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MapItemTrackedActorUniqueID: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MaterialReducerDataEntry: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MolangVariableMap: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorAbsoluteData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MoveActorDeltaData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkBlockPosition: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkItemInstanceDescriptor: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkItemStackDescriptor: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkItemStackDescriptorV2: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NetworkPermissions: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PackedItemUseLegacyInventoryTransaction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerBlockActionData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PositionTrackingId: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PotionMixDataEntry: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PropertySyncData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RecipeIngredient: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type RecipeUnlockingRequirement: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ScoreboardId: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SerializedAbilitiesData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SerializedSkin: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShapedRecipe: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShapelessRecipe: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShulkerBoxRecipe: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SmithingTransformRecipe: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SmithingTrimRecipe: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureEditorData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkPos: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SubChunkPosOffset: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SyncedPlayerMovementSettings: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type WebSocketPacketData: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
}
pub trait ProtoVersionEnums {
    type AbilitiesIndex: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorBlockSyncMessageID: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorDamageCause: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorDataIDs: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorEvent: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorFlags: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorLinkType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ActorType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AgentActionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AimAssistAction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimatedTextureType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimationExpression: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AnimationMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AttributeModifierOperation: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AttributeOperands: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type AuthoritativeMovementMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BookEditAction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BossEventUpdateType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type BuildPlatform: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraAimAssistOperation: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraShakeAction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraShakeType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplineEaseType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CameraSplineType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ChatRestrictionLevel: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderCodeStatus: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderStorageCategory: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CodeBuilderStorageOperation: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandBlockMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandOriginType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandOutputType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandParameterOption: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CommandPermissionLevel: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ComplexInventoryTransactionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ConnectionFailReason: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerEnumName: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerID: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ContainerType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ControlScheme: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type CraftingDataEntryType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type DataItemType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Difficulty: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EasingType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EditorWorldType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EducationEditionOffer: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type EnchantType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GamePublishSetting: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GameType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type GeneratorType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type HudElement: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type HudVisibility: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type IdentityDefinitionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InputMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InteractionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryLayout: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryLeftTabIndex: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventoryRightTabIndex: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySourceFlags: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type InventorySourceType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemDescriptorType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemReleaseInventoryTransactionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackNetResult: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemStackRequestActionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseInventoryTransactionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseMethod: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemUseOnActorInventoryTransactionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ItemVersion: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LabTableReactionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LessonAction: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelEvent: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type LevelSoundEventType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MinecraftPacketIds: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Mirror: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;
    type ModalFormCancelReason: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MolangVersion: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MovementEffectType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type MultiplayerSettingsPacketType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type NewInteractionModel: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ObjectiveSortOrder: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type POIBlockInteractionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PackType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketCompressionAlgorithm: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketViolationSeverity: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PacketViolationType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ParticleType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PhotoType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayStatus: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerPermissionLevel: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerPositionMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PlayerRespawnState: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type PredictionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ResourcePackResponse: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type Rotation: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ServerAuthMovementMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type ShowStoreOfferRedirectType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SimulationType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SoftEnumUpdateType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnBiomeType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type SpawnPositionType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureBlockType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureRedstoneSaveMode: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureTemplateRequestOperation: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type StructureTemplateResponseType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TeleportationCause: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TextPacketType: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type TextProcessingEventOrigin: bedrock_protocol_core::ProtoCodec
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;
    type UIProfile: bedrock_protocol_core::ProtoCodec
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
mod unknown;
pub use unknown::*;
mod v662;
pub use v662::*;
mod v671;
pub use v671::*;
mod v685;
pub use v685::*;
mod v686;
pub use v686::*;
mod v712;
pub use v712::*;
mod v729;
pub use v729::*;
mod v748;
pub use v748::*;
mod v766;
pub use v766::*;
mod v776;
pub use v776::*;
mod v786;
pub use v786::*;
mod v800;
pub use v800::*;
mod v818;
pub use v818::*;
mod v819;
pub use v819::*;
mod v827;
pub use v827::*;
mod v844;
pub use v844::*;
mod v859;
pub use v859::*;
mod v898;
pub use v898::*;
mod v924;
pub use v924::*;
mod v944;
pub use v944::*;
mod v975;
pub use v975::*;
mod v1001;
pub use v1001::*;
