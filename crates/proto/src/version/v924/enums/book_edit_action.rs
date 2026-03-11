use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum BookEditAction {
    ReplacePage {
        #[endianness(var)]
        page_index: i32,
        text: String,
        photo_name: String,
    } = 0,
    AddPage {
        #[endianness(var)]
        page_index: i32,
        text: String,
        photo_name: String,
    } = 1,
    DeletePage {
        #[endianness(var)]
        page_index: i32,
    } = 2,
    SwapPages {
        #[endianness(var)]
        page_index_a: i32,
        #[endianness(var)]
        page_index_b: i32,
    } = 3,
    Finalize {
        title: String,
        author: String,
        xuid: String,
    } = 4,
}
