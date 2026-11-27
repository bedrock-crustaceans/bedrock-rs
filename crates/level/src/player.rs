//! Player records. A world stores a player's data under one of two key
//! shapes:
//!
//! - `~local_player` -- the singleplayer host (or, on a server, the one
//!   player the game itself tracks outside the client/server split below).
//!   Its value is a [`PlayerData`] compound directly, no indirection.
//! - **Remote players**, reached through a client id first: `player_<id>`
//!   holds a [`PlayerClient`] compound naming a `player_server_<...>` record
//!   ([`PlayerClient::server_id`]) that holds the actual [`PlayerData`].
//!   [`read_remote_player`] performs both hops; [`remote_player_client_ids`]
//!   enumerates the client ids a world has by scanning for the `player_`
//!   prefix. This crate's own fixture corpus carries no remote-player
//!   records at all -- every fixture's only player key is `~local_player`
//!   -- so this half is implemented from the on-disk key/value shapes as
//!   documented elsewhere, not cross-checked against a real remote-player
//!   record the way the rest of this crate's format claims are.
//!
//! Neither shape's payload is modeled field by field beyond what
//! [`PlayerData`] and [`PlayerClient`] declare -- both accept unknown keys
//! (`#[facet(nbtx::allow_unknown_fields)]`) so a field this crate has not
//! catalogued does not fail the read.

use std::io::Read;

use facet::Facet;

use crate::db::Database;
use crate::error::Result;
use crate::settings::Abilities;

/// ASCII prefix on a `player_<client-id>` key: a remote player's
/// client-side record ([`PlayerClient`]), the first hop of the two-hop
/// lookup this module's `remote_*` functions perform. [`PLAYER_SERVER_PREFIX`]
/// below begins with these same seven bytes, so a scan for this prefix must
/// reject it explicitly rather than matching greedily -- see
/// [`parse_player_client_key`].
pub const PLAYER_CLIENT_PREFIX: &[u8] = b"player_";

/// ASCII prefix on a `player_server_<server-id>` key: the record that holds
/// a remote player's actual [`PlayerData`], reached from a `player_<id>`
/// record's [`PlayerClient::server_id`].
pub const PLAYER_SERVER_PREFIX: &[u8] = b"player_server_";

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct InventoryItemWithSlot {
    pub damage: i16,
    pub slot: i8,
    #[facet(rename = "id")]
    pub id: i16,
    pub count: i8,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct InventoryItem {
    pub damage: i16,
    #[facet(rename = "id")]
    pub id: i16,
    pub count: i8,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct StatusEffect {
    pub id: i8,
    pub duration: i32,
    pub duration_easy: i32,
    pub duration_normal: i32,
    pub duration_hard: i32,
    pub ambient: bool,
    pub amplifier: i8,
    pub show_particles: bool,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct Attribute {
    pub current: f32,
    pub max: f32,
    pub name: String,
    pub base: f32,
    /// TODO: Figure out what this is supposed to be. It seems to be linked to `minecraft:attack_damage`
    ///
    /// Held as a raw [`nbtx::Value`] until that is known: `usize` has no NBT tag
    /// (its width varies by target), and guessing a concrete one would decide the
    /// question by accident. A `Value` accepts whatever is actually there.
    pub modifiers: Option<nbtx::Value>,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the player NBT compound"
)]
pub struct PlayerData {
    pub inventory_version: String,
    pub is_swimming: bool,
    pub enchantment_seed: i32,
    pub fall_distance: f32,
    pub mark_variant: i32,
    #[facet(rename = "LeasherID")]
    pub leasher_id: i64,
    #[facet(rename = "DimensionID")]
    pub dimension_id: i32,
    pub sheared: bool,
    pub sleep_timer: i16,
    pub spawn_y: i32,
    pub chested: bool,
    pub spawn_forced: bool,
    pub is_global: bool,
    pub strength: i32,
    pub cursor_selected_items: InventoryItem,
    pub surface: bool,
    pub strength_max: i32,
    pub color2: i8,
    #[facet(rename = "boundY")]
    pub bound_y: i32,
    #[facet(rename = "boundZ")]
    pub bound_z: i32,
    #[facet(rename = "limitedLife")]
    pub limited_life: i32,
    pub armor: [InventoryItem; 4],
    pub hurt_time: i16,
    pub is_gliding: bool,
    pub player_game_mode: i32,
    pub color: i8,
    /// `[yaw, pitch]`. A plain array rather than [`bedrock_shared::vector::Rotation`]: that is a tuple
    /// struct, which facet reflects as a compound of `0`/`1` fields, while the
    /// tag on disk is a list of floats. Convert with `Rotation::from`.
    pub rotation: [f32; 2],
    pub invulnerable: bool,
    pub is_angry: bool,
    pub active_effects: Vec<StatusEffect>,
    // pub mainhand: i16,
    pub natural_spawn: bool,
    pub death_time: i16,
    pub is_baby: bool,
    pub variant: i32,
    pub spawn_z: i32,
    pub loot_dropped: bool,
    pub selected_inventory_slot: i32,
    pub on_ground: bool,
    pub sneaking: bool,
    pub sleeping: bool,
    pub attributes: Vec<Attribute>,
    pub bed_position_z: i32,
    pub abilities: Abilities,
    #[facet(rename = "UniqueID")]
    pub unique_id: i64,
    pub definitions: Vec<String>,
    pub saddled: bool,
    pub show_bottom: bool,
    #[facet(rename = "TargetID")]
    pub target_id: i64,
    pub bed_position_y: i32,
    pub bed_position_x: i32,
    #[facet(rename = "hasBoundOrigin")]
    pub has_bound_origin: bool,
    pub is_tamed: bool,
    /// `[x, y, z]`. See [`Self::rotation`] for why this is not a [`bedrock_shared::vector::Position`].
    pub pos: [f32; 3],
    pub map_index: i32,
    #[facet(rename = "boundX")]
    pub bound_x: i32,
    pub fire: i16,
    pub ender_chest_inventory: [InventoryItemWithSlot; 27],
    pub is_autonomous: bool,
    pub persistent: bool,
    pub offhand: Vec<InventoryItem>,
    #[facet(rename = "SelectedContainerID")]
    pub selected_contain_id: i32,
    pub owner_new: i64,
    pub player_level: i32,
    /// Thank you Mojang.
    pub is_pregnant: bool,
    #[facet(rename = "id")]
    pub id: i32,
    pub air: i16,
    pub attack_time: i16,
    pub portal_cooldown: i32,
    pub spawn_x: i32,
    pub sitting: bool,
    pub player_level_progress: f32,
}

impl PlayerData {
    /// Reads a player record from little-endian (on-disk) NBT.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from `data` fails or the NBT does not
    /// match the [`PlayerData`] shape.
    pub fn read<R: Read>(mut data: R) -> Result<Self> {
        Ok(nbtx::from_le_bytes(&mut data)?)
    }
}

/// A `player_<client-id>` record: not the player's data itself, but the
/// three ids the game uses to reach it. [`server_id`](Self::server_id) is
/// observed elsewhere (see the module docs) to hold the complete on-disk key
/// of the matching `player_server_<...>` record -- not a bare id to be
/// prefixed -- which is what [`read_remote_player`] looks it up as.
#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct PlayerClient {
    pub msa_id: String,
    pub self_signed_id: String,
    /// The full on-disk key of this client's `player_server_<...>` record
    /// (see the struct docs), e.g. `player_server_<uuid>` -- not just the
    /// `<uuid>` suffix.
    pub server_id: String,
}

impl PlayerClient {
    /// Reads a `player_<client-id>` record from little-endian (on-disk) NBT.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from `data` fails or the NBT does not
    /// match the [`PlayerClient`] shape.
    pub fn read<R: Read>(mut data: R) -> Result<Self> {
        Ok(nbtx::from_le_bytes(&mut data)?)
    }
}

/// Returns the client id suffix of `key` if it is a `player_<id>` key --
/// [`PLAYER_CLIENT_PREFIX`] but *not* the longer [`PLAYER_SERVER_PREFIX`],
/// which is a different record family that happens to share the same
/// leading bytes. `None` for anything else -- like `parse_actorprefix_key`,
/// a key that is not this shape is simply some other kind of record, not a
/// malformed one.
pub fn parse_player_client_key(key: &[u8]) -> Option<&[u8]> {
    if key.starts_with(PLAYER_SERVER_PREFIX) {
        return None;
    }
    key.strip_prefix(PLAYER_CLIENT_PREFIX)
}

/// Enumerates every remote player's client id by scanning `db` for
/// `player_<id>` keys. The backend offers no prefix seek (see
/// [`crate::iter::Keys`]), so this walks the whole database once; that is
/// fine for how rarely this is called, but it is not a hot-path
/// operation. Returns the raw id bytes (the part of the key after
/// [`PLAYER_CLIENT_PREFIX`]), in whatever order the underlying scan visits
/// them.
///
/// # Errors
///
/// Returns an error if the database cannot create a key iterator.
pub fn remote_player_client_ids(db: &Database) -> Result<Vec<Vec<u8>>> {
    let mut keys = db.keys()?;
    let mut ids = Vec::new();
    for kv in &mut keys {
        let key = kv.key();
        if let Some(id) = parse_player_client_key(&key) {
            ids.push(id.to_vec());
        }
    }
    Ok(ids)
}

/// Reads one remote player's full data via the client/server two-hop:
/// fetches `player_<client_id>` (a [`PlayerClient`]), then follows its
/// [`server_id`](PlayerClient::server_id) to fetch and decode the
/// [`PlayerData`] stored there.
///
/// Returns `Ok(None)` if no `player_<client_id>` record exists. A
/// `player_<client_id>` record whose `server_id` names a `player_server_`
/// record that is itself missing is a dangling reference rather than a
/// malformed one, so it is not an error here either -- the same principle
/// [`crate::actor::ChunkActors::read`] applies to a `digp` entry with no
/// matching `actorprefix` record. Unlike that function, this one does not
/// distinguish the two `Ok(None)` causes (no client record vs. a dangling
/// server id) in its return type -- there is no real record in this
/// crate's corpus to say a caller needs that distinction, so it collapses
/// both into one `None` rather than preserving it on spec alone.
///
/// # Errors
///
/// Returns an error if either database read fails, or if either record is
/// present but does not decode as its NBT type.
pub fn read_remote_player(db: &Database, client_id: &[u8]) -> Result<Option<PlayerData>> {
    let mut key = PLAYER_CLIENT_PREFIX.to_vec();
    key.extend_from_slice(client_id);

    let Some(client_bytes) = db.get(&key)? else {
        return Ok(None);
    };
    let client = PlayerClient::read(&*client_bytes)?;

    let Some(server_bytes) = db.get(client.server_id.as_bytes())? else {
        return Ok(None);
    };
    Ok(Some(PlayerData::read(&*server_bytes)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbtx::{Compound, Value};

    /// A `bool` field is `true` only for the exact byte `1`; any other byte,
    /// `2` included, is `false`. This is nbtx's own decode rule for `bool` —
    /// unlike a numeric field's tag width, nothing here widens it.
    #[test]
    fn flag_byte_is_true_only_for_exactly_one() {
        let effect = Value::Compound(Compound::from_iter([
            ("Id".into(), Value::Byte(1)),
            ("Duration".into(), Value::Int(100)),
            ("DurationEasy".into(), Value::Int(100)),
            ("DurationNormal".into(), Value::Int(100)),
            ("DurationHard".into(), Value::Int(100)),
            ("Ambient".into(), Value::Byte(2)),
            ("Amplifier".into(), Value::Byte(0)),
            ("ShowParticles".into(), Value::Byte(1)),
        ]));
        let bytes = nbtx::to_le_bytes(&effect).unwrap();

        let decoded: StatusEffect = nbtx::from_le_bytes(&mut bytes.as_slice()).unwrap();
        assert!(!decoded.ambient);
        assert!(decoded.show_particles);
    }

    // The remote-player two-hop below has no corpus example to check
    // against -- every fixture's only player record is `~local_player`
    // (see the module docs) -- so these are synthetic, marked as such by
    // their names, built from the on-disk key/value shapes rather than from
    // a real record.

    #[test]
    fn parse_player_client_key_accepts_the_client_prefix() {
        assert_eq!(
            parse_player_client_key(b"player_11111111-2222-3333-4444-555555555555"),
            Some(&b"11111111-2222-3333-4444-555555555555"[..])
        );
    }

    #[test]
    fn parse_player_client_key_rejects_the_longer_server_prefix() {
        // `player_server_...` starts with `player_`; a naive prefix strip
        // would misparse it as a client id of `server_...`.
        assert_eq!(
            parse_player_client_key(b"player_server_11111111-2222-3333-4444-555555555555"),
            None
        );
    }

    #[test]
    fn parse_player_client_key_rejects_unrelated_keys() {
        assert_eq!(parse_player_client_key(b"~local_player"), None);
        assert_eq!(parse_player_client_key(b"scoreboard"), None);
        assert_eq!(parse_player_client_key(b"player"), None); // no trailing underscore
    }

    #[test]
    fn player_client_round_trips_through_nbt() {
        let client = Value::Compound(Compound::from_iter([
            ("MsaId".into(), Value::String("msa-1".into())),
            ("SelfSignedId".into(), Value::String("self-signed-1".into())),
            (
                "ServerId".into(),
                Value::String("player_server_11111111-2222-3333-4444-555555555555".into()),
            ),
        ]));
        let bytes = nbtx::to_le_bytes(&client).unwrap();
        let decoded = PlayerClient::read(bytes.as_slice()).unwrap();
        assert_eq!(decoded.msa_id, "msa-1");
        assert_eq!(decoded.self_signed_id, "self-signed-1");
        assert_eq!(
            decoded.server_id,
            "player_server_11111111-2222-3333-4444-555555555555"
        );
    }

    /// End-to-end synthetic two-hop: a `player_<id>` record whose `ServerId`
    /// names a `player_server_<...>` record holding a minimal but complete
    /// [`PlayerData`] compound, resolved through a real (temp) database the
    /// same way a caller would.
    #[test]
    fn read_remote_player_resolves_the_two_hop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let db = Database::open(path.to_str().unwrap()).unwrap();

        let server_key = b"player_server_11111111-2222-3333-4444-555555555555";
        let client_id = b"11111111-2222-3333-4444-555555555555";

        let client_nbt = Value::Compound(Compound::from_iter([
            ("MsaId".into(), Value::String("msa-1".into())),
            ("SelfSignedId".into(), Value::String("self-signed-1".into())),
            (
                "ServerId".into(),
                Value::String(String::from_utf8(server_key.to_vec()).unwrap().into()),
            ),
        ]));
        let mut client_key = PLAYER_CLIENT_PREFIX.to_vec();
        client_key.extend_from_slice(client_id);
        db.insert(&client_key, nbtx::to_le_bytes(&client_nbt).unwrap())
            .unwrap();

        db.insert(server_key, minimal_player_data_bytes()).unwrap();

        let found = read_remote_player(&db, client_id).unwrap();
        assert!(found.is_some());

        let ids = remote_player_client_ids(&db).unwrap();
        assert_eq!(ids, vec![client_id.to_vec()]);
    }

    #[test]
    fn read_remote_player_returns_none_for_an_unknown_client_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let db = Database::open(path.to_str().unwrap()).unwrap();
        assert!(read_remote_player(&db, b"nonexistent").unwrap().is_none());
    }

    /// A `player_<id>` record whose `ServerId` names a `player_server_`
    /// record that does not exist is a dangling reference, not an error.
    #[test]
    fn read_remote_player_returns_none_for_a_dangling_server_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let db = Database::open(path.to_str().unwrap()).unwrap();

        let client_id = b"11111111-2222-3333-4444-555555555555";
        let client_nbt = Value::Compound(Compound::from_iter([
            ("MsaId".into(), Value::String("msa-1".into())),
            ("SelfSignedId".into(), Value::String("self-signed-1".into())),
            (
                "ServerId".into(),
                Value::String("player_server_00000000-0000-0000-0000-000000000000".into()),
            ),
        ]));
        let mut client_key = PLAYER_CLIENT_PREFIX.to_vec();
        client_key.extend_from_slice(client_id);
        db.insert(&client_key, nbtx::to_le_bytes(&client_nbt).unwrap())
            .unwrap();

        assert!(read_remote_player(&db, client_id).unwrap().is_none());
    }

    /// The minimal set of [`PlayerData`] fields, NBT-encoded, so the
    /// two-hop tests above can decode a real (if minimal) record at the end
    /// of the chain rather than an arbitrary compound.
    #[expect(
        clippy::too_many_lines,
        reason = "one line per required PlayerData field"
    )]
    fn minimal_player_data_bytes() -> Vec<u8> {
        nbtx::to_le_bytes(&Value::Compound(Compound::from_iter([
            ("InventoryVersion".into(), Value::String("1.0.0".into())),
            ("IsSwimming".into(), Value::Byte(0)),
            ("EnchantmentSeed".into(), Value::Int(0)),
            ("FallDistance".into(), Value::Float(0.0)),
            ("MarkVariant".into(), Value::Int(0)),
            ("LeasherID".into(), Value::Long(0)),
            ("DimensionID".into(), Value::Int(0)),
            ("Sheared".into(), Value::Byte(0)),
            ("SleepTimer".into(), Value::Short(0)),
            ("SpawnY".into(), Value::Int(0)),
            ("Chested".into(), Value::Byte(0)),
            ("SpawnForced".into(), Value::Byte(0)),
            ("IsGlobal".into(), Value::Byte(0)),
            ("Strength".into(), Value::Int(0)),
            (
                "CursorSelectedItems".into(),
                Value::Compound(Compound::from_iter([
                    ("Damage".into(), Value::Short(0)),
                    ("id".into(), Value::Short(0)),
                    ("Count".into(), Value::Byte(0)),
                ])),
            ),
            ("Surface".into(), Value::Byte(0)),
            ("StrengthMax".into(), Value::Int(0)),
            ("Color2".into(), Value::Byte(0)),
            ("boundY".into(), Value::Int(0)),
            ("boundZ".into(), Value::Int(0)),
            ("limitedLife".into(), Value::Int(0)),
            (
                "Armor".into(),
                Value::List(
                    (0..4)
                        .map(|_| {
                            Value::Compound(Compound::from_iter([
                                ("Damage".into(), Value::Short(0)),
                                ("id".into(), Value::Short(0)),
                                ("Count".into(), Value::Byte(0)),
                            ]))
                        })
                        .collect::<Vec<Value>>()
                        .try_into()
                        .unwrap(),
                ),
            ),
            ("HurtTime".into(), Value::Short(0)),
            ("IsGliding".into(), Value::Byte(0)),
            ("PlayerGameMode".into(), Value::Int(0)),
            ("Color".into(), Value::Byte(0)),
            (
                "Rotation".into(),
                Value::List(
                    nbtx::ValueList::try_from(vec![Value::Float(0.0), Value::Float(0.0)]).unwrap(),
                ),
            ),
            ("Invulnerable".into(), Value::Byte(0)),
            ("IsAngry".into(), Value::Byte(0)),
            ("ActiveEffects".into(), Value::List(nbtx::ValueList::End)),
            ("NaturalSpawn".into(), Value::Byte(0)),
            ("DeathTime".into(), Value::Short(0)),
            ("IsBaby".into(), Value::Byte(0)),
            ("Variant".into(), Value::Int(0)),
            ("SpawnZ".into(), Value::Int(0)),
            ("LootDropped".into(), Value::Byte(0)),
            ("SelectedInventorySlot".into(), Value::Int(0)),
            ("OnGround".into(), Value::Byte(1)),
            ("Sneaking".into(), Value::Byte(0)),
            ("Sleeping".into(), Value::Byte(0)),
            ("Attributes".into(), Value::List(nbtx::ValueList::End)),
            ("BedPositionZ".into(), Value::Int(0)),
            (
                "Abilities".into(),
                Value::Compound(Compound::from_iter([
                    ("attackmobs".into(), Value::Byte(1)),
                    ("attackplayers".into(), Value::Byte(1)),
                    ("build".into(), Value::Byte(1)),
                    ("doorsandswitches".into(), Value::Byte(1)),
                    ("flying".into(), Value::Byte(0)),
                    ("flySpeed".into(), Value::Float(0.05)),
                    ("instabuild".into(), Value::Byte(0)),
                    ("invulnerable".into(), Value::Byte(0)),
                    ("lightning".into(), Value::Byte(0)),
                    ("mayfly".into(), Value::Byte(0)),
                    ("mine".into(), Value::Byte(1)),
                    ("op".into(), Value::Byte(0)),
                    ("opencontainers".into(), Value::Byte(1)),
                    ("permissionsLevel".into(), Value::Int(0)),
                    ("playerPermissionsLevel".into(), Value::Int(1)),
                    ("teleport".into(), Value::Byte(0)),
                    ("walkSpeed".into(), Value::Float(0.1)),
                    ("verticalFlySpeed".into(), Value::Float(1.0)),
                ])),
            ),
            ("UniqueID".into(), Value::Long(0)),
            ("Definitions".into(), Value::List(nbtx::ValueList::End)),
            ("Saddled".into(), Value::Byte(0)),
            ("ShowBottom".into(), Value::Byte(0)),
            ("TargetID".into(), Value::Long(0)),
            ("BedPositionY".into(), Value::Int(0)),
            ("BedPositionX".into(), Value::Int(0)),
            ("hasBoundOrigin".into(), Value::Byte(0)),
            ("IsTamed".into(), Value::Byte(0)),
            (
                "Pos".into(),
                Value::List(
                    nbtx::ValueList::try_from(vec![
                        Value::Float(0.0),
                        Value::Float(0.0),
                        Value::Float(0.0),
                    ])
                    .unwrap(),
                ),
            ),
            ("MapIndex".into(), Value::Int(0)),
            ("boundX".into(), Value::Int(0)),
            ("Fire".into(), Value::Short(0)),
            (
                "EnderChestInventory".into(),
                Value::List(
                    (0..27)
                        .map(|_| {
                            Value::Compound(Compound::from_iter([
                                ("Damage".into(), Value::Short(0)),
                                ("Slot".into(), Value::Byte(0)),
                                ("id".into(), Value::Short(0)),
                                ("Count".into(), Value::Byte(0)),
                            ]))
                        })
                        .collect::<Vec<Value>>()
                        .try_into()
                        .unwrap(),
                ),
            ),
            ("IsAutonomous".into(), Value::Byte(0)),
            ("Persistent".into(), Value::Byte(1)),
            ("Offhand".into(), Value::List(nbtx::ValueList::End)),
            ("SelectedContainerID".into(), Value::Int(0)),
            ("OwnerNew".into(), Value::Long(0)),
            ("PlayerLevel".into(), Value::Int(0)),
            ("IsPregnant".into(), Value::Byte(0)),
            ("id".into(), Value::Int(0)),
            ("Air".into(), Value::Short(0)),
            ("AttackTime".into(), Value::Short(0)),
            ("PortalCooldown".into(), Value::Int(0)),
            ("SpawnX".into(), Value::Int(0)),
            ("Sitting".into(), Value::Byte(0)),
            ("PlayerLevelProgress".into(), Value::Float(0.0)),
        ])))
        .unwrap()
    }
}
