macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(inventory_content);
export!(inventory_slot);
export!(mob_effect);
export!(movement_effect);
export!(player_auth_input);
export!(resource_packs_info);
export!(set_movement_authority);
