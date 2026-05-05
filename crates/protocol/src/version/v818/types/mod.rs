macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_instruction);
export!(camera_preset);
export!(debug_shape);
export!(level_settings);
export!(synced_player_movement_settings);
