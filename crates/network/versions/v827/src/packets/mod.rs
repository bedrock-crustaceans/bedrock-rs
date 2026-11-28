macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(camera_aim_assist);
export!(camera_instruction);
export!(correct_player_move_prediction);
export!(start_game);
