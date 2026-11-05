//! What the sound families share: the JSON rules of the game's JSON reader (a jsoncpp variant)
//! the three resource-pack documents are read with, the reporting while one is read, the merge
//! of their copies across the stack, and the writing of their typed values.

pub mod fork;
pub mod merge;
pub mod read;
pub mod report;
pub mod write;
