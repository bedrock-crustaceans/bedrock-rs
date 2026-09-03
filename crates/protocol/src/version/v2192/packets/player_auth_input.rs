use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use player_auth_input_packet::PerformItemStackRequestData;

#[packet(id = 144)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerAuthInputPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub player_rotation: (f32, f32),
    #[endianness(le)]
    pub player_position: (f32, f32, f32),
    #[endianness(le)]
    pub move_vector: (f32, f32),
    #[endianness(le)]
    pub player_head_rotation: f32,
    pub input_data: Vec<V::PlayerAuthInputData>,
    pub input_mode: V::InputMode,
    pub play_mode: ClientPlayMode,
    pub new_interaction_model: V::NewInteractionModel,
    #[endianness(le)]
    pub interact_rotation: (f32, f32),
    #[endianness(var)]
    pub client_tick: u64,
    #[endianness(le)]
    pub pos_delta: (f32, f32, f32),
    pub item_use_transaction: Option<V::PackedItemUseLegacyInventoryTransaction>,
    pub item_stack_request: Option<PerformItemStackRequestData<V>>,
    pub player_block_actions: Option<Vec<V::PlayerBlockActionData>>,
    #[endianness(le)]
    pub vehicle_rotation: Option<(f32, f32)>,
    pub client_predicted_vehicle: Option<V::ActorUniqueID>,
    #[endianness(le)]
    pub analog_move_vector: (f32, f32),
    #[endianness(le)]
    pub camera_orientation: (f32, f32, f32),
    #[endianness(le)]
    pub raw_move_vector: (f32, f32),
}

pub mod player_auth_input_packet {
    use crate::ProtoVersion;
    use bedrock_macros::ProtoCodec;

    #[derive(ProtoCodec, Clone, Debug)]
    pub struct PerformItemStackRequestData<V: ProtoVersion> {
        #[endianness(var)]
        pub client_request_id: i32,
        pub actions: Vec<V::ItemStackRequestActionType>,
        pub strings_to_filter: Vec<String>,
        pub strings_to_filter_origin: V::TextProcessingEventOrigin,
    }
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ClientPlayMode {
    Normal = 0,
    Teaser = 1,
    Screen = 2,
    Viewer = 3,
    Reality = 4,
    Placement = 5,
    LivingRoom = 6,
    ExitLevel = 7,
    ExitLevelLivingRoom = 8,
    NumModes = 9,
}
