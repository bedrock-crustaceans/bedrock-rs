macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_instruction);
export!(change_dimension);
export!(current_structure_feature);
export!(disconnect);
export!(editor_network);
export!(inventory_content);
export!(inventory_slot);
export!(jigsaw_structure_data);
export!(mob_armor_equipment);
export!(player_armor_damage);
export!(resource_packs_info);
export!(server_bound_diagnostics);
export!(server_bound_loading_screen);
export!(set_title);
export!(stop_sound);
