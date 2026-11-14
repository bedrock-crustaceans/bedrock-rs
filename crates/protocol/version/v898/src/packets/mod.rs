macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(animate);
export!(available_commands);
export!(client_bound_data_store);
export!(command_output);
export!(command_request);
export!(interact);
export!(legacy_telemetry_event);
export!(mob_effect);
export!(resource_pack_stack);
export!(server_bound_data_store);
export!(start_game);
export!(text);
