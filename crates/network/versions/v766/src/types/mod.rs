macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist_categories);
export!(camera_aim_assist_category);
export!(camera_aim_assist_item_settings);
export!(camera_aim_assist_preset);
export!(camera_aim_assist_preset_definition);
export!(camera_aim_assist_priority);
export!(camera_preset);
export!(item_stack_response_slot_info);
