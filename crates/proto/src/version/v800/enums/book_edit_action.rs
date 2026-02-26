use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum BookEditAction {
    ReplacePage {
        page_index: i8,
        text_a: String,
        text_b: String,
    } = 0,
    AddPage {
        page_index: i8,
        text_a: String,
        text_b: String,
    } = 1,
    DeletePage {
        page_index: i8,
    } = 2,
    SwapPages {
        page_index_a: i8,
        page_index_b: i8,
    } = 3,
    Finalize {
        text_a: String,
        text_b: String,
        xuid: String,
    } = 4,
}