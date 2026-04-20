use bedrockrs_macros::{ProtoCodec, packet};

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
    Balancer = 5,
    BlockTickingQueues = 6,
    BiomeStorage = 7,
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
    Network = 43,
    Marketplace = 44,
    MaterialDragonCompiledDefinition = 45,
    MaterialDragonMaterial = 46,
    MaterialDragonResource = 47,
    MaterialDragonUniformMap = 48,
    MaterialRenderMaterial = 49,
    MaterialRenderMaterialGroup = 50,
    MaterialVariationManager = 51,
    MoLang = 52,
    OreUI = 53,
    Persona = 54,
    Player = 55,
    RenderChunk = 56,
    RenderChunkIndexBuffer = 57,
    RenderChunkVertexBuffer = 58,
    Rendering = 59,
    RenderingLibrary = 60,
    RequestLog = 61,
    ResourcePacks = 62,
    Sound = 63,
    SubChunkBiomeData = 64,
    SubChunkBlockData = 65,
    SubChunkLightData = 66,
    Textures = 67,
    VR = 68,
    WeatherRenderer = 69,
    WorldGenerator = 70,
    Tasks = 71,
    Test = 72,
    Scripting = 73,
    ScriptingRuntime = 74,
    ScriptingContext = 75,
    ScriptingContextBindingsMC = 76,
    ScriptingContextBindingsGT = 77,
    ScriptingContextRun = 78,
    DataDrivenUI = 79,
    DataDrivenUIDefs = 80,
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
