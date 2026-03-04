macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(client_bound_debug_renderer);
export!(correct_player_move_prediction);
export!(legacy_telemetry_event);
export!(resource_pack_stack);
export!(update_player_game_type);
