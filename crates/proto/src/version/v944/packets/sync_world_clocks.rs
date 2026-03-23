use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 344)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SyncWorldClocksPacket {
    pub data: SyncWorldClocks 
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(le)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum SyncWorldClocks {
    SyncState {
        clock_data: Vec<SyncWorldClockState>
    } = 0,
    InitializeRegistry {
        clock_data: Vec<WorldClock>
    } = 1,
    AddTimeMarker {
        #[endianness(var)]
        clock_id: u64,
        time_markers: Vec<TimeMarker>,
    } = 2,
    RemoveTimeMarker {
        #[endianness(var)]
        clock_id: u64,
        #[endianness(var)]
        time_marker_ids: Vec<u64>
    } = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SyncWorldClockState {
    #[endianness(var)]
    pub clock_id: u64,
    #[endianness(var)]
    pub time: i32,
    pub paused: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct WorldClock {
    #[endianness(var)]
    pub id: u64,
    pub name: String,
    #[endianness(var)]
    pub time: i32,
    pub paused: bool,
    pub time_markers: Vec<TimeMarker>
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct TimeMarker {
    #[endianness(var)]
    pub id: u64,
    pub name: String,
    #[endianness(var)]
    pub time: i32,
    #[endianness(var)]
    pub period: i32,
}