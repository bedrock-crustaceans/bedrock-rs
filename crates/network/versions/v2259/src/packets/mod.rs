macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(chat);
export!(client_bound_attribute_layer_sync);
export!(client_bound_data_store);
export!(client_bound_music_playback_failure);
export!(client_bound_online_music_playback_response);
export!(client_bound_play_audio_content);
export!(command_block_update);
export!(command_output);
export!(network_settings);
export!(player_action);
export!(player_toggle_armor_visibility_request);
export!(play_sound);
export!(resource_pack_stack);
export!(server_bound_data_store);
export!(server_bound_request_online_music_playback);
export!(start_game);
export!(structure_block_update);
export!(transfer_player);
export!(virtual_container_close);
export!(virtual_container_open);
