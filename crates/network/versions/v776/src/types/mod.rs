macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist_preset_definition);
export!(camera_preset);
export!(serialized_abilities_data);
export!(structure_editor_data);
