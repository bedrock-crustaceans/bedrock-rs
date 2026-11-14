macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(biome_definition_chunk_gen_data);
export!(biome_replacement_data);
export!(camera_instruction);
export!(debug_shape);
