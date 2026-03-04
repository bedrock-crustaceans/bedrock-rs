macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_instruction);
export!(camera_preset);
export!(shulker_box_recipe);
