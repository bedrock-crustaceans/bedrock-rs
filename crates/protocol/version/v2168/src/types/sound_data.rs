use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum SoundData {
    Stop = 0,
    SetVolume {
        #[endianness(le)]
        volume: f32,
    } = 1,
    SetPitch {
        #[endianness(le)]
        pitch: f32,
    } = 2,
    Fade {
        #[endianness(le)]
        duration: f32,
        #[endianness(le)]
        target_volume: f32,
    } = 3,
    SeekTo {
        #[endianness(le)]
        seconds: f32,
    } = 4,
    Pause = 5,
    Resume = 6,
}
