macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(player_armor_damage);
export!(server_bound_pack_setting_change);
