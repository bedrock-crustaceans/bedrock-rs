use bedrock_macros::{ProtoCodec, packet};

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
    PersonaCharacters = 58,
    PersonaSkinPacks = 59,
    PersonaRepo = 60,
    Player = 61,
    RenderChunk = 62,
    RenderChunkIndexBuffer = 63,
    RenderChunkVertexBuffer = 64,
    Rendering = 65,
    RenderingBgfxInit = 66,
    RenderingBgfxStartFrame = 67,
    RenderingBgfxTessellator = 68,
    RenderingBgfxEndFrame = 69,
    RenderingBgfxGraphicsTasksInit = 70,
    RenderingLibrary = 71,
    RenderingPolygonOperatorPool = 72,
    RenderingPbrTextureData = 73,
    RenderingRenderRegistry = 74,
    RenderingSetup = 75,
    RenderingVertices = 76,
    RequestLog = 77,
    ResourcePacks = 78,
    Sound = 79,
    SubChunkBiomeData = 80,
    SubChunkBlockData = 81,
    SubChunkLightData = 82,
    Textures = 83,
    WeatherRenderer = 84,
    WorldGenerator = 85,
    Tasks = 86,
    Test = 87,
    TestLoadTestFlags = 88,
    Scripting = 89,
    ScriptingRuntime = 90,
    ScriptingContext = 91,
    ScriptingContextBindingsMC = 92,
    ScriptingContextBindingsGT = 93,
    ScriptingContextRun = 94,
    DataDrivenUI = 95,
    DataDrivenUIDefs = 96,
    Gameface = 97,
    GamefaceSystem = 98,
    GamefaceDom = 99,
    GamefaceCss = 100,
    GamefaceDisplay = 101,
    GamefaceTempAllocator = 102,
    GamefacePoolAllocator = 103,
    GamefaceDump = 104,
    GamefaceMedia = 105,
    GamefaceJson = 106,
    GamefaceScriptEngine = 107,
    GamefaceScript = 108,
    GamefaceLayout = 109,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct EntityDiagnosticTimingInfo {
    pub display_name: String,
    pub entity: String,
    #[endianness(le)]
    pub ns_time: i64,
    pub total_percent: u8,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    pub dimension: String,
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
