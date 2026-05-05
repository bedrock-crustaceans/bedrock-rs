macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(authoritative_movement_mode);
export!(movement_effect_type);
