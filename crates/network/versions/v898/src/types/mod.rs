macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist_category);
export!(camera_aim_assist_preset_definition);
export!(command_origin_data);
