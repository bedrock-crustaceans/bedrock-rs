use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 315)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundDiagnosticsPacket {
    #[endianness(le)]
    pub avg_fps: f32,
    #[endianness(le)]
    pub avg_server_tick_time_ms: f32,
    #[endianness(le)]
    pub avg_client_tick_time_ms: f32,
    #[endianness(le)]
    pub avg_begin_frame_time_ms: f32,
    #[endianness(le)]
    pub avg_input_time_ms: f32,
    #[endianness(le)]
    pub avg_render_time_ms: f32,
    #[endianness(le)]
    pub avg_end_frame_time_ms: f32,
    #[endianness(le)]
    pub avg_remainder_time_percent: f32,
    #[endianness(le)]
    pub avg_unnacounted_time_percent: f32,
    pub memory_category_values: Vec<MemoryCategoryCounter>,
    pub entity_diagnostics: Vec<EntityDiagnosticTimingInfo>,
    pub system_diagnostics: Vec<SystemDiagnosticTimingInfo>,
    pub system_categories: Vec<SystemCategory>,
    pub whisker_scopes: Vec<WhiskerScopeDataSummary>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct WhiskerScopeDataSummary {
    pub indentation: String,
    pub label: String,
    #[endianness(le)]
    pub total_high_cost_ns: i64,
    #[endianness(le)]
    pub total_mid_cost_ns: i64,
    #[endianness(le)]
    pub total_low_cost_ns: i64,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SystemCategory {
    pub category_name: String,
    #[endianness(le)]
    pub system_index: i64,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MemoryCategoryCounter {
    pub category: MemoryCategoryCounterType,
    #[endianness(le)]
    pub current_bytes: i64,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum MemoryCategoryCounterType {
    Unknown = 0,
    InvalidSizeUnknown = 1,
    Actor = 2,
    ActorAnimation = 3,
    ActorRendering = 4,
    BlockTickingQueues = 5,
    BiomeStorage = 6,
    Blobs = 7,
    Cereal = 8,
    CircuitSystem = 9,
    Client = 10,
    Commands = 11,
    DBStorage = 12,
    Debug = 13,
    Documentation = 14,
    EcsSystems = 15,
    FMOD = 16,
    Fonts = 17,
    ImGUI = 18,
    Input = 19,
    JsonUI = 20,
    JsonUIControlFactoryJson = 21,
    JsonUIControlTree = 22,
    JsonUIControlTreeControlElement = 23,
    JsonUIControlTreePopulateDataBinding = 24,
    JsonUIControlTreePopulateFocus = 25,
    JsonUIControlTreePopulateLayout = 26,
    JsonUIControlTreePopulateOther = 27,
    JsonUIControlTreePopulateSprite = 28,
    JsonUIControlTreePopulateText = 29,
    JsonUIControlTreePopulateTTS = 30,
    JsonUIControlTreeVisibility = 31,
    JsonUICreateUI = 32,
    JsonUIDefs = 33,
    JsonUILayoutManager = 34,
    JsonUILayoutManagerRemoveDependencies = 35,
    JsonUILayoutManagerInitVariable = 36,
    Languages = 37,
    Level = 38,
    LevelStructures = 39,
    LevelChunk = 40,
    LevelChunkGen = 41,
    LevelChunkGenThreadLocal = 42,
    LightVolumeManager = 43,
    Network = 44,
    Marketplace = 45,
    MaterialDragonCompiledDefinition = 46,
    MaterialDragonMaterial = 47,
    MaterialDragonResource = 48,
    MaterialDragonUniformMap = 49,
    MaterialRenderMaterial = 50,
    MaterialRenderMaterialGroup = 51,
    MaterialVariationManager = 52,
    MoLang = 53,
    OreUI = 54,
    OreUIClient = 55,
    PersonaPieces = 56,
    PersonaAnimations = 57,
    PersonaTextures = 58,
    PersonaCharacters = 59,
    PersonaSkinPacks = 60,
    PersonaRepo = 61,
    Player = 62,
    RenderChunk = 63,
    RenderChunkIndexBuffer = 64,
    RenderChunkVertexBuffer = 65,
    Rendering = 66,
    RenderingBgfxInit = 67,
    RenderingBgfxStartFrame = 68,
    RenderingBgfxTessellator = 69,
    RenderingBgfxEndFrame = 70,
    RenderingBgfxGraphicsTasksInit = 71,
    RenderingLibrary = 72,
    RenderingPolygonOperatorPool = 73,
    RenderingPbrTextureData = 74,
    RenderingRenderRegistry = 75,
    RenderingSetup = 76,
    RenderingVertices = 77,
    RequestLog = 78,
    ResourcePacks = 79,
    Sound = 80,
    SubChunkBiomeData = 81,
    SubChunkBlockData = 82,
    SubChunkLightData = 83,
    Textures = 84,
    WeatherRenderer = 85,
    WorldGenerator = 86,
    Tasks = 87,
    Test = 88,
    TestLoadTestFlags = 89,
    Scripting = 90,
    ScriptingRuntime = 91,
    ScriptingContext = 92,
    ScriptingContextBindingsMC = 93,
    ScriptingContextBindingsGT = 94,
    ScriptingContextRun = 95,
    DataDrivenUI = 96,
    DataDrivenUIDefs = 97,
    Gameface = 98,
    GamefaceSystem = 99,
    GamefaceDom = 100,
    GamefaceCss = 101,
    GamefaceDisplay = 102,
    GamefaceTempAllocator = 103,
    GamefacePoolAllocator = 104,
    GamefaceDump = 105,
    GamefaceMedia = 106,
    GamefaceJson = 107,
    GamefaceScriptEngine = 108,
    GamefaceScript = 109,
    GamefaceLayout = 110,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct EntityDiagnosticTimingInfo {
    pub display_name: String,
    pub entity: String,
    #[endianness(le)]
    pub ns_time: i64,
    pub total_percent: u8,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SystemDiagnosticTimingInfo {
    pub display_name: String,
    #[endianness(le)]
    pub system_index: i64,
    #[endianness(le)]
    pub ns_time: i64,
    pub total_percent: u8,
}
