//! The published script `metadata/` files as written, one module per file family.
//! Each file type has a strict `parse(&[u8])`: the files are written
//! by the game's metadata generator, so a key no field takes is an error, not a member to keep.
//! Nothing here is read at pack-load time; the generated tables and chorus's code generator are
//! built from these models.

pub mod commands;
pub mod engine;
pub mod molang;
pub mod script;
pub mod vanilla;
