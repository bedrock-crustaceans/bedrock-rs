macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_preset);
export!(debug_shape);
export!(dimension_definition_group);
export!(item_stack_response_info);
export!(item_stack_response_slot_info);
export!(move_actor_delta_data);
export!(packed_item_use_legacy_inventory_transaction);
