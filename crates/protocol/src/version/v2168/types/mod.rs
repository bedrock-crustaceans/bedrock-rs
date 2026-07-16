macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(crafting_recipe_ingredient);
export!(dimension_definition_group);
export!(gatherings_config);
export!(item_stack_request_network_item_instance_descriptor);
export!(item_stack_request_slot_info);
export!(item_stack_response_info);
export!(item_stack_response_slot_info);
export!(level_settings);
export!(map_decoration);
export!(map_item_tracked_actor_unique_id);
export!(move_actor_delta_data);
export!(move_player_teleport_data);
export!(multi_recipe);
export!(network_item_instance_descriptor);
export!(network_item_stack_descriptor);
export!(network_item_stack_descriptor_v2);
export!(player_block_action_data);
export!(recipe_ingredient);
export!(recipe_unlocking_requirement);
export!(redactable_string);
export!(serialized_skin);
export!(shaped_recipe);
export!(shapeless_recipe);
export!(smithing_transform_recipe);
export!(smithing_trim_recipe);
export!(sound_data);
export!(structure_editor_data);
export!(sub_chunk_pos);
