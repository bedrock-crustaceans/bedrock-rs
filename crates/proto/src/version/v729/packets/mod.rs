macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist);
export!(container_registry_cleanup);
export!(emote);
export!(inventory_content);
export!(inventory_slot);
export!(resource_packs_info);
export!(transfer_player);
export!(update_attributes);
