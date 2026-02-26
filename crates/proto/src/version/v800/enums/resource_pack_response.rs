use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ResourcePackResponse {
    Cancel = 1,
    Downloading = 2,
    DownloadingFinished = 3,
    ResourcePackStackFinished = 4,
}