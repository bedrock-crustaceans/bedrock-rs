//! `crate::player` against real fixture worlds.
//!
//! Unlike the other Phase 4 keys this crate's corpus establishes a byte
//! layout for, the remote-player two-hop (`player_<id>` / `player_server_
//! <id>`) has no real example anywhere in the corpus: every fixture's only
//! player record is `~local_player`. This file pins that absence as an
//! explicit, checked fact (so it is noticed if a future fixture import ever
//! changes it) rather than leaving it as an unstated assumption.
//!
//! It does not assert that every fixture's `~local_player` record decodes
//! as [`bedrock_level::player::PlayerData`] -- that is pre-existing
//! behavior outside this file's scope, and at least one fixture (`v1_12`,
//! a pre-flattening world) fails that decode today (an `InventoryItem`
//! field `PlayerData` requires that the real record does not carry in that
//! era's shape). That gap belongs to `PlayerData`/`InventoryItem`, not to
//! the remote-player scan this file actually checks.

pub mod support;

use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::player::{parse_player_client_key, remote_player_client_ids};

use std::io::Cursor;

/// No fixture has any `player_`-prefixed (remote player) key, and every
/// fixture has exactly one `~local_player` key.
#[test]
fn every_fixture_has_only_a_local_player_and_no_remote_players() {
    for fixture in support::fixtures() {
        let (_tmp, db) = fixture.open();

        let remote_ids =
            remote_player_client_ids(&db).unwrap_or_else(|e| panic!("{}: {e}", fixture.name));
        assert!(
            remote_ids.is_empty(),
            "{}: expected no remote player_<id> keys, found {}",
            fixture.name,
            remote_ids.len()
        );

        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: {e}", fixture.name));
        let mut local_player_count = 0usize;
        for kv in &mut keys {
            let key_bytes = kv.key();
            assert!(
                parse_player_client_key(&key_bytes).is_none(),
                "{}: found a player_<id> key even though remote_player_client_ids saw none",
                fixture.name
            );

            let mut cursor = Cursor::new(key_bytes.as_ref());
            if let Ok(key) = Key::deserialize(&mut cursor)
                && key.data == KeyVariant::LocalPlayer
            {
                local_player_count += 1;
            }
        }
        assert_eq!(
            local_player_count, 1,
            "{}: expected exactly one ~local_player record",
            fixture.name
        );
    }
}
