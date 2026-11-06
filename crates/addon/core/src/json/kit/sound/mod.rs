//! The reader kit of the sound families: the scalar members of a merged sound document as the
//! game's reader (a jsoncpp variant) reads them, the reporting while one is read, and the
//! writing of their typed values. The JSON rules themselves are
//! [`jsoncpp::fork`](crate::json::jsoncpp::fork), the merge of the copies across the stack
//! `load::sound`.

pub mod read;
pub mod report;
pub mod write;
