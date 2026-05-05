macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(biome_definition_chunk_gen_data);
export!(camera_aim_assist_category);
export!(camera_aim_assist_preset_definition);
export!(camera_instruction);
export!(camera_spline_instruction);
export!(debug_shape);
export!(level_settings);
