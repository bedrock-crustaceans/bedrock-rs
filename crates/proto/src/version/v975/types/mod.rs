macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(biome_definition_chunk_gen_data);
export!(biome_noise_gradient_surface_data);
export!(biome_surface_builder_data);
export!(client_store_entry_point_configuration);
export!(debug_shape);
export!(dimension_definition_group);
export!(presence_configuration);
