macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist_instruction);
export!(camera_aim_assist_presets);
export!(command_block_update);
export!(creative_content);
export!(item_component);
export!(movement_prediction_sync);
export!(start_game);
