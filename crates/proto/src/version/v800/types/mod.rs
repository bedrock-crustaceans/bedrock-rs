macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(biome_capped_surface_data);
export!(biome_climate_data);
export!(biome_conditional_transformation_data);
export!(biome_consolidated_feature_list);
export!(biome_coordinate_data);
export!(biome_definition);
export!(biome_definition_chunk_gen_data);
export!(biome_element_data);
export!(biome_legacy_world_gen_rules_data);
export!(biome_mesa_surface_data);
export!(biome_mountain_params_data);
export!(biome_multinoise_gen_rules_data);
export!(biome_overworld_gen_rules_data);
export!(biome_scatter_param_data);
export!(biome_surface_material_data);
export!(biome_surface_material_adjustment_data);
export!(biome_weighted_data);
export!(biome_weighted_temperature_data);
export!(camera_preset);
export!(color);
