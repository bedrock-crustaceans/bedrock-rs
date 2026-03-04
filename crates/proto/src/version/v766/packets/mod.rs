macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist);
export!(camera_aim_assist_presets);
export!(player_auth_input);
export!(resource_packs_info);
