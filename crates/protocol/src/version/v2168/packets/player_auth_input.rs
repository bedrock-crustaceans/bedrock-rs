use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use player_auth_input_packet::PerformItemStackRequestData;
use std::io::{Read, Write};

#[packet(id = 144)]
#[derive(Clone, Debug)]
pub struct PlayerAuthInputPacket<V: ProtoVersion> {
    pub player_rotation: (f32, f32),
    pub player_position: (f32, f32, f32),
    pub move_vector: (f32, f32),
    pub player_head_rotation: f32,
    pub input_data: Vec<V::PlayerAuthInputData>,
    pub input_mode: V::InputMode,
    pub play_mode: ClientPlayMode,
    pub new_interaction_model: V::NewInteractionModel,
    pub interact_rotation: (f32, f32),
    pub client_tick: u64,
    pub pos_delta: (f32, f32, f32),
    pub item_use_transaction: Option<V::PackedItemUseLegacyInventoryTransaction>,
    pub item_stack_request: Option<PerformItemStackRequestData<V>>,
    pub player_block_actions: Option<Vec<V::PlayerBlockActionData>>,
    pub vehicle_rotation: Option<(f32, f32)>,
    pub client_predicted_vehicle: Option<V::ActorUniqueID>,
    pub analog_move_vector: (f32, f32),
    pub camera_orientation: (f32, f32, f32),
    pub raw_move_vector: (f32, f32),
}

pub mod player_auth_input_packet {
    use crate::ProtoVersion;
    use bedrock_macros::ProtoCodec;

    #[derive(ProtoCodec, Clone, Debug)]
    pub struct PerformItemStackRequestData<V: ProtoVersion> {
        #[endianness(var)]
        pub client_request_id: u32,
        pub actions: Vec<V::ItemStackRequestActionType>,
        pub strings_to_filter: Vec<String>,
        pub strings_to_filter_origin: V::TextProcessingEventOrigin,
    }
}

impl<V: ProtoVersion> ProtoCodec for PlayerAuthInputPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <(f32, f32) as ProtoCodecLE>::serialize(&self.player_rotation, stream)?;
        <(f32, f32, f32) as ProtoCodecLE>::serialize(&self.player_position, stream)?;
        <(f32, f32) as ProtoCodecLE>::serialize(&self.move_vector, stream)?;
        <f32 as ProtoCodecLE>::serialize(&self.player_head_rotation, stream)?;
        <bool as ProtoCodec>::serialize(&true, stream)?;
        <Vec<V::PlayerAuthInputData> as ProtoCodec>::serialize(&self.input_data, stream)?;
        <V::InputMode as ProtoCodec>::serialize(&self.input_mode, stream)?;
        <ClientPlayMode as ProtoCodec>::serialize(&self.play_mode, stream)?;
        <V::NewInteractionModel as ProtoCodec>::serialize(&self.new_interaction_model, stream)?;
        <(f32, f32) as ProtoCodecLE>::serialize(&self.interact_rotation, stream)?;
        <u64 as ProtoCodecVAR>::serialize(&self.client_tick, stream)?;
        <(f32, f32, f32) as ProtoCodecLE>::serialize(&self.pos_delta, stream)?;
        <bool as ProtoCodec>::serialize(&true, stream)?;
        <Option<V::PackedItemUseLegacyInventoryTransaction> as ProtoCodec>::serialize(
            &self.item_use_transaction,
            stream,
        )?;
        <bool as ProtoCodec>::serialize(&true, stream)?;
        <Option<PerformItemStackRequestData<V>> as ProtoCodec>::serialize(
            &self.item_stack_request,
            stream,
        )?;
        <bool as ProtoCodec>::serialize(&true, stream)?;
        <Option<Vec<V::PlayerBlockActionData>> as ProtoCodec>::serialize(
            &self.player_block_actions,
            stream,
        )?;
        <bool as ProtoCodec>::serialize(&true, stream)?;
        <Option<(f32, f32)> as ProtoCodecLE>::serialize(&self.vehicle_rotation, stream)?;
        <bool as ProtoCodec>::serialize(&true, stream)?;
        <Option<V::ActorUniqueID> as ProtoCodec>::serialize(
            &self.client_predicted_vehicle,
            stream,
        )?;
        <(f32, f32) as ProtoCodecLE>::serialize(&self.analog_move_vector, stream)?;
        <(f32, f32, f32) as ProtoCodecLE>::serialize(&self.camera_orientation, stream)?;
        <(f32, f32) as ProtoCodecLE>::serialize(&self.raw_move_vector, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let player_rotation = <(f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let player_position = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let move_vector = <(f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let player_head_rotation = <f32 as ProtoCodecLE>::deserialize(stream)?;
        let input_data = match <bool as ProtoCodec>::deserialize(stream)? {
            true => <Vec<V::PlayerAuthInputData> as ProtoCodec>::deserialize(stream)?,
            false => Vec::new(),
        };
        let input_mode = <V::InputMode as ProtoCodec>::deserialize(stream)?;
        let play_mode = <ClientPlayMode as ProtoCodec>::deserialize(stream)?;
        let new_interaction_model = <V::NewInteractionModel as ProtoCodec>::deserialize(stream)?;
        let interact_rotation = <(f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let client_tick = <u64 as ProtoCodecVAR>::deserialize(stream)?;
        let pos_delta = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let item_use_transaction = match <bool as ProtoCodec>::deserialize(stream)? {
            true => <Option<V::PackedItemUseLegacyInventoryTransaction> as ProtoCodec>::deserialize(stream)?,
            false => None,
        };
        let item_stack_request = match <bool as ProtoCodec>::deserialize(stream)? {
            true => <Option<PerformItemStackRequestData<V>> as ProtoCodec>::deserialize(stream)?,
            false => None,
        };
        let player_block_actions = match <bool as ProtoCodec>::deserialize(stream)? {
            true => <Option<Vec<V::PlayerBlockActionData>> as ProtoCodec>::deserialize(stream)?,
            false => None,
        };
        let vehicle_rotation = match <bool as ProtoCodec>::deserialize(stream)? {
            true => <Option<(f32, f32)> as ProtoCodecLE>::deserialize(stream)?,
            false => None,
        };
        let client_predicted_vehicle = match <bool as ProtoCodec>::deserialize(stream)? {
            true => <Option<V::ActorUniqueID> as ProtoCodec>::deserialize(stream)?,
            false => None,
        };
        let analog_move_vector = <(f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let camera_orientation = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let raw_move_vector = <(f32, f32) as ProtoCodecLE>::deserialize(stream)?;

        Ok(Self {
            player_rotation,
            player_position,
            move_vector,
            player_head_rotation,
            input_data,
            input_mode,
            play_mode,
            new_interaction_model,
            interact_rotation,
            client_tick,
            pos_delta,
            item_use_transaction,
            item_stack_request,
            player_block_actions,
            vehicle_rotation,
            client_predicted_vehicle,
            analog_move_vector,
            camera_orientation,
            raw_move_vector,
        })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecLE::size_hint(&self.player_rotation)
            + ProtoCodecLE::size_hint(&self.player_position)
            + ProtoCodecLE::size_hint(&self.move_vector)
            + ProtoCodecLE::size_hint(&self.player_head_rotation)
            + 5
            + ProtoCodec::size_hint(&self.input_data)
            + self.input_mode.size_hint()
            + self.play_mode.size_hint()
            + self.new_interaction_model.size_hint()
            + ProtoCodecLE::size_hint(&self.interact_rotation)
            + ProtoCodecVAR::size_hint(&self.client_tick)
            + ProtoCodecLE::size_hint(&self.pos_delta)
            + ProtoCodec::size_hint(&self.item_use_transaction)
            + ProtoCodec::size_hint(&self.item_stack_request)
            + ProtoCodec::size_hint(&self.player_block_actions)
            + ProtoCodecLE::size_hint(&self.vehicle_rotation)
            + ProtoCodec::size_hint(&self.client_predicted_vehicle)
            + ProtoCodecLE::size_hint(&self.analog_move_vector)
            + ProtoCodecLE::size_hint(&self.camera_orientation)
            + ProtoCodecLE::size_hint(&self.raw_move_vector)
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
